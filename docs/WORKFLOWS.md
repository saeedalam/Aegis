# Workflow Guide

Workflows let you chain multiple tools together, passing data between steps and creating powerful automation pipelines.

---

## Overview

| Tool | Purpose |
|------|---------|
| `workflow.run` | Execute a workflow inline |
| `workflow.define` | Save a reusable workflow |
| `workflow.execute` | Run a saved workflow |
| `workflow.list` | List saved workflows |

---

## Quick Start

### Simple Workflow

```json
{
  "name": "workflow.run",
  "arguments": {
    "name": "my-first-workflow",
    "steps": [
      {"id": "step1", "tool": "get_time"},
      {"id": "step2", "tool": "echo", "args": {"text": "Time is: {{step1.time}}"}}
    ]
  }
}
```

**What happens:**
1. Step 1 calls `get_time`, stores result as `step1`
2. Step 2 uses `{{step1.time}}` to access the time
3. Both results are returned

---

## Workflow Structure

```json
{
  "name": "workflow.run",
  "arguments": {
    "name": "workflow-name",           // Optional: for logging
    "steps": [...],                     // Required: array of steps
    "context": {...}                    // Optional: initial variables
  }
}
```

### Step Structure

```json
{
  "id": "unique-id",           // Optional: for referencing output
  "tool": "tool-name",         // Required: which tool to call
  "args": {...},               // Optional: tool arguments
  "condition": "expression"    // Optional: skip condition
}
```

---

## Variable Substitution

Reference previous step outputs using `{{...}}` syntax.

### Simple Reference

```json
{
  "steps": [
    {"id": "time", "tool": "get_time"},
    {"id": "echo", "tool": "echo", "args": {"text": "{{time}}"}}
  ]
}
```

### Nested Access

```json
{
  "steps": [
    {"id": "status", "tool": "git.status"},
    {"id": "echo", "tool": "echo", "args": {
      "text": "Branch: {{status.branch}}, Changes: {{status.changes_count}}"
    }}
  ]
}
```

### Special Variables

| Variable | Description |
|----------|-------------|
| `{{_last}}` | Output of previous step |
| `{{step_id}}` | Full output of step |
| `{{step_id.field}}` | Specific field |

### Initial Context

Pass variables at runtime:

```json
{
  "name": "workflow.run",
  "arguments": {
    "context": {
      "user_name": "Alice",
      "target_env": "production"
    },
    "steps": [
      {"tool": "echo", "args": {"text": "Deploying for {{user_name}} to {{target_env}}"}}
    ]
  }
}
```

---

## Conditions

Skip steps based on conditions.

### Condition Syntax

| Format | Description |
|--------|-------------|
| `key exists` | Key is in context |
| `key empty` | Key is null/empty |
| `key == value` | Equals |
| `key != value` | Not equals |
| `key > value` | Greater than |
| `key < value` | Less than |
| `key >= value` | Greater or equal |
| `key <= value` | Less or equal |

### Examples

**Skip if already processed:**
```json
{
  "steps": [
    {"id": "check", "tool": "memory.recall", "args": {"key": "processed:{{id}}"}},
    {"id": "process", "tool": "http.request", "condition": "check.found == false"}
  ]
}
```

**Only notify on changes:**
```json
{
  "steps": [
    {"id": "status", "tool": "git.status"},
    {"id": "notify", "tool": "notify.slack", 
     "args": {"text": "Changes detected!"},
     "condition": "status.clean == false"}
  ]
}
```

**Numeric comparison:**
```json
{
  "steps": [
    {"id": "status", "tool": "git.status"},
    {"id": "warn", "tool": "notify.slack",
     "args": {"text": "Too many changes!"},
     "condition": "status.changes_count > 10"}
  ]
}
```

---

## Saved Workflows

### Define

Save a workflow for reuse:

```json
{
  "name": "workflow.define",
  "arguments": {
    "name": "deploy-pipeline",
    "description": "Deploy and notify",
    "inputs": ["branch", "environment"],
    "steps": [
      {"id": "status", "tool": "git.status"},
      {"id": "checkout", "tool": "git.branch", "args": {"checkout": "{{branch}}"}},
      {"id": "notify", "tool": "notify.slack", "args": {
        "text": "Deployed {{branch}} to {{environment}}"
      }}
    ]
  }
}
```

