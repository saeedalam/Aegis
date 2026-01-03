//! Scheduler for automated task execution.
//!
//! Provides cron-like scheduling for tools and workflows.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::core::RuntimeState;

/// A scheduled task definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// Unique task ID.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Cron expression (e.g., "0 * * * *" for every hour).
    pub cron: String,
    /// Tool to execute.
    pub tool: String,
    /// Tool arguments.
    pub args: serde_json::Value,
    /// Whether the task is enabled.
    pub enabled: bool,
    /// When the task was created.
    pub created_at: String,
    /// Last execution time.
    pub last_run: Option<String>,
    /// Last execution result.
    pub last_result: Option<TaskResult>,
}

/// Result of a task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Whether execution was successful.
    pub success: bool,
    /// Output or error message.
    pub output: String,
    /// Execution timestamp.
    pub executed_at: String,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

/// Scheduler for managing automated tasks.
#[derive(Debug)]
pub struct Scheduler {
    tasks: RwLock<HashMap<String, ScheduledTask>>,
    running: std::sync::atomic::AtomicBool,
}

impl Scheduler {
    /// Creates a new scheduler.
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Adds a scheduled task.
    pub fn add_task(&self, task: ScheduledTask) -> Result<(), String> {
        // Validate cron expression
        Self::validate_cron(&task.cron)?;

        let id = task.id.clone();
        self.tasks.write().insert(id.clone(), task);
        info!("Added scheduled task: {}", id);
        Ok(())
    }

    /// Removes a scheduled task.
    pub fn remove_task(&self, id: &str) -> bool {
        self.tasks.write().remove(id).is_some()
    }

    /// Gets a scheduled task by ID.
    pub fn get_task(&self, id: &str) -> Option<ScheduledTask> {
        self.tasks.read().get(id).cloned()
    }

    /// Lists all scheduled tasks.
    pub fn list_tasks(&self) -> Vec<ScheduledTask> {
        self.tasks.read().values().cloned().collect()
    }

    /// Enables or disables a task.
    pub fn set_enabled(&self, id: &str, enabled: bool) -> bool {
        if let Some(task) = self.tasks.write().get_mut(id) {
            task.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Validates a cron expression.
    fn validate_cron(cron: &str) -> Result<(), String> {
        let parts: Vec<&str> = cron.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(format!(
                "Invalid cron expression: expected 5 parts, got {}",
                parts.len()
            ));
        }
        // Basic validation - could be more thorough
        Ok(())
    }

    /// Checks if a cron expression should trigger at the given time.
    fn should_trigger(cron: &str, time: DateTime<Utc>) -> bool {
        let parts: Vec<&str> = cron.split_whitespace().collect();
        if parts.len() != 5 {
            return false;
        }

        let minute = time.format("%M").to_string().parse::<u32>().unwrap_or(0);
        let hour = time.format("%H").to_string().parse::<u32>().unwrap_or(0);
        let day = time.format("%d").to_string().parse::<u32>().unwrap_or(1);
        let month = time.format("%m").to_string().parse::<u32>().unwrap_or(1);
        let weekday = time.format("%u").to_string().parse::<u32>().unwrap_or(1); // 1-7

        Self::matches_cron_part(parts[0], minute)
            && Self::matches_cron_part(parts[1], hour)
            && Self::matches_cron_part(parts[2], day)
            && Self::matches_cron_part(parts[3], month)
            && Self::matches_cron_part(parts[4], weekday)
    }

    /// Checks if a value matches a cron part.
    fn matches_cron_part(part: &str, value: u32) -> bool {
        if part == "*" {
            return true;
        }

        // Handle step values like */5
        if let Some(step) = part.strip_prefix("*/") {
            if let Ok(step_val) = step.parse::<u32>() {
                return value % step_val == 0;
            }
        }

        // Handle ranges like 1-5
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                if let (Ok(start), Ok(end)) = (range[0].parse::<u32>(), range[1].parse::<u32>()) {
                    return value >= start && value <= end;
                }
            }
        }

        // Handle lists like 1,3,5
        if part.contains(',') {
            return part.split(',').any(|p| p.parse::<u32>().ok() == Some(value));
        }

        // Exact match
        part.parse::<u32>().ok() == Some(value)
    }

    /// Starts the scheduler loop.
    pub async fn start(&self, state: Arc<RuntimeState>) {
        if self
            .running
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            warn!("Scheduler already running");
            return;
        }

        info!("Starting scheduler");

        loop {
            if !self.running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            let now = Utc::now();
            let tasks_to_run: Vec<ScheduledTask> = self
                .tasks
                .read()
                .values()
                .filter(|t| t.enabled && Self::should_trigger(&t.cron, now))
                .cloned()
                .collect();

            for task in tasks_to_run {
                let state_clone = state.clone();
                let task_id = task.id.clone();

                tokio::spawn(async move {
                    let start = std::time::Instant::now();
                    debug!("Executing scheduled task: {}", task_id);

                    // Get the tool first, release the lock before await
                    let tool = {
                        let registry = state_clone.tool_registry.read();
                        registry.get(&task.tool).cloned()
                    };
                    
                    let result = match tool {
                        Some(t) => t.execute(task.args.clone(), state_clone.clone()).await,
                        None => Err(crate::tools::ToolError::NotFound(task.tool.clone())),
                    };

                    let duration = start.elapsed().as_millis() as u64;

                    match result {
                        Ok(output) => {
                            info!(
                                "Task {} completed successfully in {}ms",
                                task_id, duration
                            );
                            debug!("Task output: {:?}", output);
                        }
                        Err(e) => {
                            error!("Task {} failed: {}", task_id, e);
                        }
                    }
                });
            }

            // Check every minute
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }

        info!("Scheduler stopped");
    }

    /// Stops the scheduler.
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_matching() {
        // Test wildcard
        assert!(Scheduler::matches_cron_part("*", 5));
        assert!(Scheduler::matches_cron_part("*", 0));

        // Test exact match
        assert!(Scheduler::matches_cron_part("5", 5));
        assert!(!Scheduler::matches_cron_part("5", 6));

        // Test step
        assert!(Scheduler::matches_cron_part("*/5", 0));
        assert!(Scheduler::matches_cron_part("*/5", 5));
        assert!(Scheduler::matches_cron_part("*/5", 10));
        assert!(!Scheduler::matches_cron_part("*/5", 3));

        // Test range
        assert!(Scheduler::matches_cron_part("1-5", 3));
        assert!(!Scheduler::matches_cron_part("1-5", 6));

        // Test list
        assert!(Scheduler::matches_cron_part("1,3,5", 3));
        assert!(!Scheduler::matches_cron_part("1,3,5", 4));
    }

    #[test]
    fn test_validate_cron() {
        assert!(Scheduler::validate_cron("* * * * *").is_ok());
        assert!(Scheduler::validate_cron("0 * * * *").is_ok());
        assert!(Scheduler::validate_cron("*/5 * * * *").is_ok());
        assert!(Scheduler::validate_cron("bad").is_err());
    }
}

