use anyhow::{Context, Result};

/// Parse a `raw:<location_id>` string and return the location ID.
///
/// Only `raw:` prefixed strings are currently supported. Other formats
/// (e.g. location name lookup) can be added here in future.
#[allow(dead_code)]
pub fn parse_location(location: &str) -> Result<&str> {
    location
        .strip_prefix("raw:")
        .with_context(|| format!("unsupported location format '{location}' — expected 'raw:<id>'"))
}
