# Using Webhooks

> **Status: Complete**

Webhooks let external services send messages into Chatalot channels over HTTP. This page covers creating, using, managing, and securing webhooks.

## What Are Webhooks?

A webhook is a unique URL that, when called with an HTTP POST request, sends a message to a specific channel. This allows tools like CI/CD systems, monitoring alerts, bots, and custom scripts to post updates directly into your Chatalot conversations without needing a full user account.

Webhook messages appear in the channel with a special "Webhook" message type, displaying the webhook's name (or an overridden username) as the sender.

## Who Can Create Webhooks?

> **Permission required:** You must be an **Owner** or **Admin** of the channel's community to create, view, edit, or delete webhooks.

Regular members cannot see or manage webhooks.

## Creating a Webhook

1. In the channel sidebar, right-click (or click the settings icon on) the channel you want to add a webhook to.
2. In the **Channel Settings** card that appears, scroll down to the **Webhooks** section and click it to expand.
3. Type a name for the webhook (1-64 characters) in the input field.
4. Click **Add**.
5. The webhook appears in the list. **Immediately click "Click to copy URL"** to copy the full webhook URL to your clipboard.

> **Important:** The webhook token is only visible when the webhook is first created. If you navigate away without copying the URL, you will not be able to retrieve the token again. You would need to delete the webhook and create a new one.

## Webhook URL Format

The webhook URL follows this pattern:

```
https://your-chatalot-instance.com/api/webhooks/execute/{token}
```

The `{token}` is a 64-character random string that serves as both the identifier and the authentication secret.

## Sending a Message via Webhook

To send a message, make an HTTP POST request to the webhook URL with a JSON body.

### Payload Format

```json
{
  "content": "Hello from a webhook!",
  "username": "CI Bot",
  "avatar_url": "https://example.com/bot-avatar.png"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | string | Yes | The message text (1-4000 characters). Supports Markdown. |
| `username` | string | No | Override the display name (max 64 characters). Defaults to the webhook's name. |
| `avatar_url` | string | No | Override the avatar URL for this message. Defaults to the webhook's avatar. |

### Example: curl

```bash
curl -X POST \
  https://your-instance.com/api/webhooks/execute/YOUR_TOKEN_HERE \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Build #142 passed successfully.",
    "username": "CI Pipeline"
  }'
```

### Example: Python

```python
import requests

webhook_url = "https://your-instance.com/api/webhooks/execute/YOUR_TOKEN_HERE"

requests.post(webhook_url, json={
    "content": "Deployment complete. Version 2.1.0 is live.",
    "username": "Deploy Bot"
})
```

### Example: JavaScript (Node.js)

```javascript
const response = await fetch(
  "https://your-instance.com/api/webhooks/execute/YOUR_TOKEN_HERE",
  {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      content: "New issue opened: Fix login timeout",
      username: "Issue Tracker"
    })
  }
);
```

### Response

- **200 OK** -- message was sent successfully (empty response body).
- **404 Not Found** -- the webhook token is invalid or the webhook has been deactivated.
- **422 Validation Error** -- the payload is invalid (e.g., `content` is empty or exceeds 4000 characters).

## Managing Webhooks

### Viewing Webhooks

1. Open the channel settings card for the channel.
2. Expand the **Webhooks** section.
3. All webhooks for that channel are listed with their name and active status.

### Enabling / Disabling a Webhook

Each webhook has an **ON/OFF** toggle. Click it to enable or disable the webhook. A disabled webhook will return a 404 error when called, effectively pausing it without deleting it.

### Editing a Webhook

Webhooks can be updated via the API:

```
PATCH /api/webhooks/{webhook_id}
```

Updatable fields:

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | New webhook name (1-64 characters) |
| `avatar_url` | string or null | New default avatar URL, or null to remove |
| `active` | boolean | Enable or disable the webhook |

This endpoint requires authentication (a logged-in admin/owner session).

### Deleting a Webhook

In the webhooks list in the channel settings card, click the **X** button next to the webhook you want to delete. A confirmation dialog will appear. Deleting a webhook is permanent and immediately invalidates its URL.

## Security Considerations

> **Treat your webhook URL like a password.** Anyone who has the URL can post messages to your channel.

Best practices:

- **Never commit webhook URLs to public repositories.** Use environment variables or secret management tools.
- **Rotate webhooks periodically.** Delete the old webhook and create a new one if you suspect the URL has been exposed.
- **Disable unused webhooks** rather than leaving them active. This prevents abuse if an old URL is discovered.
- **Monitor webhook usage.** If unexpected messages appear in a channel, check the webhooks list and revoke any compromised ones.
- **Use HTTPS.** Always ensure your Chatalot instance is served over HTTPS so webhook tokens are encrypted in transit.

## Limitations

- Webhook messages support plain text and Markdown but cannot include file attachments.
- The `content` field is limited to 4000 characters per message.
- The `username` override is limited to 64 characters.
- Webhook execution does not require authentication -- the token in the URL is the only credential.
- The webhook token is only shown once at creation time.

## Related

- [Channels](../channels/README.md) -- where webhook messages are delivered
- [Security](../security/README.md) -- general account and platform security
