//! Git integration tools.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool to get git status.
#[derive(Debug)]
pub struct GitStatusTool;

#[async_trait]
impl Tool for GitStatusTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "git.status".to_string(),
            description: Some("Gets the git status of a repository.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Repository path (default: current directory)"
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
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let output = Command::new("git")
            .args(["status", "--porcelain", "-b"])
            .current_dir(path)
            .output()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run git: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ToolError::ExecutionFailed(format!("Git error: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        let branch = lines
            .first()
            .and_then(|l| l.strip_prefix("## "))
            .map(|b| b.split("...").next().unwrap_or(b))
            .unwrap_or("unknown");

        let changes: Vec<Value> = lines
            .iter()
            .skip(1)
            .filter(|l| !l.is_empty())
            .map(|line| {
                let status = &line[..2];
                let file = &line[3..];
                json!({
                    "status": status.trim(),
                    "file": file
                })
            })
            .collect();

        let result = json!({
            "branch": branch,
            "clean": changes.is_empty(),
            "changes_count": changes.len(),
            "changes": changes
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to get git log.
#[derive(Debug)]
pub struct GitLogTool;

#[async_trait]
impl Tool for GitLogTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "git.log".to_string(),
            description: Some("Gets recent git commits.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Repository path"
                    },
                    "count": {
                        "type": "integer",
                        "description": "Number of commits (default: 10)"
                    },
                    "format": {
                        "type": "string",
                        "description": "Output format: 'short' or 'full' (default: short)"
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
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let count = arguments
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        let output = Command::new("git")
            .args([
                "log",
                &format!("-{}", count),
                "--pretty=format:%H|%h|%an|%ae|%at|%s",
            ])
            .current_dir(path)
            .output()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run git: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ToolError::ExecutionFailed(format!("Git error: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let commits: Vec<Value> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 6 {
                    Some(json!({
                        "hash": parts[0],
                        "short_hash": parts[1],
                        "author": parts[2],
                        "email": parts[3],
                        "timestamp": parts[4].parse::<i64>().ok(),
                        "message": parts[5]
                    }))
                } else {
                    None
                }
            })
            .collect();

        let result = json!({
            "count": commits.len(),
            "commits": commits
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to get git diff.
#[derive(Debug)]
pub struct GitDiffTool;

#[async_trait]
impl Tool for GitDiffTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "git.diff".to_string(),
            description: Some("Gets git diff for changes.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Repository path"
                    },
                    "file": {
                        "type": "string",
                        "description": "Specific file to diff (optional)"
                    },
                    "staged": {
                        "type": "boolean",
                        "description": "Show staged changes only"
                    },
                    "commit": {
                        "type": "string",
                        "description": "Commit to diff against (e.g., HEAD~1)"
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
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let mut args = vec!["diff", "--stat"];

        if arguments.get("staged").and_then(|v| v.as_bool()).unwrap_or(false) {
            args.push("--cached");
        }

        if let Some(commit) = arguments.get("commit").and_then(|v| v.as_str()) {
            args.push(commit);
        }

        let file = arguments.get("file").and_then(|v| v.as_str());
        if let Some(f) = file {
            args.push("--");
            args.push(f);
        }

        let output = Command::new("git")
            .args(&args)
            .current_dir(path)
            .output()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run git: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(ToolError::ExecutionFailed(format!("Git error: {}", stderr)));
        }

        let result = json!({
            "diff": stdout.to_string()
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to commit changes.
#[derive(Debug)]
pub struct GitCommitTool;

#[async_trait]
impl Tool for GitCommitTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "git.commit".to_string(),
            description: Some("Creates a git commit with staged changes.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Repository path"
                    },
                    "message": {
                        "type": "string",
                        "description": "Commit message"
                    },
                    "add_all": {
                        "type": "boolean",
                        "description": "Stage all changes before committing"
                    }
                },
                "required": ["message"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'message'".to_string()))?;

        // Stage all if requested
        if arguments.get("add_all").and_then(|v| v.as_bool()).unwrap_or(false) {
            let add_output = Command::new("git")
                .args(["add", "-A"])
                .current_dir(path)
                .output()
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to stage: {}", e)))?;

            if !add_output.status.success() {
                let stderr = String::from_utf8_lossy(&add_output.stderr);
                return Err(ToolError::ExecutionFailed(format!("Git add error: {}", stderr)));
            }
        }

        // Commit
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(path)
            .output()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to commit: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(ToolError::ExecutionFailed(format!("Git commit error: {}", stderr)));
        }

        // Get the new commit hash
        let hash_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(path)
            .output()
            .ok();

        let commit_hash = hash_output
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let result = json!({
            "success": true,
            "message": message,
            "commit": commit_hash,
            "output": stdout.to_string()
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to get current branch.
#[derive(Debug)]
pub struct GitBranchTool;

#[async_trait]
impl Tool for GitBranchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "git.branch".to_string(),
            description: Some("Lists git branches or creates a new branch.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Repository path"
                    },
                    "create": {
                        "type": "string",
                        "description": "Name of new branch to create"
                    },
                    "checkout": {
                        "type": "string",
                        "description": "Branch to checkout"
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
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        // Create new branch
        if let Some(name) = arguments.get("create").and_then(|v| v.as_str()) {
            let output = Command::new("git")
                .args(["checkout", "-b", name])
                .current_dir(path)
                .output()
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ToolError::ExecutionFailed(format!("Git error: {}", stderr)));
            }

            let result = json!({
                "success": true,
                "action": "created",
                "branch": name
            });
            return Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()));
        }

        // Checkout branch
        if let Some(name) = arguments.get("checkout").and_then(|v| v.as_str()) {
            let output = Command::new("git")
                .args(["checkout", name])
                .current_dir(path)
                .output()
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ToolError::ExecutionFailed(format!("Git error: {}", stderr)));
            }

            let result = json!({
                "success": true,
                "action": "checkout",
                "branch": name
            });
            return Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()));
        }

        // List branches
        let output = Command::new("git")
            .args(["branch", "-a"])
            .current_dir(path)
            .output()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let branches: Vec<Value> = stdout
            .lines()
            .map(|line| {
                let is_current = line.starts_with('*');
                let name = line.trim_start_matches('*').trim();
                json!({
                    "name": name,
                    "current": is_current
                })
            })
            .collect();

        let current = branches
            .iter()
            .find(|b| b.get("current").and_then(|v| v.as_bool()).unwrap_or(false))
            .and_then(|b| b.get("name").and_then(|v| v.as_str()))
            .unwrap_or("unknown");

        let result = json!({
            "current": current,
            "branches": branches
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

