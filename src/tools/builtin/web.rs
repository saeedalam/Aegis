//! Web extraction tools for fetching and parsing web content.

use async_trait::async_trait;
use regex::Regex;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool to fetch and extract content from web pages.
#[derive(Debug)]
pub struct WebExtractTool;

#[async_trait]
impl Tool for WebExtractTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web.extract".to_string(),
            description: Some(
                "Fetches a web page and extracts clean text content. Removes HTML tags, scripts, and styles."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to fetch"
                    },
                    "selector": {
                        "type": "string",
                        "description": "CSS-like selector hint (e.g., 'article', 'main', 'body'). Not full CSS - just element name."
                    },
                    "format": {
                        "type": "string",
                        "enum": ["text", "html", "markdown", "links"],
                        "description": "Output format (default: text)"
                    },
                    "max_length": {
                        "type": "integer",
                        "description": "Max characters to return (default: 50000)"
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
            .ok_or_else(|| ToolError::InvalidInput("Missing 'url'".to_string()))?;

        let format = arguments
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        let max_length = arguments
            .get("max_length")
            .and_then(|v| v.as_u64())
            .unwrap_or(50000) as usize;

        let selector = arguments
            .get("selector")
            .and_then(|v| v.as_str());

        // Fetch the page
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; NexusBot/1.0)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ToolError::ExecutionFailed(format!("Client error: {}", e)))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Fetch error: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            return Err(ToolError::ExecutionFailed(format!(
                "HTTP {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let html = response
            .text()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Read error: {}", e)))?;

        // Extract based on selector
        let html_to_parse = if let Some(sel) = selector {
            extract_by_tag(&html, sel).unwrap_or_else(|| html.clone())
        } else {
            // Try to find main content
            extract_by_tag(&html, "article")
                .or_else(|| extract_by_tag(&html, "main"))
                .or_else(|| extract_by_tag(&html, "body"))
                .unwrap_or_else(|| html.clone())
        };

        let result = match format {
            "html" => {
                let mut output = html_to_parse;
                if output.len() > max_length {
                    output.truncate(max_length);
                    output.push_str("...[truncated]");
                }
                json!({
                    "url": url,
                    "content_type": content_type,
                    "format": "html",
                    "content": output
                })
            }
            "links" => {
                let links = extract_links(&html, url);
                json!({
                    "url": url,
                    "format": "links",
                    "count": links.len(),
                    "links": links
                })
            }
            "markdown" => {
                let text = html_to_text(&html_to_parse);
                let markdown = text_to_markdown(&text);
                let mut output = markdown;
                if output.len() > max_length {
                    output.truncate(max_length);
                    output.push_str("\n\n...[truncated]");
                }
                json!({
                    "url": url,
                    "format": "markdown",
                    "length": output.len(),
                    "content": output
                })
            }
            _ => {
                // Default: text
                let mut text = html_to_text(&html_to_parse);
                if text.len() > max_length {
                    text.truncate(max_length);
                    text.push_str("...[truncated]");
                }
                json!({
                    "url": url,
                    "format": "text",
                    "length": text.len(),
                    "content": text
                })
            }
        };

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Extract content within a specific HTML tag.
fn extract_by_tag(html: &str, tag: &str) -> Option<String> {
    let open_pattern = format!("<{}[^>]*>", regex::escape(tag));
    let close_pattern = format!("</{}>", regex::escape(tag));

    let open_re = Regex::new(&open_pattern).ok()?;
    let close_re = Regex::new(&close_pattern).ok()?;

    let start_match = open_re.find(html)?;
    let start = start_match.end();

    let search_area = &html[start..];
    let end_match = close_re.find(search_area)?;
    let end = start + end_match.start();

    Some(html[start..end].to_string())
}

/// Convert HTML to plain text.
fn html_to_text(html: &str) -> String {
    let mut text = html.to_string();

    // Remove script and style tags with content
    let script_re = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    text = script_re.replace_all(&text, "").to_string();

    let style_re = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    text = style_re.replace_all(&text, "").to_string();

    // Remove comments
    let comment_re = Regex::new(r"<!--.*?-->").unwrap();
    text = comment_re.replace_all(&text, "").to_string();

    // Convert common elements to newlines
    let block_re = Regex::new(r"(?i)</(p|div|h[1-6]|li|tr|br|hr)[^>]*>").unwrap();
    text = block_re.replace_all(&text, "\n").to_string();

    let br_re = Regex::new(r"(?i)<br[^>]*/?>").unwrap();
    text = br_re.replace_all(&text, "\n").to_string();

    // Remove all HTML tags
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    text = tag_re.replace_all(&text, "").to_string();

    // Decode common HTML entities
    text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");

    // Decode numeric entities
    let numeric_re = Regex::new(r"&#(\d+);").unwrap();
    text = numeric_re
        .replace_all(&text, |caps: &regex::Captures| {
            caps.get(1)
                .and_then(|m| m.as_str().parse::<u32>().ok())
                .and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default()
        })
        .to_string();

    // Collapse whitespace
    let ws_re = Regex::new(r"[ \t]+").unwrap();
    text = ws_re.replace_all(&text, " ").to_string();

    // Collapse multiple newlines
    let nl_re = Regex::new(r"\n\s*\n+").unwrap();
    text = nl_re.replace_all(&text, "\n\n").to_string();

    text.trim().to_string()
}

/// Convert text to basic markdown format.
fn text_to_markdown(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            result.push(String::new());
        } else {
            result.push(trimmed.to_string());
        }
    }

    result.join("\n")
}

/// Extract all links from HTML.
fn extract_links(html: &str, base_url: &str) -> Vec<Value> {
    let link_re = Regex::new(r#"<a[^>]+href=["']([^"']+)["'][^>]*>([^<]*)</a>"#).unwrap();
    let mut links = Vec::new();

    for cap in link_re.captures_iter(html) {
        let href = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let text = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();

        // Resolve relative URLs
        let full_url = if href.starts_with("http://") || href.starts_with("https://") {
            href.to_string()
        } else if href.starts_with("//") {
            format!("https:{}", href)
        } else if href.starts_with('/') {
            // Get base domain
            if let Some(domain_end) = base_url
                .find("://")
                .and_then(|i| base_url[i + 3..].find('/').map(|j| i + 3 + j))
            {
                format!("{}{}", &base_url[..domain_end], href)
            } else {
                format!("{}{}", base_url.trim_end_matches('/'), href)
            }
        } else {
            format!("{}/{}", base_url.trim_end_matches('/'), href)
        };

        if !href.is_empty() && !href.starts_with('#') && !href.starts_with("javascript:") {
            links.push(json!({
                "href": full_url,
                "text": text
            }));
        }
    }

    links
}

/// Tool to search the web using a search engine.
#[derive(Debug)]
pub struct WebSearchTool;

#[async_trait]
impl Tool for WebSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web.search".to_string(),
            description: Some(
                "Searches the web using DuckDuckGo. Returns search results with titles and URLs."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results (default: 10)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'query'".to_string()))?;

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        // Use DuckDuckGo HTML interface
        let encoded_query = urlencoding::encode(query);
        let url = format!("https://html.duckduckgo.com/html/?q={}", encoded_query);

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; NexusBot/1.0)")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| ToolError::ExecutionFailed(format!("Client error: {}", e)))?;

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Search error: {}", e)))?;

        let html = response
            .text()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Read error: {}", e)))?;

        // Parse DuckDuckGo results
        let results = parse_ddg_results(&html, limit);

        let output = json!({
            "query": query,
            "count": results.len(),
            "results": results
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&output).unwrap()))
    }
}

