# Git Integration Guide

Nexus provides full git integration for version control operations.

---

## Overview

| Tool         | Purpose                 |
| ------------ | ----------------------- |
| `git.status` | Check repository status |
| `git.log`    | View commit history     |
| `git.diff`   | See changes             |
| `git.commit` | Create commits          |
| `git.branch` | Manage branches         |

---

## git.status

Check the current state of a repository.

### Usage

```json
{
  "name": "git.status",
  "arguments": {
    "path": "/path/to/repo"
  }
}
```

### Response

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

### Status Codes

| Code | Meaning             |
| ---- | ------------------- |
| `M`  | Modified            |
| `A`  | Added (staged)      |
| `D`  | Deleted             |
| `R`  | Renamed             |
| `??` | Untracked           |
| `UU` | Unmerged (conflict) |

---

## git.log

View recent commits.

### Usage

```json
{
  "name": "git.log",
  "arguments": {
    "path": "/path/to/repo",
    "count": 5
  }
}
```

### Response

```json
{
  "count": 5,
  "commits": [
    {
      "hash": "a1b2c3d4e5f6...",
      "short_hash": "a1b2c3d",
      "author": "John Doe",
      "email": "john@example.com",
      "timestamp": 1767400000,
      "message": "Add new feature"
    },
    ...
  ]
}
```

---

## git.diff

View changes in the repository.

### Unstaged Changes

```json
{
  "name": "git.diff",
  "arguments": {
    "path": "/path/to/repo"
  }
}
```

### Staged Changes

```json
{
  "name": "git.diff",
  "arguments": {
    "path": "/path/to/repo",
    "staged": true
  }
}
```

### Compare to Commit

```json
{
  "name": "git.diff",
  "arguments": {
    "path": "/path/to/repo",
    "commit": "HEAD~5"
  }
}
```

### Specific File

```json
{
  "name": "git.diff",
  "arguments": {
    "path": "/path/to/repo",
    "file": "src/main.rs"
  }
}
```

---

## git.commit

Create a commit.

### Basic Commit

```json
{
  "name": "git.commit",
  "arguments": {
    "path": "/path/to/repo",
    "message": "feat: add new feature"
  }
}
```

### Stage All and Commit

```json
{
  "name": "git.commit",
  "arguments": {
    "path": "/path/to/repo",
    "message": "fix: resolve bug",
    "add_all": true
  }
}
```

### Response

```json
{
  "success": true,
  "message": "feat: add new feature",
  "commit": "a1b2c3d4e5f6..."
}
```

### Commit Message Convention

Follow [Conventional Commits](https://www.conventionalcommits.org/):

| Type       | Description      |
| ---------- | ---------------- |
| `feat`     | New feature      |
| `fix`      | Bug fix          |
| `docs`     | Documentation    |
| `style`    | Formatting       |
| `refactor` | Code refactoring |
| `test`     | Adding tests     |
| `chore`    | Maintenance      |

---

## git.branch

Manage branches.

### List Branches

```json
{
  "name": "git.branch",
  "arguments": {
    "path": "/path/to/repo"
  }
}
```

### Response

```json
{
  "current": "main",
  "branches": [
    { "name": "main", "current": true },
    { "name": "develop", "current": false },
    { "name": "feature/new-thing", "current": false }
  ]
}
```

### Create Branch

```json
{
  "name": "git.branch",
  "arguments": {
    "path": "/path/to/repo",
    "create": "feature/new-feature"
  }
}
```

### Switch Branch

```json
{
  "name": "git.branch",
  "arguments": {
    "path": "/path/to/repo",
    "checkout": "develop"
  }
}
```

---

## Workflow Examples

### Daily Status Report

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      { "id": "status", "tool": "git.status", "args": { "path": "/project" } },
      {
        "id": "log",
        "tool": "git.log",
        "args": { "path": "/project", "count": 10 }
      },
      {
        "id": "report",
        "tool": "llm.openai",
        "args": {
          "messages": [
            {
              "role": "system",
              "content": "Create a brief status report from git data."
            },
            {
              "role": "user",
              "content": "Status: {{status}}\nRecent commits: {{log}}"
            }
          ]
        }
      },
      {
        "id": "notify",
        "tool": "notify.slack",
        "args": { "text": "📋 *Daily Report*\n{{report.content}}" }
      }
    ]
  }
}
```

### Auto-Commit

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      { "id": "status", "tool": "git.status", "args": { "path": "/project" } },
      {
        "id": "commit",
        "tool": "git.commit",
        "args": {
          "path": "/project",
          "message": "chore: auto-save",
          "add_all": true
        },
        "condition": "status.clean == false"
      }
    ]
  }
}
```

