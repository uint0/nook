use anyhow::{Context, Result};
use chrono::{Local, NaiveDate, TimeZone};
use chrono_tz::Tz;

/// ISO 8601 format used when sending timestamps to the Envoy API.
const API_DATETIME_FMT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

/// Parse a date string of the form "YYYY-MM-DD" or "latest" (= today in the given timezone).
///
/// Uses the provided IANA timezone so "latest" means today in the office timezone,
/// not the system timezone. This ensures CI (which runs in UTC) books the correct date.
pub fn parse_date(date_str: &str, timezone: &str) -> Result<NaiveDate> {
    if date_str == "latest" {
        let today = if let Ok(tz) = timezone.parse::<Tz>() {
            chrono::Utc::now().with_timezone(&tz).date_naive()
        } else {
            Local::now().date_naive()
        };
        return Ok(today);
    }
    NaiveDate::parse_from_str(date_str, DateFormat::DateOnly.fmt_str())
        .with_context(|| format!("invalid date '{date_str}' — expected YYYY-MM-DD or 'latest'"))
}

/// Format a NaiveDate as the expected arrival time for the Envoy API (UTC).
///
/// Uses the provided IANA timezone string (e.g. `"Australia/Sydney"`) to
/// convert local midnight to UTC, correctly handling DST. Falls back to the
/// system local timezone if the timezone string is empty or unrecognised.
pub fn arrival_time_for_date(date: NaiveDate, timezone: &str) -> Result<String> {
    let midnight = date
        .and_hms_opt(0, 0, 0)
        .with_context(|| format!("failed to construct midnight for {date}"))?;

    let utc_midnight = if let Ok(tz) = timezone.parse::<Tz>() {
        tz.from_local_datetime(&midnight)
            .single()
            .with_context(|| format!("ambiguous local time for {date} in {timezone}"))?
            .with_timezone(&chrono::Utc)
    } else {
        tracing::warn!(
            timezone,
            "Unrecognised IANA timezone, falling back to system local time"
        );
        Local
            .from_local_datetime(&midnight)
            .single()
            .with_context(|| format!("failed to compute arrival time for {date}"))?
            .with_timezone(&chrono::Utc)
    };

    Ok(utc_midnight.format(API_DATETIME_FMT).to_string())
}

/// Date/time format strings used throughout the application.
pub enum DateFormat {
    /// "2026-05-04" — used for parsing CLI date arguments
    DateOnly,
    /// "2026-05-04 (Mon)"
    DateWithDow,
    /// "2026-05-04 (Mon) 14:00"
    DateTimeWithDow,
}

impl DateFormat {
    fn fmt_str(&self) -> &'static str {
        match self {
            DateFormat::DateOnly => "%Y-%m-%d",
            DateFormat::DateWithDow => "%Y-%m-%d (%a)",
            DateFormat::DateTimeWithDow => "%Y-%m-%d (%a) %H:%M",
        }
    }
}

/// Format an ISO 8601 timestamp from the API into local time using the given format.
/// Falls back to the raw string if parsing fails.
pub fn format_date(date_str: &str, format: DateFormat) -> String {
    chrono::DateTime::parse_from_rfc3339(date_str)
        .map(|dt| {
            dt.with_timezone(&Local)
                .format(format.fmt_str())
                .to_string()
        })
        .unwrap_or_else(|_| date_str.to_owned())
}

/// Returns midnight today in the given timezone as a UTC API timestamp.
///
/// Uses the profile timezone so CI (UTC) and local (Sydney) produce the same window.
pub fn default_start_date(timezone: &str) -> String {
    let today = tz_today(timezone);
    arrival_time_for_date(today, timezone)
        .unwrap_or_else(|_| chrono::Utc::now().format(API_DATETIME_FMT).to_string())
}

/// Returns 23:59 fourteen days from today in the given timezone as a UTC API timestamp.
pub fn default_end_date(timezone: &str) -> String {
    let end_day = tz_today(timezone) + chrono::Duration::days(14);
    let end_midnight = end_day
        .and_hms_opt(23, 59, 0)
        .map(|naive| {
            if let Ok(tz) = timezone.parse::<Tz>() {
                tz.from_local_datetime(&naive)
                    .single()
                    .map(|dt| {
                        dt.with_timezone(&chrono::Utc)
                            .format(API_DATETIME_FMT)
                            .to_string()
                    })
                    .unwrap_or_else(|| chrono::Utc::now().format(API_DATETIME_FMT).to_string())
            } else {
                Local
                    .from_local_datetime(&naive)
                    .single()
                    .map(|dt| {
                        dt.with_timezone(&chrono::Utc)
                            .format(API_DATETIME_FMT)
                            .to_string()
                    })
                    .unwrap_or_else(|| chrono::Utc::now().format(API_DATETIME_FMT).to_string())
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().format(API_DATETIME_FMT).to_string());
    end_midnight
}

/// Returns today's date in the given IANA timezone (falls back to system local).
fn tz_today(timezone: &str) -> NaiveDate {
    if let Ok(tz) = timezone.parse::<Tz>() {
        chrono::Utc::now().with_timezone(&tz).date_naive()
    } else {
        Local::now().date_naive()
    }
}
