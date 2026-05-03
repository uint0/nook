use std::time::Duration;

use anyhow::{Context, Result};
use rand::Rng;
use reqwest::StatusCode;
use serde_json::json;
use tracing::{info, warn};

use super::types::{
    CreateInviteReservationData, DesksResponse, EmployeeRegistrationPartialDayData,
    InviteReservationResponse, LocationResponse, MeResponse, RegistrationDate, SignInEntry,
    SignInError, SignInInviteData, UserInfo,
};
use super::util::parse_graphql_result;
use crate::profile::token_store::TokenStore;

const GRAPHQL_URL: &str = "https://app.envoy.com/a/graphql_federated";
const ME_URL: &str = "https://app.envoy.com/a/visitors/api/v2/users/me";
const LOCATION_URL: &str = "https://app.envoy.com/a/visitors/api/v2/locations";
const DESKS_URL: &str = "https://app.envoy.com/a/rms/desks";

const QUERY_EMPLOYEE_REGISTRATION: &str =
    include_str!("queries/EmployeeRegistrationPartialDay.gql");

const MUTATION_CREATE_INVITE_RESERVATION: &str =
    include_str!("queries/CreateInviteReservation.gql");

const MUTATION_SIGN_IN_INVITE: &str = include_str!("queries/SignInInvite.gql");

/// Maximum number of retry attempts on 5xx errors before giving up.
const MAX_RETRIES: u32 = 4;

/// Base delay in milliseconds for exponential backoff. Actual delay per attempt:
/// attempt 1: 500–1000ms, attempt 2: 1000–2000ms, attempt 3: 2000–4000ms, attempt 4: 4000–8000ms.
const BASE_BACKOFF_MS: u64 = 500;

/// Timeout for establishing a connection to the server.
const CONNECT_TIMEOUT_SECS: u64 = 10;

/// Timeout for a complete request (connect + read).
const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct EnvoyClient {
    http: reqwest::Client,
    store: TokenStore,
}

