# Aegis Architecture

**Clean, modular, and intentionally simple.**

---

## What is Aegis?

Aegis is an **MCP Tool Server** — it provides tools that AI agents can use via the Model Context Protocol.

**Aegis does NOT run agents.** Agents connect TO Aegis to use its tools.

```
┌──────────────────┐
│   AI Agent       │  ← Claude, GPT, Ollama, your agent
│   (runs here)    │
└────────┬─────────┘
         │ MCP Protocol
         ▼
┌──────────────────┐
│      AEGIS       │  ← Provides tools
│  (tool server)   │
└──────────────────┘
         │
         ▼
    Files, Commands, Memory, HTTP...
```

---

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         AEGIS                                │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   STDIO     │  │    HTTP     │  │     SSE     │         │
│  │  Transport  │  │  Transport  │  │   Stream    │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         └────────────────┴────────────────┘                 │
│                          │                                   │
│                  ┌───────▼───────┐                          │
│                  │  MCP Router   │                          │
│                  │  (handlers)   │                          │
│                  └───────┬───────┘                          │
│         ┌────────────────┼────────────────┐                 │
│         │                │                │                 │
│  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐         │
│  │    Core     │  │   Extras    │  │   Plugins   │         │
│  │   Tools     │  │   Tools     │  │   (config)  │         │
│  │    (21)     │  │    (36)     │  │     (n)     │         │
│  └──────┬──────┘  └─────────────┘  └─────────────┘         │
│         │                                                    │
│  ┌──────▼──────────────────────────────────────────┐        │
│  │              RuntimeState                        │        │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │        │
│  │  │  Memory  │ │ Secrets  │ │ Scheduler│        │        │
│  │  │ (SQLite) │ │ Manager  │ │          │        │        │
│  │  └──────────┘ └──────────┘ └──────────┘        │        │
│  └──────────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
aegis/
├── src/
│   ├── main.rs              # Entry point, CLI
│   ├── lib.rs               # Library exports
│   │
│   ├── core/                # Core runtime
│   │   ├── mod.rs
│   │   ├── config.rs        # Configuration structs
│   │   ├── errors.rs        # Error types
│   │   └── state.rs         # RuntimeState
│   │
│   ├── protocol/            # MCP/JSON-RPC
│   │   ├── mod.rs
│   │   ├── jsonrpc.rs       # JSON-RPC 2.0 types
│   │   └── mcp.rs           # MCP types & methods
│   │
│   ├── transport/           # I/O layer
│   │   ├── mod.rs
│   │   ├── transport.rs     # Transport trait
│   │   ├── stdio.rs         # Stdin/stdout transport
│   │   ├── sse.rs           # HTTP/SSE server
│   │   └── middleware.rs    # Auth, rate limiting
│   │
│   ├── handlers/            # MCP method handlers
│   │   ├── mod.rs
│   │   ├── router.rs        # Request routing
│   │   ├── initialize.rs    # MCP handshake
│   │   ├── tools.rs         # tools/list
│   │   ├── tools_call.rs    # tools/call
│   │   ├── prompts.rs       # prompts/list
│   │   ├── resources.rs     # resources/*
│   │   └── ping.rs          # health check
│   │
│   ├── tools/               # Tool system
│   │   ├── mod.rs
│   │   ├── registry.rs      # Tool trait & registry
│   │   ├── process_manager.rs
│   │   ├── core/            # Core tools (21)
│   │   └── extras/          # Extra tools (36)
│   │
│   ├── memory/              # Storage layer
│   │   ├── mod.rs
│   │   ├── store.rs         # MemoryStore trait
│   │   ├── schema.rs        # SQL schema
│   │   └── sqlite.rs        # SQLite implementation
│   │
│   ├── secrets/             # Secrets management
│   │   └── mod.rs
│   │
│   ├── scheduler/           # Task scheduling
│   │   └── mod.rs
│   │
│   └── dashboard/           # Web UI (optional)
│       └── mod.rs
│
├── config/                  # Example configs
├── docs/                    # Documentation
├── examples/                # Python SDK & examples
└── sdk/                     # Language SDKs
```

## Key Components

### 1. RuntimeState

Holds all shared state:

```rust
pub struct RuntimeState {
    pub config: Config,
    pub tool_registry: RwLock<ToolRegistry>,
    pub memory_store: Arc<dyn MemoryStore>,
    pub secrets: Arc<SecretsManager>,
    pub scheduler: Arc<Scheduler>,
}
```

### 2. Tool Trait

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, args: Value, state: Arc<RuntimeState>) -> Result<ToolOutput, ToolError>;
}
```

### 3. Transport Trait

```rust
#[async_trait]
pub trait Transport {
    async fn read_request(&mut self) -> Result<Option<Request>, AegisError>;
    async fn write_response(&mut self, response: Response) -> Result<(), AegisError>;
    async fn close(&mut self) -> Result<(), AegisError>;
}
```

## Data Flow

```
1. Client sends JSON-RPC request
       │
       ▼
2. Transport reads & parses
       │
       ▼
3. Router dispatches to handler
       │
       ▼
4. Handler executes (may call tools)
       │
       ▼
5. Response flows back through transport
       │
       ▼
6. Client receives JSON-RPC response
```

## Tool Loading

```rust
// In RuntimeState::new()

// Always load core
register_core_tools(&mut registry, &config);

// Conditionally load extras
if config.extras_enabled {
    register_extra_tools(&mut registry, &config);
}

// Load plugins from config
for plugin in &config.plugins {
    registry.register(ScriptTool::new(plugin));
}
```

## Security Model

1. **Filesystem** - Allowlist of readable/writable paths
2. **Commands** - Allowlist of executable commands
3. **HTTP** - Optional URL pattern restrictions
4. **Auth** - API key authentication for HTTP transport
5. **Rate Limiting** - Per-client request limits

## Extension Points

| Extension        | How                            |
| ---------------- | ------------------------------ |
| Add a tool       | Implement `Tool` trait         |
| Custom transport | Implement `Transport` trait    |
| Custom storage   | Implement `MemoryStore` trait  |
| Plugins          | Add to `plugins` in config     |
| Middleware       | Add to Axum router in `sse.rs` |

## Design Principles

1. **Single binary** - No external dependencies at runtime
2. **Zero config works** - Sensible defaults
3. **Security by default** - Restrictive unless configured
4. **Core is stable** - Extras can change
5. **Composition over inheritance** - Traits, not class hierarchies
