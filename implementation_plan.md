# Implementation Plan - Phase 1: The Skeleton & Protocol

## Goal
Establish the project foundation and implement the basic network transport for the Model Context Protocol (MCP). By the end of this phase, we will have a compiled binary that runs and can accept a basic JSON-RPC handshake.

## User Review Required
> [!IMPORTANT]
> **Transport Decision:** We will implementing BOTH `stdio` (for local agent CLI usage) and `SSE` (Server-Sent Events via Axum) for remote connections. This covers the most common MCP use cases.

## Proposed Changes

### 1. Project Initialization
#### [NEW] [Cargo.toml](file:///Users/monitum/.gemini/antigravity/brain/673cd3ec-2ffa-40ac-9faf-95e16bd9df57/nexus/Cargo.toml)
- Initialize new Rust binary crate `nexus`.
- Add dependencies:
    - `tokio` (full features) for async runtime.
    - `axum` for HTTP/SSE server.
    - `serde`, `serde_json` for JSON-RPC.
    - `tracing`, `tracing-subscriber` for observability.
    - `clap` for CLI argument parsing (`--stdio` vs `--port 9000`).
    - `thiserror` for error handling.

### 2. Core Structure
#### [NEW] [src/main.rs](file:///Users/monitum/.gemini/antigravity/brain/673cd3ec-2ffa-40ac-9faf-95e16bd9df57/nexus/src/main.rs)
- CLI entry point.
- Logic to switch between Stdio mode and HTTP mode based on flags.

#### [NEW] [src/server/mod.rs](file:///Users/monitum/.gemini/antigravity/brain/673cd3ec-2ffa-40ac-9faf-95e16bd9df57/nexus/src/server/mod.rs)
- Module definition for server components.

#### [NEW] [src/server/transport.rs](file:///Users/monitum/.gemini/antigravity/brain/673cd3ec-2ffa-40ac-9faf-95e16bd9df57/nexus/src/server/transport.rs)
- Definition of the `Transport` trait (read message, write message).

#### [NEW] [src/server/jsonrpc.rs](file:///Users/monitum/.gemini/antigravity/brain/673cd3ec-2ffa-40ac-9faf-95e16bd9df57/nexus/src/server/jsonrpc.rs)
- Basic types for JSON-RPC 2.0 (`Request`, `Response`, `Error`).

### 3. Implementation Logic
- **Stdio Loop:** Read lines from stdin, parse as JSON-RPC, log them, write back to stdout.
- **HTTP Loop:** Simple Axum route that accepts POST requests (for now) as a placeholder for full SSE.

## Verification Plan

### Automated Tests
- **Unit Tests:** Test JSON-RPC serialization/deserialization.
- **Integration Test:** 
    - Compile binary.
    - Run `./nexus --stdio`.
    - Pipe in `{"jsonrpc": "2.0", "method": "initialize", "id": 1}`.
    - Verify output contains valid JSON response.

### Manual Verification
- Run `cargo run` and interact via terminal.
- Verify logs appear in stderr (not stdout, to avoid breaking JSON-RPC pipe).
