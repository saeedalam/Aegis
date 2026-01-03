# Nexus Tools Reference

Complete reference for all 48 built-in tools.

---

## Table of Contents

1. [Core Tools](#core-tools)
2. [File System Tools](#file-system-tools)
3. [Memory Tools](#memory-tools)
4. [Secrets Tools](#secrets-tools)
5. [Conversation Tools](#conversation-tools)
6. [Scheduler Tools](#scheduler-tools)
7. [LLM Tools](#llm-tools)
8. [Notification Tools](#notification-tools)
9. [Workflow Tools](#workflow-tools)
10. [Git Tools](#git-tools)
11. [HTTP Tools](#http-tools)
12. [Data Tools](#data-tools)
13. [Crypto Tools](#crypto-tools)
14. [Text Tools](#text-tools)
15. [System Tools](#system-tools)

---

## Core Tools

### `echo`

Echoes back the input text. Useful for testing.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | string | Yes | Text to echo back |

**Example:**

```json
{
  "name": "echo",
  "arguments": {
    "text": "Hello, World!"
  }
}
```

**Response:**

```
Hello, World!
```

---

### `get_time`

Returns the current server time in ISO 8601 format.

**Parameters:** None

**Example:**

```json
{
  "name": "get_time",
  "arguments": {}
}
```

**Response:**

```json
{
  "time": "2026-01-03T12:00:00+00:00",
  "timestamp": 1767434400,
  "timezone": "UTC"
}
```

---

### `uuid.generate`

Generates a new UUID v4.

**Parameters:** None

**Example:**

```json
{
  "name": "uuid.generate",
  "arguments": {}
}
```

**Response:**

```json
{
  "uuid": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

## File System Tools

### `fs.read_file`

Reads the contents of a file. Only allowed paths can be accessed.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | Yes | File path to read |
| `encoding` | string | No | Encoding (default: utf-8) |

**Example:**

```json
{
  "name": "fs.read_file",
  "arguments": {
    "path": "/home/user/config.json"
  }
}
```

---

### `fs.write_file`

Writes content to a file. Only allowed paths can be accessed.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | Yes | File path to write |
| `content` | string | Yes | Content to write |
| `append` | boolean | No | Append instead of overwrite |

**Example:**

```json
{
  "name": "fs.write_file",
  "arguments": {
    "path": "/home/user/output.txt",
    "content": "Hello, World!"
  }
}
```

---

## Memory Tools

### `memory.store`

Stores a value in the key-value memory store.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | string | Yes | Storage key |
| `value` | any | Yes | Value to store (any JSON type) |
| `ttl` | integer | No | Time-to-live in seconds |

**Example:**

```json
{
  "name": "memory.store",
  "arguments": {
    "key": "user_preferences",
    "value": { "theme": "dark", "language": "en" },
    "ttl": 86400
  }
}
```

---

### `memory.recall`

Recalls a value from the key-value memory store.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | string | Yes | Key to recall |

**Example:**

```json
{
  "name": "memory.recall",
  "arguments": {
    "key": "user_preferences"
  }
}
```

---

### `memory.list`

Lists all keys in the memory store.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `prefix` | string | No | Filter keys by prefix |

**Example:**

```json
{
  "name": "memory.list",
  "arguments": {
    "prefix": "user_"
  }
}
```

---

### `memory.delete`

Deletes a key from the memory store.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | string | Yes | Key to delete |

---

## Secrets Tools

### `secrets.set`

Securely stores a secret (API key, token, password).

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | string | Yes | Secret name (e.g., OPENAI_KEY) |
| `value` | string | Yes | Secret value |
| `description` | string | No | Optional description |

**Example:**

```json
{
  "name": "secrets.set",
  "arguments": {
    "key": "OPENAI_KEY",
    "value": "sk-xxx...",
    "description": "OpenAI API key for production"
  }
}
```

**Usage:** Reference secrets in configs as `${secrets.OPENAI_KEY}`

---

### `secrets.get`

Retrieves a stored secret value.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | string | Yes | Secret name to retrieve |

---

### `secrets.list`

Lists all stored secret keys (not values).

**Parameters:** None

---

### `secrets.delete`

Deletes a stored secret.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | string | Yes | Secret name to delete |

---

## Conversation Tools

### `conversation.create`

Creates a new conversation thread.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `title` | string | No | Conversation title |
| `metadata` | object | No | Optional metadata |

**Example:**

```json
{
  "name": "conversation.create",
  "arguments": {
    "title": "Project Discussion"
  }
}
```

**Response:**

```json
{
  "success": true,
  "conversation_id": "abc-123-def",
  "title": "Project Discussion"
}
```

---

### `conversation.add`

Adds a message to a conversation.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `conversation_id` | string | Yes | Conversation ID |
| `role` | string | Yes | Message role: user, assistant, system |
| `content` | string | Yes | Message content |

**Example:**

```json
{
  "name": "conversation.add",
  "arguments": {
    "conversation_id": "abc-123-def",
    "role": "user",
    "content": "What's the project status?"
  }
}
```

---

### `conversation.get`

Gets messages from a conversation.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `conversation_id` | string | Yes | Conversation ID |
| `limit` | integer | No | Max messages (default: 50) |

---

### `conversation.list`

Lists all conversations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `limit` | integer | No | Max results (default: 20) |

---

### `conversation.search`

Searches messages across all conversations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `query` | string | Yes | Search query |
| `limit` | integer | No | Max results (default: 20) |

**Example:**

```json
{
  "name": "conversation.search",
  "arguments": {
    "query": "project deadline"
  }
}
```

---

## Scheduler Tools

### `scheduler.create`

Creates a new scheduled task with cron expression.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | Yes | Task name |
| `cron` | string | Yes | Cron expression |
| `tool` | string | Yes | Tool to execute |
| `args` | object | No | Tool arguments |

**Cron Format:** `minute hour day month weekday`

**Examples:**
| Expression | Description |
|------------|-------------|
| `* * * * *` | Every minute |
| `0 * * * *` | Every hour |
| `0 9 * * *` | Every day at 9 AM |
| `*/5 * * * *` | Every 5 minutes |
| `0 9 * * 1` | Every Monday at 9 AM |

**Example:**

```json
{
  "name": "scheduler.create",
  "arguments": {
    "name": "hourly-report",
    "cron": "0 * * * *",
    "tool": "http.request",
    "args": { "url": "https://api.example.com/report" }
  }
}
```

---

### `scheduler.list`

Lists all scheduled tasks.

**Parameters:** None

---

### `scheduler.delete`

Deletes a scheduled task.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | string | Yes | Task ID |

---

### `scheduler.toggle`

Enables or disables a scheduled task.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | string | Yes | Task ID |
| `enabled` | boolean | Yes | Enable/disable |

---

### `scheduler.run`

Manually triggers a scheduled task immediately.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | string | Yes | Task ID |

---

## LLM Tools

### `llm.openai`

Calls OpenAI Chat API (GPT-4, GPT-3.5, etc.).

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `prompt` | string | No* | Simple prompt |
| `messages` | array | No* | Chat messages array |
| `model` | string | No | Model (default: gpt-4o-mini) |
| `temperature` | number | No | Temperature 0-2 (default: 0.7) |
| `max_tokens` | integer | No | Max tokens to generate |
| `api_key` | string | No | API key (uses OPENAI_KEY secret) |

\*Either `prompt` or `messages` is required.

**Example with prompt:**

```json
{
  "name": "llm.openai",
  "arguments": {
    "prompt": "Explain quantum computing in simple terms"
  }
}
```

**Example with messages:**

```json
{
  "name": "llm.openai",
  "arguments": {
    "messages": [
      { "role": "system", "content": "You are a helpful assistant." },
      { "role": "user", "content": "What is 2+2?" }
    ],
    "model": "gpt-4o",
    "temperature": 0.3
  }
}
```

**Response:**

```json
{
  "content": "Quantum computing uses quantum mechanics...",
  "model": "gpt-4o-mini",
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 50,
    "total_tokens": 60
  }
}
```

---

### `llm.anthropic`

Calls Anthropic Claude API.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `prompt` | string | No* | Simple prompt |
| `messages` | array | No* | Chat messages array |
| `system` | string | No | System prompt |
| `model` | string | No | Model (default: claude-3-haiku-20240307) |
| `max_tokens` | integer | No | Max tokens (default: 1024) |
| `api_key` | string | No | API key (uses ANTHROPIC_KEY secret) |

**Example:**

```json
{
  "name": "llm.anthropic",
  "arguments": {
    "prompt": "Write a haiku about programming",
    "model": "claude-3-sonnet-20240229"
  }
}
```

---

### `llm.embed`

Generates text embeddings using OpenAI.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | string | No* | Single text to embed |
| `texts` | array | No* | Multiple texts to embed |
| `model` | string | No | Model (default: text-embedding-3-small) |
| `api_key` | string | No | API key (uses OPENAI_KEY secret) |

**Example:**

```json
{
  "name": "llm.embed",
  "arguments": {
    "texts": ["Hello world", "Goodbye world"]
  }
}
```

**Response:**

```json
{
  "model": "text-embedding-3-small",
  "embeddings": [
    {"index": 0, "embedding": [0.1, 0.2, ...], "dimensions": 1536},
    {"index": 1, "embedding": [0.3, 0.4, ...], "dimensions": 1536}
  ]
}
```

---

## Notification Tools

### `notify.slack`

Sends a Slack notification via webhook.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | string | Yes | Message text |
| `channel` | string | No | Channel override |
| `username` | string | No | Bot username override |
| `icon_emoji` | string | No | Icon emoji (e.g., `:robot_face:`) |
| `blocks` | array | No | Slack Block Kit blocks |
| `webhook_url` | string | No | Webhook URL (uses SLACK_WEBHOOK_URL) |

**Setup:**

```json
{
  "name": "secrets.set",
  "arguments": {
    "key": "SLACK_WEBHOOK_URL",
    "value": "https://hooks.slack.com/services/T.../B.../xxx"
  }
}
```

**Example:**

```json
{
  "name": "notify.slack",
  "arguments": {
    "text": "Deployment complete! 🚀",
    "username": "Deploy Bot",
    "icon_emoji": ":rocket:"
  }
}
```

---

### `notify.discord`

Sends a Discord notification via webhook.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `content` | string | Yes | Message content |
| `username` | string | No | Bot username override |
| `avatar_url` | string | No | Avatar URL override |
| `embeds` | array | No | Discord embeds |
| `webhook_url` | string | No | Webhook URL (uses DISCORD_WEBHOOK_URL) |

**Example:**

```json
{
  "name": "notify.discord",
  "arguments": {
    "content": "Build passed! ✅"
  }
}
```

---

### `notify.email`

Sends an email via Resend or SendGrid.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `to` | string | Yes | Recipient email |
| `subject` | string | Yes | Email subject |
| `body` | string | Yes\* | Plain text body |
| `html` | string | No | HTML body |
| `from` | string | No | Sender (uses EMAIL_FROM secret) |
| `provider` | string | No | resend or sendgrid (default: resend) |

**Setup:**

```json
{
  "name": "secrets.set",
  "arguments": { "key": "RESEND_KEY", "value": "re_xxx" }
}
```

**Example:**

```json
{
  "name": "notify.email",
  "arguments": {
    "to": "team@company.com",
    "subject": "Daily Report",
    "body": "Everything is running smoothly.",
    "html": "<h1>Daily Report</h1><p>All systems operational.</p>"
  }
}
```

---

### `webhook.send`

Sends a generic webhook notification.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `url` | string | Yes | Webhook URL |
| `event` | string | No | Event type (default: notification) |
| `data` | object | No | Event payload |
| `headers` | object | No | Custom headers |

**Example:**

```json
{
  "name": "webhook.send",
  "arguments": {
    "url": "https://api.example.com/webhook",
    "event": "build.completed",
    "data": {
      "status": "success",
      "commit": "abc123"
    },
    "headers": {
      "Authorization": "Bearer ${secrets.WEBHOOK_TOKEN}"
    }
  }
}
```

---

## Workflow Tools

### `workflow.run`

Executes a workflow - a sequence of tool calls with variable passing.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | No | Workflow name for logging |
| `steps` | array | Yes | Array of workflow steps |
| `context` | object | No | Initial context variables |

**Step Format:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | No | Step ID for referencing |
| `tool` | string | Yes | Tool to call |
| `args` | object | No | Tool arguments |
| `condition` | string | No | Skip condition |

**Variable Substitution:**
Use `{{step_id}}` or `{{step_id.field}}` to reference previous outputs.

**Example:**

```json
{
  "name": "workflow.run",
  "arguments": {
    "name": "deploy-and-notify",
    "steps": [
      {
        "id": "time",
        "tool": "get_time"
      },
      {
        "id": "status",
        "tool": "git.status",
        "args": { "path": "/app" }
      },
      {
        "id": "notify",
        "tool": "notify.slack",
        "args": {
          "text": "Deploy at {{time.time}}: {{status.changes_count}} changes"
        },
        "condition": "status.clean == false"
      }
    ]
  }
}
```

**Conditions:**
| Format | Description |
|--------|-------------|
| `key exists` | Check if key exists |
| `key empty` | Check if key is empty |
| `key == value` | Equality check |
| `key != value` | Inequality check |
| `key > value` | Greater than (numeric) |
| `key < value` | Less than (numeric) |

---

### `workflow.define`

Saves a workflow definition for later use.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | Yes | Workflow name |
| `description` | string | No | Description |
| `steps` | array | Yes | Workflow steps |
| `inputs` | array | No | Required input parameters |

**Example:**

```json
{
  "name": "workflow.define",
  "arguments": {
    "name": "daily-report",
    "description": "Send daily status report",
    "steps": [
      { "id": "time", "tool": "get_time" },
      {
        "id": "notify",
        "tool": "notify.slack",
        "args": { "text": "{{message}}" }
      }
    ],
    "inputs": ["message"]
  }
}
```

---

### `workflow.execute`

Executes a previously saved workflow.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | Yes | Workflow name |
| `inputs` | object | No | Input parameters |

**Example:**

```json
{
  "name": "workflow.execute",
  "arguments": {
    "name": "daily-report",
    "inputs": {
      "message": "All systems operational!"
    }
  }
}
```

---

### `workflow.list`

Lists all saved workflows.

**Parameters:** None

---

## Git Tools

### `git.status`

Gets the git status of a repository.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | No | Repository path (default: .) |

**Example:**

```json
{
  "name": "git.status",
  "arguments": {
    "path": "/path/to/repo"
  }
}
```

**Response:**

```json
{
  "branch": "main",
  "clean": false,
  "changes_count": 3,
  "changes": [
    { "status": "M", "file": "src/main.rs" },
    { "status": "A", "file": "src/new.rs" },
    { "status": "??", "file": "untracked.txt" }
  ]
}
```

---

### `git.log`

Gets recent git commits.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | No | Repository path |
| `count` | integer | No | Number of commits (default: 10) |

**Response:**

```json
{
  "count": 2,
  "commits": [
    {
      "hash": "abc123...",
      "short_hash": "abc123",
      "author": "John Doe",
      "email": "john@example.com",
      "timestamp": 1767400000,
      "message": "Add new feature"
    }
  ]
}
```

---

### `git.diff`

Gets git diff for changes.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | No | Repository path |
| `file` | string | No | Specific file to diff |
| `staged` | boolean | No | Show staged changes only |
| `commit` | string | No | Commit to diff against |

---

### `git.commit`

Creates a git commit with staged changes.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | No | Repository path |
| `message` | string | Yes | Commit message |
| `add_all` | boolean | No | Stage all changes first |

**Example:**

```json
{
  "name": "git.commit",
  "arguments": {
    "message": "feat: add new feature",
    "add_all": true
  }
}
```

---

### `git.branch`

Lists branches or creates/switches branches.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | No | Repository path |
| `create` | string | No | Create new branch |
| `checkout` | string | No | Switch to branch |

**Example - List:**

```json
{
  "name": "git.branch",
  "arguments": {}
}
```

**Example - Create:**

```json
{
  "name": "git.branch",
  "arguments": {
    "create": "feature/new-feature"
  }
}
```

---

## HTTP Tools

### `http.request`

Makes an HTTP request to a URL.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `url` | string | Yes | Request URL |
| `method` | string | No | GET, POST, PUT, DELETE, PATCH |
| `headers` | object | No | Request headers |
| `body` | string/object | No | Request body |
| `timeout` | integer | No | Timeout in seconds |

**Example:**

```json
{
  "name": "http.request",
  "arguments": {
    "url": "https://api.example.com/data",
    "method": "POST",
    "headers": {
      "Authorization": "Bearer ${secrets.API_KEY}",
      "Content-Type": "application/json"
    },
    "body": { "key": "value" }
  }
}
```

---

## Data Tools

### `json.parse`

Parses a JSON string.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `json` | string | Yes | JSON string to parse |

---

### `json.query`

Queries a JSON object using dot-notation path.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `json` | string/object | Yes | JSON to query |
| `path` | string | Yes | Path (e.g., `data.items[0].name`) |

**Example:**

```json
{
  "name": "json.query",
  "arguments": {
    "json": "{\"users\": [{\"name\": \"Alice\"}]}",
    "path": "users[0].name"
  }
}
```

---

### `base64.encode`

Encodes text to Base64.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | string | Yes | Text to encode |

---

### `base64.decode`

Decodes Base64 to text.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `encoded` | string | Yes | Base64 to decode |

---

## Crypto Tools

### `hash.sha256`

Computes SHA-256 hash of text.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | string | Yes | Text to hash |

**Response:**

```json
{
  "hash": "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
}
```

---

## Text Tools

### `regex.match`

Tests if text matches a regex pattern.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | string | Yes | Text to test |
| `pattern` | string | Yes | Regex pattern |

**Response:**

```json
{
  "matches": true,
  "groups": ["full match", "group1", "group2"]
}
```

---

### `regex.replace`

Replaces text matching a regex pattern.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `text` | string | Yes | Input text |
| `pattern` | string | Yes | Regex pattern |
| `replacement` | string | Yes | Replacement text |

---

## System Tools

### `cmd.exec`

Executes a shell command. Only allowed commands can be run.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `command` | string | Yes | Command to execute |
| `args` | array | No | Command arguments |
| `cwd` | string | No | Working directory |
| `timeout` | integer | No | Timeout in seconds |

**Example:**

```json
{
  "name": "cmd.exec",
  "arguments": {
    "command": "ls",
    "args": ["-la"],
    "cwd": "/home/user"
  }
}
```

**Security:**
Only commands listed in `security.allowed_commands` can be executed.

---

## Quick Reference

### By Category

| Category      | Tools                                                                                                     |
| ------------- | --------------------------------------------------------------------------------------------------------- |
| Core          | `echo`, `get_time`, `uuid.generate`                                                                       |
| Files         | `fs.read_file`, `fs.write_file`                                                                           |
| Memory        | `memory.store`, `memory.recall`, `memory.list`, `memory.delete`                                           |
| Secrets       | `secrets.set`, `secrets.get`, `secrets.list`, `secrets.delete`                                            |
| Conversations | `conversation.create`, `conversation.add`, `conversation.get`, `conversation.list`, `conversation.search` |
| Scheduler     | `scheduler.create`, `scheduler.list`, `scheduler.delete`, `scheduler.toggle`, `scheduler.run`             |
| LLM           | `llm.openai`, `llm.anthropic`, `llm.embed`                                                                |
| Notifications | `notify.slack`, `notify.discord`, `notify.email`, `webhook.send`                                          |
| Workflows     | `workflow.run`, `workflow.define`, `workflow.execute`, `workflow.list`                                    |
| Git           | `git.status`, `git.log`, `git.diff`, `git.commit`, `git.branch`                                           |
| HTTP          | `http.request`                                                                                            |
| Data          | `json.parse`, `json.query`, `base64.encode`, `base64.decode`                                              |
| Crypto        | `hash.sha256`                                                                                             |
| Text          | `regex.match`, `regex.replace`                                                                            |
| System        | `cmd.exec`                                                                                                |

### Total: 48 Tools

