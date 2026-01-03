//! MCP request router.
//!
//! Routes incoming JSON-RPC requests to the appropriate handler
//! based on the method name.

use std::sync::Arc;
use tracing::{debug, warn};

use crate::core::RuntimeState;
use crate::protocol::{Request, Response, ErrorObject, McpMethod};
use crate::handlers::{
    handle_initialize, handle_tools_list, handle_tools_call,
    handle_prompts_list, handle_ping, handle_resources_list, handle_resources_read
};

/// Router for dispatching MCP requests to handlers.
#[derive(Debug, Clone)]
pub struct Router {
    // Future: could hold handler registry, middleware chain, etc.
}

impl Router {
    /// Creates a new router.
    pub fn new() -> Self {
        Self {}
    }

    /// Handles an incoming MCP request and returns a response.
    pub async fn handle(&self, request: Request, state: Arc<RuntimeState>) -> Response {
        let method = McpMethod::from_str(&request.method);
        let id = request.id.clone();

        debug!("Routing request: method={:?}, id={:?}", method, id);

        match method {
            McpMethod::Initialize => {
                match handle_initialize(request.params, state).await {
                    Ok(result) => Response::success(id, result),
                    Err(e) => Response::from_error(id, &e),
                }
            }

            McpMethod::Initialized => {
                // Notification - just acknowledge
                debug!("Received initialized notification");
                Response::success(id, serde_json::json!({}))
            }

            McpMethod::ToolsList => {
                match handle_tools_list(request.params, state).await {
                    Ok(result) => Response::success(id, result),
                    Err(e) => Response::from_error(id, &e),
                }
            }

            McpMethod::ToolsCall => {
                match handle_tools_call(request.params, state).await {
                    Ok(result) => Response::success(id, result),
                    Err(e) => Response::from_error(id, &e),
                }
            }

            McpMethod::PromptsList => {
                match handle_prompts_list(request.params, state).await {
                    Ok(result) => Response::success(id, result),
                    Err(e) => Response::from_error(id, &e),
                }
            }

            McpMethod::Ping => {
                match handle_ping().await {
                    Ok(result) => Response::success(id, result),
                    Err(e) => Response::from_error(id, &e),
                }
            }

            McpMethod::ResourcesList => {
                match handle_resources_list(request.params, state).await {
                    Ok(result) => Response::success(id, result),
                    Err(e) => Response::from_error(id, &e),
                }
            }

            McpMethod::ResourcesRead => {
                match handle_resources_read(request.params, state).await {
                    Ok(result) => Response::success(id, result),
                    Err(e) => Response::from_error(id, &e),
                }
            }

            McpMethod::PromptsGet => {
                warn!("prompts/get not implemented yet");
                Response::error(
                    id,
                    ErrorObject::new(-32601, "prompts/get not implemented yet"),
                )
            }

            McpMethod::Unknown(method_name) => {
                warn!("Unknown method: {}", method_name);
                Response::error(id, ErrorObject::method_not_found(&method_name))
            }
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
