# Integrating Nexus with Claude Desktop

This guide shows you how to connect Nexus to Claude Desktop, giving Claude access to tools, persistent memory, file operations, shell commands, and HTTP requests.

## Quick Setup (5 minutes)

### Step 1: Build Nexus

```bash
cd /path/to/nexus
cargo build --release
```

### Step 2: Find Your Binary Path

```bash
# Get the full path to the nexus binary
realpath target/release/nexus
# Example output: /Users/you/nexus/target/release/nexus
```

### Step 3: Configure Claude Desktop

Edit Claude Desktop's configuration file:

**macOS:**
```bash
code ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

**Windows:**
```bash
code %APPDATA%\Claude\claude_desktop_config.json
```

**Linux:**
```bash
code ~/.config/Claude/claude_desktop_config.json
```

Add Nexus as an MCP server:

```json
{
  "mcpServers": {
    "nexus": {
      "command": "/full/path/to/nexus",
      "args": ["--stdio", "-c", "/path/to/nexus.json"],
      "env": {}
    }
  }
}
```

### Step 4: Create Nexus Configuration (Optional)

Create `nexus.json` in your preferred location:

```json
{
  "server_name": "nexus",
  "database_path": "~/.nexus/memory.db",
  "security": {
    "allowed_read_paths": [
      "~",
      "/tmp"
    ],
    "allowed_write_paths": [
      "/tmp",
      "~/Documents"
    ],
    "allowed_commands": [
      "ls", "cat", "head", "tail", "grep", "find",
      "echo", "date", "pwd", "which",
      "git", "npm", "node", "python3", "cargo"
    ]
  },
  "http_client": {
    "blocked_urls": []
  }
}
```

### Step 5: Restart Claude Desktop

Quit and restart Claude Desktop. Nexus tools will now be available!

---

## Available Tools for Claude

Once connected, Claude has access to these tools:

### 🔧 Basic Tools

| Tool | Description | Example Use |
|------|-------------|-------------|
| `echo` | Echo text back | Testing connectivity |
| `get_time` | Get current time | "What time is it?" |

### 📁 File Operations

| Tool | Description | Example Use |
|------|-------------|-------------|
| `fs.read_file` | Read file contents | "Read my config file" |
| `fs.write_file` | Write to a file | "Save this to notes.txt" |

### 💻 Shell Commands

| Tool | Description | Example Use |
|------|-------------|-------------|
| `cmd.exec` | Run shell commands | "List files in this directory" |

### 🧠 Memory (Persistent)

| Tool | Description | Example Use |
|------|-------------|-------------|
| `memory.store` | Remember something | "Remember my API key is xyz" |
| `memory.recall` | Recall something | "What's my API key?" |
| `memory.list` | List all memories | "What do you remember?" |
| `memory.delete` | Forget something | "Forget my API key" |

### 🌐 HTTP Requests

| Tool | Description | Example Use |
|------|-------------|-------------|
| `http.request` | Make HTTP calls | "Check if example.com is up" |

### 🛠️ Utility Tools

| Tool | Description | Example Use |
|------|-------------|-------------|
| `base64.encode` | Encode to Base64 | "Encode this in base64" |
| `base64.decode` | Decode from Base64 | "Decode this base64" |
| `json.parse` | Parse JSON | "Format this JSON nicely" |
| `json.query` | Query JSON | "Get the 'name' field from this JSON" |
| `uuid.generate` | Generate UUID | "Give me a unique ID" |
| `hash.sha256` | Hash text | "Hash my password" |
| `regex.match` | Match regex | "Find all emails in this text" |
| `regex.replace` | Replace with regex | "Replace all numbers with X" |

---

## Example Conversations with Claude

### Using Memory

```
You: Remember that my favorite programming language is Rust

Claude: I'll store that for you.
[Uses memory.store with key="favorite_language", value="Rust"]

Done! I'll remember that your favorite programming language is Rust.

---

You: What's my favorite language?

Claude: Let me check my memory.
[Uses memory.recall with key="favorite_language"]

Your favorite programming language is Rust!
```

### File Operations

```
You: Read my ~/.bashrc file

Claude: I'll read that file for you.
[Uses fs.read_file with path="~/.bashrc"]

Here's your .bashrc:
[displays file contents]
```

### Making HTTP Requests

```
You: What's the current Bitcoin price?

Claude: I'll check a crypto API.
[Uses http.request to call a price API]

The current Bitcoin price is $XX,XXX.
```

### Shell Commands

```
You: How much disk space do I have?

Claude: Let me check.
[Uses cmd.exec with command="df", args=["-h"]]

Here's your disk usage:
[displays output]
```

---

## Security Considerations

### For Personal Use

The default development config is fine:

```json
{
  "auth": { "enabled": false },
  "rate_limit": { "enabled": false },
  "security": {
    "allowed_read_paths": [".", "~", "/tmp"],
    "allowed_write_paths": [".", "/tmp"],
    "allowed_commands": ["*"]
  }
}
```

### For Shared/Production Use

Enable authentication and restrict access:

```json
{
  "auth": {
    "enabled": true,
    "api_keys": ["YOUR_HASHED_API_KEY"]
  },
  "rate_limit": {
    "enabled": true,
    "requests_per_second": 10
  },
  "security": {
    "allowed_read_paths": ["/approved/path"],
    "allowed_write_paths": ["/tmp"],
    "allowed_commands": ["ls", "cat", "echo"]
  },
  "http_client": {
    "allowed_urls": ["^https://api\\.approved\\.com"]
  }
}
```

### Generating API Key Hashes

```bash
# Generate a random API key
openssl rand -hex 32
# Output: a1b2c3d4e5f6...

# Hash it for the config
echo -n "a1b2c3d4e5f6..." | shasum -a 256
# Output: 5e884898da28047d... (use this in config)
```

---

## Troubleshooting

### Claude doesn't see Nexus tools

1. Check the config path is correct
2. Verify the binary is executable: `chmod +x /path/to/nexus`
3. Test manually: `/path/to/nexus --stdio`
4. Check Claude Desktop logs

### "Permission denied" errors

Update `security.allowed_*` in your `nexus.json`:

```json
{
  "security": {
    "allowed_read_paths": ["/the/path/you/need"],
    "allowed_write_paths": ["/the/path/you/need"],
    "allowed_commands": ["the-command-you-need"]
  }
}
```

### Memory not persisting

Check `database_path` in config:

```json
{
  "database_path": "/absolute/path/to/nexus.db"
}
```

### HTTP requests blocked

Check `http_client.blocked_urls` - by default, localhost/internal IPs are blocked:

```json
{
  "http_client": {
    "blocked_urls": []
  }
}
```

---

## Advanced: Multiple Configurations

You can have different configs for different use cases:

```json
{
  "mcpServers": {
    "nexus-work": {
      "command": "/path/to/nexus",
      "args": ["--stdio", "-c", "~/.nexus/work.json"]
    },
    "nexus-personal": {
      "command": "/path/to/nexus",
      "args": ["--stdio", "-c", "~/.nexus/personal.json"]
    }
  }
}
```

---

## What's Next?

With Nexus connected to Claude, you can:

1. **Build agents** that remember context across conversations
2. **Automate tasks** using shell commands and file operations
3. **Integrate APIs** using HTTP requests
4. **Process data** using JSON/regex/encoding tools

Claude now has persistent memory and real-world capabilities! 🚀


