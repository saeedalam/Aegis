# Aegis: Complete User Guide

## What is Aegis?

**Aegis** is a secure local MCP tool server with script plugins. It provides tools that AI agents can use via the Model Context Protocol (MCP).

**Aegis does NOT run agents.** Agents connect TO Aegis to use its tools.

```
┌──────────────────────────────────────────────────────────────┐
│                        AI AGENT                               │
│            (Claude, GPT, Ollama, your custom agent)           │
└─────────────────────────┬────────────────────────────────────┘
                          │ MCP Protocol
                          ▼
┌──────────────────────────────────────────────────────────────┐
│                         AEGIS                                 │
│      Secure local MCP tool server with script plugins         │
│  ┌─────────┐  ┌──────────┐  ┌────────┐  ┌────────────────┐   │
│  │  Tools  │  │  Memory  │  │ Files  │  │ Shell Commands │   │
│  └─────────┘  └──────────┘  └────────┘  └────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### Key Features

| Feature               | Description                                                  |
| --------------------- | ------------------------------------------------------------ |
| **Single Binary**     | No Docker, no dependencies - just one executable             |
| **Core + Extras**     | 21 core tools + 36 optional extras                           |
| **Plugin System**     | Add custom tools via config (no code changes)                |
| **Persistent Memory** | SQLite-backed key-value store                                |
| **Dual Transport**    | CLI (stdio) or HTTP/SSE server                               |
| **Security First**    | Path restrictions, command allowlisting, sandboxed execution |
| **MCP Protocol**      | Industry-standard AI agent protocol                          |

---

## Quick Start

### 1. Build Aegis

```bash
git clone https://github.com/saeedalam/Aegis
cd Aegis
cargo build --release
```

### 2. Run Your First Tool

```bash
./target/release/aegis run echo --args '{"text": "Hello, World!"}'
# Output: Hello, World!
```

### 3. Explore Available Tools

```bash
./target/release/aegis tools
```

Output:

```
Available Tools
════════════════════════════════════════════════════════════

  CORE (always loaded)
  ────────────────────────────────────────
  ▸ echo
  ▸ get_time
  ▸ fs.read_file
  ▸ fs.write_file
  ▸ cmd.exec
  ▸ memory.store / recall / list / delete
  ▸ http.request
  ...

  EXTRAS (optional)
  ────────────────────────────────────────
  ▸ llm.openai / anthropic / embed
  ▸ git.status / log / diff / commit
  ▸ vector.store / search
  ...

════════════════════════════════════════════════════════════
  21 core + 36 extras = 57 tools
```

---

## Usage Modes

### Mode 1: One-Shot CLI (`aegis run`)

Execute a single tool and get the result immediately:

```bash
# Echo text
aegis run echo --args '{"text": "Hello!"}'

# Get current time
aegis run get_time

# Store data in memory
aegis run memory.store --args '{"key": "user", "value": "Alice"}'

# Recall data from memory
aegis run memory.recall --args '{"key": "user"}'

# Get JSON output
aegis run get_time --format json
```

### Mode 2: HTTP Server (`aegis serve`)

Start an HTTP server for remote clients:

```bash
aegis serve --port 9000
```

Then connect via HTTP:

```bash
# Health check
curl http://localhost:9000/health

# List tools
curl -X POST http://localhost:9000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'

# Call a tool
curl -X POST http://localhost:9000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"tools/call",
    "params":{"name":"echo","arguments":{"text":"Hello via HTTP!"}},
    "id":2
  }'
