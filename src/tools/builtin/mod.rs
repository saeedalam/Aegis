//! Built-in tools for Nexus.
//!
//! NOTE: This module is being deprecated in favor of:
//! - `tools::core` - Essential tools
//! - `tools::extras` - Optional tools
//!
//! This file remains for backwards compatibility.

mod echo;
mod get_time;
mod fs_read;
mod fs_write;
mod cmd_exec;
mod memory;
mod http_request;
mod utils;
mod script_tool;
mod secrets;
mod conversation;
mod scheduler;
mod llm;
mod webhook;
mod workflow;
mod git;
mod web;
mod env;
mod vector;

use std::sync::Arc;
use tracing::info;
use crate::tools::ToolRegistry;
use crate::core::Config;

// Re-export script tool (still needed for plugins)
pub use script_tool::ScriptTool;

/// Registers plugins from config.
/// Core and extra tools are registered separately now.
pub fn register_plugins(registry: &mut ToolRegistry, config: &Config) {
    // Register custom plugins from config
    for plugin in &config.plugins {
        info!("Registering plugin tool: {}", plugin.name);
        registry.register(Arc::new(ScriptTool::new(plugin.clone())));
    }

    if !config.plugins.is_empty() {
        info!("Loaded {} plugin tool(s)", config.plugins.len());
    }
}

/// Legacy function - now calls core + extras registration.
#[deprecated(note = "Use register_core_tools and register_extra_tools instead")]
pub fn register_builtin_tools(registry: &mut ToolRegistry, config: &Config) {
    crate::tools::core::register_core_tools(registry, config);
    if config.extras_enabled {
        crate::tools::extras::register_extra_tools(registry, config);
    }
    register_plugins(registry, config);
}
