use std::path::Path;

use anyhow::{Context, Result};

use crate::envoy::client::EnvoyClient;
use crate::profile::decrypt::Cipher;
use crate::profile::loader::read_profiles_file;
use crate::profile::token_store::TokenStore;
use crate::util::date::{DateFormat, format_date};

fn format_expiry(expiry: &str) -> String {
    if expiry.is_empty() {
        "unknown (not yet refreshed)".to_owned()
    } else {
        format_date(expiry, DateFormat::DateTimeWithDow)
    }
}

pub async fn run(profile_name: &str, cipher: &Cipher, path: &Path) -> Result<()> {
    tracing::debug!(profile_name, "Running profile info");
    let file = read_profiles_file(path)?;

    let raw = file
        .profiles
        .iter()
        .find(|p| p.name == profile_name)
        .with_context(|| format!("profile '{profile_name}' not found"))?;

    let access_expiry = format_expiry(&raw.auth.token.access_token.expiry);
    let refresh_expiry = format_expiry(&raw.auth.token.refresh_token.expiry);
    let last_refreshed = if raw.auth.last_refreshed_at.is_empty() {
        "never".to_owned()
    } else {
        format_date(&raw.auth.last_refreshed_at, DateFormat::DateTimeWithDow)
    };

    let access_token = cipher
        .decrypt(&raw.auth.token.access_token.aes256)
        .context("failed to decrypt access token")?;
    let refresh_token = cipher
        .decrypt(&raw.auth.token.refresh_token.aes256)
        .context("failed to decrypt refresh token")?;

    let store = TokenStore::new(profile_name, access_token, refresh_token, cipher, path);
    let mut client = EnvoyClient::new(store)?;
    let me = client.get_me().await?;
    let timezone = client.get_location_timezone(&raw.location_id).await?;

    println!("Profile: {profile_name}");
    println!("Location ID: {}", raw.location_id);
    println!("Timezone: {timezone}");
    println!("Profiles file: {}", path.display());
    println!();
    println!("User");
    println!("  full name : {}", me.full_name);
    println!("  email     : {}", me.email);
    println!();
    println!("Tokens");
    println!("  last refreshed       : {last_refreshed}");
    println!("  access token expiry  : {access_expiry}");
    println!("  refresh token expiry : {refresh_expiry}");

    Ok(())
}
