# LLM Integration Guide

Nexus provides direct integration with major LLM providers, enabling AI agents to leverage other AI models for reasoning, code generation, and more.

---

## Overview

| Tool            | Provider  | Use Case                 |
| --------------- | --------- | ------------------------ |
| `llm.openai`    | OpenAI    | Chat with GPT-4, GPT-3.5 |
| `llm.anthropic` | Anthropic | Chat with Claude         |
| `llm.embed`     | OpenAI    | Generate embeddings      |

---

## Setup

### 1. Store API Keys

```bash
# OpenAI
nexus run secrets.set --args '{"key": "OPENAI_KEY", "value": "sk-xxx..."}'

# Anthropic
nexus run secrets.set --args '{"key": "ANTHROPIC_KEY", "value": "sk-ant-xxx..."}'
```

### 2. Verify

```bash
nexus run secrets.list
```

---

## OpenAI Integration

### Basic Usage

```json
{
  "name": "llm.openai",
  "arguments": {
    "prompt": "What is the meaning of life?"
  }
}
```

### With System Prompt

```json
{
  "name": "llm.openai",
  "arguments": {
    "messages": [
      { "role": "system", "content": "You are a helpful coding assistant." },
      { "role": "user", "content": "Write a function to sort an array" }
    ]
  }
}
```

### Model Selection

```json
{
  "name": "llm.openai",
  "arguments": {
    "prompt": "Analyze this code",
    "model": "gpt-4o"
  }
}
```

**Available Models:**

- `gpt-4o` - Most capable
- `gpt-4o-mini` - Fast and cheap (default)
- `gpt-4-turbo` - Latest GPT-4
- `gpt-3.5-turbo` - Legacy, very fast

### Parameters

| Parameter     | Default | Description         |
| ------------- | ------- | ------------------- |
| `temperature` | 0.7     | Creativity (0-2)    |
| `max_tokens`  | None    | Max response length |

**Low temperature (0.1-0.3):** Deterministic, focused answers
**High temperature (0.7-1.0):** Creative, varied responses

### Multi-turn Conversation

```json
{
  "name": "llm.openai",
  "arguments": {
    "messages": [
      { "role": "system", "content": "You are a Python expert." },
      { "role": "user", "content": "What is a decorator?" },
      {
        "role": "assistant",
        "content": "A decorator is a function that wraps another function..."
      },
      { "role": "user", "content": "Show me an example" }
    ]
  }
}
```

---

## Anthropic (Claude) Integration

### Basic Usage

```json
{
  "name": "llm.anthropic",
  "arguments": {
    "prompt": "Explain quantum entanglement"
  }
}
```

### With System Prompt

```json
{
  "name": "llm.anthropic",
  "arguments": {
    "system": "You are a physics professor who explains complex topics simply.",
    "prompt": "What is dark matter?"
  }
}
```

### Model Selection

```json
{
  "name": "llm.anthropic",
  "arguments": {
    "prompt": "Write a poem",
    "model": "claude-3-sonnet-20240229"
  }
}
```

**Available Models:**

- `claude-3-opus-20240229` - Most capable
- `claude-3-sonnet-20240229` - Balanced
- `claude-3-haiku-20240307` - Fast and cheap (default)

---

## Embeddings

Generate vector embeddings for semantic search and similarity.

### Single Text

```json
{
  "name": "llm.embed",
  "arguments": {
    "text": "Hello world"
  }
}
```

### Multiple Texts

```json
{
  "name": "llm.embed",
  "arguments": {
    "texts": [
      "Machine learning is amazing",
      "Deep learning is a subset of ML",
      "I love pizza"
    ]
  }
}
```

### Response

```json
{
  "model": "text-embedding-3-small",
  "embeddings": [
    {
      "index": 0,
      "embedding": [0.0123, -0.0456, ...],
      "dimensions": 1536
    }
  ]
}
```

### Use Cases

1. **Semantic Search** - Find similar documents
2. **Clustering** - Group similar items
3. **Recommendations** - Find related content
4. **Anomaly Detection** - Find outliers

---

## Workflows with LLM

### Analyze and Notify

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      {
        "id": "analyze",
        "tool": "llm.openai",
        "args": {
          "messages": [
            {
              "role": "system",
              "content": "Analyze this log and summarize issues."
            },
            { "role": "user", "content": "{{log_content}}" }
          ]
        }
      },
      {
        "id": "notify",
        "tool": "notify.slack",
        "args": {
          "text": "Log Analysis:\n{{analyze.content}}"
        }
      }
    ],
    "context": {
      "log_content": "ERROR: Connection timeout at 12:00..."
    }
  }
}
```

### Code Review Bot

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      {
        "id": "diff",
        "tool": "git.diff",
        "args": { "path": "/project" }
      },
      {
        "id": "review",
        "tool": "llm.openai",
        "args": {
          "messages": [
            {
              "role": "system",
              "content": "You are a code reviewer. Point out bugs, improvements, and security issues."
            },
            { "role": "user", "content": "Review this diff:\n{{diff.diff}}" }
          ],
          "model": "gpt-4o"
        }
      }
    ]
  }
}
```

