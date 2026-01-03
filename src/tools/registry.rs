//! Tool registry for managing and executing tools.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;

/// Error type for tool execution.
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Input for a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    /// The name of the tool to call.
    pub name: String,
    /// The arguments to pass to the tool.
    pub arguments: Value,
}

/// Output from a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// The content returned by the tool.
    pub content: Vec<ToolContent>,
    /// Whether the tool execution resulted in an error.
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

/// Content item in tool output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
}

impl ToolOutput {
    /// Creates a successful text output.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: text.into() }],
            is_error: false,
        }
    }

    /// Creates an error output.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: message.into() }],
            is_error: true,
        }
    }
}

/// Trait for executable tools.
#[async_trait]
pub trait Tool: Send + Sync + Debug {
    /// Returns the tool definition for MCP.
    fn definition(&self) -> ToolDefinition;

    /// Executes the tool with the given arguments.
    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError>;
}

/// Registry for managing tools.
#[derive(Debug)]
pub struct ToolRegistry {
    /// All registered tools (public for iteration).
    pub tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Creates a new empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a tool.
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.definition().name.clone();
        tracing::debug!("Registering tool: {}", name);
        self.tools.insert(name, tool);
    }

    /// Gets a tool by name.
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Executes a tool by name.
    pub async fn execute(
        &self,
        name: &str,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        match self.tools.get(name) {
            Some(tool) => tool.execute(arguments, state).await,
            None => Err(ToolError::NotFound(name.to_string())),
        }
    }

    /// Returns all tool definitions for MCP.
    pub fn list_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

