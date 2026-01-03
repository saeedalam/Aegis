//! Transport layer for MCP communication.
//!
//! This module provides:
//! - Transport trait defining the interface for message exchange
//! - Stdio transport for CLI/pipe-based communication
//! - SSE transport for HTTP-based communication
//! - Middleware for auth, rate limiting, and observability

/// Transport trait definition.
mod transport;

/// Stdio transport implementation.
pub mod stdio;

/// SSE (Server-Sent Events) transport via Axum.
pub mod sse;

/// HTTP middleware (auth, rate limiting, metrics).
pub mod middleware;

// Re-exports
pub use transport::Transport;
pub use stdio::StdioTransport;
pub use middleware::{AuthState, RateLimiter, RateLimitState, Metrics};

