//! Tool system for Nexus.
//!
//! Tools are organized into two categories:
//! - **Core**: Essential primitives (always loaded)
//! - **Extras**: Optional capabilities (loaded if enabled)

pub mod registry;
pub mod process_manager;
pub mod core;
pub mod extras;

// Keep builtin for backwards compatibility during transition
mod builtin;

pub use registry::{Tool, ToolRegistry, ToolError, ToolOutput, ToolContent, ToolInput};
pub use process_manager::ProcessManager;

// Re-export for convenience
pub use core::register_core_tools;
pub use extras::register_extra_tools;

// Re-export script tool for plugins
pub use builtin::ScriptTool;
