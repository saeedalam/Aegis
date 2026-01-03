//! HTTP request tool for making web requests.

use async_trait::async_trait;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::core::{Config, RuntimeState};
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool for making HTTP requests.
#[derive(Debug)]
pub struct HttpRequestTool {
    client: reqwest::Client,
    config: Config,
}

impl HttpRequestTool {
    /// Creates a new HTTP request tool with the given configuration.
    pub fn new(config: &Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.http_client.timeout_secs))
            .user_agent(&config.http_client.user_agent)
            .build()
            .unwrap_or_default();

        Self {
            client,
            config: config.clone(),
        }
    }

    fn is_url_allowed(&self, url: &str) -> Result<(), ToolError> {
        // Check blocked patterns first
        for pattern in &self.config.http_client.blocked_urls {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(url) {
                    return Err(ToolError::PermissionDenied(format!(
                        "URL blocked by pattern: {}",
                        pattern
                    )));
                }
            }
        }

        // If allowed_urls is empty, allow all (except blocked)
        if self.config.http_client.allowed_urls.is_empty() {
            return Ok(());
        }

        // Check allowed patterns
        for pattern in &self.config.http_client.allowed_urls {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(url) {
                    return Ok(());
                }
            }
        }

        Err(ToolError::PermissionDenied(format!(
            "URL not in allowed list: {}",
            url
        )))
    }
}

#[async_trait]
impl Tool for HttpRequestTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "http.request".to_string(),
            description: Some(
                "Makes an HTTP request to a URL. Supports GET, POST, PUT, DELETE, PATCH methods."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to request"
                    },
                    "method": {
                        "type": "string",
                        "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"],
                        "default": "GET",
                        "description": "HTTP method"
                    },
                    "headers": {
                        "type": "object",
                        "additionalProperties": { "type": "string" },
                        "description": "Request headers"
                    },
                    "body": {
                        "type": "string",
                        "description": "Request body (for POST/PUT/PATCH)"
                    },
                    "json": {
                        "type": "object",
                        "description": "JSON body (alternative to body, sets Content-Type)"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let url = arguments
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'url' parameter".to_string()))?;

        // Validate URL
        self.is_url_allowed(url)?;

        let method = arguments
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_uppercase();

        // Build request
        let mut request = match method.as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            "HEAD" => self.client.head(url),
            _ => {
                return Err(ToolError::InvalidInput(format!(
                    "Invalid HTTP method: {}",
                    method
                )))
            }
        };

        // Add headers
        if let Some(headers) = arguments.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val) = value.as_str() {
                    request = request.header(key, val);
                }
            }
        }

        // Add body
        if let Some(json_body) = arguments.get("json") {
            request = request.json(json_body);
        } else if let Some(body) = arguments.get("body").and_then(|v| v.as_str()) {
            request = request.body(body.to_string());
        }

        // Execute request
        let response = request.send().await.map_err(|e| {
            ToolError::ExecutionFailed(format!("HTTP request failed: {}", e))
        })?;

        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason().unwrap_or("Unknown");

        // Get headers
        let mut response_headers: HashMap<String, String> = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                response_headers.insert(key.to_string(), v.to_string());
            }
        }

        // Get body with size limit
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read response: {}", e)))?;

        if body_bytes.len() > self.config.http_client.max_response_bytes {
            return Err(ToolError::ExecutionFailed(format!(
                "Response too large: {} bytes (max: {})",
                body_bytes.len(),
                self.config.http_client.max_response_bytes
            )));
        }

        // Try to parse as text
        let body = String::from_utf8_lossy(&body_bytes).to_string();

        // Try to parse as JSON for pretty output
        let body_json: Option<Value> = serde_json::from_str(&body).ok();

        let result = json!({
            "status": status,
            "statusText": status_text,
            "headers": response_headers,
            "body": body_json.unwrap_or_else(|| Value::String(body)),
            "size": body_bytes.len()
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}


