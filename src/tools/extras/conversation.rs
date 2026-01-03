//! Conversation history tools for multi-turn agent memory.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool to create a new conversation.
#[derive(Debug)]
pub struct ConversationCreateTool;

#[async_trait]
impl Tool for ConversationCreateTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "conversation.create".to_string(),
            description: Some("Creates a new conversation thread.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Conversation title"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Optional metadata"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let title = arguments
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let metadata = arguments
            .get("metadata")
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        let id = state
            .memory_store
            .create_conversation(title.clone(), metadata)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let result = json!({
            "success": true,
            "conversation_id": id,
            "title": title
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to add a message to a conversation.
#[derive(Debug)]
pub struct ConversationAddTool;

#[async_trait]
impl Tool for ConversationAddTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "conversation.add".to_string(),
            description: Some("Adds a message to a conversation.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "conversation_id": {
                        "type": "string",
                        "description": "Conversation ID"
                    },
                    "role": {
                        "type": "string",
                        "enum": ["user", "assistant", "system"],
                        "description": "Message role"
                    },
                    "content": {
                        "type": "string",
                        "description": "Message content"
                    }
                },
                "required": ["conversation_id", "role", "content"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let conversation_id = arguments
            .get("conversation_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'conversation_id'".to_string()))?;

        let role = arguments
            .get("role")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'role'".to_string()))?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'content'".to_string()))?;

        let message_id = state
            .memory_store
            .add_message(conversation_id, role, content, None)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let result = json!({
            "success": true,
            "message_id": message_id,
            "conversation_id": conversation_id
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to get conversation history.
#[derive(Debug)]
pub struct ConversationGetTool;

#[async_trait]
impl Tool for ConversationGetTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "conversation.get".to_string(),
            description: Some("Gets messages from a conversation.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "conversation_id": {
                        "type": "string",
                        "description": "Conversation ID"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max messages to return (default: 50)"
                    }
                },
                "required": ["conversation_id"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let conversation_id = arguments
            .get("conversation_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'conversation_id'".to_string()))?;

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;

        let messages = state
            .memory_store
            .get_messages(conversation_id, limit)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let messages_json: Vec<Value> = messages
            .iter()
            .map(|m| {
                json!({
                    "id": m.id,
                    "role": m.role,
                    "content": m.content,
                    "created_at": m.created_at
                })
            })
            .collect();

        let result = json!({
            "conversation_id": conversation_id,
            "count": messages.len(),
            "messages": messages_json
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to list conversations.
#[derive(Debug)]
pub struct ConversationListTool;

#[async_trait]
impl Tool for ConversationListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "conversation.list".to_string(),
            description: Some("Lists all conversations.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Max conversations to return (default: 20)"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;

        let conversations = state
            .memory_store
            .list_conversations(limit)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let conv_json: Vec<Value> = conversations
            .iter()
            .map(|c| {
                json!({
                    "id": c.id,
                    "title": c.title,
                    "created_at": c.created_at,
                    "updated_at": c.updated_at
                })
            })
            .collect();

        let result = json!({
            "count": conversations.len(),
            "conversations": conv_json
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to search conversations.
#[derive(Debug)]
pub struct ConversationSearchTool;

#[async_trait]
impl Tool for ConversationSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "conversation.search".to_string(),
            description: Some("Searches messages across all conversations.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results (default: 20)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'query'".to_string()))?;

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;

        let results = state
            .memory_store
            .search_messages(query, limit)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let results_json: Vec<Value> = results
            .iter()
            .map(|m| {
                json!({
                    "message_id": m.id,
                    "conversation_id": m.conversation_id,
                    "role": m.role,
                    "content": m.content,
                    "created_at": m.created_at
                })
            })
            .collect();

        let result = json!({
            "query": query,
            "count": results.len(),
            "results": results_json
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

