# Aegis + Claude Integration Guide

## Overview

Aegis provides 57 tools that Claude (Code, Desktop, or API) can use via the Model Context Protocol (MCP). This guide covers all integration methods.

---

## Claude Code (CLI) — Recommended

Claude Code is the terminal-based Claude interface. This is the easiest integration.

### Step 1: Install Aegis

```bash
# Clone and build
git clone https://github.com/saeedalam/Aegis
cd Aegis
cargo build --release

# Copy to a permanent location
mkdir -p ~/bin
cp target/release/aegis ~/bin/aegis
```

### Step 2: Add to Claude Code

```bash
# Add globally (available in all projects)
claude mcp add --scope user aegis -- ~/bin/aegis --stdio

# Verify connection
claude mcp list
```

Expected output:
```
aegis: /Users/you/bin/aegis --stdio - ✓ Connected
```

### Step 3: Use in Claude Code

Start Claude Code and ask:

```
> What tools do you have from aegis?
```

Claude will list all 57 available tools.

### Example Usage

```
> Use aegis to store "hello world" in memory with key "greeting"

⏺ I'll use the memory_store tool from aegis.

⏺ mcp__aegis__memory_store({"key": "greeting", "value": "hello world"})
  ⎿ Stored 'greeting' successfully

> Now recall it

⏺ mcp__aegis__memory_recall({"key": "greeting"})
  ⎿ hello world
```

### Managing Aegis in Claude Code

```bash
# List all MCP servers
claude mcp list

# Get details about aegis
claude mcp get aegis

# Remove aegis
claude mcp remove aegis --scope user

# Re-add with different path
claude mcp add --scope user aegis -- /new/path/to/aegis --stdio
```

### Troubleshooting Claude Code

**"Failed to connect"**
```bash
# Test aegis manually
echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | ~/bin/aegis --stdio
```

**"No MCP servers configured"**
```bash
# Check if added correctly
cat ~/.claude.json | grep -A5 aegis
```

**Tools not appearing**
```bash
# Restart Claude Code after adding
# Make sure to use --stdio flag
claude mcp remove aegis --scope user
claude mcp add --scope user aegis -- ~/bin/aegis --stdio
```

---

## Claude Desktop (macOS App)

### Step 1: Locate Config File

```bash
# Open config file
open ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

### Step 2: Add Aegis

Edit the file to include:

```json
{
  "mcpServers": {
    "aegis": {
      "command": "/Users/YOUR_USERNAME/bin/aegis",
      "args": ["--stdio"]
    }
  }
}
```

**Important:** Replace `YOUR_USERNAME` with your actual username.

### Step 3: Restart Claude Desktop

Quit Claude Desktop completely (Cmd+Q) and reopen.

### Step 4: Verify

In Claude Desktop, ask:
```
What MCP servers do you have access to?
```

---

## Cursor IDE

### Step 1: Open MCP Settings

In Cursor:
1. Open Settings (Cmd+,)
2. Search for "MCP"
3. Click "Edit in settings.json"

### Step 2: Add Aegis

```json
{
  "mcp.servers": {
    "aegis": {
      "command": "/Users/YOUR_USERNAME/bin/aegis",
      "args": ["--stdio"]
    }
  }
}
```

### Step 3: Restart Cursor

Reload the window or restart Cursor.

---

## Available Tools

Once connected, Claude has access to these tool categories:

| Category | Tools | Use For |
|----------|-------|---------|
| **Memory** | `memory_store`, `memory_recall`, `memory_list`, `memory_delete` | Persistent storage across sessions |
| **Files** | `fs_read_file`, `fs_write_file` | Reading/writing files |
| **Git** | `git_status`, `git_diff`, `git_log`, `git_commit`, `git_branch` | Version control |
| **Web** | `http_request`, `web_search`, `web_extract` | API calls, web scraping |
| **LLM** | `llm_openai`, `llm_anthropic`, `llm_embed` | Call other LLMs |
| **Vector** | `vector_store`, `vector_search` | Semantic search |
| **Secrets** | `secrets_set`, `secrets_get`, `secrets_list` | Secure credential storage |
| **Conversations** | `conversation_create`, `conversation_add`, `conversation_get` | Multi-turn history |
| **Workflows** | `workflow_run`, `workflow_define`, `workflow_execute` | Multi-step automation |
| **Scheduler** | `scheduler_create`, `scheduler_list`, `scheduler_run` | Cron-like tasks |
| **Utilities** | `echo`, `get_time`, `uuid_generate`, `hash_sha256`, `base64_*`, `json_*`, `regex_*` | Data processing |
| **System** | `cmd_exec`, `env_get`, `env_list`, `sys_info` | System interaction |
| **Notifications** | `notify_slack`, `notify_discord`, `notify_email`, `webhook_send` | Send alerts |

---

## Security Configuration

By default, Aegis restricts file and command access. To customize:

Create `~/bin/aegis.json`:

```json
{
  "security": {
    "allowed_read_paths": [
      "/Users/YOUR_USERNAME/projects",
      "/tmp"
    ],
    "allowed_write_paths": [
      "/tmp",
      "/Users/YOUR_USERNAME/projects/output"
    ],
    "allowed_commands": [
      "ls", "cat", "echo", "date", "pwd", "git",
      "python", "node", "npm", "cargo"
    ],
    "tool_timeout_secs": 60
  }
}
```

Then update the MCP config to use `-c`:

```bash
claude mcp remove aegis --scope user
claude mcp add --scope user aegis -- ~/bin/aegis --stdio -c ~/bin/aegis.json
```

---

## Common Use Cases

### 1. Persistent Memory

```
> Remember that the project deadline is January 15th
⏺ mcp__aegis__memory_store({"key": "deadline", "value": "January 15th"})

# Later session...
> What's the project deadline?
⏺ mcp__aegis__memory_recall({"key": "deadline"})
⎿ January 15th
```

### 2. Save Conversation Context

```
> Start a new conversation called "feature-discussion"
⏺ mcp__aegis__conversation_create({"title": "feature-discussion"})

> Add this to the conversation: "We decided to use React for the frontend"
⏺ mcp__aegis__conversation_add({"conversation_id": "...", "role": "user", "content": "..."})

# Later...
> What did we decide about the frontend?
⏺ mcp__aegis__conversation_search({"query": "frontend"})
```

### 3. Web Search

```
> Search for "Rust async best practices"
⏺ mcp__aegis__web_search({"query": "Rust async best practices"})
```

### 4. API Calls

```
> Get the current weather from wttr.in
⏺ mcp__aegis__http_request({"url": "https://wttr.in/?format=3", "method": "GET"})
```

### 5. Git Operations

```
> What's the git status?
⏺ mcp__aegis__git_status({"path": "."})

> Show me the last 5 commits
⏺ mcp__aegis__git_log({"path": ".", "limit": 5})
```

---

## Disabling Extra Tools

For a minimal setup (21 core tools only):

```bash
claude mcp remove aegis --scope user
claude mcp add --scope user aegis -- ~/bin/aegis --stdio --core-only
```

---

## Uninstalling

```bash
# Remove from Claude Code
claude mcp remove aegis --scope user

# Remove binary
rm ~/bin/aegis

# Remove database (optional)
rm ~/bin/aegis.db
```

---

<p align="center">
  <strong>Aegis — Secure local MCP tool server with script plugins</strong>
</p>
