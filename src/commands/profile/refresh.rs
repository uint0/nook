use std::path::Path;

use anyhow::{Context, Result};
use tracing::info;

use crate::profile::decrypt::Cipher;
use crate::profile::loader::read_profiles_file;
use crate::profile::token_store::TokenStore;

pub async fn run(profile_name: &str, cipher: &Cipher, path: &Path) -> Result<()> {
    let file = read_profiles_file(path)?;

    let raw = file
        .profiles
        .iter()
        .find(|p| p.name == profile_name)
        .with_context(|| format!("profile '{profile_name}' not found"))?;

    if raw.auth.token.refresh_token.aes256.is_empty() {
        anyhow::bail!(
            "refresh_token for profile '{profile_name}' is empty — have you run `nook profile create`?"
        );
    }

    let access_token = cipher
        .decrypt(&raw.auth.token.access_token.aes256)
        .with_context(|| format!("failed to decrypt access_token for '{profile_name}'"))?;
    let refresh_token = cipher
        .decrypt(&raw.auth.token.refresh_token.aes256)
        .with_context(|| format!("failed to decrypt refresh_token for '{profile_name}'"))?;

    tracing::debug!(profile_name, "Running profile refresh");
    info!("Refreshing token for profile '{profile_name}'...");
    let http = reqwest::Client::new();
    let resp = crate::envoy::auth::refresh(&http, &refresh_token)
        .await
        .context("token refresh failed")?;

    let expiry = resp.expires_in;
    let refresh_expiry = resp.refresh_token_expires_in;

    // Delegate encrypt + persist to TokenStore — single source of truth
    let mut store = TokenStore::new(profile_name, access_token, &refresh_token, cipher, path);
    store.update(
        &resp.access_token,
        expiry,
        &resp.refresh_token,
        refresh_expiry,
    )?;

    info!(
        "Token refreshed successfully for profile '{profile_name}'. \
         Access token valid for {expiry}s."
    );
    Ok(())
}
