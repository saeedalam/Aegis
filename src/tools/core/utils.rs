//! Utility tools: base64, JSON, UUID, hash, regex.

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

// ============================================================================
// Base64 Encode Tool
// ============================================================================

#[derive(Debug)]
pub struct Base64EncodeTool;

#[async_trait]
impl Tool for Base64EncodeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "base64.encode".to_string(),
            description: Some("Encodes text to Base64.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to encode"
                    }
                },
                "required": ["text"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'text' parameter".to_string()))?;

        let encoded = BASE64.encode(text.as_bytes());
        Ok(ToolOutput::text(encoded))
    }
}

// ============================================================================
// Base64 Decode Tool
// ============================================================================

#[derive(Debug)]
pub struct Base64DecodeTool;

#[async_trait]
impl Tool for Base64DecodeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "base64.decode".to_string(),
            description: Some("Decodes Base64 to text.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "encoded": {
                        "type": "string",
                        "description": "Base64 encoded string"
                    }
                },
                "required": ["encoded"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let encoded = arguments
            .get("encoded")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'encoded' parameter".to_string()))?;

        let decoded = BASE64
            .decode(encoded)
            .map_err(|e| ToolError::ExecutionFailed(format!("Invalid Base64: {}", e)))?;

        let text = String::from_utf8(decoded)
            .map_err(|e| ToolError::ExecutionFailed(format!("Invalid UTF-8: {}", e)))?;

        Ok(ToolOutput::text(text))
    }
}

// ============================================================================
// JSON Parse Tool
// ============================================================================

#[derive(Debug)]
pub struct JsonParseTool;

#[async_trait]
impl Tool for JsonParseTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "json.parse".to_string(),
            description: Some("Parses a JSON string and returns formatted output.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "JSON string to parse"
                    }
                },
                "required": ["text"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'text' parameter".to_string()))?;

        let parsed: Value = serde_json::from_str(text)
            .map_err(|e| ToolError::ExecutionFailed(format!("Invalid JSON: {}", e)))?;

        let formatted = serde_json::to_string_pretty(&parsed)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to format: {}", e)))?;

        Ok(ToolOutput::text(formatted))
    }
}

// ============================================================================
// JSON Query Tool (simple path-based access)
// ============================================================================

#[derive(Debug)]
pub struct JsonQueryTool;

#[async_trait]
impl Tool for JsonQueryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "json.query".to_string(),
            description: Some(
                "Queries a JSON object using a dot-notation path (e.g., 'data.items[0].name')."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "json": {
                        "description": "JSON object or string to query"
                    },
                    "path": {
                        "type": "string",
                        "description": "Dot-notation path (e.g., 'data.items[0].name')"
                    }
                },
                "required": ["json", "path"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let json_input = arguments
            .get("json")
            .ok_or_else(|| ToolError::InvalidInput("Missing 'json' parameter".to_string()))?;

        // If it's a string, parse it first
        let json_value: Value = if let Some(s) = json_input.as_str() {
            serde_json::from_str(s)
                .map_err(|e| ToolError::InvalidInput(format!("Invalid JSON string: {}", e)))?
        } else {
            json_input.clone()
        };

        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'path' parameter".to_string()))?;

        // Simple path parser
        let mut current = &json_value;
        for part in path.split('.') {
            // Check for array index
            if let Some(idx_start) = part.find('[') {
                let key = &part[..idx_start];
                if !key.is_empty() {
                    current = current.get(key).ok_or_else(|| {
                        ToolError::ExecutionFailed(format!("Key not found: {}", key))
                    })?;
                }

                let idx_end = part.find(']').ok_or_else(|| {
                    ToolError::InvalidInput("Invalid array index syntax".to_string())
                })?;
                let idx: usize = part[idx_start + 1..idx_end]
                    .parse()
                    .map_err(|_| ToolError::InvalidInput("Invalid array index".to_string()))?;

                current = current.get(idx).ok_or_else(|| {
                    ToolError::ExecutionFailed(format!("Index {} out of bounds", idx))
                })?;
            } else if !part.is_empty() {
                current = current.get(part).ok_or_else(|| {
                    ToolError::ExecutionFailed(format!("Key not found: {}", part))
                })?;
            }
        }

        let result = serde_json::to_string_pretty(current)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to format: {}", e)))?;

        Ok(ToolOutput::text(result))
    }
}

