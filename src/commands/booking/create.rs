use anyhow::{Context, Result};
use chrono::Local;
use comfy_table::{Attribute, Cell, Color, Table, presets::UTF8_FULL};
use tracing::info;

use crate::envoy::client::EnvoyClient;
use crate::envoy::types::{InviteReservationResponse, RegistrationDate, ScreeningCard, UserInfo};
use crate::profile::Profile;
use crate::util::date::{
    DateFormat, arrival_time_for_date, default_end_date, default_start_date, format_date,
    parse_date,
};
use crate::util::location::parse_location;
use crate::util::spinner;

/// GraphQL error message returned when a date is outside the bookable window.
const SCHEDULING_LIMIT_ERROR: &str = "Scheduling Limit Check";

// ── Table output ──────────────────────────────────────────────────────────────

fn make_table() -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Date", "Desk", "Invite ID", "Status"]);
    table
}

/// Format a desk display string from a location name and optional desk name.
fn format_desk(location_name: &str, desk_name: Option<&str>, location_id: &str) -> String {
    match desk_name {
        Some(desk) => format!("{location_name} / {desk}"),
        None => format!("{location_name} (raw:{location_id})"),
    }
}

/// Extract desk display string from a reservation response.
fn desk_display(result: &InviteReservationResponse, location_id: &str) -> String {
    let location_name = result
        .invite
        .location
        .as_ref()
        .map(|l| l.name.as_str())
        .unwrap_or(location_id);
    let desk_name = result
        .reservation
        .as_ref()
        .and_then(|r| r.desk.as_ref())
        .map(|d| d.name.as_str());
    format_desk(location_name, desk_name, location_id)
}

/// Extract desk display string from an existing registration date.
fn desk_display_from_reg(reg: &RegistrationDate, location_id: &str) -> String {
    let location_name = match &reg.screening_card {
        Some(ScreeningCard::Invite(inv)) => inv
            .location
            .as_ref()
            .map(|l| l.name.as_str())
            .unwrap_or(location_id),
        _ => location_id,
    };
    let desk_name = reg
        .reservations
        .first()
        .and_then(|r| r.desk.as_ref())
        .map(|d| d.name.as_str());
    format_desk(location_name, desk_name, location_id)
}

fn add_new_row(table: &mut Table, date: &str, desk: &str, invite_id: &str) {
    table.add_row(vec![
        Cell::new(date),
        Cell::new(desk),
        Cell::new(invite_id),
        Cell::new("New").fg(Color::Green),
    ]);
}

fn add_wrong_desk_row(
    table: &mut Table,
    date: &str,
    desk: &str,
    invite_id: &str,
    requested_desk_id: &str,
) {
    table.add_row(vec![
        Cell::new(date),
        Cell::new(desk),
        Cell::new(invite_id),
        Cell::new(format!("New (wanted raw:{requested_desk_id})")).fg(Color::Yellow),
    ]);
}

/// Add a booking row, checking whether the booked desk matches the requested one.
fn add_booking_row(
    table: &mut Table,
    date: &str,
    desk: &str,
    invite_id: &str,
    result: &InviteReservationResponse,
    requested_desk_id: Option<&str>,
) {
    let got_wrong_desk = requested_desk_id.is_some_and(|requested_id| {
        // Only flag as wrong if we got a desk back AND it doesn't match.
        // If no desk was assigned in the response, we can't verify — don't flag it.
        result
            .reservation
            .as_ref()
            .and_then(|r| r.desk.as_ref())
            .is_some_and(|d| d.id.as_str() != requested_id)
    });

    if got_wrong_desk {
        tracing::warn!(
            invite_id,
            requested_desk_id,
            actual_desk = desk,
            "Booking landed on wrong desk"
        );
        add_wrong_desk_row(table, date, desk, invite_id, requested_desk_id.unwrap());
    } else {
        add_new_row(table, date, desk, invite_id);
    }
}

fn add_existing_row(table: &mut Table, date: &str, desk: &str, invite_id: &str) {
    table.add_row(vec![
        Cell::new(date),
        Cell::new(desk),
        Cell::new(invite_id),
        Cell::new("Existing").add_attribute(Attribute::Dim),
    ]);
}

/// Parse a `raw:<desk_id>` string and return the desk ID.
fn parse_desk(desk: &str) -> Result<&str> {
    parse_location(desk)
        .with_context(|| format!("invalid --desk format '{desk}' — expected 'raw:<id>'"))
}

// ── Command entry points ──────────────────────────────────────────────────────

