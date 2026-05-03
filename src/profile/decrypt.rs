use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecryptError {
    #[error("key must be exactly 32 bytes when base64-decoded")]
    InvalidKeyLength,
    #[error("ciphertext is not valid base64: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
    #[error("AES-256-GCM decryption failed")]
    DecryptionFailed,
}

/// A loaded AES-256-GCM key, ready to encrypt and decrypt.
///
/// Construct via [`Cipher::from_base64`] with a key resolved at the application
/// edge (e.g. from an environment variable read in `main.rs`).
#[derive(Clone, Debug)]
pub struct Cipher {
    key_bytes: [u8; 32],
}

impl Cipher {
    /// Load the key from a base64-encoded string.
    pub fn from_base64(b64: &str) -> Result<Self> {
        let bytes = STANDARD.decode(b64).context("key is not valid base64")?;
        if bytes.len() != 32 {
            bail!(DecryptError::InvalidKeyLength);
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);
        Ok(Self { key_bytes })
    }

    /// Generate a fresh random 32-byte key, returning both the [`Cipher`] and
    /// its base64 representation (for display to the user).
    pub fn generate() -> (Self, String) {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let b64 = STANDARD.encode(key_bytes);
        (Self { key_bytes }, b64)
    }

    /// Encrypt `plaintext` with AES-256-GCM.
    ///
    /// Output is base64-encoded `[ 12-byte nonce | ciphertext+tag ]`.
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key_bytes);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| DecryptError::DecryptionFailed)?;

        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);
        Ok(STANDARD.encode(&combined))
    }

    /// Decrypt a base64-encoded AES-256-GCM ciphertext.
    ///
    /// Expects the format `[ 12-byte nonce | ciphertext+tag ]`.
    pub fn decrypt(&self, ciphertext_b64: &str) -> Result<String> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key_bytes);
        let cipher = Aes256Gcm::new(key);

        let data = STANDARD.decode(ciphertext_b64.trim())?;
        if data.len() < 12 {
            bail!("ciphertext too short to contain a nonce");
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| DecryptError::DecryptionFailed)?;

        Ok(String::from_utf8(plaintext).context("decrypted token is not valid UTF-8")?)
    }
}
