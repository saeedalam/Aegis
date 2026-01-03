//! Built-in tools for Aegis.
//!
//! This module only contains the ScriptTool for plugin support.
//! Core and Extra tools are in their respective modules.

mod script_tool;

// Re-export script tool (needed for plugins)
pub use script_tool::ScriptTool;
