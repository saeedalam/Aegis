//! Memory tools for storing and recalling data.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::{Tool, ToolError, ToolOutput};

// ============================================================================
// Memory Store Tool
// ============================================================================

/// Tool for storing key-value data in memory.
#[derive(Debug)]
pub struct MemoryStoreTool;

#[derive(Deserialize)]
struct MemoryStoreArgs {
    key: String,
    value: Value,
    #[serde(default)]
    ttl_secs: Option<u64>,
}

#[async_trait]
impl Tool for MemoryStoreTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "memory.store".to_string(),
            description: Some("Stores a value in the key-value memory store.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The key to store the value under"
                    },
                    "value": {
                        "description": "The value to store (any JSON value)"
                    },
                    "ttl_secs": {
                        "type": "integer",
                        "description": "Optional time-to-live in seconds"
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
        let args: MemoryStoreArgs = serde_json::from_value(arguments)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        debug!("Storing key: {}", args.key);

        state.memory_store
            .kv_set(&args.key, args.value.clone(), args.ttl_secs)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        Ok(ToolOutput::text(serde_json::json!({
            "success": true,
            "key": args.key,
            "stored": true
        }).to_string()))
    }
}

// ============================================================================
// Memory Recall Tool
// ============================================================================

/// Tool for recalling data from memory.
#[derive(Debug)]
pub struct MemoryRecallTool;

#[derive(Deserialize)]
struct MemoryRecallArgs {
    key: String,
}

#[async_trait]
impl Tool for MemoryRecallTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "memory.recall".to_string(),
            description: Some("Recalls a value from the key-value memory store.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The key to recall"
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
        let args: MemoryRecallArgs = serde_json::from_value(arguments)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        debug!("Recalling key: {}", args.key);

        let result = state.memory_store
            .kv_get(&args.key)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        match result {
            Some(kv) => Ok(ToolOutput::text(serde_json::json!({
                "found": true,
                "key": args.key,
                "value": kv.value,
                "created_at": kv.created_at,
                "updated_at": kv.updated_at
            }).to_string())),
            None => Ok(ToolOutput::text(serde_json::json!({
                "found": false,
                "key": args.key
            }).to_string())),
        }
    }
}

// ============================================================================
// Memory Delete Tool
// ============================================================================

/// Tool for deleting data from memory.
#[derive(Debug)]
pub struct MemoryDeleteTool;

#[derive(Deserialize)]
struct MemoryDeleteArgs {
    key: String,
}

#[async_trait]
impl Tool for MemoryDeleteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "memory.delete".to_string(),
            description: Some("Deletes a key from the memory store.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "The key to delete"
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
        let args: MemoryDeleteArgs = serde_json::from_value(arguments)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        debug!("Deleting key: {}", args.key);

        state.memory_store
            .kv_delete(&args.key)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        Ok(ToolOutput::text(serde_json::json!({
            "success": true,
            "key": args.key,
            "deleted": true
        }).to_string()))
    }
}

// ============================================================================
// Memory List Tool
// ============================================================================

/// Tool for listing keys in memory.
#[derive(Debug)]
pub struct MemoryListTool;

#[derive(Deserialize)]
struct MemoryListArgs {
    #[serde(default)]
    prefix: Option<String>,
}

#[async_trait]
impl Tool for MemoryListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "memory.list".to_string(),
            description: Some("Lists all keys in the memory store, optionally filtered by prefix.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "prefix": {
                        "type": "string",
                        "description": "Optional prefix to filter keys"
                    }
                },
                "required": []
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let args: MemoryListArgs = serde_json::from_value(arguments)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        debug!("Listing keys with prefix: {:?}", args.prefix);

        let keys = state.memory_store
            .kv_list(args.prefix.as_deref())
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        Ok(ToolOutput::text(serde_json::json!({
            "keys": keys,
            "count": keys.len()
        }).to_string()))
    }
}

