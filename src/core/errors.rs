//! Error types for Aegis operations.

use thiserror::Error;

/// The primary error type for Aegis operations.
#[derive(Error, Debug)]
pub enum AegisError {
    /// JSON-RPC protocol error.
    #[error("JSON-RPC error: {code} - {message}")]
    JsonRpc { code: i32, message: String },

    /// Failed to parse JSON.
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// I/O error during transport operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid MCP method.
    #[error("Unknown method: {0}")]
    UnknownMethod(String),

    /// Missing required field.
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid request format.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Transport error.
    #[error("Transport error: {0}")]
    Transport(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AegisError {
    /// Returns the JSON-RPC error code for this error.
    pub fn code(&self) -> i32 {
        match self {
            AegisError::JsonParse(_) => -32700,       // Parse error
            AegisError::InvalidRequest(_) => -32600,  // Invalid Request
            AegisError::UnknownMethod(_) => -32601,   // Method not found
            AegisError::MissingField(_) => -32602,    // Invalid params
            AegisError::JsonRpc { code, .. } => *code,
            AegisError::Io(_) => -32000,              // Server error
            AegisError::Transport(_) => -32001,
            AegisError::Config(_) => -32002,
            AegisError::Internal(_) => -32603,        // Internal error
        }
    }

    /// Creates a new JSON-RPC error with the given code and message.
    pub fn json_rpc(code: i32, message: impl Into<String>) -> Self {
        AegisError::JsonRpc {
            code,
            message: message.into(),
        }
    }
}

/// A specialized Result type for Aegis operations.
pub type AegisResult<T> = Result<T, AegisError>;

// Backwards compatibility aliases
#[doc(hidden)]
pub type NexusError = AegisError;
#[doc(hidden)]
pub type NexusResult<T> = AegisResult<T>;

/// Standard JSON-RPC error codes as defined in the specification.
pub mod codes {
    /// Invalid JSON was received by the server.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist / is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;
}
