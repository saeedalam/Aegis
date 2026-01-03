# Nexus Architecture & Design Philosophy

## 1. Core Identity
**Nexus is an MCP Runtime, not an Agent Framework.**
It is a single-binary infrastructure component (like Redis or Nginx) responsible for the *execution* of agentic capabilities, not the *orchestration* of them.

* **We build:** The engine that runs tools, stores memory, and traces execution.
* **We do NOT build:** The prompt chains, the agent personas, or the UI.

## 2. System Boundaries

### Input (The "Brain")
Nexus does not know *why* it is running a tool. It receives commands via **MCP (Model Context Protocol)** over JSON-RPC (Stdio or SSE).
* Clients: Python scripts, Node apps, CLI tools, LLM UIs.

### Core (The "Runtime")
* **Process Manager:** Spawns and supervises tool processes.
* **Memory Store:** SQLite (Relational) + LanceDB (Vector) embedded.
* **Protocol Adapter:** Translates high-level generic intentions into specific system calls.

### Output (The "Hands")
* **Stdio Tools:** Executing local binaries/scripts (Python/Bash) via standard input/output.
* **Wasm Modules:** (Future) Sandboxed execution of `.wasm` plugins.

## 3. Technology Stack (Fixed Constraints)
* **Language:** Rust (Stable)
* **Async Runtime:** Tokio
* **Web Server:** Axum (for SSE transport)
* **Database:** Rusqlite (Bundled, no external db process required)
* **Protocol:** `mcp-sdk-rs` (or custom JSON-RPC implementation if needed)

## 4. The "Anti-Features" (Do Not Implement)
If a feature falls into this list, reject it immediately.

1.  **Agent Marketplace:** We are not an App Store.
2.  **Graph/Workflow Editor:** Leave that to LangGraph/n8n.
3.  **Prompt Engineering DSL:** We accept JSON, not Jinja2 templates.
4.  **Web UI:** The interface is the API. Use `inspector` or generic MCP clients for UI.
5.  **Cloud Sync:** Local-first only for MVP.

## 5. Decision Records

### ADR-001: Server vs Library
* **Decision:** Standalone Binary.
* **Reasoning:** Cross-language support (Polyglot), persistence independence, and clean resource isolation.

### ADR-002: Tooling Interface
* **Decision:** Native MCP support first.
* **Reasoning:** Trying to invent a custom tool schema fails adoption. Piggyback on Anthropic's standard.