pub async fn run(profile: Profile, backfill: bool, date: &str, desk: Option<&str>) -> Result<()> {
    let location_id = profile.location_id.clone();
    let timezone = profile.timezone.clone();
    // Parse desk_id upfront so we fail fast on bad input before any API calls
    let desk_id = desk.map(parse_desk).transpose()?;
    tracing::debug!(
        location_id,
        timezone,
        backfill,
        desk_id,
        "Running booking create"
    );
    let mut client = EnvoyClient::new(profile.token_store)?;

    // Fetch user info once upfront — used for all booking requests
    let sp = spinner::start("Loading user info...");
    let user = client.get_me().await?;
    sp.finish_and_clear();

    if backfill {
        run_backfill(&mut client, &location_id, &timezone, desk_id, &user).await
    } else {
        run_single(&mut client, &location_id, &timezone, desk_id, date, &user).await
    }
}

async fn run_single(
    client: &mut EnvoyClient,
    location_id: &str,
    timezone: &str,
    desk_id: Option<&str>,
    date: &str,
    user: &UserInfo,
) -> Result<()> {
    let target_date = parse_date(date, timezone)?;
    let arrival_time = arrival_time_for_date(target_date, timezone)?;

    let sp = spinner::start("Creating booking...");
    let result = client
        .create_invite_reservation(
            location_id,
            &arrival_time,
            &user.full_name,
            &user.email,
            desk_id,
        )
        .await?;
    sp.finish_and_clear();

    let date_display = format_date(
        &result.invite.expected_arrival_time,
        DateFormat::DateWithDow,
    );
    let desk = desk_display(&result, location_id);

    let mut table = make_table();
    add_booking_row(
        &mut table,
        &date_display,
        &desk,
        &result.invite.id,
        &result,
        desk_id,
    );
    println!("{table}");

    Ok(())
}

async fn run_backfill(
    client: &mut EnvoyClient,
    location_id: &str,
    timezone: &str,
    desk_id: Option<&str>,
    user: &UserInfo,
) -> Result<()> {
    let start_date = default_start_date(timezone);
    let end_date = default_end_date(timezone);

    // Note: backfill only covers the default 14-day window. Envoy previously allowed booking
    // further ahead, but this no longer appears to work — possibly a server-side policy change.
    let sp = spinner::start("Fetching registration dates...");
    let dates = client
        .get_registration_dates(location_id, &start_date, &end_date)
        .await?;
    sp.finish_and_clear();

    let mut table = make_table();

    for reg in &dates {
        let date_display = format_date(&reg.date, DateFormat::DateWithDow);

        match &reg.screening_card {
            Some(ScreeningCard::Invite(inv)) => {
                // Already booked — show as existing
                let desk = desk_display_from_reg(reg, location_id);
                add_existing_row(&mut table, &date_display, &desk, &inv.id);
            }
            _ => {
                // Not booked — attempt to create
                // Convert the API's UTC timestamp to a date in the profile timezone.
                // Using Local here would give the wrong date on UTC CI systems.
                let target_date = chrono::DateTime::parse_from_rfc3339(&reg.date)
                    .map(|dt| {
                        if let Ok(tz) = timezone.parse::<chrono_tz::Tz>() {
                            dt.with_timezone(&tz).date_naive()
                        } else {
                            dt.with_timezone(&Local).date_naive()
                        }
                    })
                    .with_context(|| format!("failed to parse date '{}'", reg.date))?;

                let arrival_time = arrival_time_for_date(target_date, timezone)?;
                let sp = spinner::start(format!("Booking {}...", target_date));
                match client
                    .create_invite_reservation(
                        location_id,
                        &arrival_time,
                        &user.full_name,
                        &user.email,
                        desk_id,
                    )
                    .await
                {
                    Ok(result) => {
                        sp.finish_and_clear();
                        let desk = desk_display(&result, location_id);
                        add_booking_row(
                            &mut table,
                            &date_display,
                            &desk,
                            &result.invite.id,
                            &result,
                            desk_id,
                        );
                    }
                    Err(e) => {
                        sp.finish_and_clear();
                        let msg = e.to_string();
                        if msg.contains(SCHEDULING_LIMIT_ERROR) {
                            info!("Reached scheduling limit at {target_date}, stopping backfill.");
                            // Remaining dates are outside the bookable window — omit them
                            break;
                        }
                        return Err(e);
                    }
                }
            }
        }
    }

    println!("{table}");
    Ok(())
}
