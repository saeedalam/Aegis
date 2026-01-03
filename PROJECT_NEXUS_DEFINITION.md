# Project Nexus: The Rust MCP Runtime
**"The Infrastructure Server for AI Agents"**

## 1. Executive Summary
**Nexus** is a high-performance, single-binary server that provides the essential runtime environment for AI agents.
It adopts the **Model Context Protocol (MCP)** as its native language, acting as a standardized host for tools, memory, and execution resources.

*   **Identiy:** Infrastructure, not Framework.
*   **Analogy:** "Ollama for Tools" or "Redis for Agent Memory".
*   **Target Audience:** Systems Engineers, DevOps, and developers building production-grade agents who are tired of Python dependency hell.

---

## 2. The Core Problem
Building robust agents today requires importing heavy, opinionated libraries (LangChain, Rig) that needlessly couple your business logic to a specific language runtime (usually Python).
Every new agent project forces developers to re-implement:
1.  **Tool Execution:** How to run code safely?
2.  **Memory persistence:** How to store history?
3.  **Observability:** How to trace actions?

## 3. The Nexus Solution
We decouple the **"Brain"** (Protocol/Logic) from the **"Body"** (Execution/Runtime).

Nexus runs as a standalone background service (like a database):
```bash
./nexus serve
# 🟢 Active on http://localhost:9000 (MCP/SSE)
```

Clients (Agents) connect via standard MCP to:
*   **Execute Tools:** Safe, sandboxed process execution.
*   **Store/Recall Memory:** Built-in SQLite and Vector handling.
*   **Trace Execution:** Structured logs of every tool call.

---

## 4. Technical Architecture

### Stack Constraints (Rust)
*   **Language:** Rust (for safety, single-binary distribution, and concurrency).
*   **Runtime:** Tokio (Async I/O).
*   **API:** Axum (HTTP/SSE).
*   **Protocol:** JSON-RPC 2.0 (MCP Compliance).
*   **Storage:** Rusqlite (Bundled, zero-config).

### System Components
1.  **Protocol Adapter:** Accepts MCP requests via Stdio (CLI) or SSE (Remote).
2.  **Process Manager:** A robust supervisor for spawning and monitoring tool subprocesses with timeouts.
3.  **Memory Subsystem:** A relational + vector engine exposed as standard MCP Resources (`resources/read`).

---

## 5. Scope & Roadmap (The "Honest MVP")

We are building a **System Engineer's Tool**, focusing on correctness and performance, not hype features.

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| **1. Skeleton** | 3 Weeks | Binary that speaks strict MCP (JSON-RPC) over Stdio/HTTP. |
| **2. Execution** | 4 Weeks | Secure process manager for running local scripts/binaries as tools. |
| **3. Memory** | 3 Weeks | SQLite integration for persisting conversation history via standard MCP resources. |
| **4. Polish** | 2 Weeks | CLI UX, Python Client SDK (thin wrapper), and Documentation. |

**Total Timeline:** ~12 Weeks (Solo Dev Pace)

---

## 6. Critical Anti-Features (Non-Goals)
To ensure completion, we **WILL NOT** build:

❌ **Agent Marketplace:** We are a runtime, not an app store.
❌ **Graph Editor/Orchestration:** Use LangGraph for logic; use Nexus for execution.
❌ **Web UI:** The interface is the API.
❌ **Prompt Engineering DSLs:** No Jinja templates; just JSON.
❌ **Cloud Sync:** Local-first focus for MVP.

---

## 7. Why This Project Matters
*   **Career Value:** Demonstrates mastery of Rust systems programming, protocol implementation, and modern AI infrastructure standards.
*   **Utility:** Solves the real "deployment gap" for polyglot AI agents.
*   **Honesty:** It doesn't promise "AGI"; it promises a stable binary that runs tools reliably.
