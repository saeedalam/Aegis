//! Get time tool - returns the current server time.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::{Tool, ToolError, ToolOutput};

/// Get time tool - returns the current server time in ISO 8601 format.
#[derive(Debug)]
pub struct GetTimeTool;

#[async_trait]
impl Tool for GetTimeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "get_time".to_string(),
            description: Some("Returns the current server time in ISO 8601 format.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let now = Utc::now();
        let iso_time = now.to_rfc3339();

        Ok(ToolOutput::text(serde_json::json!({
            "time": iso_time,
            "timestamp": now.timestamp(),
            "timezone": "UTC"
        }).to_string()))
    }
}


