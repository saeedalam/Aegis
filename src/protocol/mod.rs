//! Protocol module for JSON-RPC 2.0 and MCP message types.
//!
//! This module provides:
//! - JSON-RPC 2.0 request/response types
//! - MCP-specific message types and capabilities

/// JSON-RPC 2.0 protocol types.
pub mod jsonrpc;

/// Model Context Protocol (MCP) message types.
pub mod mcp;

// Re-exports for convenience
pub use jsonrpc::{Request, Response, ErrorObject, RequestId};
pub use mcp::{McpMethod, ServerCapabilities, ClientCapabilities};


