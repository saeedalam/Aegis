//! Request handlers for MCP methods.
//!
//! This module provides:
//! - A router for dispatching requests to handlers
//! - Individual handlers for each MCP method

mod router;
mod initialize;
mod tools;
mod tools_call;
mod prompts;
mod ping;
mod resources;

pub use router::Router;
pub use initialize::handle_initialize;
pub use tools::handle_tools_list;
pub use tools_call::handle_tools_call;
pub use prompts::handle_prompts_list;
pub use ping::handle_ping;
pub use resources::{handle_resources_list, handle_resources_read};
