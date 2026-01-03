//! Scheduler tools for managing automated tasks.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::scheduler::ScheduledTask;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool to create a scheduled task.
#[derive(Debug)]
pub struct SchedulerCreateTool;

#[async_trait]
impl Tool for SchedulerCreateTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "scheduler.create".to_string(),
            description: Some(
                "Creates a new scheduled task with cron expression. Format: 'minute hour day month weekday'"
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Task name"
                    },
                    "cron": {
                        "type": "string",
                        "description": "Cron expression (e.g., '*/5 * * * *' for every 5 mins)"
                    },
                    "tool": {
                        "type": "string",
                        "description": "Tool to execute"
                    },
                    "args": {
                        "type": "object",
                        "description": "Tool arguments"
                    }
                },
                "required": ["name", "cron", "tool"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'name'".to_string()))?;

        let cron = arguments
            .get("cron")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'cron'".to_string()))?;

        let tool = arguments
            .get("tool")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'tool'".to_string()))?;

        let args = arguments
            .get("args")
            .cloned()
            .unwrap_or(json!({}));

        let task = ScheduledTask {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            cron: cron.to_string(),
            tool: tool.to_string(),
            args,
            enabled: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_run: None,
            last_result: None,
        };

        let task_id = task.id.clone();
        state
            .scheduler
            .add_task(task)
            .map_err(|e| ToolError::ExecutionFailed(e))?;

        let result = json!({
            "success": true,
            "task_id": task_id,
            "name": name,
            "cron": cron,
            "message": format!("Scheduled task '{}' created", name)
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to list scheduled tasks.
#[derive(Debug)]
pub struct SchedulerListTool;

#[async_trait]
impl Tool for SchedulerListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "scheduler.list".to_string(),
            description: Some("Lists all scheduled tasks.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn execute(
        &self,
        _arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let tasks = state.scheduler.list_tasks();

        let tasks_json: Vec<Value> = tasks
            .iter()
            .map(|t| {
                json!({
                    "id": t.id,
                    "name": t.name,
                    "cron": t.cron,
                    "tool": t.tool,
                    "enabled": t.enabled,
                    "last_run": t.last_run,
                    "created_at": t.created_at
                })
            })
            .collect();

        let result = json!({
            "count": tasks.len(),
            "tasks": tasks_json
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to delete a scheduled task.
#[derive(Debug)]
pub struct SchedulerDeleteTool;

#[async_trait]
impl Tool for SchedulerDeleteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "scheduler.delete".to_string(),
            description: Some("Deletes a scheduled task.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Task ID to delete"
                    }
                },
                "required": ["id"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let id = arguments
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'id'".to_string()))?;

        let deleted = state.scheduler.remove_task(id);

        let result = json!({
            "success": deleted,
            "id": id,
            "message": if deleted { "Task deleted" } else { "Task not found" }
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to enable/disable a scheduled task.
#[derive(Debug)]
pub struct SchedulerToggleTool;

#[async_trait]
impl Tool for SchedulerToggleTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "scheduler.toggle".to_string(),
            description: Some("Enables or disables a scheduled task.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Task ID"
                    },
                    "enabled": {
                        "type": "boolean",
                        "description": "Whether to enable the task"
                    }
                },
                "required": ["id", "enabled"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let id = arguments
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'id'".to_string()))?;

        let enabled = arguments
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'enabled'".to_string()))?;

        let updated = state.scheduler.set_enabled(id, enabled);

        let result = json!({
            "success": updated,
            "id": id,
            "enabled": enabled,
            "message": if updated {
                format!("Task {}", if enabled { "enabled" } else { "disabled" })
            } else {
                "Task not found".to_string()
            }
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to manually trigger a scheduled task.
#[derive(Debug)]
pub struct SchedulerRunTool;

#[async_trait]
impl Tool for SchedulerRunTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "scheduler.run".to_string(),
            description: Some("Manually triggers a scheduled task immediately.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Task ID to run"
                    }
                },
                "required": ["id"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let id = arguments
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'id'".to_string()))?;

        let task = state
            .scheduler
            .get_task(id)
            .ok_or_else(|| ToolError::ExecutionFailed("Task not found".to_string()))?;

        // Execute the tool - get the tool first, then release the lock before await
        let tool = {
            let registry = state.tool_registry.read();
            registry.get(&task.tool).cloned()
        };
        
        let tool = tool.ok_or_else(|| ToolError::ExecutionFailed(format!("Tool not found: {}", task.tool)))?;
        let output = tool.execute(task.args.clone(), state.clone())
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let output_text = match output.content.first() {
            Some(crate::tools::registry::ToolContent::Text { text }) => Some(text.clone()),
            _ => None,
        };

        let result = json!({
            "success": true,
            "task_id": id,
            "task_name": task.name,
            "tool": task.tool,
            "output": output_text
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

