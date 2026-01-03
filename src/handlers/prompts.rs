//! Handler for the `prompts/list` MCP method.
//!
//! Returns the list of available prompts that the server provides.

use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

use crate::core::{NexusResult, RuntimeState};
use crate::protocol::mcp::{Prompt, PromptsListResult};

/// Handles the `prompts/list` request.
///
/// Returns a list of available prompts. For the MVP, we return an empty list
/// as prompts are not the primary focus of Nexus (tools are).
pub async fn handle_prompts_list(
    _params: Option<Value>,
    _state: Arc<RuntimeState>,
) -> NexusResult<Value> {
    debug!("Handling prompts/list request");

    // MVP: Return empty list (prompts are not our focus)
    let prompts: Vec<Prompt> = vec![];

    let result = PromptsListResult { prompts };

    debug!("Returning {} prompts", result.prompts.len());

    serde_json::to_value(result)
        .map_err(|e| crate::core::NexusError::Internal(format!("Failed to serialize: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Config;

    #[tokio::test]
    async fn test_prompts_list() {
        let state = Arc::new(RuntimeState::new(Config::default()));
        let result = handle_prompts_list(None, state).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        let prompts = value.get("prompts").unwrap().as_array().unwrap();
        assert!(prompts.is_empty()); // MVP returns empty
    }
}


