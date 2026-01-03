//! Secret management tools.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool to store a secret.
#[derive(Debug)]
pub struct SecretsSetTool;

#[async_trait]
impl Tool for SecretsSetTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "secrets.set".to_string(),
            description: Some(
                "Securely stores a secret (API key, token, password). Use ${secrets.KEY} to reference it."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Secret name (e.g., OPENAI_KEY)"
                    },
                    "value": {
                        "type": "string",
                        "description": "Secret value"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional description"
                    }
                },
                "required": ["key", "value"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let key = arguments
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'key' parameter".to_string()))?;

        let value = arguments
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'value' parameter".to_string()))?;

        let description = arguments.get("description").and_then(|v| v.as_str());

        state.secrets.set(key, value, description);

        let result = json!({
            "success": true,
            "key": key,
            "message": format!("Secret '{}' stored securely. Reference with ${{secrets.{}}}", key, key)
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to get a secret.
#[derive(Debug)]
pub struct SecretsGetTool;

#[async_trait]
impl Tool for SecretsGetTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "secrets.get".to_string(),
            description: Some("Retrieves a stored secret value.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Secret name to retrieve"
                    }
                },
                "required": ["key"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let key = arguments
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'key' parameter".to_string()))?;

        match state.secrets.get(key) {
            Some(value) => {
                // Mask the value for display (show first 4 and last 4 chars)
                let masked = if value.len() > 12 {
                    format!(
                        "{}...{}",
                        &value[..4],
                        &value[value.len() - 4..]
                    )
                } else {
                    "*".repeat(value.len())
                };

                let result = json!({
                    "found": true,
                    "key": key,
                    "value": value,
                    "masked": masked
                });
                Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
            }
            None => {
                let result = json!({
                    "found": false,
                    "key": key,
                    "error": "Secret not found"
                });
                Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
            }
        }
    }
}

/// Tool to list secrets.
#[derive(Debug)]
pub struct SecretsListTool;

#[async_trait]
impl Tool for SecretsListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "secrets.list".to_string(),
            description: Some("Lists all stored secret keys (not values).".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let keys = state.secrets.list();

        let result = json!({
            "count": keys.len(),
            "keys": keys
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to delete a secret.
#[derive(Debug)]
pub struct SecretsDeleteTool;

#[async_trait]
impl Tool for SecretsDeleteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "secrets.delete".to_string(),
            description: Some("Deletes a stored secret.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Secret name to delete"
                    }
                },
                "required": ["key"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let key = arguments
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'key' parameter".to_string()))?;

        let deleted = state.secrets.delete(key);

        let result = json!({
            "success": deleted,
            "key": key,
            "message": if deleted { "Secret deleted" } else { "Secret not found" }
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

