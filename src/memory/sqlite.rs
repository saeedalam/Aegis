//! SQLite implementation of the MemoryStore trait.

use async_trait::async_trait;
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::memory::schema::initialize_schema;
use crate::memory::store::{Conversation, KeyValue, MemoryError, MemoryStore, Message};

/// SQLite-based memory store.
#[derive(Debug)]
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    /// Creates a new SQLite store with the given database path.
    /// Use ":memory:" for an in-memory database.
    pub fn new(path: &str) -> Result<Self, MemoryError> {
        info!("Opening SQLite database: {}", path);

        let conn = if path == ":memory:" {
            Connection::open_in_memory()
        } else {
            // Create parent directories if needed
            if let Some(parent) = Path::new(path).parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    MemoryError::Database(format!("Failed to create database directory: {}", e))
                })?;
            }
            Connection::open(path)
        }
        .map_err(|e| MemoryError::Database(e.to_string()))?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        // Initialize schema
        initialize_schema(&conn).map_err(|e| MemoryError::Database(e.to_string()))?;

        info!("SQLite database initialized successfully");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Creates an in-memory SQLite store.
    pub fn in_memory() -> Result<Self, MemoryError> {
        Self::new(":memory:")
    }
}

#[async_trait]
impl MemoryStore for SqliteStore {
    async fn create_conversation(
        &self,
        title: Option<String>,
        metadata: Option<String>,
    ) -> Result<String, MemoryError> {
        let id = Uuid::new_v4().to_string();
        let now_str = Utc::now().to_rfc3339();

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO conversations (id, name, created_at, updated_at, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&id, &title, &now_str, &now_str, &metadata),
        )
        .map_err(|e| MemoryError::Database(e.to_string()))?;

        debug!("Created conversation: {}", id);

