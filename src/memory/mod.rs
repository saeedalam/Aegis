//! Memory module for persistent storage.
//!
//! This module provides:
//! - SQLite-based storage for conversations, messages, and key-value data
//! - Memory trait for abstraction over storage backends
//! - Resource types for MCP resources/list and resources/read

mod store;
mod sqlite;
mod schema;

pub use store::{MemoryStore, Conversation, Message, KeyValue};
pub use sqlite::SqliteStore;
pub use schema::initialize_schema;


