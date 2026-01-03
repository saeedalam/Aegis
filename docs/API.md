# Nexus API Reference

Complete reference for all Nexus tools and MCP methods.

---

## MCP Methods

### `initialize`

Initialize the MCP session.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "clientInfo": {
      "name": "my-client",
      "version": "1.0.0"
    },
    "capabilities": {}
  },
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": {
      "name": "nexus",
      "version": "0.2.0"
    },
    "capabilities": {
      "tools": {},
      "resources": {},
      "prompts": {}
    }
  },
  "id": 1
}
```

---

### `tools/list`

List all available tools.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "id": 2
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "name": "echo",
        "description": "Echoes back the input text",
        "inputSchema": {
          "type": "object",
          "properties": {
            "text": {"type": "string"}
          },
          "required": ["text"]
        }
      }
    ]
  },
  "id": 2
}
```

---

### `tools/call`

Execute a tool.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "echo",
    "arguments": {
      "text": "Hello!"
    }
  },
  "id": 3
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Hello!"
      }
    ]
  },
  "id": 3
}
```

---

### `resources/list`

List available resources (memory).

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/list",
  "id": 4
}
```

---

### `resources/read`

Read a specific resource.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "nexus://kv"
  },
  "id": 5
}
```

---

### `ping`

Health check.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "ping",
  "id": 6
}
```

---

## Tool Reference

### `echo`

Echo input text back.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to echo |

**Example:**
```json
{"text": "Hello, World!"}
```

---

### `get_time`

Get current server time.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| (none) | | | |

**Response:**
```json
{
  "time": "2026-01-03T00:00:00+00:00",
  "timestamp": 1767398400,
  "timezone": "UTC"
}
```

---

### `fs.read_file`

Read file contents.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | File path to read |

**Example:**
```json
{"path": "/tmp/data.txt"}
```

**Note:** Path must be in `allowed_read_paths`.

---

### `fs.write_file`

Write content to file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | File path to write |
| `content` | string | Yes | Content to write |

**Example:**
```json
{"path": "/tmp/output.txt", "content": "Hello!"}
```

**Note:** Path must be in `allowed_write_paths`.

---

### `cmd.exec`

Execute shell command.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | Command to run |
| `args` | array | No | Command arguments |

**Example:**
```json
{"command": "ls", "args": ["-la", "/tmp"]}
```

**Note:** Command must be in `allowed_commands`.

---

### `memory.store`

Store key-value pair.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `key` | string | Yes | Key to store under |
| `value` | string | Yes | Value to store |

**Example:**
```json
{"key": "user_name", "value": "Alice"}
```

---

### `memory.recall`

Recall stored value.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `key` | string | Yes | Key to recall |

**Example:**
```json
{"key": "user_name"}
```

**Response:**
```json
{
  "found": true,
  "key": "user_name",
  "value": "Alice",
  "created_at": "...",
  "updated_at": "..."
}
```

---

### `memory.list`

List stored keys.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `prefix` | string | No | Filter by prefix |

**Example:**
```json
{"prefix": "user_"}
```

---

### `memory.delete`

Delete stored key.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `key` | string | Yes | Key to delete |

---

### `http.request`

Make HTTP request.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url` | string | Yes | URL to request |
| `method` | string | No | HTTP method (default: GET) |
| `headers` | object | No | Request headers |
| `body` | string | No | Request body |
| `json` | object | No | JSON body (sets Content-Type) |

**Example:**
```json
{
  "url": "https://api.example.com/data",
  "method": "POST",
  "headers": {"Authorization": "Bearer token"},
  "json": {"name": "test"}
}
```

---

### `base64.encode`

Encode text to Base64.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to encode |

---

### `base64.decode`

Decode Base64 to text.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `encoded` | string | Yes | Base64 string |

---

### `json.parse`

Parse and format JSON.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | JSON string |

---

### `json.query`

Query JSON by path.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `json` | any | Yes | JSON object or string |
| `path` | string | Yes | Dot-notation path |

**Example:**
```json
{
  "json": {"data": {"items": [{"name": "first"}]}},
  "path": "data.items[0].name"
}
```

---

### `uuid.generate`

Generate UUID(s).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `count` | integer | No | Number to generate (default: 1, max: 100) |

---

### `hash.sha256`

Compute SHA-256 hash.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to hash |

---

### `regex.match`

Match regex pattern.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to search |
| `pattern` | string | Yes | Regex pattern |
| `global` | boolean | No | Find all matches |

---

### `regex.replace`

Replace regex matches.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to process |
| `pattern` | string | Yes | Regex pattern |
| `replacement` | string | Yes | Replacement text |
| `global` | boolean | No | Replace all (default: true) |

---

## HTTP Endpoints

### `GET /health`

Health check.

**Response:**
```json
{"status": "ok", "service": "nexus", "version": "0.2.0"}
```

### `POST /mcp`

MCP JSON-RPC endpoint.

### `GET /sse`

Server-Sent Events stream.

### `GET /metrics`

Server metrics.

**Response:**
```json
{
  "requests": {"POST /mcp": 42},
  "tool_calls": {"echo": 10, "memory.store": 5},
  "total_requests": 42
}
```

---

## Error Codes

| Code | Meaning |
|------|---------|
| -32700 | Parse error |
| -32600 | Invalid request |
| -32601 | Method not found |
| -32602 | Invalid params |
| -32603 | Internal error |

---

## Configuration Reference

See [GUIDE.md](GUIDE.md) for complete configuration options.