/// Parse DuckDuckGo HTML search results.
fn parse_ddg_results(html: &str, limit: usize) -> Vec<Value> {
    let mut results = Vec::new();

    // DuckDuckGo result pattern
    let result_re = Regex::new(
        r#"class="result__a"[^>]*href="([^"]+)"[^>]*>([^<]+)</a>.*?class="result__snippet"[^>]*>([^<]+)"#
    ).ok();

    // Simpler fallback pattern
    let simple_re = Regex::new(
        r#"<a[^>]+class="result__a"[^>]*href="([^"]+)"[^>]*>([^<]+)</a>"#
    ).ok();

    if let Some(re) = result_re {
        for cap in re.captures_iter(html) {
            if results.len() >= limit {
                break;
            }

            let url = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let title = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();
            let snippet = cap.get(3).map(|m| m.as_str()).unwrap_or("").trim();

            // Decode DuckDuckGo redirect URL
            let actual_url = decode_ddg_url(url);

            if !actual_url.is_empty() {
                results.push(json!({
                    "title": html_to_text(title),
                    "url": actual_url,
                    "snippet": html_to_text(snippet)
                }));
            }
        }
    }

    // Fallback if no results with snippets
    if results.is_empty() {
        if let Some(re) = simple_re {
            for cap in re.captures_iter(html) {
                if results.len() >= limit {
                    break;
                }

                let url = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let title = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();

                let actual_url = decode_ddg_url(url);

                if !actual_url.is_empty() {
                    results.push(json!({
                        "title": html_to_text(title),
                        "url": actual_url,
                        "snippet": ""
                    }));
                }
            }
        }
    }

    results
}

/// Decode DuckDuckGo redirect URL.
fn decode_ddg_url(url: &str) -> String {
    // DuckDuckGo uses //duckduckgo.com/l/?uddg=ENCODED_URL
    if url.contains("uddg=") {
        if let Some(start) = url.find("uddg=") {
            let encoded = &url[start + 5..];
            let end = encoded.find('&').unwrap_or(encoded.len());
            let encoded = &encoded[..end];
            return urlencoding::decode(encoded)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| url.to_string());
        }
    }
    url.to_string()
}


