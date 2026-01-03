//! LLM integration tools for AI providers.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool to call OpenAI API.
#[derive(Debug)]
pub struct OpenAiChatTool;

#[async_trait]
impl Tool for OpenAiChatTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "llm.openai".to_string(),
            description: Some(
                "Calls OpenAI Chat API. Requires OPENAI_KEY secret or api_key parameter."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "messages": {
                        "type": "array",
                        "description": "Array of message objects with 'role' and 'content'",
                        "items": {
                            "type": "object",
                            "properties": {
                                "role": {"type": "string", "enum": ["system", "user", "assistant"]},
                                "content": {"type": "string"}
                            }
                        }
                    },
                    "prompt": {
                        "type": "string",
                        "description": "Simple prompt (alternative to messages array)"
                    },
                    "model": {
                        "type": "string",
                        "description": "Model to use (default: gpt-4o-mini)"
                    },
                    "temperature": {
                        "type": "number",
                        "description": "Temperature (0-2, default: 0.7)"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Max tokens to generate"
                    },
                    "api_key": {
                        "type": "string",
                        "description": "API key (optional, uses OPENAI_KEY secret if not provided)"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        // Get API key from arguments or secrets
        let api_key = arguments
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| state.secrets.get("OPENAI_KEY"))
            .ok_or_else(|| {
                ToolError::InvalidInput(
                    "No API key provided. Set OPENAI_KEY secret or pass api_key parameter."
                        .to_string(),
                )
            })?;

        // Build messages
        let messages: Vec<Value> = if let Some(msgs) = arguments.get("messages") {
            msgs.as_array()
                .ok_or_else(|| ToolError::InvalidInput("messages must be an array".to_string()))?
                .clone()
        } else if let Some(prompt) = arguments.get("prompt").and_then(|v| v.as_str()) {
            vec![json!({"role": "user", "content": prompt})]
        } else {
            return Err(ToolError::InvalidInput(
                "Either 'messages' or 'prompt' is required".to_string(),
            ));
        };

        let model = arguments
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("gpt-4o-mini");

        let temperature = arguments
            .get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);

        let max_tokens = arguments.get("max_tokens").and_then(|v| v.as_u64());

        // Build request
        let mut request_body = json!({
            "model": model,
            "messages": messages,
            "temperature": temperature
        });

        if let Some(mt) = max_tokens {
            request_body["max_tokens"] = json!(mt);
        }

        // Make request
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP error: {}", e)))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse response: {}", e)))?;

        if !status.is_success() {
            let error_msg = body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ToolError::ExecutionFailed(format!(
                "OpenAI API error: {}",
                error_msg
            )));
        }

        // Extract response
        let content = body
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("");

        let usage = body.get("usage").cloned().unwrap_or(json!({}));

        let result = json!({
            "content": content,
            "model": model,
            "usage": usage
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to call Anthropic Claude API.
#[derive(Debug)]
pub struct AnthropicChatTool;

#[async_trait]
impl Tool for AnthropicChatTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "llm.anthropic".to_string(),
            description: Some(
                "Calls Anthropic Claude API. Requires ANTHROPIC_KEY secret or api_key parameter."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "messages": {
                        "type": "array",
                        "description": "Array of message objects with 'role' and 'content'",
                        "items": {
                            "type": "object",
                            "properties": {
                                "role": {"type": "string", "enum": ["user", "assistant"]},
                                "content": {"type": "string"}
                            }
                        }
                    },
                    "prompt": {
                        "type": "string",
                        "description": "Simple prompt (alternative to messages array)"
                    },
                    "system": {
                        "type": "string",
                        "description": "System prompt"
                    },
                    "model": {
                        "type": "string",
                        "description": "Model to use (default: claude-3-haiku-20240307)"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Max tokens to generate (default: 1024)"
                    },
                    "api_key": {
                        "type": "string",
                        "description": "API key (optional, uses ANTHROPIC_KEY secret if not provided)"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        // Get API key
        let api_key = arguments
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| state.secrets.get("ANTHROPIC_KEY"))
            .ok_or_else(|| {
                ToolError::InvalidInput(
                    "No API key provided. Set ANTHROPIC_KEY secret or pass api_key parameter."
                        .to_string(),
                )
            })?;

        // Build messages
        let messages: Vec<Value> = if let Some(msgs) = arguments.get("messages") {
            msgs.as_array()
                .ok_or_else(|| ToolError::InvalidInput("messages must be an array".to_string()))?
                .clone()
        } else if let Some(prompt) = arguments.get("prompt").and_then(|v| v.as_str()) {
            vec![json!({"role": "user", "content": prompt})]
        } else {
            return Err(ToolError::InvalidInput(
                "Either 'messages' or 'prompt' is required".to_string(),
            ));
        };

        let model = arguments
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("claude-3-haiku-20240307");

        let max_tokens = arguments
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024);

        let system = arguments.get("system").and_then(|v| v.as_str());

        // Build request
        let mut request_body = json!({
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens
        });

        if let Some(sys) = system {
            request_body["system"] = json!(sys);
        }

        // Make request
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP error: {}", e)))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse response: {}", e)))?;

        if !status.is_success() {
            let error_msg = body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ToolError::ExecutionFailed(format!(
                "Anthropic API error: {}",
                error_msg
            )));
        }

        // Extract response
        let content = body
            .get("content")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let usage = body.get("usage").cloned().unwrap_or(json!({}));

        let result = json!({
            "content": content,
            "model": model,
            "usage": usage
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to generate embeddings using OpenAI.
#[derive(Debug)]
pub struct EmbeddingsTool;

#[async_trait]
impl Tool for EmbeddingsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "llm.embed".to_string(),
            description: Some("Generates text embeddings using OpenAI. Useful for semantic search.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to embed"
                    },
                    "texts": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Multiple texts to embed"
                    },
                    "model": {
                        "type": "string",
                        "description": "Model to use (default: text-embedding-3-small)"
                    },
                    "api_key": {
                        "type": "string",
                        "description": "API key (optional, uses OPENAI_KEY secret if not provided)"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let api_key = arguments
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| state.secrets.get("OPENAI_KEY"))
            .ok_or_else(|| {
                ToolError::InvalidInput("No API key. Set OPENAI_KEY secret.".to_string())
            })?;

        let texts: Vec<String> = if let Some(text) = arguments.get("text").and_then(|v| v.as_str()) {
            vec![text.to_string()]
        } else if let Some(arr) = arguments.get("texts").and_then(|v| v.as_array()) {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            return Err(ToolError::InvalidInput("Either 'text' or 'texts' is required".to_string()));
        };

        let model = arguments
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("text-embedding-3-small");

        let request_body = json!({
            "model": model,
            "input": texts
        });

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP error: {}", e)))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Parse error: {}", e)))?;

        if !status.is_success() {
            let error_msg = body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ToolError::ExecutionFailed(format!("OpenAI error: {}", error_msg)));
        }

        let embeddings: Vec<Value> = body
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|item| {
                        json!({
                            "index": item.get("index"),
                            "embedding": item.get("embedding"),
                            "dimensions": item.get("embedding").and_then(|e| e.as_array()).map(|a| a.len())
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let result = json!({
            "model": model,
            "embeddings": embeddings,
            "usage": body.get("usage")
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}


