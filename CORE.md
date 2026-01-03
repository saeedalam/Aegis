# Aegis Core

**The essential tools that define Aegis as an MCP tool server.**

---

## Philosophy

Aegis Core follows these principles:

1. **Minimal** - Only tools that are universally needed
2. **Unopinionated** - No assumptions about how you'll use them
3. **Stable** - These APIs won't change often
4. **Fast** - No unnecessary dependencies

## Core Tools (21 total)

### Basic Utilities

| Tool | Purpose |
|------|---------|
| `echo` | Testing and debugging |
| `get_time` | Current timestamp (ISO 8601) |
| `uuid.generate` | Generate unique identifiers |

### Filesystem

| Tool | Purpose |
|------|---------|
| `fs.read_file` | Read file contents |
| `fs.write_file` | Write file contents |

> **Security**: Restricted to configured `allowed_read_paths` and `allowed_write_paths`.

### Command Execution

| Tool | Purpose |
|------|---------|
| `cmd.exec` | Run shell commands |

> **Security**: Restricted to configured `allowed_commands`.

### Memory (Key-Value)

| Tool | Purpose |
|------|---------|
| `memory.store` | Store a value |
| `memory.recall` | Retrieve a value |
| `memory.delete` | Remove a value |
| `memory.list` | List all keys |

> Backed by SQLite. Persists across restarts.

### HTTP Client

| Tool | Purpose |
|------|---------|
| `http.request` | Make HTTP requests |

> Supports GET, POST, PUT, DELETE, PATCH. Configurable timeouts and URL restrictions.

### Environment & System

| Tool | Purpose |
|------|---------|
| `env.get` | Get environment variable |
| `env.list` | List environment variable names |
| `sys.info` | System information (OS, arch, hostname) |

### Data Utilities

| Tool | Purpose |
|------|---------|
| `base64.encode` | Encode text to Base64 |
| `base64.decode` | Decode Base64 to text |
| `json.parse` | Parse JSON string |
| `json.query` | Query JSON with dot notation |
| `hash.sha256` | Compute SHA-256 hash |
| `regex.match` | Match text against pattern |
| `regex.replace` | Replace text using pattern |

## What's NOT in Core

These are explicitly **not** core tools. They are useful but opinionated:

| Category | Why Not Core |
|----------|--------------|
| **LLM Tools** | Provider-specific (OpenAI, Anthropic) |
| **Vector/RAG** | Use-case specific |
| **Git** | Not all agents need git |
| **Notifications** | Integration-specific |
| **Workflows** | Orchestration, not primitives |
| **Web Scraping** | Specialized use case |

These tools are available as **Extras** - enable them when needed.

## Using Core Only

```bash
aegis --core-only serve
```

Or in config:

```json
{
  "extras_enabled": false
}
```

This gives you:
- 21 essential tools
- Faster startup
- Smaller attack surface
- Clear, predictable behavior

## Extending Core

Don't modify core tools. Instead:

1. **Use Plugins** - Add custom tools via config
2. **Enable Extras** - Turn on when needed
3. **Build on Top** - Create your own MCP server that proxies to Aegis

## Stability Promise

Core tools will:
- ✅ Maintain backwards compatibility
- ✅ Keep the same input/output schema
- ✅ Work reliably without external dependencies
- ❌ Not be removed without major version bump
- ❌ Not gain opinionated features
