//! Command execution tool - runs shell commands safely.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::{Tool, ToolError, ToolOutput, ToolContent, ProcessManager};

/// Command execution tool - runs allowed commands.
#[derive(Debug)]
pub struct CmdExecTool {
    allowed_commands: Vec<String>,
}

#[derive(Deserialize)]
struct CmdExecArgs {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default = "default_timeout")]
    timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

impl CmdExecTool {
    /// Creates a new CmdExecTool with the given allowed commands.
    pub fn new(allowed_commands: Vec<String>) -> Self {
        Self { allowed_commands }
    }

    /// Checks if a command is in the allowed list.
    fn is_command_allowed(&self, command: &str) -> bool {
        if self.allowed_commands.is_empty() {
            return false;
        }

        // Check exact match or wildcard
        for allowed in &self.allowed_commands {
            if allowed == "*" || allowed == command {
                return true;
            }
            // Support prefix matching (e.g., "git*" matches "git", "git-log")
            if allowed.ends_with('*') {
                let prefix = &allowed[..allowed.len() - 1];
                if command.starts_with(prefix) {
                    return true;
                }
            }
        }

        false
    }
}

#[async_trait]
impl Tool for CmdExecTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "cmd.exec".to_string(),
            description: Some("Executes a shell command. Only allowed commands can be run.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The command to execute"
                    },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Arguments to pass to the command",
                        "default": []
                    },
                    "timeout_secs": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 30, max: 300)",
                        "default": 30
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let args: CmdExecArgs = serde_json::from_value(arguments)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        debug!("Executing command: {} {:?}", args.command, args.args);

        // Check if command is allowed
        if !self.is_command_allowed(&args.command) {
            return Err(ToolError::PermissionDenied(format!(
                "Command not in allowed list: {}",
                args.command
            )));
        }

        // Limit timeout to 5 minutes max
        let timeout_secs = args.timeout_secs.min(300);
        let pm = ProcessManager::with_timeout(timeout_secs);

        // Convert args to &str slice
        let arg_refs: Vec<&str> = args.args.iter().map(|s| s.as_str()).collect();

        // Execute the command
        let output = pm.execute(&args.command, &arg_refs).await?;

        // Format the output
        let result = serde_json::json!({
            "exit_code": output.exit_code,
            "success": output.success,
            "stdout": output.stdout,
            "stderr": output.stderr
        });

        if output.success {
            Ok(ToolOutput::text(result.to_string()))
        } else {
            Ok(ToolOutput {
                content: vec![ToolContent::Text { text: result.to_string() }],
                is_error: true,
            })
        }
    }
}

