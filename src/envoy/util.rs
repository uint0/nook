use anyhow::{Context, Result};

use super::types::GraphQlResult;

/// Parse a raw GraphQL response body for operations where partial success is
/// not expected (i.e. either `data` is present or `errors` is non-empty — not both).
///
/// Returns `Err` if GraphQL errors are present, or if `data` is null.
pub fn parse_graphql_result<T: serde::de::DeserializeOwned>(
    response_body: &str,
    operation: &str,
) -> Result<T> {
    let result: GraphQlResult<T> = serde_json::from_str(response_body)
        .with_context(|| format!("failed to parse GraphQL response for {operation}"))?;

    if !result.errors.is_empty() {
        let messages: Vec<&str> = result.errors.iter().map(|e| e.message.as_str()).collect();
        anyhow::bail!("{operation} failed: {}", messages.join("; "));
    }

    result
        .data
        .with_context(|| format!("{operation} returned null data with no errors"))
}
