use serde::Deserialize;

// ── Top-level response wrappers ───────────────────────────────────────────────

/// A GraphQL response that may contain errors (e.g. mutations that return
/// `data: null` with an `errors` array on failure).
#[derive(Debug, Deserialize)]
pub struct GraphQlResult<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<GraphQlError>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQlError {
    pub message: String,
}

// ── /desks endpoint ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct DesksResponse {
    pub data: Vec<DeskEntry>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct DeskEntry {
    pub id: String,
    pub attributes: DeskEntryAttributes,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct DeskEntryAttributes {
    pub name: String,
}

// ── /locations endpoint ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LocationResponse {
    pub data: LocationData,
}

#[derive(Debug, Deserialize)]
pub struct LocationData {
    pub attributes: LocationAttributes,
}

#[derive(Debug, Deserialize)]
pub struct LocationAttributes {
    pub timezone: String,
}

// ── /me endpoint ─────────────────────────────────────────────────────────────

/// Minimal user info from the /me endpoint — what we need to create bookings.
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub full_name: String,
    pub email: String,
}

/// JSON:API response wrapper for /me
#[derive(Debug, Deserialize)]
pub struct MeResponse {
    pub data: MeData,
}

#[derive(Debug, Deserialize)]
pub struct MeData {
    pub attributes: MeAttributes,
}

#[derive(Debug, Deserialize)]
pub struct MeAttributes {
    #[serde(rename = "full-name")]
    pub full_name: String,
    pub email: String,
}

// ── SignInInvite ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInInviteData {
    pub sign_in_invite: Vec<SignInEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInEntry {
    pub id: String,
    pub signed_in_at: String,
}

/// Errors we recognise from the SignInInvite mutation.
#[derive(Debug, PartialEq)]
pub enum SignInError {
    AlreadySignedIn,
    InviteNotFound,
    FutureInvite,
    Other(String),
}

impl SignInError {
    pub fn from_message(msg: &str) -> Self {
        if msg.contains("Resource already exists") || msg.contains("ConflictError") {
            SignInError::AlreadySignedIn
        } else if msg.contains("Invite not found") {
            SignInError::InviteNotFound
        } else if msg.contains("Cannot check in for a future invite") {
            SignInError::FutureInvite
        } else {
            SignInError::Other(msg.to_owned())
        }
    }
}

// ── CreateInviteReservation ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteReservationData {
    pub create_invite_reservation: InviteReservationResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteReservationResponse {
    pub invite: CreatedInvite,
    pub reservation: Option<CreatedReservation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedInvite {
    pub id: String,
    pub expected_arrival_time: String,
    pub location: Option<CreatedInviteLocation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedInviteLocation {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedReservation {
    pub desk: Option<CreatedDesk>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedDesk {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeRegistrationPartialDayData {
    pub employee_registration_partial_day: EmployeeRegistrationPartialDay,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeRegistrationPartialDay {
    pub registration_dates: Vec<RegistrationDate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationDate {
    pub date: String,
    pub people_registered: u32,
    pub reservations: Vec<Reservation>,
    pub screening_card: Option<ScreeningCard>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reservation {
    pub desk: Option<Desk>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Desk {
    pub name: String,
    pub floor: Floor,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Floor {
    pub name: String,
}

/// The screeningCard field is a union of Invite | SelfCertify.
/// We tag-dispatch on the `__typename` field.
#[derive(Debug, Deserialize)]
#[serde(tag = "__typename")]
pub enum ScreeningCard {
    Invite(Invite),
    SelfCertify(SelfCertify),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invite {
    pub id: String,
    pub location: Option<InviteLocation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteLocation {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SelfCertify {}
