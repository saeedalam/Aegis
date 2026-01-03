//! Handler for the `tools/call` MCP method.
//!
//! Executes a tool with the given arguments.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::core::{NexusError, NexusResult, RuntimeState};
use crate::tools::{Tool, ToolOutput, ToolContent};

/// Parameters for tools/call request.
#[derive(Debug, Deserialize)]
pub struct ToolsCallParams {
    /// The name of the tool to call.
    pub name: String,
    /// The arguments to pass to the tool.
    #[serde(default)]
    pub arguments: Value,
}

/// Result of tools/call request.
#[derive(Debug, Serialize)]
pub struct ToolsCallResult {
    /// The content returned by the tool.
    pub content: Vec<ToolContentItem>,
    /// Whether the tool execution resulted in an error.
    #[serde(rename = "isError")]
    pub is_error: bool,
}

/// Content item in tool output (MCP format).
#[derive(Debug, Serialize)]
pub struct ToolContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Handles the `tools/call` request.
///
/// Looks up the tool in the registry and executes it with the given arguments.
pub async fn handle_tools_call(
    params: Option<Value>,
    state: Arc<RuntimeState>,
) -> NexusResult<Value> {
    debug!("Handling tools/call request");

    // Parse parameters
    let call_params: ToolsCallParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| NexusError::InvalidRequest(format!("Invalid tools/call params: {}", e)))?,
        None => {
            return Err(NexusError::MissingField("params".to_string()));
        }
    };

    info!("Calling tool: {} with args: {:?}", call_params.name, call_params.arguments);

    // Get the tool from registry (clone the Arc to release the lock before await)
    let tool: Arc<dyn Tool> = {
        let registry = state.tool_registry.read();
        match registry.get(&call_params.name) {
            Some(t) => t.clone(),
            None => {
                warn!("Tool not found: {}", call_params.name);
                let output = ToolOutput::error(format!("Tool not found: {}", call_params.name));
                return format_output(output);
            }
        }
    };

    // Execute the tool (lock is released)
    let output = match tool.execute(call_params.arguments, state.clone()).await {
        Ok(output) => output,
        Err(e) => {
            warn!("Tool execution failed: {}", e);
            ToolOutput::error(e.to_string())
        }
    };

    format_output(output)
}

/// Converts tool output to MCP format.
fn format_output(output: ToolOutput) -> NexusResult<Value> {
    let content: Vec<ToolContentItem> = output.content.iter().map(|c| {
        match c {
            ToolContent::Text { text } => ToolContentItem {
                content_type: "text".to_string(),
                text: text.clone(),
            },
            ToolContent::Image { data, mime_type } => ToolContentItem {
                content_type: "text".to_string(),
                text: format!("data:{};base64,{}", mime_type, data),
            },
        }
    }).collect();

    let result = ToolsCallResult {
        content,
        is_error: output.is_error,
    };

    serde_json::to_value(result)
        .map_err(|e| NexusError::Internal(format!("Failed to serialize: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Config;

    #[tokio::test]
    async fn test_tools_call_echo() {
        let state = Arc::new(RuntimeState::new(Config::default()));

        let params = serde_json::json!({
            "name": "echo",
            "arguments": {
                "text": "Hello, Nexus!"
            }
        });

        let result = handle_tools_call(Some(params), state).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("isError").unwrap(), false);
        
        let content = value.get("content").unwrap().as_array().unwrap();
        assert!(!content.is_empty());
        assert!(content[0].get("text").unwrap().as_str().unwrap().contains("Hello, Nexus!"));
    }

    #[tokio::test]
    async fn test_tools_call_get_time() {
        let state = Arc::new(RuntimeState::new(Config::default()));

        let params = serde_json::json!({
            "name": "get_time",
            "arguments": {}
        });

        let result = handle_tools_call(Some(params), state).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("isError").unwrap(), false);
    }

    #[tokio::test]
    async fn test_tools_call_unknown() {
        let state = Arc::new(RuntimeState::new(Config::default()));

        let params = serde_json::json!({
            "name": "nonexistent_tool",
            "arguments": {}
        });

        let result = handle_tools_call(Some(params), state).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        // Should return error in output, not fail the request
        assert_eq!(value.get("isError").unwrap(), true);
    }
}
