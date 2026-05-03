use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Duration, Utc};

use super::decrypt::Cipher;
use super::loader::{EncryptedValue, encrypt_value, read_profiles_file, save};

/// Owns the responsibility of persisting refreshed tokens for a single profile.
///
/// Call [`TokenStore::update`] after a successful token refresh to atomically
/// encrypt and write the new tokens back to the profiles file.
#[derive(Debug)]
pub struct TokenStore {
    pub profile_name: String,
    pub access_token: String,
    pub refresh_token: String,
    cipher: Cipher,
    profiles_path: PathBuf,
}

impl TokenStore {
    pub fn new(
        profile_name: impl Into<String>,
        access_token: impl Into<String>,
        refresh_token: impl Into<String>,
        cipher: &Cipher,
        profiles_path: &Path,
    ) -> Self {
        Self {
            profile_name: profile_name.into(),
            access_token: access_token.into(),
            refresh_token: refresh_token.into(),
            cipher: cipher.clone(),
            profiles_path: profiles_path.to_owned(),
        }
    }

    /// Encrypt and persist new access/refresh tokens to the profiles file,
    /// updating the in-memory state only after a successful write.
    pub fn update(
        &mut self,
        access_token: &str,
        expires_in: u64,
        refresh_token: &str,
        refresh_token_expires_in: u64,
    ) -> Result<()> {
        let new_access_expiry = (Utc::now() + Duration::seconds(expires_in as i64)).to_rfc3339();
        let new_refresh_expiry =
            (Utc::now() + Duration::seconds(refresh_token_expires_in as i64)).to_rfc3339();

        let encrypted_access = encrypt_value(&self.cipher, access_token, &new_access_expiry)?;
        let encrypted_refresh = encrypt_value(&self.cipher, refresh_token, &new_refresh_expiry)?;

        // Persist first — only update in-memory state if write succeeds
        self.persist(encrypted_access, encrypted_refresh)?;
        self.access_token = access_token.to_owned();
        self.refresh_token = refresh_token.to_owned();
        Ok(())
    }

    fn persist(&self, access: EncryptedValue, refresh: EncryptedValue) -> Result<()> {
        let mut file = read_profiles_file(&self.profiles_path)?;

        let raw = file
            .profiles
            .iter_mut()
            .find(|p| p.name == self.profile_name)
            .with_context(|| format!("profile '{}' not found", self.profile_name))?;

        raw.auth.token.access_token = access;
        raw.auth.token.refresh_token = refresh;
        raw.auth.last_refreshed_at = Utc::now().to_rfc3339();

        save(&self.profiles_path, &file)
    }
}
