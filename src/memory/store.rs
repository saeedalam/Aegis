//! Memory store trait and data types.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for memory operations.
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// A conversation (session) in the memory store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique identifier for the conversation.
    pub id: String,
    /// Human-readable title.
    pub title: Option<String>,
    /// When the conversation was created.
    pub created_at: String,
    /// When the conversation was last updated.
    pub updated_at: String,
    /// Optional metadata as JSON.
    pub metadata: Option<String>,
}

/// A message within a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for the message.
    pub id: String,
    /// The conversation this message belongs to.
    pub conversation_id: String,
    /// Role of the sender (user, assistant, system, tool).
    pub role: String,
    /// The message content.
    pub content: String,
    /// When the message was created.
    pub created_at: String,
    /// Optional metadata as JSON (e.g., tool call info).
    pub metadata: Option<String>,
}

/// A key-value pair for simple storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    /// The key.
    pub key: String,
    /// The value (stored as JSON).
    pub value: serde_json::Value,
    /// When the entry was created.
    pub created_at: String,
    /// When the entry was last updated.
    pub updated_at: String,
    /// Optional TTL (expiration time).
    pub expires_at: Option<String>,
}

/// Trait for memory storage backends.
#[async_trait]
pub trait MemoryStore: Send + Sync + std::fmt::Debug {
    // Conversation operations
    
    /// Creates a new conversation.
    async fn create_conversation(&self, name: Option<String>, metadata: Option<String>) -> Result<String, MemoryError>;
    
    /// Gets a conversation by ID.
    async fn get_conversation(&self, id: &str) -> Result<Conversation, MemoryError>;
    
    /// Lists all conversations (most recent first).
    async fn list_conversations(&self, limit: usize) -> Result<Vec<Conversation>, MemoryError>;
    
    /// Deletes a conversation and all its messages.
    async fn delete_conversation(&self, id: &str) -> Result<(), MemoryError>;

    // Message operations
    
    /// Adds a message to a conversation.
    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: Option<String>,
    ) -> Result<String, MemoryError>;
    
    /// Gets messages for a conversation.
    async fn get_messages(&self, conversation_id: &str, limit: usize) -> Result<Vec<Message>, MemoryError>;
    
    /// Gets the last N messages across all conversations.
    async fn get_recent_messages(&self, limit: usize) -> Result<Vec<Message>, MemoryError>;

    /// Searches messages by content.
    async fn search_messages(&self, query: &str, limit: usize) -> Result<Vec<Message>, MemoryError>;

    // Key-Value operations
    
    /// Sets a key-value pair.
    async fn kv_set(&self, key: &str, value: serde_json::Value, ttl_secs: Option<u64>) -> Result<(), MemoryError>;
    
    /// Gets a value by key.
    async fn kv_get(&self, key: &str) -> Result<Option<KeyValue>, MemoryError>;
    
    /// Deletes a key.
    async fn kv_delete(&self, key: &str) -> Result<(), MemoryError>;
    
    /// Lists all keys (optionally with prefix filter).
    async fn kv_list(&self, prefix: Option<&str>) -> Result<Vec<String>, MemoryError>;
}

