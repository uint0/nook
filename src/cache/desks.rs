use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::envoy::client::EnvoyClient;
use crate::envoy::types::DesksResponse;

/// Cache file path for a given location's desk list.
fn cache_path(location_id: &str) -> PathBuf {
    super::cache_dir().join(format!("desks-{location_id}.json"))
}

/// Load the desk name→id map from cache, fetching from the API if not present.
///
/// The cache is stored at `$XDG_CACHE_HOME/nook/cache/desks-<location-id>.json`.
/// It is invalidated manually (by deleting the file) or will be re-fetched on
/// the next run if the file is absent.
pub async fn lookup_desk_id(
    client: &mut EnvoyClient,
    location_id: &str,
    desk_name: &str,
) -> Result<String> {
    let path = cache_path(location_id);

    if path.exists() {
        // Try the cache first
        let map = load_or_fetch(client, location_id).await?;
        if let Some(id) = map.get(desk_name) {
            return Ok(id.clone());
        }

        // Cache miss — could be stale. Invalidate and re-fetch once.
        tracing::info!(
            desk_name,
            location_id,
            path = %path.display(),
            "Desk not found in cache, invalidating and re-fetching"
        );
        let _ = std::fs::remove_file(&path);
    }

    // Fetch fresh (either cache was absent or just invalidated)
    let map = load_or_fetch(client, location_id).await?;
    map.get(desk_name).cloned().with_context(|| {
        format!(
            "desk '{desk_name}' not found for location {location_id}.\n\
             Check the desk name with `nook booking show` or the Envoy UI.\n\
             Available desks were just fetched fresh from the API."
        )
    })
}

/// Build a name→id map, using the cache file if present.
async fn load_or_fetch(
    client: &mut EnvoyClient,
    location_id: &str,
) -> Result<HashMap<String, String>> {
    let path = cache_path(location_id);

    if path.exists() {
        tracing::debug!(path = %path.display(), "Loading desk cache from file");
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read desk cache: {}", path.display()))?;
        let response: DesksResponse = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse desk cache: {}", path.display()))?;
        return Ok(build_map(response));
    }

    tracing::info!(location_id, "Desk cache not found, fetching from API");
    let response = client.get_desks(location_id).await?;

    // Persist to cache
    std::fs::create_dir_all(path.parent().unwrap()).context("failed to create cache directory")?;
    let json = serde_json::to_string_pretty(&response).context("failed to serialise desk cache")?;
    std::fs::write(&path, &json)
        .with_context(|| format!("failed to write desk cache: {}", path.display()))?;
    tracing::debug!(path = %path.display(), "Desk cache written");

    Ok(build_map(response))
}

fn build_map(response: DesksResponse) -> HashMap<String, String> {
    response
        .data
        .into_iter()
        .map(|d| (d.attributes.name, d.id))
        .collect()
}
