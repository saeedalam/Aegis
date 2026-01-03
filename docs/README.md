# Aegis Documentation

**MCP Tool Server for AI Agents**

---

## What is Aegis?

Aegis is an **MCP Tool Server** — it provides tools that AI agents can use via the Model Context Protocol.

**Aegis does NOT run agents.** Agents connect TO Aegis to use its tools.

```
┌──────────────────┐
│   AI Agent       │  ← Claude, GPT, Ollama, your agent
└────────┬─────────┘
         │ MCP Protocol
         ▼
┌──────────────────┐
│      AEGIS       │  ← Provides tools
└──────────────────┘
```

---

## Quick Links

| Document | Description |
|----------|-------------|
| [README](../README.md) | Project overview and quick start |
| [CORE.md](../CORE.md) | Core tools philosophy and design |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System architecture |
| [QUICKSTART.md](QUICKSTART.md) | Getting started guide |
| [CONFIGURATION.md](CONFIGURATION.md) | Configuration reference |
| [API.md](API.md) | MCP API reference |
| [PLUGINS.md](PLUGINS.md) | Plugin system guide |
| [CLAUDE_INTEGRATION.md](CLAUDE_INTEGRATION.md) | Claude Desktop setup |

---

## Tool Categories

### Core Tools (Always Loaded)

Essential primitives that every agent needs:

| Category | Tools |
|----------|-------|
| Basic | `echo`, `get_time`, `uuid.generate` |
| Files | `fs.read_file`, `fs.write_file` |
| Commands | `cmd.exec` |
| Memory | `memory.store`, `memory.recall`, `memory.delete`, `memory.list` |
| HTTP | `http.request` |
| System | `env.get`, `env.list`, `sys.info` |
| Data | `base64.*`, `json.*`, `hash.*`, `regex.*` |

### Extra Tools (Optional)

Higher-level capabilities, enable with `extras_enabled: true` or disable with `--core-only`:

| Document | Tools |
|----------|-------|
| [LLM.md](LLM.md) | `llm.openai`, `llm.anthropic`, `llm.embed` |
| [GIT.md](GIT.md) | `git.status`, `git.log`, `git.diff`, `git.commit`, `git.branch` |
| [NOTIFICATIONS.md](NOTIFICATIONS.md) | `notify.slack`, `notify.discord`, `notify.email`, `webhook.send` |
| [WORKFLOWS.md](WORKFLOWS.md) | `workflow.run`, `workflow.define`, `workflow.execute`, `workflow.list` |
| [TOOLS.md](TOOLS.md) | Complete tool reference |

---

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│                  AEGIS                       │
├─────────────────────────────────────────────┤
│  Transport Layer (stdio / HTTP / SSE)        │
├─────────────────────────────────────────────┤
│  MCP Router & Handlers                       │
├─────────────────┬───────────────────────────┤
│   Core Tools    │   Extra Tools (optional)   │
│     (21)        │        (36)                │
├─────────────────┴───────────────────────────┤
│  Runtime State (Memory, Secrets, Scheduler)  │
└─────────────────────────────────────────────┘
```

---

## Getting Started

```bash
# Build
cargo build --release

# Run with all tools
./target/release/aegis serve

# Run with core only
./target/release/aegis --core-only serve

# List tools
./target/release/aegis tools

# Test a tool
./target/release/aegis run echo --args '{"text": "Hello!"}'
```

---

## Configuration

Minimal config (core only):

```json
{
  "extras_enabled": false
}
```

Full config:

```json
{
  "host": "127.0.0.1",
  "port": 9000,
  "extras_enabled": true,
  "security": {
    "allowed_read_paths": ["/home/user/data"],
    "allowed_write_paths": ["/home/user/output"],
    "allowed_commands": ["ls", "cat", "grep"]
  }
}
```

See [CONFIGURATION.md](CONFIGURATION.md) for full reference.
