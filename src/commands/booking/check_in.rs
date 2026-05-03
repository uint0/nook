use anyhow::{Context, Result, bail};
use chrono::TimeZone;
use tracing::info;

use crate::envoy::client::EnvoyClient;
use crate::envoy::types::{RegistrationDate, ScreeningCard, SignInError};
use crate::profile::Profile;
use crate::util::date::{
    DateFormat, default_end_date, default_start_date, format_date, parse_date,
};
use crate::util::spinner;

/// Whether the user specified an explicit date or requested "latest".
enum DateMode {
    Latest,
    Explicit(chrono::NaiveDate),
}

pub async fn run(profile: Profile, date: &str) -> Result<()> {
    let timezone = profile.timezone.clone();
    let date_mode = if date == "latest" {
        DateMode::Latest
    } else {
        DateMode::Explicit(parse_date(date, &timezone)?)
    };

    tracing::debug!(location_id = %profile.location_id, date, "Running booking check-in");
    let location_id = profile.location_id.clone();
    let mut client = EnvoyClient::new(profile.token_store)?;

    match date_mode {
        DateMode::Explicit(target_date) => {
            run_explicit(&mut client, &location_id, &timezone, target_date).await
        }
        DateMode::Latest => run_latest(&mut client, &location_id, &timezone).await,
    }
}

/// Check in for a specific date. Errors on all failure cases.
async fn run_explicit(
    client: &mut EnvoyClient,
    location_id: &str,
    timezone: &str,
    target_date: chrono::NaiveDate,
) -> Result<()> {
    // Use the profile timezone for start/end to correctly bracket the day
    let tz = timezone.parse::<chrono_tz::Tz>().unwrap_or(chrono_tz::UTC);
    let start = target_date
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| tz.from_local_datetime(&naive).single())
        .with_context(|| format!("failed to compute start time for {target_date}"))?
        .to_rfc3339();
    let end = target_date
        .and_hms_opt(23, 59, 59)
        .and_then(|naive| tz.from_local_datetime(&naive).single())
        .with_context(|| format!("failed to compute end time for {target_date}"))?
        .to_rfc3339();

    let sp = spinner::start(format!("Fetching invite for {target_date}..."));
    let dates = client
        .get_registration_dates(location_id, &start, &end)
        .await?;
    sp.finish_and_clear();

    let invite_id = find_invite_id(&dates).ok_or_else(|| {
        anyhow::anyhow!(
            "No booking found for {target_date} — book first with `nook booking create`"
        )
    })?;

    match do_check_in(client, &invite_id).await? {
        Ok(()) => Ok(()),
        Err(SignInError::FutureInvite) => {
            bail!("Cannot check in yet — this booking is in the future")
        }
        Err(e) => bail!("{}", sign_in_error_message(&invite_id, e)),
    }
}

/// Check in in temporal order, stopping at the first future invite.
async fn run_latest(client: &mut EnvoyClient, location_id: &str, timezone: &str) -> Result<()> {
    let start = default_start_date(timezone);
    let end = default_end_date(timezone);

    let sp = spinner::start("Fetching bookings...");
    let dates = client
        .get_registration_dates(location_id, &start, &end)
        .await?;
    sp.finish_and_clear();

    let invites: Vec<(String, String)> = dates
        .iter()
        .filter_map(|d| {
            if let Some(ScreeningCard::Invite(inv)) = &d.screening_card {
                Some((d.date.clone(), inv.id.clone()))
            } else {
                None
            }
        })
        .collect();

    if invites.is_empty() {
        println!("No bookings found in the next 14 days — book first with `nook booking create`");
        return Ok(());
    }

    let mut checked_in_any = false;
    for (date_str, invite_id) in &invites {
        let date_display = format_date(date_str, DateFormat::DateWithDow);
        info!(invite_id, "Attempting check-in for {date_display}");

        let sp = spinner::start(format!("Checking in for {date_display}..."));
        match do_check_in(client, invite_id).await? {
            Ok(()) => {
                sp.finish_and_clear();
                checked_in_any = true;
            }
            Err(SignInError::FutureInvite) => {
                sp.finish_and_clear();
                if !checked_in_any {
                    println!(
                        "No bookings available to check in to yet \
                         (earliest booking is in the future)."
                    );
                }
                return Ok(());
            }
            Err(e) => {
                sp.finish_and_clear();
                bail!("{}", sign_in_error_message(invite_id, e));
            }
        }
    }

    Ok(())
}

/// Attempt a single check-in, returning `Ok(Ok(()))` on success,
/// `Ok(Err(SignInError))` for known domain errors, or `Err` for I/O failures.
/// `AlreadySignedIn` is handled here as an idempotent success.
async fn do_check_in(
    client: &mut EnvoyClient,
    invite_id: &str,
) -> Result<std::result::Result<(), SignInError>> {
    match client.sign_in_invite(invite_id).await? {
        Ok(entry) => {
            let signed_in_at = format_date(&entry.signed_in_at, DateFormat::DateTimeWithDow);
            println!(
                "✓ Checked in — signed in at {signed_in_at} (entry id: {})",
                entry.id
            );
            Ok(Ok(()))
        }
        Err(SignInError::AlreadySignedIn) => {
            println!("Already checked in for this booking.");
            Ok(Ok(()))
        }
        Err(e) => Ok(Err(e)),
    }
}

/// Format a user-facing message for a `SignInError`.
fn sign_in_error_message(invite_id: &str, err: SignInError) -> String {
    match err {
        SignInError::InviteNotFound => format!(
            "Invite {invite_id} not found — it may have been deleted or the profile is wrong"
        ),
        SignInError::FutureInvite => {
            "Cannot check in yet — this booking is in the future".to_owned()
        }
        SignInError::AlreadySignedIn => "Already checked in for this booking.".to_owned(),
        SignInError::Other(msg) => format!("Check-in failed: {msg}"),
    }
}

/// Find the invite ID from a registration date list.
fn find_invite_id(dates: &[RegistrationDate]) -> Option<String> {
    dates.iter().find_map(|d| {
        if let Some(ScreeningCard::Invite(inv)) = &d.screening_card {
            Some(inv.id.clone())
        } else {
            None
        }
    })
}
