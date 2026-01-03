# Building an MCP Tool Server in Rust: Lessons from Aegis

> How I built a production-ready MCP server that gives any LLM access to files, shell, APIs, and memory — in a single 8MB binary.

---

## The Problem: LLMs Can Think, But They Can't Act

Large Language Models are remarkable thinkers but terrible actors. GPT-4 can write perfect shell commands but can't execute them. Claude can explain how to read a file but can't actually open one. 

The gap between "knowing" and "doing" is where **Model Context Protocol (MCP)** comes in.

MCP is a standardized protocol (developed by Anthropic) that lets AI assistants call external tools. Think of it as an API contract between the brain (LLM) and the hands (your computer).

I built **Aegis** — an MCP tool server in Rust that provides 50+ tools for file I/O, shell execution, HTTP requests, memory persistence, and more. This article shares the architecture decisions, patterns, and lessons learned.

**GitHub**: [github.com/saeedalam/Aegis](https://github.com/saeedalam/Aegis)

---

## Why Rust?

Before diving into architecture, let me address the obvious question: why Rust for an MCP server?

1. **Single Binary Distribution**: No Python virtualenvs, no Node.js dependencies. Just copy and run.
2. **Memory Safety**: When an AI agent has shell access, you want your runtime to be bulletproof.
3. **Performance**: Sub-millisecond tool dispatch matters when you're in a conversation loop.
4. **Async Native**: Tokio's async runtime handles concurrent tool calls elegantly.

The result? An 8MB binary that starts in 50ms and handles thousands of requests per second.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                         LLM                              │
│              (Claude, GPT-4, Ollama, etc.)               │
└──────────────────────┬──────────────────────────────────┘
                       │ MCP Protocol (JSON-RPC 2.0)
                       ▼
┌─────────────────────────────────────────────────────────┐
│                       Aegis                              │
├─────────────────────────────────────────────────────────┤
│  Transport Layer     │  stdio (CLI) or HTTP/SSE        │
├──────────────────────┼──────────────────────────────────┤
│  Protocol Layer      │  JSON-RPC parsing, routing      │
├──────────────────────┼──────────────────────────────────┤
│  Tool Registry       │  Dynamic tool dispatch          │
├──────────────────────┼──────────────────────────────────┤
│  Core Tools (21)     │  fs, cmd, memory, http, utils   │
│  Extra Tools (36)    │  llm, git, vector, workflows    │
│  Plugins             │  User-defined script tools      │
├──────────────────────┼──────────────────────────────────┤
│  Runtime State       │  Config, SQLite, shared state   │
└─────────────────────────────────────────────────────────┘
```

---

## The Tool Trait: Heart of the System

Every capability in Aegis is a `Tool`. Here's the trait definition:

```rust
#[async_trait]
pub trait Tool: Send + Sync + Debug {
    /// Returns the MCP tool definition (name, description, input schema)
    fn definition(&self) -> ToolDefinition;

    /// Executes the tool with JSON arguments
    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError>;
}
```

This design gives us:
- **Async execution**: Tools can perform I/O without blocking
- **Shared state**: Access to config, database, and other tools
- **Type-safe errors**: Rich error types with context
- **Self-describing**: Each tool provides its own JSON Schema

### Implementing a Simple Tool

Here's the complete implementation of the `echo` tool:

```rust
#[derive(Debug)]
pub struct EchoTool;

#[derive(Deserialize)]
struct EchoArgs {
    text: String,
}

#[async_trait]
impl Tool for EchoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "echo".to_string(),
            description: Some("Echoes back the input text.".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to echo back"
                    }
                },
                "required": ["text"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        _state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let args: EchoArgs = serde_json::from_value(arguments)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        Ok(ToolOutput::text(args.text))
    }
}
```

**Key patterns**:
1. Define a struct (can hold configuration)
2. Define an args struct with `#[derive(Deserialize)]`
3. Implement `definition()` with JSON Schema
4. Implement `execute()` with business logic

---

## The Tool Registry: Dynamic Dispatch

Tools are registered at startup and stored in a `HashMap`:

```rust
pub struct ToolRegistry {
    pub tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.definition().name.clone();
        self.tools.insert(name, tool);
    }

    pub async fn execute(
        &self,
        name: &str,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        match self.tools.get(name) {
            Some(tool) => tool.execute(arguments, state).await,
            None => Err(ToolError::NotFound(name.to_string())),
        }
    }

    pub fn list_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }
}
```

The `Arc<dyn Tool>` pattern enables:
- **Polymorphism**: Different tool types in one collection
- **Thread safety**: Safe sharing across async tasks
- **Zero-cost abstraction**: Vtable dispatch is fast

---

## Core vs Extras: The Modular Architecture

After building 57 tools, I learned an important lesson: **more is not better for infrastructure projects**.

I refactored into a clear separation:

### Core Tools (Always Loaded)
```rust
pub fn register_core_tools(registry: &mut ToolRegistry, config: &Config) {
    // Essential primitives only
    registry.register(Arc::new(EchoTool));
    registry.register(Arc::new(GetTimeTool));
    registry.register(Arc::new(FsReadTool::new(config.security.allowed_read_paths.clone())));
    registry.register(Arc::new(FsWriteTool::new(config.security.allowed_write_paths.clone())));
    registry.register(Arc::new(CmdExecTool::new(config.security.allowed_commands.clone())));
    registry.register(Arc::new(MemoryStoreTool));
    // ... 18 tools total
}
```

