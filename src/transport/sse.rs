//! SSE (Server-Sent Events) transport via Axum.
//!
//! This module provides an HTTP-based transport for MCP using
//! Server-Sent Events for streaming responses to clients.

use axum::{
    extract::State,
    middleware as axum_mw,
    response::Sse,
    routing::{get, post},
    Json, Router,
};
use futures::stream;
use serde_json::Value;
use std::convert::Infallible;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info};

use crate::core::{Config, AegisError, AegisResult, RuntimeState};
use crate::dashboard::dashboard_routes;
use crate::handlers::Router as McpRouter;
use crate::protocol::{Request, Response, RequestId, ErrorObject};
use crate::transport::middleware::{
    AuthState, RateLimiter, RateLimitState, Metrics,
    auth_middleware, rate_limit_middleware, logging_middleware,
};

/// Shared state for the SSE server.
#[derive(Clone)]
pub struct SseState {
    /// The runtime state.
    pub runtime: Arc<RuntimeState>,
    /// The MCP request router.
    pub router: Arc<McpRouter>,
    /// Metrics collector.
    pub metrics: Metrics,
}

/// Creates the Axum router for SSE transport.
pub fn create_router(state: SseState, config: &Config) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create auth state
    let auth_state = AuthState {
        config: Arc::new(config.clone()),
    };

    // Create rate limiter state
    let rate_limit_state = RateLimitState {
        limiter: RateLimiter::new(config),
    };

    // Build dashboard routes separately (has its own state)
    let dashboard = dashboard_routes(state.runtime.clone());
    
    // Build main router with middleware layers
    let mut router = Router::new()
        .route("/health", get(health_handler))
        .route("/mcp", post(mcp_handler))
        .route("/sse", get(sse_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
        .nest("/dashboard", dashboard);

    // Add rate limiting (if enabled)
    if config.rate_limit.enabled {
        router = router.layer(axum_mw::from_fn_with_state(
            rate_limit_state,
            rate_limit_middleware,
        ));
    }

    // Add authentication (if enabled)
    if config.auth.enabled {
        router = router.layer(axum_mw::from_fn_with_state(
            auth_state,
            auth_middleware,
        ));
    }

    // Add logging and CORS
    router = router
        .layer(axum_mw::from_fn(logging_middleware))
        .layer(cors);

    router
}

/// Health check endpoint.
#[axum::debug_handler]
async fn health_handler() -> Json<Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "aegis",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Metrics endpoint.
#[axum::debug_handler]
async fn metrics_handler(
    State(state): State<SseState>,
) -> Json<Value> {
    Json(state.metrics.snapshot())
}

/// Main MCP endpoint - handles JSON-RPC requests.
#[axum::debug_handler]
async fn mcp_handler(
    State(state): State<SseState>,
    Json(body): Json<Value>,
) -> Json<Value> {
    debug!("Received MCP request: {:?}", body);

    // Parse the request
    let request: Request = match serde_json::from_value(body) {
        Ok(req) => req,
        Err(e) => {
            error!("Failed to parse request: {}", e);
            let error_response = Response::error(
                RequestId::Null,
                ErrorObject::parse_error(e.to_string()),
            );
            return Json(serde_json::to_value(error_response).unwrap_or_default());
        }
    };

    // Validate the request
    if let Err(e) = request.validate() {
        error!("Invalid request: {}", e);
        let error_response = Response::from_error(request.id.clone(), &e);
        return Json(serde_json::to_value(error_response).unwrap_or_default());
    }

    // Route and handle the request
    let response = state.router.handle(request, state.runtime.clone()).await;
    Json(serde_json::to_value(response).unwrap_or_default())
}

/// SSE endpoint for streaming (placeholder for future implementation).
async fn sse_handler(
    State(_state): State<SseState>,
) -> Sse<impl futures::Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    // For MVP, we return a simple stream that sends a ping every 30 seconds
    let stream = stream::unfold(0u64, |counter| async move {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        let event = axum::response::sse::Event::default()
            .event("ping")
            .data(format!("{{\"count\": {}}}", counter));
        Some((Ok::<_, Infallible>(event), counter + 1))
    });

    Sse::new(stream)
}

/// Starts the SSE server.
pub async fn start_server(state: SseState, config: &Config, addr: std::net::SocketAddr) -> AegisResult<()> {
    info!("Starting SSE server on http://{}", addr);

    let router = create_router(state, config);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AegisError::Transport(format!("Failed to bind: {}", e)))?;

    // Log security status
    if config.auth.enabled {
        info!("🔐 Authentication enabled ({} API keys configured)", config.auth.api_keys.len());
    } else {
        info!("⚠️  Authentication disabled - server is open");
    }

    if config.rate_limit.enabled {
        info!("🚦 Rate limiting enabled ({} req/s, burst: {})", 
              config.rate_limit.requests_per_second,
              config.rate_limit.burst_size);
    }

    info!("🟢 Aegis SSE server listening on http://{}", addr);
    info!("📊 Dashboard available at http://{}/dashboard", addr);

    axum::serve(listener, router)
        .await
        .map_err(|e| AegisError::Transport(format!("Server error: {}", e)))?;

    Ok(())
}
