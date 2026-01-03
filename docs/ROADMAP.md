# Nexus Roadmap: What's Next

## Current Status: v0.2.0 ✅

Nexus is **production-ready for Claude Desktop integration** with:
- 18 built-in tools
- Plugin system for custom tools
- Persistent memory (SQLite)
- Authentication & rate limiting
- Dual transport (stdio + HTTP)

---

## What's Missing for Enterprise/Production Use

### 🔴 Critical (Should Have)

#### 1. Conversation History & Context

**Problem:** Currently only key-value memory. No structured conversation storage.

**Solution:**
```rust
// Add conversation tools
memory.conversation.create  // Start new conversation
memory.conversation.add     // Add message to conversation  
memory.conversation.get     // Get conversation history
memory.conversation.search  // Search across conversations
```

**Impact:** Agents can maintain coherent multi-turn conversations.

---

#### 2. Vector Search / Semantic Memory

**Problem:** Current memory is exact-match only. Can't find "similar" things.

**Solution:**
- Integrate `lance` or `sqlite-vec` for vector storage
- Add embedding generation (local model or API)
- Enable semantic search:

```rust
memory.embed     // Generate embedding for text
memory.search    // Find semantically similar items
```

**Impact:** Agents can find related memories, not just exact matches.

---

#### 3. Better Error Messages & Debugging

**Problem:** When things fail, errors are often cryptic.

**Solution:**
- Structured error responses with codes
- Debug mode with verbose logging
- Tool execution tracing
- Request/response logging to file

**Impact:** Easier debugging and monitoring.

---

#### 4. Streaming Responses

**Problem:** Long-running tools block until complete.

**Solution:**
- Implement SSE streaming for tool output
- Progress updates for slow operations
- Cancellation support

**Impact:** Better UX for long operations.

---

### 🟡 Important (Nice to Have)

#### 5. Tool Chaining / Workflows

**Problem:** Each tool call is independent. Can't compose tools.

**Solution:**
```json
{
  "name": "workflow.run",
  "steps": [
    {"tool": "http.request", "args": {"url": "..."}},
    {"tool": "json.query", "args": {"path": "data.items"}},
    {"tool": "memory.store", "args": {"key": "result"}}
  ]
}
```

**Impact:** Complex multi-step operations in one call.

---

#### 6. WebAssembly Plugins

**Problem:** Script plugins have overhead and security concerns.

**Solution:**
- Support `.wasm` plugins
- Sandboxed execution
- Better performance
- Cross-platform

**Impact:** Safer, faster custom tools.

---

#### 7. Multi-Tenant Mode

**Problem:** Single user/agent only. No isolation.

**Solution:**
- Per-client memory isolation
- Tenant-aware authentication
- Resource quotas per tenant

**Impact:** Shared Nexus instance for multiple agents.

---

#### 8. Observability Dashboard

**Problem:** No visibility into what's happening.

**Solution:**
- Web UI for metrics
- Tool usage graphs
- Memory browser
- Request logs viewer

**Impact:** Easier monitoring and debugging.

---

### 🟢 Future (Could Have)

#### 9. Tool Discovery / Registry

**Problem:** Tools are static. No dynamic discovery.

**Solution:**
- Tool marketplace/registry
- Install plugins from URL
- Version management

---

#### 10. Agent-to-Agent Communication

**Problem:** Agents can't coordinate.

**Solution:**
- Message queues between agents
- Shared state coordination
- Event subscriptions

---

#### 11. Scheduled Tasks

**Problem:** Tools only run on-demand.

**Solution:**
```json
{
  "schedules": [
    {
      "tool": "http.request",
      "cron": "0 * * * *",
      "args": {"url": "..."}
    }
  ]
}
```

---

#### 12. File Watching

**Problem:** Can't react to file changes.

**Solution:**
- Watch directories for changes
- Trigger tools on file events
- Real-time sync

---

## Priority Matrix

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| Conversation History | High | Medium | P0 |
| Vector Search | High | High | P1 |
| Better Errors | Medium | Low | P0 |
| Streaming | Medium | Medium | P1 |
| Tool Chaining | High | Medium | P1 |
| WASM Plugins | Medium | High | P2 |
| Multi-Tenant | Medium | High | P2 |
| Dashboard | Low | Medium | P3 |
| Tool Registry | Low | Medium | P3 |
| Agent Comms | Low | High | P3 |
| Schedules | Low | Medium | P3 |
| File Watch | Low | Low | P3 |

---

## Implementation Plan

### v0.3.0 - Memory Enhancement
- [ ] Conversation history storage
- [ ] Conversation-aware tools
- [ ] Better error messages
- [ ] Debug mode

### v0.4.0 - Intelligence
- [ ] Vector embeddings (local model)
- [ ] Semantic search
- [ ] Memory clustering

### v0.5.0 - Workflows
- [ ] Tool chaining
- [ ] Streaming responses
- [ ] Cancellation

### v1.0.0 - Enterprise
- [ ] Multi-tenant
- [ ] WASM plugins
- [ ] Dashboard
- [ ] Production hardening

---

## Quick Wins (Can Do Now)

1. **Add `memory.search` with SQLite FTS5** - Full-text search without vectors
2. **Add `--debug` flag** - Verbose logging mode
3. **Add `tools.chain`** - Simple sequential execution
4. **Add `/dashboard` endpoint** - Basic HTML metrics page

---

## Community Wishlist

Share your feature requests:
- Open an issue on GitHub
- Tag with `feature-request`
- Vote on existing requests

---

## Contributing

Want to help? Here's how:

1. **Easy**: Documentation, examples, bug fixes
2. **Medium**: New built-in tools, better tests
3. **Hard**: Vector search, WASM, streaming

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.


