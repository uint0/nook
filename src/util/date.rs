use anyhow::{Context, Result};
use chrono::{Local, NaiveDate, TimeZone};
use chrono_tz::Tz;

/// ISO 8601 format used when sending timestamps to the Envoy API.
const API_DATETIME_FMT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

/// Parse a date string of the form "YYYY-MM-DD" or "latest" (= today).
pub fn parse_date(date_str: &str) -> Result<NaiveDate> {
    if date_str == "latest" {
        return Ok(Local::now().date_naive());
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

/// Returns the current local midnight as an RFC 3339 string suitable for the API.
pub fn default_start_date() -> String {
    // and_hms_opt(0,0,0) only returns None for invalid h/m/s values, which 0,0,0 never are.
    // from_local_datetime returns None only in ambiguous DST gaps, which midnight rarely hits.
    // Both are safe to unwrap_or_else to a reasonable fallback.
    Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| Local.from_local_datetime(&naive).single())
        .unwrap_or_else(|| Local::now())
        .format(API_DATETIME_FMT)
        .to_string()
}

/// Returns local 23:59 14 days from now as an RFC 3339 string.
pub fn default_end_date() -> String {
    (Local::now().date_naive() + chrono::Duration::days(14))
        .and_hms_opt(23, 59, 0)
        .and_then(|naive| Local.from_local_datetime(&naive).single())
        .unwrap_or_else(|| Local::now() + chrono::Duration::days(14))
        .format(API_DATETIME_FMT)
        .to_string()
}
