# Notifications Guide

Send notifications to Slack, Discord, Email, and custom webhooks.

---

## Overview

| Tool             | Service           | Setup Required |
| ---------------- | ----------------- | -------------- |
| `notify.slack`   | Slack             | Webhook URL    |
| `notify.discord` | Discord           | Webhook URL    |
| `notify.email`   | Resend/SendGrid   | API Key        |
| `webhook.send`   | Any HTTP endpoint | None           |

---

## Slack

### Setup

1. Go to [Slack Apps](https://api.slack.com/apps)
2. Create new app → "From scratch"
3. Enable "Incoming Webhooks"
4. Add webhook to workspace
5. Copy webhook URL

Store the URL as a secret:

```json
{
  "name": "secrets.set",
  "arguments": {
    "key": "SLACK_WEBHOOK_URL",
    "value": "https://hooks.slack.com/services/T.../B.../xxx"
  }
}
```

### Basic Message

```json
{
  "name": "notify.slack",
  "arguments": {
    "text": "Hello from Nexus! 🚀"
  }
}
```

### With Formatting

```json
{
  "name": "notify.slack",
  "arguments": {
    "text": "*Build Status*\n✅ All tests passed\n📦 Version: 1.0.0",
    "username": "Build Bot",
    "icon_emoji": ":robot_face:"
  }
}
```

### With Block Kit

For rich formatting, use [Slack Block Kit](https://api.slack.com/block-kit):

```json
{
  "name": "notify.slack",
  "arguments": {
    "text": "Deployment Alert",
    "blocks": [
      {
        "type": "header",
        "text": { "type": "plain_text", "text": "🚀 Deployment Complete" }
      },
      {
        "type": "section",
        "fields": [
          { "type": "mrkdwn", "text": "*Environment:*\nProduction" },
          { "type": "mrkdwn", "text": "*Version:*\n1.2.3" }
        ]
      },
      {
        "type": "divider"
      },
      {
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "View <https://dashboard.example.com|Dashboard>"
        }
      }
    ]
  }
}
```

### Channel Override

```json
{
  "name": "notify.slack",
  "arguments": {
    "text": "Alert!",
    "channel": "#alerts"
  }
}
```

---

## Discord

### Setup

1. Open Discord server settings
2. Go to Integrations → Webhooks
3. Create new webhook
4. Copy webhook URL

Store the URL:

```json
{
  "name": "secrets.set",
  "arguments": {
    "key": "DISCORD_WEBHOOK_URL",
    "value": "https://discord.com/api/webhooks/xxx/yyy"
  }
}
```

### Basic Message

```json
{
  "name": "notify.discord",
  "arguments": {
    "content": "Hello from Nexus! 🎮"
  }
}
```

### With Custom Bot

```json
{
  "name": "notify.discord",
  "arguments": {
    "content": "Build complete!",
    "username": "Build Bot",
    "avatar_url": "https://example.com/bot-avatar.png"
  }
}
```

### With Embeds

For rich content:

```json
{
  "name": "notify.discord",
  "arguments": {
    "content": "New deployment",
    "embeds": [
      {
        "title": "Deployment Status",
        "description": "Successfully deployed to production",
        "color": 5763719,
        "fields": [
          { "name": "Version", "value": "1.2.3", "inline": true },
          { "name": "Environment", "value": "Production", "inline": true }
        ],
        "footer": { "text": "Nexus Bot" },
        "timestamp": "2026-01-03T12:00:00Z"
      }
    ]
  }
}
```

**Embed Colors:**

- Green: 5763719 (0x57F287)
- Red: 15548997 (0xED4245)
- Blue: 5793266 (0x5865F2)
- Yellow: 16705372 (0xFEE75C)

---

## Email

### Providers

| Provider | Secret Key     | Features              |
| -------- | -------------- | --------------------- |
| Resend   | `RESEND_KEY`   | Simple, great DX      |
| SendGrid | `SENDGRID_KEY` | Enterprise, analytics |

### Resend Setup

1. Sign up at [resend.com](https://resend.com)
2. Create API key
3. Verify your domain

```json
{
  "name": "secrets.set",
  "arguments": {
    "key": "RESEND_KEY",
    "value": "re_xxx..."
  }
}
```

```json
{
  "name": "secrets.set",
  "arguments": {
    "key": "EMAIL_FROM",
    "value": "notifications@yourdomain.com"
  }
}
```

### SendGrid Setup

1. Sign up at [sendgrid.com](https://sendgrid.com)
2. Create API key with Mail Send permission
3. Verify sender identity

```json
{
  "name": "secrets.set",
  "arguments": {
    "key": "SENDGRID_KEY",
    "value": "SG.xxx..."
  }
}
```

### Plain Text Email

```json
{
  "name": "notify.email",
  "arguments": {
    "to": "user@example.com",
    "subject": "Daily Report",
    "body": "Everything is running smoothly.\n\nBest,\nNexus Bot"
  }
}
```

### HTML Email

```json
{
  "name": "notify.email",
  "arguments": {
    "to": "user@example.com",
    "subject": "Weekly Summary",
    "body": "Plain text fallback",
    "html": "<h1>Weekly Summary</h1><p>Here are the highlights...</p>"
  }
}
```

### Using SendGrid

```json
{
  "name": "notify.email",
  "arguments": {
    "to": "user@example.com",
    "subject": "Alert",
    "body": "Something needs attention.",
    "provider": "sendgrid"
  }
}
```

---

## Generic Webhooks

For any HTTP endpoint.

### Basic Webhook

```json
{
  "name": "webhook.send",
  "arguments": {
    "url": "https://api.example.com/webhook",
    "event": "build.completed",
    "data": {
      "status": "success",
      "version": "1.2.3"
    }
  }
}
```

### Payload Format

The webhook sends:

```json
{
  "event": "build.completed",
  "timestamp": "2026-01-03T12:00:00Z",
  "data": {
    "status": "success",
    "version": "1.2.3"
  }
}
```

### With Authentication

```json
{
  "name": "webhook.send",
  "arguments": {
    "url": "https://api.example.com/webhook",
    "event": "deploy",
    "data": { "env": "production" },
    "headers": {
      "Authorization": "Bearer ${secrets.WEBHOOK_TOKEN}",
      "X-Custom-Header": "value"
    }
  }
}
```

### GitHub Webhook

```json
{
  "name": "webhook.send",
  "arguments": {
    "url": "https://api.github.com/repos/owner/repo/dispatches",
    "headers": {
      "Authorization": "Bearer ${secrets.GITHUB_TOKEN}",
      "Accept": "application/vnd.github.v3+json"
    },
    "data": {
      "event_type": "deploy",
      "client_payload": { "version": "1.0" }
    }
  }
}
```

---

## Workflow Integration

### Notify on Git Changes

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      { "id": "status", "tool": "git.status" },
      {
        "id": "notify",
        "tool": "notify.slack",
        "args": { "text": "📝 {{status.changes_count}} uncommitted changes" },
        "condition": "status.clean == false"
      }
    ]
  }
}
```

### Multi-Channel Alert

```json
{
  "name": "workflow.run",
  "arguments": {
    "context": { "message": "Server is down!" },
    "steps": [
      { "tool": "notify.slack", "args": { "text": "🚨 {{message}}" } },
      { "tool": "notify.discord", "args": { "content": "🚨 {{message}}" } },
      {
        "tool": "notify.email",
        "args": {
          "to": "oncall@company.com",
          "subject": "ALERT: {{message}}",
          "body": "{{message}}\n\nPlease investigate immediately."
        }
      }
    ]
  }
}
```

### Scheduled Reports

```json
{
  "name": "scheduler.create",
  "arguments": {
    "name": "daily-slack-report",
    "cron": "0 9 * * 1-5",
    "tool": "workflow.execute",
    "args": {
      "name": "daily-report"
    }
  }
}
```

---

## Templates

### Success Message

```json
{
  "name": "notify.slack",
  "arguments": {
    "text": "✅ *Success*\nOperation completed successfully.",
    "icon_emoji": ":white_check_mark:"
  }
}
```

### Error Alert

````json
{
  "name": "notify.slack",
  "arguments": {
    "text": "🚨 *Error*\nSomething went wrong.\n```{{error_details}}```",
    "icon_emoji": ":rotating_light:"
  }
}
````

### Deployment Notification

```json
{
  "name": "notify.slack",
  "arguments": {
    "text": "🚀 *Deployment*\n*Version:* {{version}}\n*Environment:* {{env}}\n*Status:* Complete",
    "icon_emoji": ":rocket:"
  }
}
```

### PR Notification

```json
{
  "name": "notify.discord",
  "arguments": {
    "content": "📋 New PR: {{pr_title}}",
    "embeds": [
      {
        "title": "{{pr_title}}",
        "url": "{{pr_url}}",
        "description": "{{pr_description}}",
        "color": 5793266,
        "author": { "name": "{{author}}" }
      }
    ]
  }
}
```

---

## Error Handling

### Webhook Failure

Check the response:

```json
{
  "success": false,
  "status_code": 401,
  "url": "...",
  "event": "..."
}
```

### Common Issues

| Status | Cause         | Solution             |
| ------ | ------------- | -------------------- |
| 401    | Invalid token | Check API key/secret |
| 403    | Forbidden     | Check permissions    |
| 404    | Bad URL       | Verify webhook URL   |
| 429    | Rate limited  | Add delays           |
| 500    | Server error  | Retry later          |

### Retry Pattern

```json
{
  "name": "workflow.run",
  "arguments": {
    "steps": [
      { "id": "try1", "tool": "notify.slack", "args": { "text": "{{msg}}" } },
      {
        "id": "try2",
        "tool": "notify.slack",
        "args": { "text": "{{msg}}" },
        "condition": "try1.success == false"
      },
      {
        "id": "try3",
        "tool": "notify.slack",
        "args": { "text": "{{msg}}" },
        "condition": "try2.success == false"
      }
    ]
  }
}
```

---

## Best Practices

### 1. Use Secrets for URLs/Keys

```json
// Good
{"key": "SLACK_WEBHOOK_URL", "value": "https://..."}

// Bad - hardcoding in tool calls
{"webhook_url": "https://..."}
```

### 2. Include Context

```json
{
  "text": "*Alert*\nService: {{service}}\nTime: {{time}}\nDetails: {{details}}"
}
```

### 3. Use Appropriate Channels

```
#alerts → Critical issues only
#builds → Build notifications
#deploys → Deployment updates
```

### 4. Rate Limit Yourself

Don't spam channels. Aggregate messages when possible.

### 5. Test in Dev First

```json
{
  "channel": "#test-notifications"
}
```

---

## Quick Reference

| Action           | Tool             | Required Secret              |
| ---------------- | ---------------- | ---------------------------- |
| Slack message    | `notify.slack`   | `SLACK_WEBHOOK_URL`          |
| Discord message  | `notify.discord` | `DISCORD_WEBHOOK_URL`        |
| Email (Resend)   | `notify.email`   | `RESEND_KEY`, `EMAIL_FROM`   |
| Email (SendGrid) | `notify.email`   | `SENDGRID_KEY`, `EMAIL_FROM` |
| Custom webhook   | `webhook.send`   | (varies)                     |

