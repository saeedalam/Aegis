# Nexus Quick Start

Get up and running with Nexus in 5 minutes.

---

## 1. Build

```bash
cd rustapp/nexus
cargo build --release
```

Binary location: `target/release/nexus`

---

## 2. Run

### HTTP Server (Recommended)

```bash
./target/release/nexus serve
```

Output:
```
🟢 Nexus SSE server listening on http://127.0.0.1:9000
📊 Dashboard available at http://127.0.0.1:9000/dashboard
```

### Stdio Mode (for Claude/Cursor)

```bash
./target/release/nexus --stdio
```

---

## 3. Verify

```bash
# Health check
curl http://localhost:9000/health

# Expected:
# {"status":"ok","service":"nexus","version":"0.2.0"}
```

---

## 4. First Tool Call

```bash
curl -X POST http://localhost:9000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "1.0",
      "capabilities": {},
      "clientInfo": {"name": "test", "version": "1.0"}
    }
  }'

curl -X POST http://localhost:9000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "get_time",
      "arguments": {}
    }
  }'
```

---

## 5. List Tools

```bash
./target/release/nexus tools
```

Shows all 48 available tools.

---

## 6. Use the Dashboard

Open in browser: http://localhost:9000/dashboard

---

## 7. Store a Secret

```bash
curl -X POST http://localhost:9000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "secrets.set",
      "arguments": {
        "key": "OPENAI_KEY",
        "value": "sk-your-key-here"
      }
    }
  }'
```

---

## 8. Call an LLM

```bash
curl -X POST http://localhost:9000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "llm.openai",
      "arguments": {
        "prompt": "Say hello in 3 languages"
      }
    }
  }'
```

---

## 9. Create a Workflow

```bash
curl -X POST http://localhost:9000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 5,
    "method": "tools/call",
    "params": {
      "name": "workflow.run",
      "arguments": {
        "name": "hello-workflow",
        "steps": [
          {"id": "time", "tool": "get_time"},
          {"id": "greet", "tool": "echo", "args": {"text": "Hello at {{time.time}}"}}
        ]
      }
    }
  }'
```

---

## 10. Schedule a Task

```bash
curl -X POST http://localhost:9000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 6,
    "method": "tools/call",
    "params": {
      "name": "scheduler.create",
      "arguments": {
        "name": "hourly-check",
        "cron": "0 * * * *",
        "tool": "get_time"
      }
    }
  }'
```

---

## Next Steps

- **[Tools Reference](TOOLS.md)** - All 48 tools documented
- **[LLM Guide](LLM.md)** - OpenAI & Anthropic integration
- **[Workflows](WORKFLOWS.md)** - Chain tools together
- **[Notifications](NOTIFICATIONS.md)** - Slack, Discord, Email
- **[Configuration](CONFIGURATION.md)** - Full config reference
- **[Claude Integration](CLAUDE_INTEGRATION.md)** - Use with Claude Desktop
- **[Plugins](PLUGINS.md)** - Create custom tools

---

## Claude Desktop Setup

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "nexus": {
      "command": "/path/to/nexus",
      "args": ["--stdio"]
    }
  }
}
```

Restart Claude Desktop.

---

## Cursor IDE Setup

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "nexus": {
      "command": "/path/to/nexus",
      "args": ["--stdio"]
    }
  }
}
```

---

## Tool Categories

| Category | Examples |
|----------|----------|
| **Core** | echo, get_time, uuid.generate |
| **Memory** | memory.store, memory.recall |
| **Secrets** | secrets.set, secrets.get |
| **LLM** | llm.openai, llm.anthropic |
| **Notify** | notify.slack, notify.discord |
| **Workflow** | workflow.run, workflow.define |
| **Git** | git.status, git.commit |
| **Files** | fs.read_file, fs.write_file |
| **HTTP** | http.request |
| **Schedule** | scheduler.create, scheduler.run |

**Total: 48 tools**
