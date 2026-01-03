//! Configuration management for Nexus.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration for Nexus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server name reported during initialization.
    #[serde(default = "default_server_name")]
    pub server_name: String,

    /// Server version reported during initialization.
    #[serde(default = "default_server_version")]
    pub server_version: String,

    /// Host address for SSE transport.
    #[serde(default = "default_host")]
    pub host: String,

    /// Port for SSE transport.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable structured JSON logging.
    #[serde(default)]
    pub json_logs: bool,

    /// Security configuration for tools.
    #[serde(default)]
    pub security: SecurityConfig,

    /// Authentication configuration.
    #[serde(default)]
    pub auth: AuthConfig,

    /// Rate limiting configuration.
    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// HTTP client configuration (for http.request tool).
    #[serde(default)]
    pub http_client: HttpClientConfig,

    /// Path to the SQLite database file. Use ":memory:" for in-memory.
    #[serde(default)]
    pub database_path: Option<String>,

    /// Custom tool plugins.
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,

    /// Enable extra tools (LLM, vector, git, notifications, etc.)
    /// Default: true for backwards compatibility
    #[serde(default = "default_extras_enabled")]
    pub extras_enabled: bool,
}

fn default_extras_enabled() -> bool {
    true // Enable by default for backwards compatibility
}

/// Configuration for a custom tool plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Unique name for the tool.
    pub name: String,

    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,

    /// Command to execute (path to script or binary).
    pub command: String,

    /// Arguments template. Use ${param} for substitution.
    #[serde(default)]
    pub args_template: Vec<String>,

    /// Working directory for the command.
    #[serde(default)]
    pub working_dir: Option<String>,

    /// Environment variables to set.
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Timeout in seconds (default: 30).
    #[serde(default = "default_plugin_timeout")]
    pub timeout_secs: u64,

    /// JSON Schema for input parameters.
    #[serde(default = "default_plugin_schema")]
    pub input_schema: serde_json::Value,

    /// How to pass input to the script: "args", "stdin", or "env".
    #[serde(default)]
    pub input_mode: String,

    /// How to parse output: "text" or "json".
    #[serde(default)]
    pub output_mode: String,
}

fn default_plugin_timeout() -> u64 { 30 }
fn default_plugin_schema() -> serde_json::Value {
    serde_json::json!({ "type": "object", "properties": {} })
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable API key authentication for HTTP endpoints.
    #[serde(default)]
    pub enabled: bool,

    /// List of valid API keys (hashed with SHA-256).
    /// Generate with: echo -n "your-api-key" | sha256sum
    #[serde(default)]
    pub api_keys: Vec<String>,

    /// Allow unauthenticated access to health endpoint.
    #[serde(default = "default_true")]
    pub allow_health_unauthenticated: bool,

    /// Header name for API key (default: X-API-Key).
    #[serde(default = "default_api_key_header")]
    pub api_key_header: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_keys: vec![],
            allow_health_unauthenticated: true,
            api_key_header: default_api_key_header(),
        }
    }
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting.
    #[serde(default)]
    pub enabled: bool,

    /// Requests per second per client.
    #[serde(default = "default_requests_per_second")]
    pub requests_per_second: u32,

    /// Burst size (max requests in burst).
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_second: default_requests_per_second(),
            burst_size: default_burst_size(),
        }
    }
}

/// HTTP client configuration for the http.request tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    /// Request timeout in seconds.
    #[serde(default = "default_http_timeout")]
    pub timeout_secs: u64,

    /// Maximum response body size in bytes (default 10MB).
    #[serde(default = "default_max_response_size")]
    pub max_response_bytes: usize,

    /// Allowed URL patterns (regex). Empty = allow all.
    #[serde(default)]
    pub allowed_urls: Vec<String>,

    /// Blocked URL patterns (regex). Takes precedence over allowed.
    #[serde(default)]
    pub blocked_urls: Vec<String>,

    /// User-Agent header for requests.
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_http_timeout(),
            max_response_bytes: default_max_response_size(),
            allowed_urls: vec![],
            blocked_urls: vec![
                // Block internal/private networks by default
                r"^https?://localhost".to_string(),
                r"^https?://127\.".to_string(),
                r"^https?://10\.".to_string(),
                r"^https?://172\.(1[6-9]|2[0-9]|3[01])\.".to_string(),
                r"^https?://192\.168\.".to_string(),
            ],
            user_agent: default_user_agent(),
        }
    }
}

fn default_true() -> bool { true }
fn default_api_key_header() -> String { "X-API-Key".to_string() }
fn default_requests_per_second() -> u32 { 100 }
fn default_burst_size() -> u32 { 50 }
fn default_http_timeout() -> u64 { 30 }
fn default_max_response_size() -> usize { 10 * 1024 * 1024 } // 10MB
fn default_user_agent() -> String { format!("Nexus/{}", env!("CARGO_PKG_VERSION")) }

/// Security configuration for tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Paths that fs.read_file can access.
    #[serde(default)]
    pub allowed_read_paths: Vec<PathBuf>,

    /// Paths that fs.write_file can access.
    #[serde(default)]
    pub allowed_write_paths: Vec<PathBuf>,

    /// Commands that cmd.exec can run (supports wildcards like "git*").
    #[serde(default)]
    pub allowed_commands: Vec<String>,

    /// Default timeout for tool execution in seconds.
    #[serde(default = "default_tool_timeout")]
    pub tool_timeout_secs: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            // By default, allow reading current directory
            allowed_read_paths: vec![PathBuf::from(".")],
            // By default, no write access
            allowed_write_paths: vec![],
            // By default, allow safe read-only commands
            allowed_commands: vec![
                "echo".to_string(),
                "date".to_string(),
                "whoami".to_string(),
                "pwd".to_string(),
                "ls".to_string(),
                "cat".to_string(),
                "head".to_string(),
                "tail".to_string(),
                "wc".to_string(),
            ],
            tool_timeout_secs: default_tool_timeout(),
        }
    }
}

fn default_server_name() -> String {
    "aegis".to_string()
}

fn default_server_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    9000
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_tool_timeout() -> u64 {
    30
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_name: default_server_name(),
            server_version: default_server_version(),
            host: default_host(),
            port: default_port(),
            log_level: default_log_level(),
            json_logs: false,
            security: SecurityConfig::default(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            http_client: HttpClientConfig::default(),
            database_path: None,
            plugins: vec![],
            extras_enabled: default_extras_enabled(),
        }
    }
}

impl Config {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a JSON file.
    pub fn load_from_file(path: &PathBuf) -> Result<Self, crate::core::NexusError> {
        if !path.exists() {
            tracing::info!("Config file not found, using defaults");
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::core::NexusError::Config(format!("Failed to read config: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| crate::core::NexusError::Config(format!("Failed to parse config: {}", e)))
    }

    /// Returns the socket address for SSE transport.
    pub fn socket_addr(&self) -> std::net::SocketAddr {
        use std::net::{IpAddr, SocketAddr};
        let ip: IpAddr = self.host.parse().unwrap_or([127, 0, 0, 1].into());
        SocketAddr::new(ip, self.port)
    }
}
