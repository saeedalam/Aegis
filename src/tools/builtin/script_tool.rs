//! Script-based custom tools - allows users to define tools via configuration.
//!
//! Users can define custom tools that execute external scripts/commands:
//!
//! ```json
//! {
//!   "plugins": [
//!     {
//!       "name": "my_tool",
//!       "description": "Does something cool",
//!       "command": "/path/to/script.sh",
//!       "args_template": ["--input", "${input}"],
//!       "input_schema": {
//!         "type": "object",
//!         "properties": {
//!           "input": { "type": "string" }
//!         }
//!       }
//!     }
//!   ]
//! }
//! ```

use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use crate::core::{PluginConfig, RuntimeState};
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// A tool that executes an external script.
#[derive(Debug)]
pub struct ScriptTool {
    config: PluginConfig,
}

impl ScriptTool {
    /// Creates a new script tool from configuration.
    pub fn new(config: PluginConfig) -> Self {
        Self { config }
    }

    /// Substitutes ${param} placeholders in a string with argument values.
    fn substitute(&self, template: &str, arguments: &Value) -> String {
        let mut result = template.to_string();

        if let Some(obj) = arguments.as_object() {
            for (key, value) in obj {
                let placeholder = format!("${{{}}}", key);
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }

        result
    }

    fn input_mode(&self) -> &str {
        if self.config.input_mode.is_empty() {
            "args"
        } else {
            &self.config.input_mode
        }
    }

    fn output_mode(&self) -> &str {
        if self.config.output_mode.is_empty() {
            "text"
        } else {
            &self.config.output_mode
        }
    }
}

#[async_trait]
impl Tool for ScriptTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.config.name.clone(),
            description: self.config.description.clone(),
            input_schema: self.config.input_schema.clone(),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        // Build command
        let mut cmd = Command::new(&self.config.command);

        // Set working directory
        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &self.config.env {
            cmd.env(key, self.substitute(value, &arguments));
        }

        // Handle input mode
        match self.input_mode() {
            "stdin" => {
                cmd.stdin(Stdio::piped());
            }
            "env" => {
                // Pass all arguments as environment variables
                if let Some(obj) = arguments.as_object() {
                    for (key, value) in obj {
                        let env_key = format!("NEXUS_ARG_{}", key.to_uppercase());
                        let env_value = match value {
                            Value::String(s) => s.clone(),
                            _ => value.to_string(),
                        };
                        cmd.env(env_key, env_value);
                    }
                }
                // Also pass full JSON
                cmd.env("NEXUS_ARGS_JSON", serde_json::to_string(&arguments).unwrap_or_default());
                cmd.stdin(Stdio::null());
            }
            _ => {
                // "args" mode (default)
                // Substitute arguments in template
                for arg_template in &self.config.args_template {
                    cmd.arg(self.substitute(arg_template, &arguments));
                }
                cmd.stdin(Stdio::null());
            }
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| {
            ToolError::ExecutionFailed(format!("Failed to spawn command: {}", e))
        })?;

        // If stdin mode, write arguments
        if self.input_mode() == "stdin" {
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let json = serde_json::to_string(&arguments).unwrap_or_default();
                let _ = stdin.write_all(json.as_bytes()).await;
            }
        }

        // Wait with timeout
        let duration = Duration::from_secs(self.config.timeout_secs);
        let output = timeout(duration, child.wait_with_output())
            .await
            .map_err(|_| ToolError::Timeout(self.config.timeout_secs))?
            .map_err(|e| ToolError::ExecutionFailed(format!("Command failed: {}", e)))?;

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ToolError::ExecutionFailed(format!(
                "Command exited with status {}: {}",
                output.status.code().unwrap_or(-1),
                stderr
            )));
        }

        // Parse output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        match self.output_mode() {
            "json" => {
                // Validate it's valid JSON
                match serde_json::from_str::<Value>(&stdout) {
                    Ok(json) => Ok(ToolOutput::text(
                        serde_json::to_string_pretty(&json).unwrap_or(stdout),
                    )),
                    Err(_) => Ok(ToolOutput::text(stdout.trim())),
                }
            }
            _ => Ok(ToolOutput::text(stdout.trim())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_substitute() {
        let config = PluginConfig {
            name: "test".to_string(),
            description: None,
            command: "echo".to_string(),
            args_template: vec!["Hello ${name}!".to_string()],
            working_dir: None,
            env: std::collections::HashMap::new(),
            timeout_secs: 30,
            input_schema: json!({ "type": "object" }),
            input_mode: String::new(),
            output_mode: String::new(),
        };

        let tool = ScriptTool::new(config);
        let args = json!({"name": "World"});

        assert_eq!(tool.substitute("Hello ${name}!", &args), "Hello World!");
    }
}

