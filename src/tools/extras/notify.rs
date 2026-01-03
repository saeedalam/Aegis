//! Webhook tools for outbound notifications.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool to send a webhook notification.
#[derive(Debug)]
pub struct WebhookSendTool;

#[async_trait]
impl Tool for WebhookSendTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "webhook.send".to_string(),
            description: Some("Sends a webhook notification to a URL.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "Webhook URL"
                    },
                    "event": {
                        "type": "string",
                        "description": "Event type (e.g., 'task.completed')"
                    },
                    "data": {
                        "type": "object",
                        "description": "Event data payload"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Additional headers"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let url = arguments
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'url'".to_string()))?;

        // Substitute secrets in URL
        let url = state.secrets.substitute(url);

        let event = arguments
            .get("event")
            .and_then(|v| v.as_str())
            .unwrap_or("notification");

        let data = arguments.get("data").cloned().unwrap_or(json!({}));

        let payload = json!({
            "event": event,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": data
        });

        let client = reqwest::Client::new();
        let mut request = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "Nexus/0.2.0");

        // Add custom headers
        if let Some(headers) = arguments.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val) = value.as_str() {
                    let val = state.secrets.substitute(val);
                    request = request.header(key, val);
                }
            }
        }

        let response = request
            .json(&payload)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP error: {}", e)))?;

        let status = response.status();
        let status_code = status.as_u16();

        let result = json!({
            "success": status.is_success(),
            "status_code": status_code,
            "url": url,
            "event": event
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to send Slack notifications.
#[derive(Debug)]
pub struct SlackNotifyTool;

#[async_trait]
impl Tool for SlackNotifyTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "notify.slack".to_string(),
            description: Some(
                "Sends a Slack notification. Requires SLACK_WEBHOOK_URL secret.".to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Message text"
                    },
                    "channel": {
                        "type": "string",
                        "description": "Channel override (optional)"
                    },
                    "username": {
                        "type": "string",
                        "description": "Username override (optional)"
                    },
                    "icon_emoji": {
                        "type": "string",
                        "description": "Icon emoji (e.g., ':robot_face:')"
                    },
                    "blocks": {
                        "type": "array",
                        "description": "Slack Block Kit blocks (advanced)"
                    },
                    "webhook_url": {
                        "type": "string",
                        "description": "Webhook URL (optional, uses SLACK_WEBHOOK_URL secret)"
                    }
                },
                "required": ["text"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let webhook_url = arguments
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| state.secrets.get("SLACK_WEBHOOK_URL"))
            .ok_or_else(|| {
                ToolError::InvalidInput(
                    "No webhook URL. Set SLACK_WEBHOOK_URL secret or pass webhook_url.".to_string(),
                )
            })?;

        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'text'".to_string()))?;

        let mut payload = json!({
            "text": text
        });

        if let Some(channel) = arguments.get("channel").and_then(|v| v.as_str()) {
            payload["channel"] = json!(channel);
        }

        if let Some(username) = arguments.get("username").and_then(|v| v.as_str()) {
            payload["username"] = json!(username);
        }

        if let Some(emoji) = arguments.get("icon_emoji").and_then(|v| v.as_str()) {
            payload["icon_emoji"] = json!(emoji);
        }

        if let Some(blocks) = arguments.get("blocks") {
            payload["blocks"] = blocks.clone();
        }

        let client = reqwest::Client::new();
        let response = client
            .post(&webhook_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP error: {}", e)))?;

        let status = response.status();

        let result = json!({
            "success": status.is_success(),
            "status_code": status.as_u16()
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to send Discord notifications.
#[derive(Debug)]
pub struct DiscordNotifyTool;

#[async_trait]
impl Tool for DiscordNotifyTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "notify.discord".to_string(),
            description: Some(
                "Sends a Discord notification. Requires DISCORD_WEBHOOK_URL secret.".to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Message content"
                    },
                    "username": {
                        "type": "string",
                        "description": "Username override"
                    },
                    "avatar_url": {
                        "type": "string",
                        "description": "Avatar URL override"
                    },
                    "embeds": {
                        "type": "array",
                        "description": "Discord embeds (advanced)"
                    },
                    "webhook_url": {
                        "type": "string",
                        "description": "Webhook URL (optional, uses DISCORD_WEBHOOK_URL secret)"
                    }
                },
                "required": ["content"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let webhook_url = arguments
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| state.secrets.get("DISCORD_WEBHOOK_URL"))
            .ok_or_else(|| {
                ToolError::InvalidInput(
                    "No webhook URL. Set DISCORD_WEBHOOK_URL secret.".to_string(),
                )
            })?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'content'".to_string()))?;

        let mut payload = json!({
            "content": content
        });

        if let Some(username) = arguments.get("username").and_then(|v| v.as_str()) {
            payload["username"] = json!(username);
        }

        if let Some(avatar) = arguments.get("avatar_url").and_then(|v| v.as_str()) {
            payload["avatar_url"] = json!(avatar);
        }

        if let Some(embeds) = arguments.get("embeds") {
            payload["embeds"] = embeds.clone();
        }

        let client = reqwest::Client::new();
        let response = client
            .post(&webhook_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP error: {}", e)))?;

        let status = response.status();

        let result = json!({
            "success": status.is_success(),
            "status_code": status.as_u16()
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to send email notifications via SMTP or service.
#[derive(Debug)]
pub struct EmailNotifyTool;

#[async_trait]
impl Tool for EmailNotifyTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "notify.email".to_string(),
            description: Some(
                "Sends an email via SendGrid, Mailgun, or Resend. Requires appropriate API key secret."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "description": "Recipient email"
                    },
                    "subject": {
                        "type": "string",
                        "description": "Email subject"
                    },
                    "body": {
                        "type": "string",
                        "description": "Email body (text)"
                    },
                    "html": {
                        "type": "string",
                        "description": "Email body (HTML)"
                    },
                    "from": {
                        "type": "string",
                        "description": "Sender email (uses EMAIL_FROM secret if not provided)"
                    },
                    "provider": {
                        "type": "string",
                        "enum": ["resend", "sendgrid"],
                        "description": "Email provider (default: resend)"
                    }
                },
                "required": ["to", "subject", "body"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let provider = arguments
            .get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("resend");

        let to = arguments
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'to'".to_string()))?;

        let subject = arguments
            .get("subject")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'subject'".to_string()))?;

        let body = arguments.get("body").and_then(|v| v.as_str());
        let html = arguments.get("html").and_then(|v| v.as_str());

        if body.is_none() && html.is_none() {
            return Err(ToolError::InvalidInput(
                "Either 'body' or 'html' is required".to_string(),
            ));
        }

        let from = arguments
            .get("from")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| state.secrets.get("EMAIL_FROM"))
            .unwrap_or_else(|| "noreply@nexus.local".to_string());

        match provider {
            "resend" => {
                let api_key = state.secrets.get("RESEND_KEY").ok_or_else(|| {
                    ToolError::InvalidInput("RESEND_KEY secret not set".to_string())
                })?;

                let mut payload = json!({
                    "from": from,
                    "to": [to],
                    "subject": subject
                });

                if let Some(b) = body {
                    payload["text"] = json!(b);
                }
                if let Some(h) = html {
                    payload["html"] = json!(h);
                }

                let client = reqwest::Client::new();
                let response = client
                    .post("https://api.resend.com/emails")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(format!("HTTP error: {}", e)))?;

                let status = response.status();
                let response_body: Value = response.json().await.unwrap_or(json!({}));

                let result = json!({
                    "success": status.is_success(),
                    "provider": "resend",
                    "to": to,
                    "id": response_body.get("id")
                });

                Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
            }
            "sendgrid" => {
                let api_key = state.secrets.get("SENDGRID_KEY").ok_or_else(|| {
                    ToolError::InvalidInput("SENDGRID_KEY secret not set".to_string())
                })?;

                let mut content = vec![];
                if let Some(b) = body {
                    content.push(json!({"type": "text/plain", "value": b}));
                }
                if let Some(h) = html {
                    content.push(json!({"type": "text/html", "value": h}));
                }

                let payload = json!({
                    "personalizations": [{"to": [{"email": to}]}],
                    "from": {"email": from},
                    "subject": subject,
                    "content": content
                });

                let client = reqwest::Client::new();
                let response = client
                    .post("https://api.sendgrid.com/v3/mail/send")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(format!("HTTP error: {}", e)))?;

                let status = response.status();

                let result = json!({
                    "success": status.is_success(),
                    "provider": "sendgrid",
                    "to": to,
                    "status_code": status.as_u16()
                });

                Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
            }
            _ => Err(ToolError::InvalidInput(format!(
                "Unknown provider: {}",
                provider
            ))),
        }
    }
}