impl EnvoyClient {
    pub fn new(store: TokenStore) -> Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self { http, store })
    }

    /// POST a GraphQL request with the current access token.
    async fn post_graphql(&self, body: &serde_json::Value) -> Result<reqwest::Response> {
        let operation = body
            .get("operationName")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        tracing::debug!(operation, url = GRAPHQL_URL, "Sending GraphQL request");
        let resp = self
            .http
            .post(GRAPHQL_URL)
            .bearer_auth(&self.store.access_token)
            .json(body)
            .send()
            .await
            .context("failed to send GraphQL request")?;
        tracing::debug!(operation, status = %resp.status(), "Received GraphQL response");
        Ok(resp)
    }

    /// GET a REST request with the current access token.
    async fn get_rest(&self, url: &str) -> Result<reqwest::Response> {
        tracing::debug!(url, "Sending REST GET request");
        let resp = self
            .http
            .get(url)
            .bearer_auth(&self.store.access_token)
            .send()
            .await
            .context("failed to send REST request")?;
        tracing::debug!(url, status = %resp.status(), "Received REST response");
        Ok(resp)
    }

    /// Send a GraphQL request with exponential backoff on 5xx / network errors.
    async fn send_with_backoff(&self, body: &serde_json::Value) -> Result<reqwest::Response> {
        let mut attempt = 0u32;
        loop {
            if attempt > 0 {
                tracing::debug!(attempt, "Retrying GraphQL request after backoff");
            }
            match self.post_graphql(body).await {
                Ok(resp) if resp.status().is_server_error() => {
                    attempt += 1;
                    if attempt > MAX_RETRIES {
                        return Ok(resp);
                    }
                    let base = BASE_BACKOFF_MS * (1 << (attempt - 1));
                    let jitter = rand::thread_rng().gen_range(0..base.max(1));
                    let delay = Duration::from_millis(base + jitter);
                    warn!(
                        status = %resp.status(),
                        attempt,
                        delay_ms = delay.as_millis(),
                        "Server error, retrying after backoff..."
                    );
                    tokio::time::sleep(delay).await;
                }
                other => return other.context("failed to send GraphQL request"),
            }
        }
    }

    /// Send a REST GET with exponential backoff on 5xx / network errors.
    async fn get_with_backoff(&self, url: &str) -> Result<reqwest::Response> {
        let mut attempt = 0u32;
        loop {
            if attempt > 0 {
                tracing::debug!(attempt, url, "Retrying REST request after backoff");
            }
            match self.get_rest(url).await {
                Ok(resp) if resp.status().is_server_error() => {
                    attempt += 1;
                    if attempt > MAX_RETRIES {
                        return Ok(resp);
                    }
                    let base = BASE_BACKOFF_MS * (1 << (attempt - 1));
                    let jitter = rand::thread_rng().gen_range(0..base.max(1));
                    let delay = Duration::from_millis(base + jitter);
                    warn!(
                        status = %resp.status(),
                        attempt,
                        delay_ms = delay.as_millis(),
                        "Server error, retrying after backoff..."
                    );
                    tokio::time::sleep(delay).await;
                }
                other => return other.context("failed to send REST request"),
            }
        }
    }

    /// Refresh the access token and persist via the TokenStore.
    async fn do_refresh(&mut self) -> Result<()> {
        info!("Refreshing access token...");
        let resp = super::auth::refresh(&self.http, &self.store.refresh_token)
            .await
            .context("token refresh failed")?;
        self.store.update(
            &resp.access_token,
            resp.expires_in,
            &resp.refresh_token,
            resp.refresh_token_expires_in,
        )?;
        Ok(())
    }

    /// Send a GraphQL request with backoff retries, then handle 401/403 with a
    /// token refresh and a single retry.
    async fn request_with_retry(&mut self, body: &serde_json::Value) -> Result<reqwest::Response> {
        let resp = self.send_with_backoff(body).await?;
        if matches!(
            resp.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            warn!(status = %resp.status(), "Got auth error, retrying after token refresh...");
            self.do_refresh().await?;
            return self.send_with_backoff(body).await;
        }
        Ok(resp)
    }

    /// Fetch all desks for a location from the /desks endpoint.
    pub async fn get_desks(&mut self, location_id: &str) -> Result<DesksResponse> {
        let url = format!("{DESKS_URL}?filter[location-id]={location_id}");
        tracing::debug!(location_id, "Fetching desks list");
        let resp = self.get_with_backoff(&url).await?;
        if matches!(resp.status(), StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            tracing::warn!(status = %resp.status(), "Auth error on /desks, refreshing token...");
            self.do_refresh().await?;
            let resp = self.get_with_backoff(&url).await?;
            return self.parse_desks_response(resp).await;
        }
        self.parse_desks_response(resp).await
    }

    async fn parse_desks_response(&self, resp: reqwest::Response) -> Result<DesksResponse> {
        let resp = resp.error_for_status().context("/desks request failed")?;
        let body = resp.text().await.context("failed to read /desks response body")?;
        tracing::trace!(response_body = %body, "/desks response");
        serde_json::from_str(&body).context("failed to parse /desks response")
    }

    /// Fetch the IANA timezone for a location from the /locations endpoint.
    pub async fn get_location_timezone(&mut self, location_id: &str) -> Result<String> {
        let url = format!("{LOCATION_URL}/{location_id}");
        let resp = self.get_with_backoff(&url).await?;
        if matches!(resp.status(), StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            warn!(status = %resp.status(), "Got auth error on /locations, retrying after token refresh...");
            self.do_refresh().await?;
            let resp = self.get_with_backoff(&url).await?;
            return self.parse_location_response(resp).await;
        }
        self.parse_location_response(resp).await
    }

    async fn parse_location_response(&self, resp: reqwest::Response) -> Result<String> {
        let resp = resp.error_for_status().context("/locations request failed")?;
        let body = resp.text().await.context("failed to read /locations response body")?;
        tracing::trace!(response_body = %body, "/locations response");
        let loc: LocationResponse = serde_json::from_str(&body)
            .context("failed to parse /locations response")?;
        Ok(loc.data.attributes.timezone)
    }

    /// Fetch the current user's profile from the /me endpoint.
    /// Uses backoff retries and auth refresh, consistent with GraphQL methods.
    pub async fn get_me(&mut self) -> Result<UserInfo> {
        let resp = self.get_with_backoff(ME_URL).await?;
        if matches!(
            resp.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            warn!(status = %resp.status(), "Got auth error on /me, retrying after token refresh...");
            self.do_refresh().await?;
            let resp = self.get_with_backoff(ME_URL).await?;
            return self.parse_me_response(resp).await;
        }
        self.parse_me_response(resp).await
    }

    async fn parse_me_response(&self, resp: reqwest::Response) -> Result<UserInfo> {
        let resp = resp.error_for_status().context("/me request failed")?;
        let body = resp
            .text()
            .await
            .context("failed to read /me response body")?;
        tracing::trace!(response_body = %body, "/me response");
        let me: MeResponse = serde_json::from_str(&body).context("failed to parse /me response")?;
        Ok(UserInfo {
            full_name: me.data.attributes.full_name,
            email: me.data.attributes.email,
        })
    }

    pub async fn get_registration_dates(
        &mut self,
        location_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<RegistrationDate>> {
        tracing::debug!(
            location_id,
            start_date,
            end_date,
            "Fetching registration dates"
        );
        let body = json!({
            "operationName": "EmployeeRegistrationPartialDay",
            "variables": {
                "locationId": location_id,
                "startDate": start_date,
                "endDate": end_date,
            },
            "query": QUERY_EMPLOYEE_REGISTRATION,
        });

        let resp = self
            .request_with_retry(&body)
            .await?
            .error_for_status()
            .context("EmployeeRegistrationPartialDay request failed")?;

        let response_body = resp.text().await.context("failed to read response body")?;
        tracing::trace!(response_body = %response_body, "EmployeeRegistrationPartialDay response");

        let parsed: EmployeeRegistrationPartialDayData =
            parse_graphql_result(&response_body, "EmployeeRegistrationPartialDay")?;

        Ok(parsed.employee_registration_partial_day.registration_dates)
    }

    /// Attempt to sign in to an invite.
    ///
    /// Returns `Ok(SignInEntry)` on success, or `Ok`-wraps a `SignInError`
    /// for known domain errors (already signed in, not found, future invite)
    /// so callers can handle them without downcasting.
    pub async fn sign_in_invite(
        &mut self,
        invite_id: &str,
    ) -> Result<std::result::Result<SignInEntry, SignInError>> {
        tracing::debug!(invite_id, "Signing in to invite");
        let body = serde_json::json!({
            "operationName": "SignInInvite",
            "variables": { "inviteID": invite_id },
            "query": MUTATION_SIGN_IN_INVITE,
        });

        let resp = self
            .request_with_retry(&body)
            .await?
            .error_for_status()
            .context("SignInInvite request failed")?;

        let response_body = resp.text().await.context("failed to read response body")?;
        tracing::trace!(response_body = %response_body, "SignInInvite response");

        // Parse as GraphQlResult to inspect errors before treating as fatal
        let result: super::types::GraphQlResult<SignInInviteData> =
            serde_json::from_str(&response_body)
                .context("failed to parse SignInInvite response")?;

        if let Some(err) = result.errors.first() {
            return Ok(Err(SignInError::from_message(&err.message)));
        }

        match result.data {
            Some(data) => match data.sign_in_invite.into_iter().next() {
                Some(entry) => Ok(Ok(entry)),
                None => Ok(Err(SignInError::Other(
                    "signInInvite returned empty array".to_owned(),
                ))),
            },
            None => Ok(Err(SignInError::Other(
                "null data with no errors".to_owned(),
            ))),
        }
    }

    pub async fn create_invite_reservation(
        &mut self,
        location_id: &str,
        expected_arrival_time: &str,
        full_name: &str,
        email: &str,
        desk_id: Option<&str>,
    ) -> Result<InviteReservationResponse> {
        tracing::debug!(
            location_id,
            expected_arrival_time,
            desk_id,
            "Creating invite reservation"
        );
        // Build variables — omit deskId entirely when not specified rather than
        // sending null, as the API ignores null deskId but respects an absent field.
        let mut variables = serde_json::json!({
            "invite": {
                "fullName": full_name,
                "email": email,
                "location": location_id,
                "userData": [
                    { "field": "Purpose of visit", "value": "Employee registration" }
                ],
                "expectedArrivalTime": expected_arrival_time,
            }
        });
        if let Some(id) = desk_id {
            variables["deskId"] = serde_json::Value::String(id.to_owned());
        }

        let body = json!({
            "operationName": "CreateInviteReservation",
            "variables": variables,
            "query": MUTATION_CREATE_INVITE_RESERVATION,
        });

        let resp = self
            .request_with_retry(&body)
            .await?
            .error_for_status()
            .context("CreateInviteReservation request failed")?;

        let response_body = resp.text().await.context("failed to read response body")?;
        tracing::trace!(response_body = %response_body, "CreateInviteReservation response");

        let data: CreateInviteReservationData =
            parse_graphql_result(&response_body, "CreateInviteReservation")?;

        Ok(data.create_invite_reservation)
    }
}
