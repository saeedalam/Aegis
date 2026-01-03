//! JSON-RPC 2.0 protocol types.
//!
//! This module implements the JSON-RPC 2.0 specification for MCP communication.
//! See: https://www.jsonrpc.org/specification

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The JSON-RPC version string (always "2.0").
pub const JSONRPC_VERSION: &str = "2.0";

/// A JSON-RPC request ID, which can be a string, number, or null.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// String identifier.
    String(String),
    /// Numeric identifier.
    Number(i64),
    /// Null identifier (for notifications, though MCP doesn't use these much).
    Null,
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        RequestId::Number(n)
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        RequestId::String(s)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        RequestId::String(s.to_string())
    }
}

/// A JSON-RPC 2.0 request object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// JSON-RPC version (must be "2.0").
    pub jsonrpc: String,

    /// The method to be invoked.
    pub method: String,

    /// The parameters for the method (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    /// The request identifier.
    pub id: RequestId,
}

impl Request {
    /// Creates a new JSON-RPC request.
    pub fn new(method: impl Into<String>, params: Option<Value>, id: RequestId) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
            id,
        }
    }

    /// Parses a JSON-RPC request from a JSON string.
    pub fn from_str(s: &str) -> Result<Self, crate::core::NexusError> {
        serde_json::from_str(s).map_err(Into::into)
    }

    /// Validates that this is a proper JSON-RPC 2.0 request.
    pub fn validate(&self) -> Result<(), crate::core::NexusError> {
        if self.jsonrpc != JSONRPC_VERSION {
            return Err(crate::core::NexusError::InvalidRequest(
                format!("Invalid jsonrpc version: expected '2.0', got '{}'", self.jsonrpc)
            ));
        }
        if self.method.is_empty() {
            return Err(crate::core::NexusError::InvalidRequest(
                "Method cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}

/// A JSON-RPC 2.0 response object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// JSON-RPC version (must be "2.0").
    pub jsonrpc: String,

    /// The result of the method invocation (mutually exclusive with error).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// The error object (mutually exclusive with result).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorObject>,

    /// The request identifier (same as the request).
    pub id: RequestId,
}

impl Response {
    /// Creates a successful response with the given result.
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Creates an error response.
    pub fn error(id: RequestId, error: ErrorObject) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }

    /// Creates an error response from a NexusError.
    pub fn from_error(id: RequestId, err: &crate::core::NexusError) -> Self {
        Self::error(id, ErrorObject::from_nexus_error(err))
    }

    /// Serializes the response to a JSON string.
    pub fn to_json(&self) -> Result<String, crate::core::NexusError> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorObject {
    /// A number indicating the error type.
    pub code: i32,

    /// A short description of the error.
    pub message: String,

    /// Additional information about the error (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ErrorObject {
    /// Creates a new error object.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Creates a new error object with additional data.
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Creates an error object from a NexusError.
    pub fn from_nexus_error(err: &crate::core::NexusError) -> Self {
        Self::new(err.code(), err.to_string())
    }

    /// Parse error (-32700).
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(crate::core::errors::codes::PARSE_ERROR, message)
    }

    /// Invalid request (-32600).
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(crate::core::errors::codes::INVALID_REQUEST, message)
    }

    /// Method not found (-32601).
    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            crate::core::errors::codes::METHOD_NOT_FOUND,
            format!("Method not found: {}", method),
        )
    }

    /// Invalid params (-32602).
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(crate::core::errors::codes::INVALID_PARAMS, message)
    }

    /// Internal error (-32603).
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(crate::core::errors::codes::INTERNAL_ERROR, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_parsing() {
        let json = r#"{"jsonrpc": "2.0", "method": "initialize", "id": 1}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "initialize");
        assert_eq!(req.id, RequestId::Number(1));
    }

    #[test]
    fn test_response_success() {
        let resp = Response::success(RequestId::Number(1), serde_json::json!({"ok": true}));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_response_error() {
        let error = ErrorObject::method_not_found("unknown");
        let resp = Response::error(RequestId::Number(1), error);
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }
}


