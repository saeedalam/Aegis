//! Core tools - Essential primitives for any MCP runtime.
//!
//! These tools are always available and represent the minimal
//! set of capabilities needed for an AI agent to interact with the world.

mod echo;
mod get_time;
mod fs_read;
mod fs_write;
mod cmd_exec;
mod memory;
mod http_request;
mod env;
mod utils;

use std::sync::Arc;
use crate::tools::ToolRegistry;
use crate::core::Config;

pub use echo::EchoTool;
pub use get_time::GetTimeTool;
pub use fs_read::FsReadTool;
pub use fs_write::FsWriteTool;
pub use cmd_exec::CmdExecTool;
pub use memory::{MemoryStoreTool, MemoryRecallTool, MemoryDeleteTool, MemoryListTool};
pub use http_request::HttpRequestTool;
pub use env::{EnvGetTool, EnvListTool, SysInfoTool};
pub use utils::{
    Base64EncodeTool, Base64DecodeTool,
    JsonParseTool, JsonQueryTool,
    UuidGenerateTool, HashTool,
    RegexMatchTool, RegexReplaceTool,
};

/// Registers all core tools with the registry.
/// These are the essential tools that define Nexus as an MCP runtime.
pub fn register_core_tools(registry: &mut ToolRegistry, config: &Config) {
    // Basic utilities (always available)
    registry.register(Arc::new(EchoTool));
    registry.register(Arc::new(GetTimeTool));
    registry.register(Arc::new(UuidGenerateTool));

    // Filesystem tools (restricted by config)
    registry.register(Arc::new(FsReadTool::new(config.security.allowed_read_paths.clone())));
    registry.register(Arc::new(FsWriteTool::new(config.security.allowed_write_paths.clone())));

    // Command execution (restricted by config)
    registry.register(Arc::new(CmdExecTool::new(config.security.allowed_commands.clone())));

    // Memory tools (always available)
    registry.register(Arc::new(MemoryStoreTool));
    registry.register(Arc::new(MemoryRecallTool));
    registry.register(Arc::new(MemoryDeleteTool));
    registry.register(Arc::new(MemoryListTool));

    // HTTP request tool (restricted by config)
    registry.register(Arc::new(HttpRequestTool::new(config)));

    // Environment/System tools
    registry.register(Arc::new(EnvGetTool));
    registry.register(Arc::new(EnvListTool));
    registry.register(Arc::new(SysInfoTool));

    // Data utilities
    registry.register(Arc::new(Base64EncodeTool));
    registry.register(Arc::new(Base64DecodeTool));
    registry.register(Arc::new(JsonParseTool));
    registry.register(Arc::new(JsonQueryTool));
    registry.register(Arc::new(HashTool));
    registry.register(Arc::new(RegexMatchTool));
    registry.register(Arc::new(RegexReplaceTool));
}

/// Returns the count of core tools.
pub fn core_tool_count() -> usize {
    18 // echo, get_time, uuid, fs.read, fs.write, cmd.exec, 
       // memory.store/recall/delete/list, http.request,
       // env.get/list, sys.info, base64.encode/decode,
       // json.parse/query, hash.sha256, regex.match/replace
}


