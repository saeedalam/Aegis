//! Model Context Protocol (MCP) message types.
//!
//! This module defines the MCP-specific types for the protocol,
//! including capabilities, tool definitions, and resource types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP protocol version.
pub const MCP_VERSION: &str = "2024-11-05";

/// Known MCP methods.
#[derive(Debug, Clone, PartialEq)]
pub enum McpMethod {
    /// Initialize the connection and negotiate capabilities.
    Initialize,
    /// Notification that initialization is complete.
    Initialized,
    /// List available tools.
    ToolsList,
    /// Call a tool.
    ToolsCall,
    /// List available prompts.
    PromptsList,
    /// Get a specific prompt.
    PromptsGet,
    /// List available resources.
    ResourcesList,
    /// Read a resource.
    ResourcesRead,
    /// Ping for health check.
    Ping,
    /// Unknown method.
    Unknown(String),
}

impl McpMethod {
    /// Parses a method string into an McpMethod.
    pub fn from_str(s: &str) -> Self {
        match s {
            "initialize" => McpMethod::Initialize,
            "initialized" => McpMethod::Initialized,
            "tools/list" => McpMethod::ToolsList,
            "tools/call" => McpMethod::ToolsCall,
            "prompts/list" => McpMethod::PromptsList,
            "prompts/get" => McpMethod::PromptsGet,
            "resources/list" => McpMethod::ResourcesList,
            "resources/read" => McpMethod::ResourcesRead,
            "ping" => McpMethod::Ping,
            _ => McpMethod::Unknown(s.to_string()),
        }
    }

    /// Returns the method string.
    pub fn as_str(&self) -> &str {
        match self {
            McpMethod::Initialize => "initialize",
            McpMethod::Initialized => "initialized",
            McpMethod::ToolsList => "tools/list",
            McpMethod::ToolsCall => "tools/call",
            McpMethod::PromptsList => "prompts/list",
            McpMethod::PromptsGet => "prompts/get",
            McpMethod::ResourcesList => "resources/list",
            McpMethod::ResourcesRead => "resources/read",
            McpMethod::Ping => "ping",
            McpMethod::Unknown(s) => s,
        }
    }
}

/// Server information returned during initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// The name of the server.
    pub name: String,
    /// The version of the server.
    pub version: String,
}

/// Client information received during initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    /// The name of the client.
    pub name: String,
    /// The version of the client.
    pub version: String,
}

/// Server capabilities advertised during initialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    /// Tool execution capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,

    /// Prompt capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,

    /// Resource capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
}

/// Tools capability details.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    /// Whether the tool list can change dynamically.
    #[serde(default)]
    pub list_changed: bool,
}

/// Prompts capability details.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptsCapability {
    /// Whether the prompt list can change dynamically.
    #[serde(default)]
    pub list_changed: bool,
}

/// Resources capability details.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    /// Whether the server supports resource subscriptions.
    #[serde(default)]
    pub subscribe: bool,
    /// Whether the resource list can change dynamically.
    #[serde(default)]
    pub list_changed: bool,
}

/// Client capabilities received during initialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// Experimental capabilities (reserved for future use).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,

    /// Sampling capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<Value>,
}

/// Parameters for the initialize request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// The protocol version the client supports.
    pub protocol_version: String,
    /// Client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client information.
    pub client_info: ClientInfo,
}

/// Result of the initialize request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// The protocol version the server supports.
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Server information.
    pub server_info: ServerInfo,
}

/// A tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// The unique name of the tool.
    pub name: String,
    /// A human-readable description of the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for the tool's input parameters.
    pub input_schema: Value,
}

/// Result of tools/list request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    /// List of available tools.
    pub tools: Vec<Tool>,
}

/// A prompt definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    /// The unique name of the prompt.
    pub name: String,
    /// A human-readable description of the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Arguments that the prompt accepts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// A prompt argument definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// The name of the argument.
    pub name: String,
    /// A description of the argument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the argument is required.
    #[serde(default)]
    pub required: bool,
}

/// Result of prompts/list request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsListResult {
    /// List of available prompts.
    pub prompts: Vec<Prompt>,
}

/// Ping result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {}

// ============================================================================
// Resource Types
// ============================================================================

/// A resource definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    /// The URI of the resource.
    pub uri: String,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Result of resources/list request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListResult {
    /// List of available resources.
    pub resources: Vec<Resource>,
}

/// Parameters for resources/read request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesReadParams {
    /// The URI of the resource to read.
    pub uri: String,
}

/// Content item in resource response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContent {
    /// The URI of the resource.
    pub uri: String,
    /// MIME type of the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content (for text resources).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Binary content as base64 (for binary resources).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// Result of resources/read request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesReadResult {
    /// The content of the resource.
    pub contents: Vec<ResourceContent>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_parsing() {
        assert_eq!(McpMethod::from_str("initialize"), McpMethod::Initialize);
        assert_eq!(McpMethod::from_str("tools/list"), McpMethod::ToolsList);
        assert_eq!(
            McpMethod::from_str("unknown/method"),
            McpMethod::Unknown("unknown/method".to_string())
        );
    }

    #[test]
    fn test_initialize_result_serialization() {
        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities::default(),
            server_info: ServerInfo {
                name: "nexus".to_string(),
                version: "0.1.0".to_string(),
            },
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("protocolVersion"));
        assert!(json.contains("serverInfo"));
    }
}