### Execute

Run the saved workflow:

```json
{
  "name": "workflow.execute",
  "arguments": {
    "name": "deploy-pipeline",
    "inputs": {
      "branch": "main",
      "environment": "production"
    }
  }
}
```

### List

```json
{"name": "workflow.list", "arguments": {}}
```

---

## Real-World Examples

### 1. Daily Report Pipeline

```json
{
  "name": "workflow.define",
  "arguments": {
    "name": "daily-report",
    "steps": [
      {
        "id": "time",
        "tool": "get_time"
      },
      {
        "id": "commits",
        "tool": "git.log",
        "args": {"count": 20}
      },
      {
        "id": "summary",
        "tool": "llm.openai",
        "args": {
          "messages": [
            {"role": "system", "content": "Summarize today's git activity."},
            {"role": "user", "content": "{{commits.commits}}"}
          ]
        }
      },
      {
        "id": "notify",
        "tool": "notify.slack",
        "args": {
          "text": "*Daily Report - {{time.time}}*\n\n{{summary.content}}"
        }
      }
    ]
  }
}
```

### 2. PR Review Bot

```json
{
  "name": "workflow.run",
  "arguments": {
    "name": "pr-review",
    "context": {
      "pr_branch": "feature/new-thing"
    },
    "steps": [
      {
        "id": "diff",
        "tool": "git.diff",
        "args": {"commit": "main"}
      },
      {
        "id": "review",
        "tool": "llm.openai",
        "args": {
          "messages": [
            {"role": "system", "content": "Review this code diff. List: 1) Bugs, 2) Security issues, 3) Improvements"},
            {"role": "user", "content": "{{diff.diff}}"}
          ],
          "model": "gpt-4o"
        }
      },
      {
        "id": "post",
        "tool": "webhook.send",
        "args": {
          "url": "https://api.github.com/repos/org/repo/issues/1/comments",
          "headers": {"Authorization": "Bearer ${secrets.GITHUB_TOKEN}"},
          "data": {"body": "## AI Review\n\n{{review.content}}"}
        }
      }
    ]
  }
}
```

### 3. Data Pipeline

```json
{
  "name": "workflow.run",
  "arguments": {
    "name": "data-sync",
    "steps": [
      {
        "id": "fetch",
        "tool": "http.request",
        "args": {"url": "https://api.source.com/data"}
      },
      {
        "id": "transform",
        "tool": "llm.openai",
        "args": {
          "messages": [
            {"role": "system", "content": "Extract key metrics as JSON."},
            {"role": "user", "content": "{{fetch.body}}"}
          ]
        }
      },
      {
        "id": "store",
        "tool": "memory.store",
        "args": {
          "key": "metrics:latest",
          "value": "{{transform.content}}"
        }
      },
      {
        "id": "push",
        "tool": "http.request",
        "args": {
          "url": "https://api.destination.com/metrics",
          "method": "POST",
          "body": "{{transform.content}}"
        }
      }
    ]
  }
}
```

### 4. Alert Pipeline

```json
{
  "name": "workflow.define",
  "arguments": {
    "name": "health-check",
    "steps": [
      {
        "id": "check",
        "tool": "http.request",
        "args": {"url": "{{endpoint}}/health", "timeout": 5}
      },
      {
        "id": "alert",
        "tool": "notify.slack",
        "args": {
          "text": "⚠️ {{service_name}} is DOWN!\nEndpoint: {{endpoint}}\nStatus: {{check.status}}"
        },
        "condition": "check.status != 200"
      },
      {
        "id": "log",
        "tool": "memory.store",
        "args": {
          "key": "health:{{service_name}}:{{check.timestamp}}",
          "value": {"status": "{{check.status}}", "time": "{{check.timestamp}}"}
        }
      }
    ],
    "inputs": ["endpoint", "service_name"]
  }
}
```

### 5. Multi-step Validation

