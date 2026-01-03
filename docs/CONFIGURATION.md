# Configuration Guide

Complete reference for configuring Nexus.

---

## Configuration File

Nexus looks for configuration in this order:
1. `--config /path/to/config.json` (CLI argument)
2. `./nexus.json` (current directory)
3. `~/.config/nexus/config.json` (user config)
4. Built-in defaults

---

## Full Configuration Reference

```json
{
  // Server identification
  "server_name": "nexus",
  "server_version": "0.2.0",

  // Network settings
  "host": "127.0.0.1",
  "port": 9000,

  // Database
  "database_path": "./data/nexus.db",

  // Security settings
  "security": {
    "allowed_read_paths": ["/home/user/projects", "/tmp"],
    "allowed_write_paths": ["/home/user/projects/output"],
    "allowed_commands": ["ls", "cat", "grep", "git", "npm", "node"],
    "max_command_timeout_secs": 30
  },

  // Authentication
  "auth": {
    "enabled": false,
    "api_keys": ["key1", "key2"]
  },

  // Rate limiting
  "rate_limit": {
    "enabled": false,
    "requests_per_second": 100,
    "burst_size": 200
  },

  // HTTP client settings
  "http_client": {
    "timeout_secs": 30,
    "allowed_hosts": ["*"],
    "blocked_hosts": ["localhost", "127.0.0.1", "169.254.169.254"]
  },

  // Custom plugins
  "plugins": []
}
```

---

## Server Settings

### `server_name`

Name reported to MCP clients.

```json
"server_name": "my-nexus-server"
```

### `server_version`

Version string for identification.

```json
"server_version": "1.0.0"
```

### `host`

IP address to bind to.

| Value | Description |
|-------|-------------|
| `127.0.0.1` | Local only (default) |
| `0.0.0.0` | All interfaces (public) |

```json
"host": "0.0.0.0"
```

### `port`

TCP port for HTTP server.

```json
"port": 9000
```

### `database_path`

Path to SQLite database file.

```json
"database_path": "./data/nexus.db"
```

Use `:memory:` for in-memory (non-persistent) storage:

```json
"database_path": ":memory:"
```

---

## Security Settings

### `allowed_read_paths`

Paths that `fs.read_file` can access.

```json
"security": {
  "allowed_read_paths": [
    "/home/user/projects",
    "/var/log",
    "/etc/nginx"
  ]
}
```

**Important:** Paths are prefix-matched. `/home/user` allows `/home/user/anything`.

### `allowed_write_paths`

Paths that `fs.write_file` can access.

```json
"security": {
  "allowed_write_paths": [
    "/home/user/projects/output",
    "/tmp/nexus"
  ]
}
```

### `allowed_commands`

Commands that `cmd.exec` can run.

```json
"security": {
  "allowed_commands": [
    "ls",
    "cat",
    "grep",
    "git",
    "npm",
    "yarn",
    "node",
    "python"
  ]
}
```

**Note:** Only the command name is checked, not arguments.

### `max_command_timeout_secs`

Maximum execution time for commands.

```json
"security": {
  "max_command_timeout_secs": 60
}
```

---

## Authentication

### Enabling Auth

```json
"auth": {
  "enabled": true,
  "api_keys": [
    "your-secure-api-key-1",
    "your-secure-api-key-2"
  ]
}
```

### Using Auth

Pass the key in the header:

```bash
curl -X POST http://localhost:9000/mcp \
  -H "Authorization: Bearer your-secure-api-key-1" \
  -H "Content-Type: application/json" \
  -d '...'
```

Or as query parameter:

```bash
curl "http://localhost:9000/mcp?api_key=your-secure-api-key-1"
```

### Generating Keys

```bash
# Generate a random key
openssl rand -hex 32
```

---

## Rate Limiting

### Enabling Rate Limits

```json
"rate_limit": {
  "enabled": true,
  "requests_per_second": 100,
  "burst_size": 200
}
```

### Parameters

| Parameter | Description |
|-----------|-------------|
| `requests_per_second` | Sustained rate limit |
| `burst_size` | Maximum burst capacity |

### How It Works

- Uses token bucket algorithm
- Per-client (by IP address)
- Returns `429 Too Many Requests` when exceeded

---

## HTTP Client Settings

Controls behavior of `http.request` tool.

### `timeout_secs`

Request timeout.

```json
"http_client": {
  "timeout_secs": 30
}
```

### `allowed_hosts`

Whitelist of allowed hosts. Use `["*"]` to allow all.

```json
"http_client": {
  "allowed_hosts": [
    "api.github.com",
    "api.openai.com",
    "*.mycompany.com"
  ]
}
```

### `blocked_hosts`

Blacklist of blocked hosts (for security).

```json
"http_client": {
  "blocked_hosts": [
    "localhost",
    "127.0.0.1",
    "0.0.0.0",
    "169.254.169.254",
    "metadata.google.internal"
  ]
}
```

**Note:** Block internal/metadata endpoints to prevent SSRF attacks.

---

## Plugins

Custom tools via external scripts.

### Basic Plugin

```json
"plugins": [
  {
    "name": "my_tool",
    "description": "My custom tool",
    "command": "/path/to/script.sh",
    "args": [],
    "timeout_secs": 30
  }
]
```

### Full Plugin Configuration

