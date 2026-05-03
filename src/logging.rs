use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{ENV_LOG_FILE, ENV_LOG_LEVEL, ENV_RUST_LOG, ENV_XDG_CACHE_HOME};

/// Resolve the log file path:
/// 1. `NOOK_LOG_FILE` env var
/// 2. `$XDG_CACHE_HOME/nook/<timestamp>.log`
/// 3. `~/.cache/nook/<timestamp>.log`
fn resolve_log_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var(ENV_LOG_FILE) {
        return Ok(PathBuf::from(path));
    }

    let cache_dir = std::env::var(ENV_XDG_CACHE_HOME)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache")
        });

    let log_dir = cache_dir.join("nook");
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("failed to create log directory: {}", log_dir.display()))?;

    let timestamp = chrono::Local::now().format("%Y%m%dT%H%M%S");
    Ok(log_dir.join(format!("{timestamp}.log")))
}

/// Initialise tracing with two layers:
///
/// - **stderr**: human-readable, level controlled by `RUST_LOG` (default: warn)
/// - **log file**: structured JSON at DEBUG level; response bodies at TRACE only
///
/// Returns the log file path and a [`WorkerGuard`] that must be held for the
/// duration of the program to ensure buffered logs are flushed on exit.
pub fn init() -> Result<(PathBuf, WorkerGuard)> {
    let log_path = resolve_log_path()?;

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("failed to open log file: {}", log_path.display()))?;

    // Update the latest.log symlink to point to this run's log file.
    // Remove the old symlink first (ignore errors if it doesn't exist).
    let latest_link = log_path.parent().unwrap().join("latest.log");
    let _ = std::fs::remove_file(&latest_link);
    std::os::unix::fs::symlink(&log_path, &latest_link).with_context(|| {
        format!(
            "failed to create latest.log symlink: {}",
            latest_link.display()
        )
    })?;

    let (non_blocking, guard) = tracing_appender::non_blocking(log_file);

    // File layer: structured JSON, level from NOOK_LOG_LEVEL (default: debug).
    // Set NOOK_LOG_LEVEL=trace to include raw API response bodies.
    // Uses EnvFilter (not LevelFilter) so the registry's global max-level hint
    // is not incorrectly capped by the stderr layer's more restrictive filter.
    let file_level_str = std::env::var(ENV_LOG_LEVEL).unwrap_or_else(|_| "debug".to_owned());
    let file_filter = EnvFilter::new(&file_level_str);

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_filter(file_filter);

    // Stderr layer: human-readable, level from RUST_LOG (default: warn)
    let stderr_filter =
        EnvFilter::try_from_env(ENV_RUST_LOG).unwrap_or_else(|_| EnvFilter::new("warn"));
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(stderr_filter);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .init();

    Ok((log_path, guard))
}
