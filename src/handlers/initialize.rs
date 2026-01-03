//! Handler for the `initialize` MCP method.
//!
//! The initialize method is the first message sent by a client to negotiate
//! capabilities and establish the connection.

use serde_json::Value;
use std::sync::Arc;
use tracing::{info, debug};

use crate::core::{NexusError, NexusResult, RuntimeState};
use crate::protocol::mcp::{
    InitializeParams, InitializeResult, ServerCapabilities,
    ToolsCapability, PromptsCapability, MCP_VERSION,
};

/// Handles the `initialize` request.
///
/// This is the first request from a client. It:
/// 1. Parses client capabilities
/// 2. Returns server capabilities
/// 3. Marks the server as initialized
pub async fn handle_initialize(
    params: Option<Value>,
    state: Arc<RuntimeState>,
) -> NexusResult<Value> {
    debug!("Handling initialize request");

    // Parse initialize parameters
    let init_params: InitializeParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| NexusError::InvalidRequest(format!("Invalid initialize params: {}", e)))?,
        None => {
            return Err(NexusError::MissingField("params".to_string()));
        }
    };

    info!(
        "Client connecting: {} v{} (protocol: {})",
        init_params.client_info.name,
        init_params.client_info.version,
        init_params.protocol_version
    );

    // Mark as initialized
    state.set_initialized();

    // Build server capabilities
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: false }),
        prompts: Some(PromptsCapability { list_changed: false }),
        resources: None, // Phase 3
    };

    // Build the response
    let result = InitializeResult {
        protocol_version: MCP_VERSION.to_string(),
        capabilities,
        server_info: state.server_info.clone(),
    };

    info!(
        "Server initialized: {} v{} (protocol: {})",
        result.server_info.name,
        result.server_info.version,
        result.protocol_version
    );

    serde_json::to_value(result)
        .map_err(|e| NexusError::Internal(format!("Failed to serialize result: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Config;

    #[tokio::test]
    async fn test_initialize() {
        let state = Arc::new(RuntimeState::new(Config::default()));

        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        });

        let result = handle_initialize(Some(params), state.clone()).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("protocolVersion").is_some());
        assert!(value.get("serverInfo").is_some());
        assert!(value.get("capabilities").is_some());
    }
}


