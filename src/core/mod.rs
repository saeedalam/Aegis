//! Core module containing fundamental components of Aegis.
//!
//! This module provides:
//! - Error types and result aliases
//! - Configuration management
//! - Runtime state management

/// Error types for Aegis operations.
pub mod errors;

/// Configuration loading and management.
pub mod config;

/// Runtime state shared across handlers.
pub mod state;

// Re-exports for convenience
pub use errors::{AegisError, AegisResult, NexusError, NexusResult};
pub use config::{Config, PluginConfig};
pub use state::RuntimeState;
