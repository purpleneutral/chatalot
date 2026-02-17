# Webhooks

Webhooks allow external services to post messages into Chatalot channels. They are the primary way to integrate third-party tools -- CI/CD pipelines, monitoring systems, bots, and custom scripts -- with your Chatalot instance.

## Sections

| Page | Description |
|------|-------------|
| [Using Webhooks](./using-webhooks.md) | Creating, executing, managing, and securing webhooks |

## Quick Overview

- A webhook is a unique URL tied to a specific channel.
- Any HTTP client can `POST` a JSON payload to that URL to send a message.
- No authentication token is needed -- the URL itself contains the secret token.
- Webhook messages appear in the channel as a special "Webhook" message type.
- Only channel admins and owners can create or manage webhooks.