### Extra Tools (Configurable)
```rust
if config.extras_enabled {
    register_extra_tools(&mut tool_registry, &config);
}
```

This gives users control:
```bash
# Minimal mode (18 tools)
aegis --core-only

# Full mode (54 tools)
aegis
```

---

## The Plugin System: User-Defined Tools

The killer feature is letting users add tools **without touching Rust code**.

Configuration-based plugins:
```json
{
  "plugins": [
    {
      "name": "code_review",
      "description": "Run code review on a file",
      "command": "/usr/local/bin/code-review.sh",
      "args_template": ["--file", "${filepath}"],
      "input_schema": {
        "type": "object",
        "properties": {
          "filepath": { "type": "string" }
        },
        "required": ["filepath"]
      },
      "timeout_secs": 60
    }
  ]
}
```

The `ScriptTool` implementation handles:
- **Argument substitution**: `${param}` → actual value
- **Multiple input modes**: args, stdin, or environment variables
- **Timeout enforcement**: Kill runaway processes
- **Output parsing**: Text or JSON

```rust
impl ScriptTool {
    fn substitute(&self, template: &str, arguments: &Value) -> String {
        let mut result = template.to_string();
        if let Some(obj) = arguments.as_object() {
            for (key, value) in obj {
                let placeholder = format!("${{{}}}", key);
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }
        result
    }
}
```

---

## Security: Because AI + Shell = Danger

Giving an AI shell access is terrifying. Here's how Aegis mitigates risks:

### 1. Path Restrictions
```rust
pub struct FsReadTool {
    allowed_paths: Vec<PathBuf>,
}

impl FsReadTool {
    fn is_path_allowed(&self, path: &Path) -> bool {
        self.allowed_paths.iter().any(|allowed| path.starts_with(allowed))
    }
}
```

### 2. Command Allowlisting
```json
{
  "security": {
    "allowed_commands": ["ls", "cat", "echo", "git"],
    "tool_timeout_secs": 30
  }
}
```

### 3. Timeouts on Everything
```rust
let duration = Duration::from_secs(config.timeout_secs);
timeout(duration, child.wait_with_output())
    .await
    .map_err(|_| ToolError::Timeout(config.timeout_secs))?
```

---

## MCP Protocol Implementation

MCP uses JSON-RPC 2.0 over stdio or HTTP/SSE. Here's the method routing:

```rust
pub enum McpMethod {
    Initialize,
    ToolsList,
    ToolsCall,
    ResourcesList,
    ResourcesRead,
    Ping,
    Unknown(String),
}

impl McpMethod {
    pub fn from_str(s: &str) -> Self {
        match s {
            "initialize" => McpMethod::Initialize,
            "tools/list" => McpMethod::ToolsList,
            "tools/call" => McpMethod::ToolsCall,
            "resources/list" => McpMethod::ResourcesList,
            "resources/read" => McpMethod::ResourcesRead,
            "ping" => McpMethod::Ping,
            _ => McpMethod::Unknown(s.to_string()),
        }
    }
}
```

A typical session:
```json
→ {"jsonrpc":"2.0","method":"initialize","params":{...},"id":1}
← {"jsonrpc":"2.0","result":{"capabilities":{"tools":{}}},"id":1}

→ {"jsonrpc":"2.0","method":"tools/list","id":2}
← {"jsonrpc":"2.0","result":{"tools":[...]},"id":2}

→ {"jsonrpc":"2.0","method":"tools/call","params":{"name":"echo","arguments":{"text":"Hello"}},"id":3}
← {"jsonrpc":"2.0","result":{"content":[{"type":"text","text":"Hello"}]},"id":3}
```

---

## Lessons Learned

### 1. Start Minimal, Add Intentionally
I built 57 tools before realizing I'd lost the project's identity. Infrastructure projects get strong with *less*, not *more*. Define your core early.

### 2. Configuration > Code for Extension
The plugin system is the most valuable feature. Users shouldn't need to write Rust to add tools.

### 3. Traits Make Everything Composable
The `Tool` trait made it trivial to add new capabilities. Rust's trait system shines for this pattern.

### 4. Security Can't Be an Afterthought
Path restrictions, command allowlisting, and timeouts were baked in from day one. Retrofitting security is painful.

### 5. Async Is Worth the Complexity
Tokio's learning curve pays off. Tool calls run concurrently, the server handles thousands of connections, and the code stays readable.

---

## Try It Yourself

```bash
# Clone and build
git clone https://github.com/saeedalam/Aegis.git
cd Aegis
cargo build --release

# List available tools
./target/release/aegis --list-tools

# Run as stdio MCP server (for Claude Desktop)
./target/release/aegis

# Run as HTTP server
./target/release/aegis serve --port 9000
```

### Connect to Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "aegis": {
      "command": "/path/to/aegis"
    }
  }
}
```

---

## What's Next?

- **WebSocket transport** for bidirectional streaming
- **Sandboxed WASM plugins** for untrusted code
- **Tool composition** (chain tools together)
- **Observability** (OpenTelemetry integration)

If you're building AI agents and want a solid foundation for tool execution, give Aegis a try. PRs welcome!

---

**Tags**: #rust #ai #mcp #llm #tooling

**Cover Image Suggestion**: A shield (Aegis) with circuit patterns, connecting to a brain icon

---

*Built with Rust 🦀, powered by Tokio, secured by paranoia.*

