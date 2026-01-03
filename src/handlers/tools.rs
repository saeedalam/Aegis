//! Handler for the `tools/list` MCP method.
//!
//! Returns the list of available tools that the server provides.

use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

use crate::core::{NexusResult, RuntimeState};
use crate::protocol::mcp::ToolsListResult;

/// Handles the `tools/list` request.
///
/// Returns a list of available tools from the tool registry.
pub async fn handle_tools_list(
    _params: Option<Value>,
    state: Arc<RuntimeState>,
) -> NexusResult<Value> {
    debug!("Handling tools/list request");

    // Get tools from the registry
    let registry = state.tool_registry.read();
    let tools = registry.list_definitions();

    let result = ToolsListResult { tools };

    debug!("Returning {} tools", result.tools.len());

    serde_json::to_value(result)
        .map_err(|e| crate::core::NexusError::Internal(format!("Failed to serialize: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Config;

    #[tokio::test]
    async fn test_tools_list() {
        let state = Arc::new(RuntimeState::new(Config::default()));
        let result = handle_tools_list(None, state).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        let tools = value.get("tools").unwrap().as_array().unwrap();
        
        // Should have built-in tools
        assert!(!tools.is_empty());

        // Check echo tool exists
        let echo_tool = tools.iter().find(|t| t.get("name").unwrap() == "echo");
        assert!(echo_tool.is_some());

        // Check get_time tool exists
        let time_tool = tools.iter().find(|t| t.get("name").unwrap() == "get_time");
        assert!(time_tool.is_some());
    }
}
