//! Handlers for MCP resource methods.
//!
//! Resources expose the memory store as readable MCP resources.

use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

use crate::core::{NexusError, NexusResult, RuntimeState};
use crate::protocol::mcp::{
    Resource, ResourcesListResult, ResourcesReadParams, ResourcesReadResult, ResourceContent,
};

/// Handles the `resources/list` request.
///
/// Returns a list of available resources including:
/// - conversations://list - List of conversations
/// - conversations://{id} - Individual conversation with messages
/// - kv://list - List of key-value keys
/// - kv://{key} - Individual key-value pair
pub async fn handle_resources_list(
    _params: Option<Value>,
    state: Arc<RuntimeState>,
) -> NexusResult<Value> {
    debug!("Handling resources/list request");

    let mut resources = Vec::new();

    // Add static resource templates
    resources.push(Resource {
        uri: "nexus://conversations".to_string(),
        name: "Conversations".to_string(),
        description: Some("List of all conversations".to_string()),
        mime_type: Some("application/json".to_string()),
    });

    resources.push(Resource {
        uri: "nexus://messages/recent".to_string(),
        name: "Recent Messages".to_string(),
        description: Some("Most recent messages across all conversations".to_string()),
        mime_type: Some("application/json".to_string()),
    });

    resources.push(Resource {
        uri: "nexus://kv".to_string(),
        name: "Key-Value Store".to_string(),
        description: Some("List of all keys in the key-value store".to_string()),
        mime_type: Some("application/json".to_string()),
    });

    // Add individual conversation resources
    let conversations = state.memory_store.list_conversations(100).await
        .map_err(|e| NexusError::Internal(e.to_string()))?;

    for conv in conversations {
        resources.push(Resource {
            uri: format!("nexus://conversations/{}", conv.id),
            name: conv.title.unwrap_or_else(|| format!("Conversation {}", &conv.id[..8])),
            description: Some(format!("Created: {}", conv.created_at)),
            mime_type: Some("application/json".to_string()),
        });
    }

    // Add individual KV resources
    let keys = state.memory_store.kv_list(None).await
        .map_err(|e| NexusError::Internal(e.to_string()))?;

    for key in keys {
        resources.push(Resource {
            uri: format!("nexus://kv/{}", key),
            name: key.clone(),
            description: Some("Key-value entry".to_string()),
            mime_type: Some("application/json".to_string()),
        });
    }

    let result = ResourcesListResult { resources };

    debug!("Returning {} resources", result.resources.len());

    serde_json::to_value(result)
        .map_err(|e| NexusError::Internal(format!("Failed to serialize: {}", e)))
}

/// Handles the `resources/read` request.
///
/// Reads a resource by URI and returns its content.
pub async fn handle_resources_read(
    params: Option<Value>,
    state: Arc<RuntimeState>,
) -> NexusResult<Value> {
    debug!("Handling resources/read request");

    let read_params: ResourcesReadParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| NexusError::InvalidRequest(format!("Invalid params: {}", e)))?,
        None => return Err(NexusError::MissingField("uri".to_string())),
    };

    debug!("Reading resource: {}", read_params.uri);

    let content = read_resource(&read_params.uri, state).await?;

    let result = ResourcesReadResult {
        contents: vec![content],
    };

    serde_json::to_value(result)
        .map_err(|e| NexusError::Internal(format!("Failed to serialize: {}", e)))
}

/// Reads a resource by URI.
async fn read_resource(uri: &str, state: Arc<RuntimeState>) -> NexusResult<ResourceContent> {
    // Parse URI
    if !uri.starts_with("nexus://") {
        return Err(NexusError::InvalidRequest(format!("Invalid URI scheme: {}", uri)));
    }

    let path = &uri[8..]; // Remove "nexus://"

    if path == "conversations" {
        // List all conversations
        let conversations = state.memory_store.list_conversations(100).await
            .map_err(|e| NexusError::Internal(e.to_string()))?;

        let json = serde_json::to_string_pretty(&conversations)
            .map_err(|e| NexusError::Internal(e.to_string()))?;

        Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(json),
            blob: None,
        })
    } else if path.starts_with("conversations/") {
        // Get specific conversation with messages
        let conv_id = &path[14..]; // Remove "conversations/"
        
        let conversation = state.memory_store.get_conversation(conv_id).await
            .map_err(|e| NexusError::Internal(e.to_string()))?;
        
        let messages = state.memory_store.get_messages(conv_id, 1000).await
            .map_err(|e| NexusError::Internal(e.to_string()))?;

        let result = serde_json::json!({
            "conversation": conversation,
            "messages": messages
        });

        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| NexusError::Internal(e.to_string()))?;

        Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(json),
            blob: None,
        })
    } else if path == "messages/recent" {
        // Get recent messages
        let messages = state.memory_store.get_recent_messages(50).await
            .map_err(|e| NexusError::Internal(e.to_string()))?;

        let json = serde_json::to_string_pretty(&messages)
            .map_err(|e| NexusError::Internal(e.to_string()))?;

        Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(json),
            blob: None,
        })
    } else if path == "kv" {
        // List all keys
        let keys = state.memory_store.kv_list(None).await
            .map_err(|e| NexusError::Internal(e.to_string()))?;

        let json = serde_json::to_string_pretty(&keys)
            .map_err(|e| NexusError::Internal(e.to_string()))?;

        Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(json),
            blob: None,
        })
    } else if path.starts_with("kv/") {
        // Get specific key
        let key = &path[3..]; // Remove "kv/"
        
        let kv = state.memory_store.kv_get(key).await
            .map_err(|e| NexusError::Internal(e.to_string()))?;

        match kv {
            Some(entry) => {
                let json = serde_json::to_string_pretty(&entry)
                    .map_err(|e| NexusError::Internal(e.to_string()))?;

                Ok(ResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(json),
                    blob: None,
                })
            }
            None => Err(NexusError::InvalidRequest(format!("Key not found: {}", key))),
        }
    } else {
        Err(NexusError::InvalidRequest(format!("Unknown resource path: {}", path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Config;

    #[tokio::test]
    async fn test_resources_list() {
        let state = Arc::new(RuntimeState::new(Config::default()));
        let result = handle_resources_list(None, state).await;
        
        assert!(result.is_ok());
        let value = result.unwrap();
        let resources = value.get("resources").unwrap().as_array().unwrap();
        
        // Should have at least the static resources
        assert!(resources.len() >= 3);
    }

    #[tokio::test]
    async fn test_resources_read_conversations() {
        let state = Arc::new(RuntimeState::new(Config::default()));
        
        let params = serde_json::json!({
            "uri": "nexus://conversations"
        });
        
        let result = handle_resources_read(Some(params), state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resources_read_kv() {
        let state = Arc::new(RuntimeState::new(Config::default()));
        
        // Store a key first
        state.memory_store.kv_set("test_key", serde_json::json!("test_value"), None).await.unwrap();
        
        let params = serde_json::json!({
            "uri": "nexus://kv/test_key"
        });
        
        let result = handle_resources_read(Some(params), state).await;
        assert!(result.is_ok());
    }
}

