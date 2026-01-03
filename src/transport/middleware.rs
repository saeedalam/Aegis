//! HTTP middleware for authentication, rate limiting, and observability.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use sha2::{Digest, Sha256};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use tracing::{info, warn};

use crate::core::Config;

// ============================================================================
// Authentication Middleware
// ============================================================================

/// State for authentication middleware.
#[derive(Clone)]
pub struct AuthState {
    pub config: Arc<Config>,
}

/// Authentication middleware that checks for valid API keys.
pub async fn auth_middleware(
    State(state): State<AuthState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Skip auth if disabled
    if !state.config.auth.enabled {
        return next.run(request).await;
    }

    // Allow health endpoint without auth
    let path = request.uri().path();
    if state.config.auth.allow_health_unauthenticated && path == "/health" {
        return next.run(request).await;
    }

    // Get API key from header
    let api_key = request
        .headers()
        .get(&state.config.auth.api_key_header)
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) => {
            // Hash the provided key
            let mut hasher = Sha256::new();
            hasher.update(key.as_bytes());
            let hash = hex::encode(hasher.finalize());

            // Check if hash matches any configured key
            if state.config.auth.api_keys.contains(&hash) {
                next.run(request).await
            } else {
                warn!("Invalid API key attempted");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Invalid API key"
                    })),
                )
                    .into_response()
            }
        }
        None => {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Missing API key",
                    "header": state.config.auth.api_key_header
                })),
            )
                .into_response()
        }
    }
}

// ============================================================================
// Rate Limiting
// ============================================================================

/// Rate limiter using token bucket algorithm.
#[derive(Clone)]
pub struct RateLimiter {
    /// Tokens per client IP
    buckets: Arc<DashMap<String, TokenBucket>>,
    /// Requests per second
    rate: f64,
    /// Maximum burst size
    burst: u32,
    /// Whether enabled
    enabled: bool,
}

struct TokenBucket {
    tokens: f64,
    last_update: Instant,
}

impl RateLimiter {
    pub fn new(config: &Config) -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            rate: config.rate_limit.requests_per_second as f64,
            burst: config.rate_limit.burst_size,
            enabled: config.rate_limit.enabled,
        }
    }

    pub fn check(&self, client_id: &str) -> bool {
        if !self.enabled {
            return true;
        }

        let now = Instant::now();
        let mut bucket = self.buckets.entry(client_id.to_string()).or_insert_with(|| {
            TokenBucket {
                tokens: self.burst as f64,
                last_update: now,
            }
        });

        // Refill tokens based on time elapsed
        let elapsed = now.duration_since(bucket.last_update).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.rate).min(self.burst as f64);
        bucket.last_update = now;

        // Try to consume a token
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Cleanup old entries (call periodically)
    pub fn cleanup(&self, max_age: Duration) {
        let now = Instant::now();
        self.buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_update) < max_age
        });
    }
}

/// Rate limit state.
#[derive(Clone)]
pub struct RateLimitState {
    pub limiter: RateLimiter,
}

/// Rate limiting middleware.
pub async fn rate_limit_middleware(
    State(state): State<RateLimitState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Get client identifier (IP or header)
    let client_id = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("unknown").trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if state.limiter.check(&client_id) {
        next.run(request).await
    } else {
        warn!("Rate limit exceeded for client: {}", client_id);
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "Rate limit exceeded",
                "retry_after": 1
            })),
        )
            .into_response()
    }
}

// ============================================================================
// Request Logging Middleware
// ============================================================================

/// Request logging middleware with timing.
pub async fn logging_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    info!(
        method = %method,
        path = %path,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}

// ============================================================================
// Metrics
// ============================================================================

/// Simple metrics collector.
#[derive(Clone, Default)]
pub struct Metrics {
    requests: Arc<DashMap<String, u64>>,
    errors: Arc<DashMap<String, u64>>,
    tool_calls: Arc<DashMap<String, u64>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_request(&self, path: &str) {
        *self.requests.entry(path.to_string()).or_insert(0) += 1;
    }

    pub fn record_error(&self, error_type: &str) {
        *self.errors.entry(error_type.to_string()).or_insert(0) += 1;
    }

    pub fn record_tool_call(&self, tool_name: &str) {
        *self.tool_calls.entry(tool_name.to_string()).or_insert(0) += 1;
    }

    pub fn snapshot(&self) -> serde_json::Value {
        let requests: std::collections::HashMap<String, u64> = 
            self.requests.iter().map(|r| (r.key().clone(), *r.value())).collect();
        let errors: std::collections::HashMap<String, u64> = 
            self.errors.iter().map(|r| (r.key().clone(), *r.value())).collect();
        let tool_calls: std::collections::HashMap<String, u64> = 
            self.tool_calls.iter().map(|r| (r.key().clone(), *r.value())).collect();

        json!({
            "requests": requests,
            "errors": errors,
            "tool_calls": tool_calls,
            "total_requests": requests.values().sum::<u64>(),
            "total_errors": errors.values().sum::<u64>(),
            "total_tool_calls": tool_calls.values().sum::<u64>()
        })
    }
}


