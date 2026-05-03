pub mod desks;

use std::path::PathBuf;

use crate::config::ENV_XDG_CACHE_HOME;

/// Returns the nook cache directory: `$XDG_CACHE_HOME/nook/cache/`
pub fn cache_dir() -> PathBuf {
    let base = std::env::var(ENV_XDG_CACHE_HOME)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache")
        });
    base.join("nook").join("cache")
}
