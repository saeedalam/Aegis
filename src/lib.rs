//! # Aegis - MCP Tool Server for AI Agents
//!
//! Aegis is a high-performance, single-binary MCP server that provides tools
//! for AI agents using the Model Context Protocol (MCP).
//!
//! ## Core Modules
//!
//! - `core`: Fundamental types, configuration, and error handling
//! - `protocol`: JSON-RPC 2.0 and MCP message types
//! - `transport`: Communication layer (Stdio, SSE)
//! - `handlers`: MCP request handlers
//! - `tools`: Tool execution and management
//! - `memory`: Persistent storage for conversations and state

/// Core module containing configuration, errors, and state management.
pub mod core;

/// Protocol module for JSON-RPC 2.0 and MCP message types.
pub mod protocol;

/// Transport layer for Stdio and SSE communication.
pub mod transport;

/// Request handlers for MCP methods.
pub mod handlers;

/// Tools module for tool execution and management.
pub mod tools;

/// Memory module for persistent storage.
pub mod memory;

/// Secrets module for secure credential storage.
pub mod secrets;

/// Scheduler module for automated tasks.
pub mod scheduler;

/// Dashboard module for web UI.
pub mod dashboard;