### PR Preparation

```json
{
  "name": "workflow.run",
  "arguments": {
    "context": { "branch": "feature/new-thing" },
    "steps": [
      {
        "id": "checkout",
        "tool": "git.branch",
        "args": { "checkout": "{{branch}}" }
      },
      { "id": "diff", "tool": "git.diff", "args": { "commit": "main" } },
      {
        "id": "describe",
        "tool": "llm.openai",
        "args": {
          "messages": [
            {
              "role": "system",
              "content": "Write a PR description from this diff."
            },
            { "role": "user", "content": "{{diff.diff}}" }
          ]
        }
      },
      {
        "id": "save",
        "tool": "memory.store",
        "args": {
          "key": "pr:{{branch}}",
          "value": "{{describe.content}}"
        }
      }
    ]
  }
}
```

### Code Review Bot

```json
{
  "name": "workflow.define",
  "arguments": {
    "name": "code-review",
    "inputs": ["path"],
    "steps": [
      { "id": "diff", "tool": "git.diff", "args": { "path": "{{path}}" } },
      {
        "id": "review",
        "tool": "llm.openai",
        "args": {
          "messages": [
            {
              "role": "system",
              "content": "Review this code. List: 1) Bugs, 2) Security issues, 3) Improvements"
            },
            { "role": "user", "content": "{{diff.diff}}" }
          ],
          "model": "gpt-4o"
        }
      }
    ]
  }
}
```

---

## Scheduling Git Tasks

### Hourly Status Check

```json
{
  "name": "scheduler.create",
  "arguments": {
    "name": "hourly-git-check",
    "cron": "0 * * * *",
    "tool": "workflow.execute",
    "args": {
      "name": "git-status-report"
    }
  }
}
```

### Daily Backup Commit

```json
{
  "name": "scheduler.create",
  "arguments": {
    "name": "daily-backup",
    "cron": "0 23 * * *",
    "tool": "git.commit",
    "args": {
      "path": "/project",
      "message": "chore: daily backup",
      "add_all": true
    }
  }
}
```

---

## Security Notes

1. **Path Restrictions**: Git commands run in the specified path
2. **Credential Handling**: Git uses your system's credential store
3. **No Push/Pull**: By design, Nexus doesn't include push/pull (use webhooks instead)

---

## Troubleshooting

### "Not a git repository"

The path doesn't contain a `.git` directory:

```json
{
  "path": "/actual/repo/path"
}
```

### "Nothing to commit"

No changes staged. Use `add_all: true`:

```json
{
  "add_all": true
}
```

### Permission Denied

Git can't access the repository. Check file permissions.

---

## Quick Reference

| Action        | Tool         | Key Args                   |
| ------------- | ------------ | -------------------------- |
| Check status  | `git.status` | `path`                     |
| View history  | `git.log`    | `path`, `count`            |
| See changes   | `git.diff`   | `path`, `staged`, `commit` |
| Create commit | `git.commit` | `message`, `add_all`       |
| List branches | `git.branch` | `path`                     |
| Create branch | `git.branch` | `create`                   |
| Switch branch | `git.branch` | `checkout`                 |