        Ok(id)
    }

    async fn get_conversation(&self, id: &str) -> Result<Conversation, MemoryError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, created_at, updated_at, metadata FROM conversations WHERE id = ?1",
            )
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        let conversation = stmt
            .query_row([id], |row| {
                Ok(Conversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    metadata: row.get(4)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    MemoryError::NotFound(format!("Conversation not found: {}", id))
                }
                _ => MemoryError::Database(e.to_string()),
            })?;

        Ok(conversation)
    }

    async fn list_conversations(&self, limit: usize) -> Result<Vec<Conversation>, MemoryError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, created_at, updated_at, metadata FROM conversations ORDER BY updated_at DESC LIMIT ?1",
            )
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        let conversations = stmt
            .query_map([limit], |row| {
                Ok(Conversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    metadata: row.get(4)?,
                })
            })
            .map_err(|e| MemoryError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        Ok(conversations)
    }

    async fn delete_conversation(&self, id: &str) -> Result<(), MemoryError> {
        let conn = self.conn.lock();

        // Delete messages first (foreign key)
        conn.execute("DELETE FROM messages WHERE conversation_id = ?1", [id])
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        // Delete conversation
        let deleted = conn
            .execute("DELETE FROM conversations WHERE id = ?1", [id])
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        if deleted == 0 {
            return Err(MemoryError::NotFound(format!(
                "Conversation not found: {}",
                id
            )));
        }

        debug!("Deleted conversation: {}", id);
        Ok(())
    }

    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: Option<String>,
    ) -> Result<String, MemoryError> {
        let id = Uuid::new_v4().to_string();
        let now_str = Utc::now().to_rfc3339();

        let conn = self.conn.lock();

        // Insert message
        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&id, conversation_id, role, content, &now_str, &metadata),
        )
        .map_err(|e| MemoryError::Database(e.to_string()))?;

        // Update conversation updated_at
        conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            (&now_str, conversation_id),
        )
        .map_err(|e| MemoryError::Database(e.to_string()))?;

        debug!("Added message {} to conversation {}", id, conversation_id);

        Ok(id)
    }

    async fn get_messages(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> Result<Vec<Message>, MemoryError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, conversation_id, role, content, created_at, metadata FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC LIMIT ?2",
            )
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        let messages = stmt
            .query_map([conversation_id, &limit.to_string()], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    metadata: row.get(5)?,
                })
            })
            .map_err(|e| MemoryError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        Ok(messages)
    }

    async fn get_recent_messages(&self, limit: usize) -> Result<Vec<Message>, MemoryError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, conversation_id, role, content, created_at, metadata FROM messages ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        let messages = stmt
            .query_map([limit], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    metadata: row.get(5)?,
                })
            })
            .map_err(|e| MemoryError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        Ok(messages)
    }

    async fn search_messages(&self, query: &str, limit: usize) -> Result<Vec<Message>, MemoryError> {
        let conn = self.conn.lock();
        
        // Use LIKE for basic search (FTS would be better for large datasets)
        let pattern = format!("%{}%", query);
        let mut stmt = conn
            .prepare(
                "SELECT id, conversation_id, role, content, created_at, metadata 
                 FROM messages 
                 WHERE content LIKE ?1 
                 ORDER BY created_at DESC 
                 LIMIT ?2",
            )
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        let messages = stmt
            .query_map([&pattern, &limit.to_string()], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    metadata: row.get(5)?,
                })
            })
            .map_err(|e| MemoryError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        Ok(messages)
    }

    async fn kv_set(
        &self,
        key: &str,
        value: serde_json::Value,
        ttl_secs: Option<u64>,
    ) -> Result<(), MemoryError> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let value_str =
            serde_json::to_string(&value).map_err(|e| MemoryError::Serialization(e.to_string()))?;
        let expires_at = ttl_secs.map(|secs| {
            (now + chrono::Duration::seconds(secs as i64)).to_rfc3339()
        });

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, value, created_at, updated_at, expires_at) VALUES (?1, ?2, COALESCE((SELECT created_at FROM kv_store WHERE key = ?1), ?3), ?3, ?4)",
            (&key, &value_str, &now_str, &expires_at),
        )
        .map_err(|e| MemoryError::Database(e.to_string()))?;

        debug!("Set key: {}", key);
        Ok(())
    }

    async fn kv_get(&self, key: &str) -> Result<Option<KeyValue>, MemoryError> {
        let conn = self.conn.lock();

        // Clean up expired entries first
        let now_str = Utc::now().to_rfc3339();
        conn.execute(
            "DELETE FROM kv_store WHERE expires_at IS NOT NULL AND expires_at < ?1",
            [&now_str],
        )
        .ok();

        let mut stmt = conn
            .prepare(
                "SELECT key, value, created_at, updated_at, expires_at FROM kv_store WHERE key = ?1",
            )
            .map_err(|e| MemoryError::Database(e.to_string()))?;

        let result = stmt.query_row([key], |row| {
            let value_str: String = row.get(1)?;

            Ok(KeyValue {
                key: row.get(0)?,
                value: serde_json::from_str(&value_str).unwrap_or(serde_json::Value::Null),
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                expires_at: row.get(4)?,
            })
        });

        match result {
            Ok(kv) => Ok(Some(kv)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(MemoryError::Database(e.to_string())),
        }
    }

    async fn kv_delete(&self, key: &str) -> Result<(), MemoryError> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM kv_store WHERE key = ?1", [key])
            .map_err(|e| MemoryError::Database(e.to_string()))?;
        debug!("Deleted key: {}", key);
        Ok(())
    }

    async fn kv_list(&self, prefix: Option<&str>) -> Result<Vec<String>, MemoryError> {
        let conn = self.conn.lock();

        // Clean up expired entries first
        let now_str = Utc::now().to_rfc3339();
        conn.execute(
            "DELETE FROM kv_store WHERE expires_at IS NOT NULL AND expires_at < ?1",
            [&now_str],
        )
        .ok();

        let keys: Vec<String> = if let Some(prefix) = prefix {
            let pattern = format!("{}%", prefix);
            let mut stmt = conn
                .prepare("SELECT key FROM kv_store WHERE key LIKE ?1 ORDER BY key")
                .map_err(|e| MemoryError::Database(e.to_string()))?;
            let rows = stmt
                .query_map([&pattern], |row| row.get(0))
                .map_err(|e| MemoryError::Database(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| MemoryError::Database(e.to_string()))?
        } else {
            let mut stmt = conn
                .prepare("SELECT key FROM kv_store ORDER BY key")
                .map_err(|e| MemoryError::Database(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| MemoryError::Database(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| MemoryError::Database(e.to_string()))?
        };

        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_conversation_crud() {
        let store = SqliteStore::in_memory().unwrap();

        // Create
        let conv_id = store
            .create_conversation(Some("Test".to_string()), None)
            .await
            .unwrap();

        // Get
        let fetched = store.get_conversation(&conv_id).await.unwrap();
        assert_eq!(fetched.id, conv_id);
        assert_eq!(fetched.title, Some("Test".to_string()));

        // List
        let list = store.list_conversations(10).await.unwrap();
        assert_eq!(list.len(), 1);

        // Delete
        store.delete_conversation(&conv_id).await.unwrap();
        let list = store.list_conversations(10).await.unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_messages() {
        let store = SqliteStore::in_memory().unwrap();

        let conv_id = store.create_conversation(None, None).await.unwrap();

        store.add_message(&conv_id, "user", "Hello", None).await.unwrap();
        store.add_message(&conv_id, "assistant", "Hi there!", None).await.unwrap();

        let messages = store.get_messages(&conv_id, 10).await.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
    }

    #[tokio::test]
    async fn test_search_messages() {
        let store = SqliteStore::in_memory().unwrap();

        let conv_id = store.create_conversation(None, None).await.unwrap();

        store.add_message(&conv_id, "user", "Hello world", None).await.unwrap();
        store.add_message(&conv_id, "assistant", "Goodbye world", None).await.unwrap();

        let results = store.search_messages("world", 10).await.unwrap();
        assert_eq!(results.len(), 2);

        let results = store.search_messages("Hello", 10).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_kv_store() {
        let store = SqliteStore::in_memory().unwrap();

        // Set
        store
            .kv_set("test_key", serde_json::json!({"foo": "bar"}), None)
            .await
            .unwrap();

        // Get
        let kv = store.kv_get("test_key").await.unwrap();
        assert!(kv.is_some());
        assert_eq!(kv.unwrap().value, serde_json::json!({"foo": "bar"}));

        // List
        let keys = store.kv_list(None).await.unwrap();
        assert_eq!(keys, vec!["test_key"]);

        // Delete
        store.kv_delete("test_key").await.unwrap();
        let kv = store.kv_get("test_key").await.unwrap();
        assert!(kv.is_none());
    }
}
