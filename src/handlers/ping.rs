//! Handler for the `ping` MCP method.
//!
//! A simple health check that returns an empty response.

use serde_json::Value;
use tracing::debug;

use crate::core::NexusResult;

/// Handles the `ping` request.
///
/// Simply returns an empty object to indicate the server is alive.
pub async fn handle_ping() -> NexusResult<Value> {
    debug!("Handling ping request");
    Ok(serde_json::json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let result = handle_ping().await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_object());
    }
}


