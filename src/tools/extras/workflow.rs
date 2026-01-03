//! Workflow/Pipeline tools for chaining multiple tool calls.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolContent, ToolError, ToolOutput};

/// Tool to execute a workflow (chain of tools).
#[derive(Debug)]
pub struct WorkflowRunTool;

#[async_trait]
impl Tool for WorkflowRunTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "workflow.run".to_string(),
            description: Some(
                "Executes a workflow - a sequence of tool calls. Each step can reference previous outputs."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Workflow name for logging"
                    },
                    "steps": {
                        "type": "array",
                        "description": "Array of workflow steps",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "string", "description": "Step ID for referencing"},
                                "tool": {"type": "string", "description": "Tool to call"},
                                "args": {"type": "object", "description": "Tool arguments"},
                                "condition": {"type": "string", "description": "Condition to check (optional)"}
                            },
                            "required": ["tool"]
                        }
                    },
                    "context": {
                        "type": "object",
                        "description": "Initial context variables"
                    }
                },
                "required": ["steps"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let workflow_name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed");

        let steps = arguments
            .get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'steps' array".to_string()))?;

        let initial_context = arguments
            .get("context")
            .cloned()
            .unwrap_or(json!({}));

        // Context to store step outputs
        let mut context: HashMap<String, Value> = HashMap::new();
        
        // Add initial context
        if let Some(obj) = initial_context.as_object() {
            for (k, v) in obj {
                context.insert(k.clone(), v.clone());
            }
        }

        let mut results: Vec<Value> = Vec::new();
        let mut success = true;

        for (index, step) in steps.iter().enumerate() {
            let default_id = format!("step_{}", index);
            let step_id = step
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or(&default_id);

            let tool_name = step
                .get("tool")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::InvalidInput(format!("Step {} missing 'tool'", step_id))
                })?;

            // Check condition if present
            if let Some(condition) = step.get("condition").and_then(|v| v.as_str()) {
                if !evaluate_condition(condition, &context) {
                    results.push(json!({
                        "step_id": step_id,
                        "tool": tool_name,
                        "skipped": true,
                        "reason": "Condition not met"
                    }));
                    continue;
                }
            }

            // Substitute context variables in arguments
            let raw_args = step.get("args").cloned().unwrap_or(json!({}));
            let args = substitute_context(&raw_args, &context);

            // Execute tool
            let tool = {
                let registry = state.tool_registry.read();
                registry.get(tool_name).cloned()
            };

            let tool = match tool {
                Some(t) => t,
                None => {
                    success = false;
                    results.push(json!({
                        "step_id": step_id,
                        "tool": tool_name,
                        "error": format!("Tool not found: {}", tool_name)
                    }));
                    break;
                }
            };

            match tool.execute(args, state.clone()).await {
                Ok(output) => {
                    // Extract text content
                    let output_text = match output.content.first() {
                        Some(ToolContent::Text { text }) => text.clone(),
                        _ => String::new(),
                    };

                    // Try to parse as JSON for context
                    let output_value: Value = serde_json::from_str(&output_text)
                        .unwrap_or(json!(output_text));

                    // Store in context
                    context.insert(step_id.to_string(), output_value.clone());
                    context.insert("_last".to_string(), output_value.clone());

                    results.push(json!({
                        "step_id": step_id,
                        "tool": tool_name,
                        "success": true,
                        "output": output_value
                    }));
                }
                Err(e) => {
                    success = false;
                    results.push(json!({
                        "step_id": step_id,
                        "tool": tool_name,
                        "error": e.to_string()
                    }));
                    // Stop on error
                    break;
                }
            }
        }

        let result = json!({
            "workflow": workflow_name,
            "success": success,
            "steps_executed": results.len(),
            "steps_total": steps.len(),
            "results": results,
            "final_context": context
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Evaluates a simple condition against context.
fn evaluate_condition(condition: &str, context: &HashMap<String, Value>) -> bool {
    // Simple condition format: "key == value", "key != value", "key exists"
    let parts: Vec<&str> = condition.split_whitespace().collect();

    match parts.len() {
        2 if parts[1] == "exists" => context.contains_key(parts[0]),
        2 if parts[1] == "empty" => {
            context.get(parts[0]).map_or(true, |v| {
                v.is_null() || v.as_str().map_or(false, |s| s.is_empty())
            })
        }
        3 => {
            let key = parts[0];
            let op = parts[1];
            let value = parts[2];

            let ctx_value = context.get(key);

            match op {
                "==" | "=" => ctx_value.map_or(false, |v| {
                    v.as_str().map_or(false, |s| s == value)
                        || v.to_string().trim_matches('"') == value
                }),
                "!=" => ctx_value.map_or(true, |v| {
                    v.as_str().map_or(true, |s| s != value)
                        && v.to_string().trim_matches('"') != value
                }),
                ">" | ">=" | "<" | "<=" => {
                    let ctx_num = ctx_value.and_then(|v| v.as_f64());
                    let cmp_num = value.parse::<f64>().ok();

                    match (ctx_num, cmp_num) {
                        (Some(a), Some(b)) => match op {
                            ">" => a > b,
                            ">=" => a >= b,
                            "<" => a < b,
                            "<=" => a <= b,
                            _ => false,
                        },
                        _ => false,
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Substitutes context variables in a JSON value.
/// Variables are referenced as {{variable_name}} or {{step_id.field}}
fn substitute_context(value: &Value, context: &HashMap<String, Value>) -> Value {
    match value {
        Value::String(s) => {
            let mut result = s.clone();

            // Find and replace {{variable}} patterns
            let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
            for cap in re.captures_iter(s) {
                let full_match = &cap[0];
                let var_path = &cap[1];

                // Handle dot notation
                let parts: Vec<&str> = var_path.split('.').collect();
                let replacement = if parts.len() == 1 {
                    context.get(parts[0]).map(|v| value_to_string(v))
                } else {
                    // Navigate nested path
                    let mut current = context.get(parts[0]);
                    for part in &parts[1..] {
                        current = current.and_then(|v| v.get(*part));
                    }
                    current.map(|v| value_to_string(v))
                };

                if let Some(repl) = replacement {
                    result = result.replace(full_match, &repl);
                }
            }

            Value::String(result)
        }
        Value::Object(obj) => {
            let new_obj: serde_json::Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), substitute_context(v, context)))
                .collect();
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| substitute_context(v, context)).collect())
        }
        _ => value.clone(),
    }
}

/// Converts a Value to string for substitution.
fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        _ => v.to_string(),
    }
}

/// Tool to define a reusable workflow template.
#[derive(Debug)]
pub struct WorkflowDefineTool;

#[async_trait]
impl Tool for WorkflowDefineTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "workflow.define".to_string(),
            description: Some(
                "Saves a workflow definition for later use.".to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Workflow name"
                    },
                    "description": {
                        "type": "string",
                        "description": "Workflow description"
                    },
                    "steps": {
                        "type": "array",
                        "description": "Workflow steps"
                    },
                    "inputs": {
                        "type": "array",
                        "description": "Required input parameters",
                        "items": {"type": "string"}
                    }
                },
                "required": ["name", "steps"]
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

        // Store workflow in memory
        let key = format!("workflow:{}", name);
        
        state
            .memory_store
            .kv_set(&key, arguments.clone(), None)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let result = json!({
            "success": true,
            "workflow": name,
            "message": format!("Workflow '{}' saved. Run with workflow.execute", name)
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to execute a saved workflow.
#[derive(Debug)]
pub struct WorkflowExecuteTool;

#[async_trait]
impl Tool for WorkflowExecuteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "workflow.execute".to_string(),
            description: Some("Executes a previously saved workflow.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Workflow name to execute"
                    },
                    "inputs": {
                        "type": "object",
                        "description": "Input parameters for the workflow"
                    }
                },
                "required": ["name"]
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

        let key = format!("workflow:{}", name);
        
        let workflow = state
            .memory_store
            .kv_get(&key)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?
            .ok_or_else(|| ToolError::ExecutionFailed(format!("Workflow '{}' not found", name)))?;

        let mut workflow_args = workflow.value.clone();
        
        // Merge inputs into context
        if let Some(inputs) = arguments.get("inputs") {
            workflow_args["context"] = inputs.clone();
        }

        // Execute using workflow.run
        let run_tool = WorkflowRunTool;
        run_tool.execute(workflow_args, state).await
    }
}

/// Tool to list saved workflows.
#[derive(Debug)]
pub struct WorkflowListTool;

#[async_trait]
impl Tool for WorkflowListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "workflow.list".to_string(),
            description: Some("Lists all saved workflows.".to_string()),
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
        let keys = state
            .memory_store
            .kv_list(Some("workflow:"))
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let workflows: Vec<String> = keys
            .iter()
            .filter_map(|k| k.strip_prefix("workflow:").map(|s| s.to_string()))
            .collect();

        let result = json!({
            "count": workflows.len(),
            "workflows": workflows
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