```json
{
  "name": "workflow.run",
  "arguments": {
    "name": "validate-deploy",
    "steps": [
      {
        "id": "tests",
        "tool": "cmd.exec",
        "args": {"command": "npm", "args": ["test"]}
      },
      {
        "id": "check_tests",
        "tool": "echo",
        "args": {"text": "Tests passed"},
        "condition": "tests.exit_code == 0"
      },
      {
        "id": "lint",
        "tool": "cmd.exec",
        "args": {"command": "npm", "args": ["run", "lint"]}
      },
      {
        "id": "build",
        "tool": "cmd.exec",
        "args": {"command": "npm", "args": ["run", "build"]},
        "condition": "lint.exit_code == 0"
      },
      {
        "id": "notify_success",
        "tool": "notify.slack",
        "args": {"text": "✅ All checks passed!"},
        "condition": "build.exit_code == 0"
      },
      {
        "id": "notify_fail",
        "tool": "notify.slack",
        "args": {"text": "❌ Build failed!"},
        "condition": "build.exit_code != 0"
      }
    ]
  }
}
```

---

## Scheduling Workflows

Run workflows on a schedule:

```json
{
  "name": "scheduler.create",
  "arguments": {
    "name": "hourly-health-check",
    "cron": "0 * * * *",
    "tool": "workflow.execute",
    "args": {
      "name": "health-check",
      "inputs": {
        "endpoint": "https://api.myapp.com",
        "service_name": "API"
      }
    }
  }
}
```

---

## Error Handling

### Workflow Stops on Error

If any step fails, the workflow stops and returns:

```json
{
  "workflow": "my-workflow",
  "success": false,
  "steps_executed": 2,
  "steps_total": 5,
  "results": [
    {"step_id": "step1", "success": true, "output": {...}},
    {"step_id": "step2", "error": "Connection timeout"}
  ]
}
```

### Conditional Error Handling

```json
{
  "steps": [
    {"id": "try", "tool": "http.request", "args": {"url": "..."}},
    {"id": "fallback", "tool": "http.request", "args": {"url": "backup..."}, "condition": "try empty"},
    {"id": "notify_error", "tool": "notify.slack", "args": {"text": "Both attempts failed"}, "condition": "fallback empty"}
  ]
}
```

---

## Best Practices

### 1. Always Use Step IDs

```json
// Good
{"id": "fetch_data", "tool": "http.request", ...}

// Bad - no ID means you can't reference it
{"tool": "http.request", ...}
```

### 2. Keep Workflows Focused

One workflow = one purpose:
- ❌ `full-deployment-with-tests-and-notifications`
- ✅ `run-tests`, `deploy`, `notify-team` (compose them)

### 3. Use Saved Workflows for Reuse

```json
// Define once
{"name": "workflow.define", "arguments": {"name": "notify-team", ...}}

// Use many times
{"name": "workflow.execute", "arguments": {"name": "notify-team", "inputs": {...}}}
```

### 4. Log Important Steps

```json
{
  "id": "log",
  "tool": "memory.store",
  "args": {
    "key": "workflow:{{workflow_id}}:{{step_id}}",
    "value": {"result": "{{_last}}", "time": "{{time}}"}
  }
}
```

### 5. Use Conditions for Control Flow

```json
{
  "condition": "previous_step.success == true"
}
```

---

## Debugging

### View Workflow Output

The response includes all step results:

```json
{
  "workflow": "my-workflow",
  "success": true,
  "steps_executed": 3,
  "steps_total": 3,
  "results": [...],
  "final_context": {...}
}
```

### Check `final_context`

Contains all variables after execution - useful for debugging variable substitution.

### Test Steps Individually

Before creating a workflow, test each tool separately:

```bash
nexus run get_time
nexus run git.status --args '{"path": "."}'
nexus run notify.slack --args '{"text": "test"}'
```

---

## Limitations

1. **No loops** - Use scheduler for repeated execution
2. **No parallel steps** - Steps execute sequentially
3. **Condition syntax is simple** - No complex expressions
4. **Context is string-based** - JSON values are stringified

---

## Summary

| Concept | Syntax |
|---------|--------|
| Define step | `{"id": "x", "tool": "y", "args": {...}}` |
| Reference output | `{{step_id.field}}` |
| Skip step | `"condition": "key == value"` |
| Initial data | `"context": {"key": "value"}` |
| Save workflow | `workflow.define` |
| Run saved | `workflow.execute` |
| Schedule | Use `scheduler.create` with `workflow.execute` |


