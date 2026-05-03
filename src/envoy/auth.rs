use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

const TOKEN_URL: &str = "https://app.envoy.com/a/auth/v0/token";
const CLIENT_ID: &str = "<your-envoy-client-id>";
const CONNECT_TIMEOUT_SECS: u64 = 10;
const REQUEST_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TokenResponse {
    pub token_type: String,
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub refresh_token_expires_in: u64,
}

/// Exchange a refresh token for a new access token.
pub async fn refresh(_http: &reqwest::Client, refresh_token: &str) -> Result<TokenResponse> {
    // Always build a client with explicit timeouts — the caller may have passed
    // a bare reqwest::Client::new() without timeouts configured.
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .context("failed to build HTTP client for token refresh")?;

    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", CLIENT_ID),
        ])
        .send()
        .await
        .context("failed to send token refresh request")?
        .error_for_status()
        .context("token refresh request failed")?
        .json::<TokenResponse>()
        .await
        .context("failed to parse token refresh response")?;

    Ok(resp)
}
