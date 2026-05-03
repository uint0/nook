mod cli;
mod commands;
mod config;
mod envoy;
mod logging;
mod profile;
mod util;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use cli::{Commands, booking::BookingCommands, profile::ProfileCommands};
use profile::decrypt::Cipher;

fn profiles_path() -> PathBuf {
    if let Ok(path) = std::env::var(config::ENV_PROFILES_FILE) {
        return PathBuf::from(path);
    }
    profile::loader::default_profiles_path()
}

fn load_cipher() -> Result<Cipher> {
    let key = std::env::var(config::ENV_AUTH_KEY).map_err(|_| {
        anyhow::anyhow!(
            "{} is not set — run `nook profile create` to set up a profile",
            config::ENV_AUTH_KEY
        )
    })?;
    Cipher::from_base64(key.trim())
}

#[tokio::main]
async fn main() {
    let (log_path, _guard) = match logging::init() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: failed to initialise logging: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = run().await {
        eprintln!("error: {e:#}");
        eprintln!("(log file: {})", log_path.display());
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = cli::Cli::parse();
    let profile_name = &cli.profile;

    match cli.command {
        Commands::Profile { command } => match command {
            ProfileCommands::Create => {
                let path = profiles_path();
                let cipher = std::env::var(config::ENV_AUTH_KEY)
                    .ok()
                    .and_then(|k| Cipher::from_base64(k.trim()).ok());
                commands::profile::create::run(profile_name, &path, cipher).await?;
            }
            ProfileCommands::Refresh => {
                let cipher = load_cipher()?;
                let path = profiles_path();
                commands::profile::refresh::run(profile_name, &cipher, &path).await?;
            }
            ProfileCommands::Info => {
                let cipher = load_cipher()?;
                let path = profiles_path();
                commands::profile::info::run(profile_name, &cipher, &path).await?;
            }
        },
        Commands::Booking { command } => match command {
            BookingCommands::Create {
                backfill,
                date,
                desk,
            } => {
                let cipher = load_cipher()?;
                let path = profiles_path();
                let loaded = profile::load(profile_name, &cipher, &path).await?;
                commands::booking::create::run(loaded, backfill, &date, desk.as_deref()).await?;
            }
            BookingCommands::CheckIn { date } => {
                let cipher = load_cipher()?;
                let path = profiles_path();
                let loaded = profile::load(profile_name, &cipher, &path).await?;
                commands::booking::check_in::run(loaded, &date).await?;
            }
            BookingCommands::Show {
                start_date,
                end_date,
            } => {
                let cipher = load_cipher()?;
                let path = profiles_path();
                let loaded = profile::load(profile_name, &cipher, &path).await?;
                commands::booking::show::run(loaded, start_date, end_date).await?;
            }
        },
    }

    Ok(())
}
