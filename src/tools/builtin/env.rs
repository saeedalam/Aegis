//! Environment variable tools.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool to get an environment variable.
#[derive(Debug)]
pub struct EnvGetTool;

#[async_trait]
impl Tool for EnvGetTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "env.get".to_string(),
            description: Some("Gets the value of an environment variable.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Environment variable name"
                    },
                    "default": {
                        "type": "string",
                        "description": "Default value if not set"
                    }
                },
                "required": ["key"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let key = arguments
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'key'".to_string()))?;

        let default = arguments.get("default").and_then(|v| v.as_str());

        let value = env::var(key).ok().or_else(|| default.map(|s| s.to_string()));

        let result = json!({
            "key": key,
            "value": value,
            "found": env::var(key).is_ok()
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to list environment variables.
#[derive(Debug)]
pub struct EnvListTool;

#[async_trait]
impl Tool for EnvListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "env.list".to_string(),
            description: Some(
                "Lists environment variable names (not values for security). Use prefix to filter."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "prefix": {
                        "type": "string",
                        "description": "Filter by prefix (e.g., 'PATH', 'HOME')"
                    },
                    "show_values": {
                        "type": "boolean",
                        "description": "Show values (default: false for security)"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let prefix = arguments.get("prefix").and_then(|v| v.as_str());
        let show_values = arguments
            .get("show_values")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let vars: Vec<Value> = env::vars()
            .filter(|(k, _)| {
                if let Some(p) = prefix {
                    k.starts_with(p)
                } else {
                    true
                }
            })
            .map(|(k, v)| {
                if show_values {
                    json!({"key": k, "value": v})
                } else {
                    json!({"key": k})
                }
            })
            .collect();

        let result = json!({
            "count": vars.len(),
            "variables": vars
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to get system information.
#[derive(Debug)]
pub struct SysInfoTool;

#[async_trait]
impl Tool for SysInfoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "sys.info".to_string(),
            description: Some("Gets system information (OS, arch, hostname, etc.).".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let result = json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "family": std::env::consts::FAMILY,
            "hostname": hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            "current_dir": std::env::current_dir()
                .ok()
                .map(|p| p.display().to_string()),
            "home_dir": std::env::var("HOME").ok().or_else(|| std::env::var("USERPROFILE").ok()),
            "temp_dir": std::env::temp_dir().display().to_string(),
            "exe_path": std::env::current_exe()
                .ok()
                .map(|p| p.display().to_string())
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}


