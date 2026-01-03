# Aegis

**Secure local MCP tool server with script plugins.**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## What is Aegis?

Aegis is a **secure local MCP tool server with script plugins**. It provides tools that AI agents can use via the [Model Context Protocol](https://modelcontextprotocol.io/).

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
    Files, Commands, Memory, HTTP, Git...
```

**Aegis does NOT run agents.** Agents connect TO Aegis to use its tools.

### Who is Aegis for?

- Developers building custom AI agents
- Projects using Ollama or raw LLM APIs that need tools
- Anyone who wants standardized, sandboxed tools for AI

---

## Features

- **Single Binary** — No runtime dependencies
- **MCP Compliant** — Works with any MCP client
- **Core + Extras** — Minimal core, optional extras
- **Plugin System** — Add custom tools via config
- **Secure** — Sandboxed file/command access
- **Persistent Memory** — SQLite-backed storage

---

## Quick Start

### Build

```bash
cd rustapp/nexus
cargo build --release
```

### Run

```bash
# Start HTTP server
./target/release/aegis serve

# Start with core tools only
./target/release/aegis --core-only serve

# Use with Claude Desktop (stdio mode)
./target/release/aegis --stdio
```

### Test

```bash
# List available tools
./target/release/aegis tools

# Run a tool
./target/release/aegis run echo --args '{"text": "Hello Aegis!"}'
```

---

## Tools

### Core (21 tools, always loaded)

| Category | Tools |
|----------|-------|
| **Basic** | `echo`, `get_time`, `uuid.generate` |
| **Files** | `fs.read_file`, `fs.write_file` |
| **Commands** | `cmd.exec` |
| **Memory** | `memory.store`, `memory.recall`, `memory.delete`, `memory.list` |
| **HTTP** | `http.request` |
| **System** | `env.get`, `env.list`, `sys.info` |
| **Data** | `base64.*`, `json.*`, `hash.sha256`, `regex.*` |

### Extras (36 tools, optional)

Enable with `extras_enabled: true` (default) or disable with `--core-only`.

| Category | Tools |
|----------|-------|
| **LLM** | `llm.openai`, `llm.anthropic`, `llm.embed` |
| **Vector** | `vector.store`, `vector.search`, `vector.delete`, `vector.list` |
| **Git** | `git.status`, `git.log`, `git.diff`, `git.commit`, `git.branch` |
| **Notifications** | `notify.slack`, `notify.discord`, `notify.email`, `webhook.send` |
| **Workflows** | `workflow.run`, `workflow.define`, `workflow.execute`, `workflow.list` |
| **Scheduler** | `scheduler.create`, `scheduler.list`, `scheduler.delete`, `scheduler.toggle`, `scheduler.run` |
| **Web** | `web.extract`, `web.search` |
| **Conversations** | `conversation.*` |
| **Secrets** | `secrets.*` |

---

## Configuration

### Minimal (Core Only)

```json
{
  "extras_enabled": false
}
```

### Full

```json
{
  "host": "127.0.0.1",
  "port": 9000,
  "extras_enabled": true,
  "security": {
    "allowed_read_paths": ["/home/user/projects"],
    "allowed_write_paths": ["/home/user/output"],
    "allowed_commands": ["ls", "cat", "grep", "git"]
  }
}
```

---

## Claude Desktop Integration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "aegis": {
      "command": "/path/to/aegis",
      "args": ["--stdio"]
    }
  }
}
```

---

## Custom Plugins

Add tools without modifying code:

```json
{
  "plugins": [
    {
      "name": "my.tool",
      "description": "My custom tool",
      "command": "/path/to/script.sh",
      "args_template": ["${input}"]
    }
  ]
}
```

---

## Project Structure

```
aegis/
├── src/
│   ├── core/           # Config, state, errors
│   ├── protocol/       # MCP/JSON-RPC
│   ├── transport/      # stdio & HTTP/SSE
│   ├── handlers/       # MCP method handlers
│   ├── tools/
│   │   ├── core/       # Essential tools (21)
│   │   └── extras/     # Optional tools (36)
│   └── memory/         # SQLite storage
├── config/             # Example configs
├── docs/               # Documentation
└── examples/           # Python SDK & examples
```

---

## Why "Aegis"?

Aegis (Greek: αἰγίς) means "shield" or "protection."

Aegis protects your agent:
- **Sandboxed execution** — Controlled access to files/commands
- **Secure secrets** — Encrypted credential storage
- **Rate limiting** — Protection against abuse

---

## License

MIT