```json
"plugins": [
  {
    "name": "weather",
    "description": "Get weather for a location",
    "command": "/usr/local/bin/weather",
    "args": ["--json"],
    "timeout_secs": 10,
    "env": {
      "API_KEY": "${secrets.WEATHER_KEY}"
    },
    "input_mode": "args",
    "output_mode": "json",
    "schema": {
      "type": "object",
      "properties": {
        "location": {"type": "string", "description": "City name"}
      },
      "required": ["location"]
    }
  }
]
```

### Plugin Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `name` | Yes | Tool name |
| `description` | No | Tool description |
| `command` | Yes | Executable path |
| `args` | No | Static arguments |
| `timeout_secs` | No | Execution timeout |
| `env` | No | Environment variables |
| `input_mode` | No | How to pass input (args, stdin, env) |
| `output_mode` | No | How to parse output (text, json) |
| `schema` | No | JSON Schema for input validation |

---

## Environment-Specific Configs

### Development

```json
{
  "server_name": "nexus-dev",
  "host": "127.0.0.1",
  "port": 9000,
  "database_path": ":memory:",
  "security": {
    "allowed_read_paths": ["."],
    "allowed_write_paths": ["."],
    "allowed_commands": ["*"]
  },
  "auth": {"enabled": false},
  "rate_limit": {"enabled": false}
}
```

### Production

```json
{
  "server_name": "nexus-prod",
  "host": "0.0.0.0",
  "port": 9000,
  "database_path": "/var/lib/nexus/data.db",
  "security": {
    "allowed_read_paths": ["/app/data"],
    "allowed_write_paths": ["/app/output"],
    "allowed_commands": ["git", "npm"]
  },
  "auth": {
    "enabled": true,
    "api_keys": ["${API_KEY}"]
  },
  "rate_limit": {
    "enabled": true,
    "requests_per_second": 50,
    "burst_size": 100
  },
  "http_client": {
    "blocked_hosts": ["localhost", "127.0.0.1", "169.254.169.254"]
  }
}
```

### Claude Desktop

For Claude Desktop integration (stdio mode):

```json
{
  "server_name": "nexus-claude",
  "database_path": "~/.nexus/data.db",
  "security": {
    "allowed_read_paths": ["~"],
    "allowed_write_paths": ["~/Documents"],
    "allowed_commands": ["ls", "cat", "git"]
  }
}
```

---

## CLI Arguments

Override config with CLI arguments:

```bash
# Custom config file
nexus serve --config /path/to/config.json

# Override port
nexus serve --port 3000

# Override host
nexus serve --host 0.0.0.0

# Set log level
nexus serve --log-level debug

# Stdio mode (ignores network settings)
nexus --stdio
```

---

## Environment Variables

Some settings can be set via environment:

```bash
export NEXUS_CONFIG=/path/to/config.json
export NEXUS_LOG_LEVEL=debug
export NEXUS_PORT=9000
```

---

## Secrets Substitution

Use `${secrets.KEY}` in config to reference stored secrets:

```json
{
  "plugins": [
    {
      "name": "my_api",
      "env": {
        "API_KEY": "${secrets.MY_API_KEY}"
      }
    }
  ]
}
```

---

## Validation

Check your config:

```bash
# Dry run - validates config without starting
nexus info --config ./nexus.json
```

---

## Default Values

| Setting | Default |
|---------|---------|
| `server_name` | "nexus" |
| `server_version` | Package version |
| `host` | "127.0.0.1" |
| `port` | 9000 |
| `database_path` | "nexus.db" |
| `allowed_read_paths` | [] (none) |
| `allowed_write_paths` | [] (none) |
| `allowed_commands` | [] (none) |
| `auth.enabled` | false |
| `rate_limit.enabled` | false |
| `rate_limit.requests_per_second` | 100 |
| `rate_limit.burst_size` | 200 |
| `http_client.timeout_secs` | 30 |

---

## Security Recommendations

### Production Checklist

- [ ] Enable authentication (`auth.enabled: true`)
- [ ] Use strong, unique API keys
- [ ] Enable rate limiting
- [ ] Restrict `allowed_read_paths` to minimum
- [ ] Restrict `allowed_write_paths` to minimum
- [ ] Whitelist only needed commands
- [ ] Block internal hosts in `http_client.blocked_hosts`
- [ ] Use HTTPS with reverse proxy (nginx)
- [ ] Set appropriate timeouts
- [ ] Rotate API keys regularly

### Example nginx config

```nginx
server {
    listen 443 ssl;
    server_name nexus.example.com;
    
    ssl_certificate /etc/ssl/certs/nexus.crt;
    ssl_certificate_key /etc/ssl/private/nexus.key;
    
    location / {
        proxy_pass http://127.0.0.1:9000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## Troubleshooting

### Config Not Loading

```bash
# Check if file exists
cat ./nexus.json

# Run with explicit path
nexus serve --config ./nexus.json

# Check for JSON errors
python -m json.tool ./nexus.json
```

### Permission Denied Errors

Check `allowed_*` settings:

```bash
# View current config
nexus info
```

### Rate Limit Issues

Increase limits or disable for development:

```json
"rate_limit": {"enabled": false}
```

### Auth Failures

Verify key format:

```bash
curl -v -H "Authorization: Bearer YOUR_KEY" http://localhost:9000/health
```


