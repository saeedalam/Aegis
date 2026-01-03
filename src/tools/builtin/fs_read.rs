//! File system read tool - reads file contents.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::debug;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::{Tool, ToolError, ToolOutput};

/// File read tool - reads content from allowed paths.
#[derive(Debug)]
pub struct FsReadTool {
    allowed_paths: Vec<PathBuf>,
}

#[derive(Deserialize)]
struct FsReadArgs {
    path: String,
}

impl FsReadTool {
    /// Creates a new FsReadTool with the given allowed paths.
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    /// Checks if a path is within the allowed directories.
    fn is_path_allowed(&self, path: &Path) -> bool {
        if self.allowed_paths.is_empty() {
            // If no paths configured, deny all
            return false;
        }

        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        for allowed in &self.allowed_paths {
            if let Ok(allowed_canonical) = allowed.canonicalize() {
                if canonical.starts_with(&allowed_canonical) {
                    return true;
                }
            }
        }

        false
    }
}

#[async_trait]
impl Tool for FsReadTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "fs.read_file".to_string(),
            description: Some("Reads the contents of a file. Only allowed paths can be accessed.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let args: FsReadArgs = serde_json::from_value(arguments)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        debug!("Reading file: {:?}", path);

        // Check if path exists first (for better error message)
        if !path.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "File not found: {}",
                args.path
            )));
        }

        // Check if path is allowed
        if !self.is_path_allowed(&path) {
            return Err(ToolError::PermissionDenied(format!(
                "Path not in allowed directories: {}",
                args.path
            )));
        }

        // Read the file
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;

        Ok(ToolOutput::text(content))
    }
}


