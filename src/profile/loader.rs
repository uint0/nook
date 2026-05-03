use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::decrypt::Cipher;
use super::token_store::TokenStore;

// ── Default profiles file path ────────────────────────────────────────────────

/// Local profiles file name — checked in the current working directory first.
const LOCAL_PROFILES_FILE: &str = "nook.yml";

/// How many seconds before expiry we proactively refresh the access token.
const TOKEN_EXPIRY_BUFFER_SECS: i64 = 60;

/// Resolve the profiles file path. Called once at the application edge (main.rs).
///
/// Resolution order:
/// 1. `./nook.yml` in the current working directory (if it exists)
/// 2. `$XDG_CONFIG_HOME/nook/nook.yml` (or `~/.config/nook/nook.yml`)
///
/// Note: creation always goes to the XDG path if no local file exists,
/// so the local file must be manually created to be used.
pub fn default_profiles_path() -> PathBuf {
    let local = PathBuf::from(LOCAL_PROFILES_FILE);
    if local.exists() {
        return local;
    }
    xdg_profiles_path()
}

/// Returns the XDG config path for the profiles file.
pub fn xdg_profiles_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        });
    base.join("nook").join("nook.yml")
}

// ── YAML types ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfilesFile {
    pub profiles: Vec<RawProfile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawProfile {
    pub name: String,
    pub location_id: String,
    pub auth: RawAuth,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawAuth {
    pub last_refreshed_at: String,
    pub token: RawToken,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawToken {
    pub token_type: String,
    pub access_token: EncryptedValue,
    pub refresh_token: EncryptedValue,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EncryptedValue {
    pub aes256: String,
    /// RFC 3339 expiry timestamp. Empty string means expiry is unknown
    /// (e.g. on first create before any refresh). An empty expiry is treated
    /// as expired, triggering an immediate refresh on next load.
    pub expiry: String,
}

// ── Decrypted profile ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct Profile {
    #[allow(dead_code)] // used by future commands (booking create, check-in)
    pub name: String,
    pub location_id: String,
    pub token_store: TokenStore,
}

impl Profile {
    #[allow(dead_code)] // used by future commands (booking create, check-in)
    pub fn access_token(&self) -> &str {
        &self.token_store.access_token
    }
}

// ── File I/O helpers ──────────────────────────────────────────────────────────

/// Read and parse the profiles file from disk.
pub fn read_profiles_file(path: &Path) -> Result<ProfilesFile> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read profiles file: {}", path.display()))?;
    serde_yaml::from_str(&contents).context("failed to parse profiles file")
}

/// Write the in-memory ProfilesFile back to disk.
pub fn save(path: &Path, file: &ProfilesFile) -> Result<()> {
    let contents = serde_yaml::to_string(file).context("failed to serialise profiles file")?;
    std::fs::write(path, contents)
        .with_context(|| format!("failed to write profiles file: {}", path.display()))
}

// ── Encrypt helper ────────────────────────────────────────────────────────────

pub fn encrypt_value(cipher: &Cipher, plaintext: &str, expiry: &str) -> Result<EncryptedValue> {
    Ok(EncryptedValue {
        aes256: cipher.encrypt(plaintext)?,
        expiry: expiry.to_owned(),
    })
}

// ── Loader ────────────────────────────────────────────────────────────────────

/// Load and decrypt the named profile, refreshing the access token if expired.
pub async fn load(profile_name: &str, cipher: &Cipher, path: &Path) -> Result<Profile> {
    let mut file = read_profiles_file(path)?;

    let idx = file
        .profiles
        .iter()
        .position(|p| p.name == profile_name)
        .with_context(|| format!("profile '{profile_name}' not found in {}", path.display()))?;

    let raw = &file.profiles[idx];

    if raw.auth.token.access_token.aes256.is_empty() {
        bail!(
            "access_token for profile '{profile_name}' is empty — have you run `nook profile create`?"
        );
    }

    let refresh_token = cipher
        .decrypt(&raw.auth.token.refresh_token.aes256)
        .with_context(|| format!("failed to decrypt refresh_token for '{profile_name}'"))?;

    let needs_refresh = is_expired(&raw.auth.token.access_token.expiry);

    let (access_token, refresh_token) = if needs_refresh {
        info!("Access token expired, refreshing...");
        let http = reqwest::Client::new();
        let resp = crate::envoy::auth::refresh(&http, &refresh_token)
            .await
            .context("token refresh failed")?;

        let new_access_expiry =
            (Utc::now() + Duration::seconds(resp.expires_in as i64)).to_rfc3339();
        let new_refresh_expiry =
            (Utc::now() + Duration::seconds(resp.refresh_token_expires_in as i64)).to_rfc3339();

        let raw_mut = &mut file.profiles[idx];
        raw_mut.auth.token.access_token =
            encrypt_value(cipher, &resp.access_token, &new_access_expiry)?;
        raw_mut.auth.token.refresh_token =
            encrypt_value(cipher, &resp.refresh_token, &new_refresh_expiry)?;
        raw_mut.auth.last_refreshed_at = Utc::now().to_rfc3339();
        save(path, &file).context("failed to save refreshed tokens")?;

        (resp.access_token, resp.refresh_token)
    } else {
        let access_token = cipher
            .decrypt(&file.profiles[idx].auth.token.access_token.aes256)
            .with_context(|| format!("failed to decrypt access_token for '{profile_name}'"))?;
        (access_token, refresh_token)
    };

    let raw = &file.profiles[idx];
    Ok(Profile {
        name: raw.name.clone(),
        location_id: raw.location_id.clone(),
        token_store: TokenStore::new(&raw.name, access_token, refresh_token, cipher, path),
    })
}

/// Returns true if the expiry string is empty or within the proactive buffer window.
fn is_expired(expiry_str: &str) -> bool {
    if expiry_str.is_empty() {
        return true;
    }
    match expiry_str.parse::<DateTime<Utc>>() {
        Ok(expiry) => Utc::now() + Duration::seconds(TOKEN_EXPIRY_BUFFER_SECS) >= expiry,
        Err(_) => true, // treat unparseable expiry as expired
    }
}