### Translation Pipeline

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      {
        "id": "translate",
        "tool": "llm.anthropic",
        "args": {
          "system": "You are a translator. Translate text to {{target_lang}}. Return only the translation.",
          "prompt": "{{text}}"
        }
      },
      {
        "id": "save",
        "tool": "memory.store",
        "args": {
          "key": "translation:{{text}}",
          "value": "{{translate.content}}"
        }
      }
    ],
    "context": {
      "text": "Hello world",
      "target_lang": "Spanish"
    }
  }
}
```

---

## Best Practices

### 1. Use System Prompts

Always define the AI's role:

```json
{
  "messages": [
    {
      "role": "system",
      "content": "You are a senior software engineer. Be concise and practical."
    },
    { "role": "user", "content": "..." }
  ]
}
```

### 2. Temperature by Task

| Task            | Temperature |
| --------------- | ----------- |
| Code generation | 0.1 - 0.3   |
| Analysis        | 0.3 - 0.5   |
| Writing         | 0.7 - 0.9   |
| Creative tasks  | 0.9 - 1.2   |

### 3. Use Cheaper Models First

```
gpt-4o-mini → if too simple → gpt-4o
claude-3-haiku → if too simple → claude-3-sonnet
```

### 4. Cache Results

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      {
        "id": "check_cache",
        "tool": "memory.recall",
        "args": { "key": "summary:{{doc_id}}" }
      },
      {
        "id": "generate",
        "tool": "llm.openai",
        "args": { "prompt": "Summarize: {{content}}" },
        "condition": "check_cache.found == false"
      },
      {
        "id": "save",
        "tool": "memory.store",
        "args": {
          "key": "summary:{{doc_id}}",
          "value": "{{generate.content}}"
        },
        "condition": "check_cache.found == false"
      }
    ]
  }
}
```

---

## Cost Optimization

### Token Estimation

| Model           | Input (1K tokens) | Output (1K tokens) |
| --------------- | ----------------- | ------------------ |
| gpt-4o-mini     | $0.00015          | $0.0006            |
| gpt-4o          | $0.005            | $0.015             |
| claude-3-haiku  | $0.00025          | $0.00125           |
| claude-3-sonnet | $0.003            | $0.015             |

### Tips

1. Use `max_tokens` to limit responses
2. Use smaller models for simple tasks
3. Cache embeddings for repeated texts
4. Batch embed operations

---

## Error Handling

### Common Errors

| Error            | Cause               | Solution               |
| ---------------- | ------------------- | ---------------------- |
| No API key       | Secret not set      | `secrets.set` with key |
| Rate limited     | Too many requests   | Add delays, use queue  |
| Context too long | Input too big       | Truncate or summarize  |
| Invalid model    | Model doesn't exist | Check model name       |

### Retry Pattern

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      { "id": "try1", "tool": "llm.openai", "args": { "prompt": "..." } },
      {
        "id": "try2",
        "tool": "llm.openai",
        "args": { "prompt": "..." },
        "condition": "try1 empty"
      },
      {
        "id": "try3",
        "tool": "llm.openai",
        "args": { "prompt": "..." },
        "condition": "try2 empty"
      }
    ]
  }
}
```

---

## Security

### API Key Protection

- Keys stored encrypted in secrets
- Never logged or exposed
- Use per-project keys if possible

### Content Safety

- OpenAI/Anthropic have built-in moderation
- Consider pre-filtering sensitive content
- Log prompts for audit (separately)

---

## Examples

### Daily Standup Generator

```json
{
  "name": "workflow.define",
  "arguments": {
    "name": "daily-standup",
    "steps": [
      { "id": "commits", "tool": "git.log", "args": { "count": 10 } },
      {
        "id": "summary",
        "tool": "llm.openai",
        "args": {
          "messages": [
            {
              "role": "system",
              "content": "Generate a brief standup summary from git commits."
            },
            { "role": "user", "content": "{{commits.commits}}" }
          ]
        }
      },
      {
        "id": "notify",
        "tool": "notify.slack",
        "args": {
          "text": "📋 *Daily Standup*\n{{summary.content}}"
        }
      }
    ]
  }
}
```

### PR Description Writer

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      { "id": "diff", "tool": "git.diff", "args": { "commit": "main" } },
      {
        "id": "describe",
        "tool": "llm.anthropic",
        "args": {
          "system": "Write a concise PR description from this diff. Include: What changed, Why, How to test.",
          "prompt": "{{diff.diff}}"
        }
      }
    ]
  }
}
```

### Smart Scheduler

```json
{
  "name": "scheduler.create",
  "arguments": {
    "name": "daily-summary",
    "cron": "0 18 * * 1-5",
    "tool": "workflow.execute",
    "args": { "name": "daily-standup" }
  }
}
```