// ============================================================================
// UUID Generate Tool
// ============================================================================

#[derive(Debug)]
pub struct UuidGenerateTool;

#[async_trait]
impl Tool for UuidGenerateTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "uuid.generate".to_string(),
            description: Some("Generates a new UUID v4.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "count": {
                        "type": "integer",
                        "default": 1,
                        "description": "Number of UUIDs to generate"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let count = arguments
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1)
            .min(100) as usize;

        let uuids: Vec<String> = (0..count)
            .map(|_| uuid::Uuid::new_v4().to_string())
            .collect();

        if count == 1 {
            Ok(ToolOutput::text(&uuids[0]))
        } else {
            Ok(ToolOutput::text(serde_json::to_string_pretty(&uuids).unwrap()))
        }
    }
}

// ============================================================================
// Hash Tool
// ============================================================================

#[derive(Debug)]
pub struct HashTool;

#[async_trait]
impl Tool for HashTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "hash.sha256".to_string(),
            description: Some("Computes SHA-256 hash of text.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to hash"
                    }
                },
                "required": ["text"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'text' parameter".to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let result = hasher.finalize();
        let hash = hex::encode(result);

        Ok(ToolOutput::text(hash))
    }
}

// ============================================================================
// Regex Match Tool
// ============================================================================

#[derive(Debug)]
pub struct RegexMatchTool;

#[async_trait]
impl Tool for RegexMatchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "regex.match".to_string(),
            description: Some("Tests if text matches a regex pattern and extracts groups.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to search"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern"
                    },
                    "global": {
                        "type": "boolean",
                        "default": false,
                        "description": "Find all matches"
                    }
                },
                "required": ["text", "pattern"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'text' parameter".to_string()))?;

        let pattern = arguments
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'pattern' parameter".to_string()))?;

        let global = arguments
            .get("global")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let re = Regex::new(pattern)
            .map_err(|e| ToolError::InvalidInput(format!("Invalid regex: {}", e)))?;

        if global {
            let matches: Vec<Value> = re
                .find_iter(text)
                .map(|m| {
                    json!({
                        "match": m.as_str(),
                        "start": m.start(),
                        "end": m.end()
                    })
                })
                .collect();

            let result = json!({
                "found": !matches.is_empty(),
                "count": matches.len(),
                "matches": matches
            });

            Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
        } else {
            match re.captures(text) {
                Some(caps) => {
                    let groups: Vec<Option<&str>> =
                        caps.iter().map(|m| m.map(|x| x.as_str())).collect();

                    let result = json!({
                        "found": true,
                        "match": caps.get(0).map(|m| m.as_str()),
                        "groups": groups
                    });

                    Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
                }
                None => {
                    let result = json!({
                        "found": false,
                        "match": null,
                        "groups": []
                    });

                    Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
                }
            }
        }
    }
}

// ============================================================================
// Regex Replace Tool
// ============================================================================

#[derive(Debug)]
pub struct RegexReplaceTool;

#[async_trait]
impl Tool for RegexReplaceTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "regex.replace".to_string(),
            description: Some("Replaces text matching a regex pattern.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to process"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to match"
                    },
                    "replacement": {
                        "type": "string",
                        "description": "Replacement text (use $1, $2 for groups)"
                    },
                    "global": {
                        "type": "boolean",
                        "default": true,
                        "description": "Replace all matches"
                    }
                },
                "required": ["text", "pattern", "replacement"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'text' parameter".to_string()))?;

        let pattern = arguments
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'pattern' parameter".to_string()))?;

        let replacement = arguments
            .get("replacement")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'replacement' parameter".to_string()))?;

        let global = arguments
            .get("global")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let re = Regex::new(pattern)
            .map_err(|e| ToolError::InvalidInput(format!("Invalid regex: {}", e)))?;

        let result = if global {
            re.replace_all(text, replacement).to_string()
        } else {
            re.replace(text, replacement).to_string()
        };

        Ok(ToolOutput::text(result))
    }
}

