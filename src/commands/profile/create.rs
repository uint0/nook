use std::path::Path;

use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::Utc;

use crate::config::ENV_AUTH_KEY;
use crate::profile::decrypt::Cipher;
use crate::profile::loader::{EncryptedValue, ProfilesFile, RawAuth, RawProfile, RawToken, save};

fn prompt(label: &str) -> Result<String> {
    eprint!("{label}: ");
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .context("failed to read from stdin")?;
    Ok(buf.trim().to_owned())
}

fn prompt_secret(label: &str) -> Result<String> {
    let val = rpassword::prompt_password(format!("{label}: "))
        .context("failed to read secret from stdin")?;
    Ok(val.trim().to_owned())
}

/// Resolve the encryption cipher — if no key was provided by the caller (i.e.
/// it wasn't set in the environment), interactively offer to generate one or
/// accept a pasted key. This is the only place user input flows into a Cipher.
fn resolve_cipher(provided: Option<Cipher>) -> Result<Cipher> {
    if let Some(cipher) = provided {
        return Ok(cipher);
    }

    eprintln!("{ENV_AUTH_KEY} is not set. This key encrypts your tokens at rest.");
    eprintln!("  [g] Generate a new key and print it  (default)");
    eprintln!("  [p] Paste an existing key");
    eprint!("Choice [g/p]: ");

    let mut choice = String::new();
    std::io::stdin()
        .read_line(&mut choice)
        .context("failed to read choice")?;

    match choice.trim().to_lowercase().as_str() {
        "p" => {
            let key = prompt_secret(ENV_AUTH_KEY)?;
            let bytes = STANDARD.decode(&key).context("key is not valid base64")?;
            if bytes.len() != 32 {
                bail!(
                    "key must be exactly 32 bytes when base64-decoded (got {})",
                    bytes.len()
                );
            }
            Cipher::from_base64(&key)
        }
        _ => {
            let (cipher, b64) = Cipher::generate();
            eprintln!(
                "\nGenerated NOOK_AUTH_KEY (store this safely — without it your tokens cannot be decrypted):"
            );
            eprintln!("\n  {b64}\n");
            Ok(cipher)
        }
    }
}

pub async fn run(profile_name: &str, path: &Path, cipher: Option<Cipher>) -> Result<()> {
    tracing::debug!(profile_name, "Running profile create");
    eprintln!("Creating profile '{profile_name}'. Press Ctrl-C to abort.\n");

    let cipher = resolve_cipher(cipher)?;

    let location_id = prompt("Location ID")?;
    let access_token = prompt_secret("Access token")?;
    let refresh_token = prompt_secret("Refresh token")?;

    let new_profile = RawProfile {
        name: profile_name.to_owned(),
        location_id,
        auth: RawAuth {
            last_refreshed_at: Utc::now().to_rfc3339(),
            token: RawToken {
                token_type: "Bearer".to_owned(),
                access_token: EncryptedValue {
                    aes256: cipher.encrypt(&access_token)?,
                    expiry: String::new(),
                },
                refresh_token: EncryptedValue {
                    aes256: cipher.encrypt(&refresh_token)?,
                    expiry: String::new(),
                },
            },
        },
    };

    let mut file: ProfilesFile = if path.exists() {
        crate::profile::loader::read_profiles_file(path)?
    } else {
        ProfilesFile { profiles: vec![] }
    };

    if let Some(existing) = file.profiles.iter_mut().find(|p| p.name == profile_name) {
        eprintln!("Profile '{profile_name}' already exists — overwriting.");
        *existing = new_profile;
    } else {
        file.profiles.push(new_profile);
    }

    save(path, &file)?;
    eprintln!("\nProfile '{profile_name}' saved to {}.", path.display());
    Ok(())
}