```

### Mode 3: Stdio (`aegis --stdio`)

For pipe-based MCP clients (like Claude Desktop):

```bash
aegis --stdio
```

Then send JSON-RPC messages on stdin:

```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": { "clientInfo": { "name": "my-agent" } },
  "id": 1
}
```

### Mode 4: Core Only (`aegis --core-only`)

Load only essential tools (21 instead of 57):

```bash
aegis --core-only serve
```

---

## Tools Reference

### Core Tools (Always Loaded)

| Category     | Tools                                     | Description                       |
| ------------ | ----------------------------------------- | --------------------------------- |
| **Basic**    | `echo`, `get_time`, `uuid.generate`       | Testing, timestamps, unique IDs   |
| **Files**    | `fs.read_file`, `fs.write_file`           | Sandboxed file I/O                |
| **Commands** | `cmd.exec`                                | Restricted shell execution        |
| **Memory**   | `memory.store/recall/list/delete`         | Persistent key-value store        |
| **HTTP**     | `http.request`                            | HTTP client (GET/POST/PUT/DELETE) |
| **System**   | `env.get`, `env.list`, `sys.info`         | Environment and system info       |
| **Data**     | `base64.*`, `json.*`, `hash.*`, `regex.*` | Data transformation               |

### Extra Tools (Optional)

| Category          | Tools                                        | Description               |
| ----------------- | -------------------------------------------- | ------------------------- |
| **LLM**           | `llm.openai`, `llm.anthropic`, `llm.embed`   | LLM API integration       |
| **Vector**        | `vector.store/search/list/delete`            | Semantic search           |
| **Git**           | `git.status/log/diff/commit/branch`          | Git operations            |
| **Notifications** | `notify.slack/discord/email`, `webhook.send` | Outbound notifications    |
| **Workflows**     | `workflow.run/define/execute/list`           | Multi-step automation     |
| **Scheduler**     | `scheduler.create/list/delete/toggle/run`    | Cron-like scheduling      |
| **Web**           | `web.extract`, `web.search`                  | Web scraping and search   |
| **Conversations** | `conversation.*`                             | Multi-turn history        |
| **Secrets**       | `secrets.set/get/list/delete`                | Secure credential storage |

---

## Configuration

Create `aegis.json` in your working directory:

```json
{
  "server_name": "aegis",
  "server_version": "0.3.0",
  "host": "127.0.0.1",
  "port": 9000,
  "database_path": "aegis.db",
  "extras_enabled": true,
  "log_level": "info",
  "security": {
    "allowed_read_paths": ["/tmp", "/home/user/data"],
    "allowed_write_paths": ["/tmp"],
    "allowed_commands": ["ls", "cat", "echo", "date", "pwd", "whoami", "git"],
    "tool_timeout_secs": 30
  }
}
```

### Configuration Options

| Option           | Description                  | Default       |
| ---------------- | ---------------------------- | ------------- |
| `server_name`    | Server name in MCP responses | `"aegis"`     |
| `server_version` | Server version               | `"0.3.0"`     |
| `host`           | HTTP server bind address     | `"127.0.0.1"` |
| `port`           | HTTP server port             | `9000`        |
| `database_path`  | SQLite database file         | `"aegis.db"`  |
| `extras_enabled` | Load extra tools             | `true`        |
| `log_level`      | Logging level                | `"info"`      |

### Security Configuration

| Option                | Description                                 |
| --------------------- | ------------------------------------------- |
| `allowed_read_paths`  | Directories where `fs.read_file` can access |
| `allowed_write_paths` | Directories where `fs.write_file` can write |
| `allowed_commands`    | Shell commands `cmd.exec` can run           |
| `tool_timeout_secs`   | Maximum execution time for commands         |

---

## Plugin System

Add custom tools without modifying code:

```json
{
  "plugins": [
    {
      "name": "my.custom_tool",
      "description": "My custom tool",
      "command": "/path/to/script.sh",
      "args_template": ["${input}"],
      "output_format": "text"
    }
  ]
}
```

See [PLUGINS.md](PLUGINS.md) for full documentation.

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

See [CLAUDE_INTEGRATION.md](CLAUDE_INTEGRATION.md) for full setup guide.

---

## Python SDK

A minimal Python client is included:

```python
from aegis_client import AegisClient

# Create client
client = AegisClient()

# Call any tool
result = client.call("echo", {"text": "Hello!"})
print(result)

# Memory operations
client.memory_store("key", "value")
value = client.memory_recall("key")
keys = client.memory_list()
```

### Install SDK

```bash
cd sdk/python
pip install -e .
```

---

## MCP Protocol

Aegis implements the [Model Context Protocol](https://modelcontextprotocol.io/) specification (version 2024-11-05).

### Supported Methods

| Method           | Description                          |
| ---------------- | ------------------------------------ |
| `initialize`     | Handshake and capability negotiation |
| `ping`           | Health check                         |
| `tools/list`     | List available tools                 |
| `tools/call`     | Execute a tool                       |
| `prompts/list`   | List prompts (empty)                 |
| `resources/list` | List memory resources                |
| `resources/read` | Read a memory resource               |

### Example Session

```json
// 1. Initialize
→ {"jsonrpc":"2.0","method":"initialize","params":{"clientInfo":{"name":"my-agent"}},"id":1}
← {"jsonrpc":"2.0","result":{"serverInfo":{"name":"aegis","version":"0.3.0"},"capabilities":{"tools":{},"resources":{}}},"id":1}

// 2. List tools
→ {"jsonrpc":"2.0","method":"tools/list","id":2}
← {"jsonrpc":"2.0","result":{"tools":[{"name":"echo",...},{"name":"get_time",...}]},"id":2}

// 3. Call a tool
→ {"jsonrpc":"2.0","method":"tools/call","params":{"name":"echo","arguments":{"text":"hi"}},"id":3}
← {"jsonrpc":"2.0","result":{"content":[{"type":"text","text":"hi"}]},"id":3}
```

---

## Architecture

```
aegis/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library exports
│   ├── core/             # Configuration, state, errors
│   ├── protocol/         # JSON-RPC and MCP types
│   ├── transport/        # Stdio and HTTP/SSE
│   ├── handlers/         # MCP method handlers
│   ├── tools/
│   │   ├── core/         # Essential tools (21)
│   │   └── extras/       # Optional tools (36)
│   └── memory/           # SQLite persistence
├── sdk/
│   └── python/           # Python client
├── examples/             # Usage examples
└── docs/                 # Documentation
```

---

## Troubleshooting

### "Tool not found"

Make sure you're using the correct tool name:

```bash
aegis tools  # List all available tools
```

### "Permission denied" for file operations

Configure allowed paths in `aegis.json`:

```json
{
  "security": {
    "allowed_read_paths": ["/your/path"],
    "allowed_write_paths": ["/your/path"]
  }
}
```

### "Command not allowed"

Add the command to the allowlist:

```json
{
  "security": {
    "allowed_commands": ["your-command"]
  }
}
```

### Database errors

The SQLite database is stored in `aegis.db`. To reset:

```bash
rm aegis.db
aegis run echo --args '{"text": "test"}'  # Creates fresh DB
```

---

## Next Steps

1. **Try the examples**: `python3 examples/hello_aegis.py`
2. **Build an agent**: See `examples/simple_agent.py`
3. **Integrate with Claude**: See [CLAUDE_INTEGRATION.md](CLAUDE_INTEGRATION.md)
4. **Add custom tools**: See [PLUGINS.md](PLUGINS.md)

---

<p align="center">
  <strong>Aegis — Secure local MCP tool server with script plugins</strong><br>
  <sub>Built with ❤️ for the AI agent ecosystem</sub>
</p>
