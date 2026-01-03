//! File system write tool - writes file contents.

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

/// File write tool - writes content to allowed paths.
#[derive(Debug)]
pub struct FsWriteTool {
    allowed_paths: Vec<PathBuf>,
}

#[derive(Deserialize)]
struct FsWriteArgs {
    path: String,
    content: String,
    #[serde(default)]
    append: bool,
}

impl FsWriteTool {
    /// Creates a new FsWriteTool with the given allowed paths.
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    /// Checks if a path is within the allowed directories.
    fn is_path_allowed(&self, path: &Path) -> bool {
        if self.allowed_paths.is_empty() {
            return false;
        }

        // For new files, check parent directory
        let check_path = if path.exists() {
            path.to_path_buf()
        } else {
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| path.to_path_buf())
        };

        let canonical = match check_path.canonicalize() {
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
impl Tool for FsWriteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "fs.write_file".to_string(),
            description: Some("Writes content to a file. Only allowed paths can be accessed.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the file"
                    },
                    "append": {
                        "type": "boolean",
                        "description": "If true, append to the file instead of overwriting",
                        "default": false
                    }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let args: FsWriteArgs = serde_json::from_value(arguments)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        debug!("Writing file: {:?} (append: {})", path, args.append);

        // Check if path is allowed
        if !self.is_path_allowed(&path) {
            return Err(ToolError::PermissionDenied(format!(
                "Path not in allowed directories: {}",
                args.path
            )));
        }

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create directories: {}", e)))?;
            }
        }

        // Write the file
        if args.append {
            use tokio::io::AsyncWriteExt;
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to open file: {}", e)))?;
            
            file.write_all(args.content.as_bytes())
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;
        } else {
            fs::write(&path, &args.content)
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;
        }

        Ok(ToolOutput::text(serde_json::json!({
            "success": true,
            "path": args.path,
            "bytes_written": args.content.len()
        }).to_string()))
    }
}


