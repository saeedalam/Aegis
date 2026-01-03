# Nexus Plugin System

Add custom tools to Nexus without writing Rust code. Define tools in your configuration file that execute external scripts, commands, or APIs.

## Quick Start

Add a `plugins` array to your `nexus.json`:

```json
{
  "plugins": [
    {
      "name": "weather.get",
      "description": "Gets weather for a city",
      "command": "curl",
      "args_template": ["-s", "wttr.in/${city}?format=3"],
      "input_schema": {
        "type": "object",
        "properties": {
          "city": { "type": "string" }
        },
        "required": ["city"]
      }
    }
  ]
}
```

Now Claude (or any MCP client) can use it:

```
You: What's the weather in Tokyo?
Claude: [Uses weather.get tool with city="Tokyo"]
The weather in Tokyo is ☀️ 12°C
```

## Plugin Configuration

### Required Fields

| Field | Description |
|-------|-------------|
| `name` | Unique tool name (e.g., `weather.get`, `my.tool`) |
| `command` | Path to executable or command name |

### Optional Fields

| Field | Default | Description |
|-------|---------|-------------|
| `description` | `null` | Human-readable description |
| `args_template` | `[]` | Command arguments with `${param}` substitution |
| `working_dir` | Current dir | Working directory for command |
| `env` | `{}` | Environment variables |
| `timeout_secs` | `30` | Maximum execution time |
| `input_schema` | `{}` | JSON Schema for parameters |
| `input_mode` | `"args"` | How to pass input: `args`, `stdin`, `env` |
| `output_mode` | `"text"` | How to parse output: `text`, `json` |

## Input Modes

### `args` (Default)

Arguments are substituted into `args_template`:

```json
{
  "name": "greet",
  "command": "echo",
  "args_template": ["Hello, ${name}!"],
  "input_mode": "args"
}
```

Call: `greet(name="World")` → Runs: `echo "Hello, World!"`

### `stdin`

Arguments are passed as JSON to stdin:

```json
{
  "name": "process_json",
  "command": "python3",
  "args_template": ["process.py"],
  "input_mode": "stdin"
}
```

The script receives `{"param1": "value1", ...}` on stdin.

### `env`

Arguments become environment variables:

```json
{
  "name": "my_script",
  "command": "./script.sh",
  "input_mode": "env"
}
```

Call: `my_script(name="Alice", age=30)` sets:
- `NEXUS_ARG_NAME=Alice`
- `NEXUS_ARG_AGE=30`
- `NEXUS_ARGS_JSON={"name":"Alice","age":30}`

## Output Modes

### `text` (Default)

Output is returned as-is:

```json
{
  "name": "uptime",
  "command": "uptime",
  "output_mode": "text"
}
```

### `json`

Output is parsed and pretty-printed as JSON:

```json
{
  "name": "api_call",
  "command": "curl",
  "args_template": ["-s", "https://api.example.com/data"],
  "output_mode": "json"
}
```

## Examples

### Python Script Tool

```json
{
  "name": "python.eval",
  "description": "Evaluate a Python expression",
  "command": "python3",
  "args_template": ["-c", "print(${expression})"],
  "timeout_secs": 5,
  "input_schema": {
    "type": "object",
    "properties": {
      "expression": { "type": "string" }
    },
    "required": ["expression"]
  }
}
```

### Shell Script Tool

Create `tools/analyze.sh`:

```bash
#!/bin/bash
# Receives JSON on stdin
INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.file')
wc -l "$FILE" | awk '{print "Lines:", $1}'
```

Register it:

```json
{
  "name": "file.analyze",
  "description": "Count lines in a file",
  "command": "./tools/analyze.sh",
  "input_mode": "stdin",
  "input_schema": {
    "type": "object",
    "properties": {
      "file": { "type": "string" }
    },
    "required": ["file"]
  }
}
```

### API Integration Tool

```json
{
  "name": "github.user",
  "description": "Get GitHub user info",
  "command": "curl",
  "args_template": [
    "-s",
    "-H", "Accept: application/vnd.github.v3+json",
    "https://api.github.com/users/${username}"
  ],
  "timeout_secs": 10,
  "output_mode": "json",
  "input_schema": {
    "type": "object",
    "properties": {
      "username": { "type": "string" }
    },
    "required": ["username"]
  }
}
```

### Database Query Tool

```json
{
  "name": "db.query",
  "description": "Run a SQLite query",
  "command": "sqlite3",
  "args_template": ["-json", "mydata.db", "${query}"],
  "output_mode": "json",
  "timeout_secs": 10,
  "input_schema": {
    "type": "object",
    "properties": {
      "query": { "type": "string" }
    },
    "required": ["query"]
  }
}
```

### Environment Variables Tool

```json
{
  "name": "notify.slack",
  "description": "Send a Slack notification",
  "command": "curl",
  "args_template": [
    "-X", "POST",
    "-H", "Content-Type: application/json",
    "-d", "{\"text\": \"${message}\"}",
    "${SLACK_WEBHOOK_URL}"
  ],
  "env": {
    "SLACK_WEBHOOK_URL": "https://hooks.slack.com/services/xxx"
  },
  "input_schema": {
    "type": "object",
    "properties": {
      "message": { "type": "string" }
    },
    "required": ["message"]
  }
}
```

## Full Configuration Example

```json
{
  "server_name": "nexus",
  "database_path": "nexus.db",
  
  "plugins": [
    {
      "name": "weather.get",
      "description": "Get weather for a city",
      "command": "curl",
      "args_template": ["-s", "wttr.in/${city}?format=3"],
      "timeout_secs": 10,
      "input_schema": {
        "type": "object",
        "properties": {
          "city": { "type": "string", "description": "City name" }
        },
        "required": ["city"]
      }
    },
    {
      "name": "system.info",
      "description": "Get system information",
      "command": "uname",
      "args_template": ["-a"],
      "timeout_secs": 5
    },
    {
      "name": "python.run",
      "description": "Run Python code",
      "command": "python3",
      "args_template": ["-c", "${code}"],
      "timeout_secs": 30,
      "input_schema": {
        "type": "object",
        "properties": {
          "code": { "type": "string" }
        },
        "required": ["code"]
      }
    }
  ],
  
  "security": {
    "allowed_commands": ["ls", "cat", "echo"]
  }
}
```

## Testing Plugins

```bash
# List all tools (including plugins)
nexus -c nexus.json tools

# Run a plugin tool
nexus -c nexus.json run weather.get --args '{"city": "Paris"}'

# With JSON output
nexus -c nexus.json run weather.get --args '{"city": "Paris"}' --format json
```

## Security Notes

1. **Plugins can execute arbitrary commands** - only add plugins you trust
2. **Validate inputs** in your scripts to prevent injection
3. **Use timeouts** to prevent runaway processes
4. **Limit network access** for scripts that don't need it

## Troubleshooting

### "Command not found"

Ensure the command is installed and in PATH, or use absolute paths:

```json
{
  "command": "/usr/local/bin/my-tool"
}
```

### "Timeout"

Increase `timeout_secs`:

```json
{
  "timeout_secs": 60
}
```

### "Exit status non-zero"

Check stderr output. The error message includes the exit code and stderr.

### Parameters not substituted

Ensure you're using `${param}` syntax in `args_template`:

```json
{
  "args_template": ["--input", "${my_param}"]
}
```

---

With plugins, Nexus becomes infinitely extensible. You can integrate any command-line tool, script, or API as an MCP tool for your AI agents! 🔌


