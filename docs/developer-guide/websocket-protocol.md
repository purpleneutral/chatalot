# WebSocket Protocol

> **Status: Complete**

The WebSocket protocol is the real-time communication layer for Chatalot. All live events -- messages, presence, typing, voice signaling -- flow through a single WebSocket connection per client.

---

## Connection

- **Endpoint**: `wss://your-instance/ws` (or `ws://` for unencrypted dev)
- The connection uses standard WebSocket upgrade
- All messages are JSON-encoded text frames
- Max frame size: **1 MB**

---

## Authentication

Authentication happens via the **first message**, not headers. The browser WebSocket API does not reliably support custom headers, and this approach also works cleanly with the Tauri desktop client.

### Flow

```
Client                                Server
  |                                      |
  |-- GET /ws (Upgrade) ---------------->|
  |<-- 101 Switching Protocols ----------|
  |                                      |
  |-- {"type":"authenticate",            |
  |     "token":"<JWT>"}  ------------->|
  |                                      |  Validate JWT (EdDSA/Ed25519)
  |<-- {"type":"authenticated"} ---------|  (success)
  |                                      |
  |  OR                                  |
  |<-- {"type":"error",                  |
  |      "code":401,                     |
  |      "message":"..."}  -------------|  (failure)
  |                                      |
```

1. Client opens WebSocket connection
2. Client sends: `{"type": "authenticate", "token": "<JWT access token>"}`
3. Server validates the JWT (EdDSA/Ed25519)
4. Server responds with `{"type": "authenticated"}` on success or `{"type": "error", "code": 401, "message": "..."}` on failure
5. If no auth message arrives within **10 seconds**, the connection is dropped

---

## Connection Limits

| Limit | Value |
|-------|-------|
| Max concurrent connections per user | **8** (multi-device support) |
| Rate limiting (burst) | **10 messages/second** |
| Rate limiting (sustained) | **5 messages/second** (token bucket refill) |
| Heartbeat interval | Server sends Ping every **30 seconds**; client must respond with Pong |
| Broadcast channel buffer | **256 messages** per channel subscription |

---

## Client Messages (Client -> Server)

Messages sent from the client to the server.

### Messaging

| Type | Fields | Description |
|------|--------|-------------|
| `send_message` | `channel_id`, `ciphertext`, `nonce`, `reply_to_id?`, `sender_key_id?`, `thread_id?` | Send an encrypted message to a channel |
| `edit_message` | `message_id`, `ciphertext`, `nonce` | Edit a previously sent message |
| `delete_message` | `message_id` | Delete a message |

### Presence

| Type | Fields | Description |
|------|--------|-------------|
| `update_presence` | `status: string` | Set status: `online`, `idle`, `dnd`, `invisible`, `offline` |
| `typing` | `channel_id` | Start typing indicator in a channel |
| `stop_typing` | `channel_id` | Stop typing indicator in a channel |

### Channel Subscriptions

| Type | Fields | Description |
|------|--------|-------------|
| `subscribe` | `channel_id` | Subscribe to real-time events for a channel |
| `unsubscribe` | `channel_id` | Unsubscribe from a channel's events |

### WebRTC Signaling

| Type | Fields | Description |
|------|--------|-------------|
| `rtc_offer` | `channel_id`, `target_user_id`, `sdp` | Send a WebRTC offer for voice/video |
| `rtc_answer` | `channel_id`, `target_user_id`, `sdp` | Send a WebRTC answer |
| `rtc_ice_candidate` | `channel_id`, `target_user_id`, `candidate` | Exchange an ICE candidate |

### Voice

| Type | Fields | Description |
|------|--------|-------------|
| `join_voice` | `channel_id` | Join a voice channel |
| `leave_voice` | `channel_id` | Leave a voice channel |
| `kick_from_voice` | `channel_id`, `user_id` | Kick a user from voice (mod+) |

### Reactions

| Type | Fields | Description |
|------|--------|-------------|
| `add_reaction` | `message_id`, `emoji` | Add a reaction to a message |
| `remove_reaction` | `message_id`, `emoji` | Remove a reaction from a message |

### Unread Tracking

| Type | Fields | Description |
|------|--------|-------------|
| `mark_read` | `channel_id`, `message_id` | Mark a channel as read up to a specific message |
| `mark_all_read` | _(none)_ | Mark all channels as read |

### Keepalive

| Type | Fields | Description |
|------|--------|-------------|
| `authenticate` | `token: string` | First message -- JWT authentication |
| `ping` | `timestamp: i64` | Client keepalive; server responds with `pong` |

---

## Server Messages (Server -> Client)

Messages sent from the server to the client.

### Auth

| Type | Fields | Description |
|------|--------|-------------|
| `authenticated` | _(none)_ | Authentication succeeded |
| `error` | `code: u16`, `message: string` | Error response |

### Messaging

| Type | Fields | Description |
|------|--------|-------------|
| `new_message` | `message object` | New message in a subscribed channel |
| `message_sent` | `message_id`, `created_at`, `thread_id?` | Confirmation of the sender's own message |
| `message_edited` | `message_id`, `ciphertext`, `nonce`, `edited_at` | A message was edited |
| `message_deleted` | `message_id`, `channel_id` | A message was deleted |

### Presence

| Type | Fields | Description |
|------|--------|-------------|
| `presence_update` | `user_id`, `status` | A user changed their presence status |
| `user_typing` | `channel_id`, `user_id` | A user started typing |
| `user_stopped_typing` | `channel_id`, `user_id` | A user stopped typing |

### WebRTC Signaling

| Type | Fields | Description |
|------|--------|-------------|
| `rtc_offer` | `channel_id`, `from_user_id`, `sdp` | Incoming WebRTC offer |
| `rtc_answer` | `channel_id`, `from_user_id`, `sdp` | Incoming WebRTC answer |
| `rtc_ice_candidate` | `channel_id`, `from_user_id`, `candidate` | Incoming ICE candidate |

### Voice

| Type | Fields | Description |
|------|--------|-------------|
| `voice_state_update` | `channel_id`, `user_id`, `state` | Voice state changed |
| `user_joined_voice` | `channel_id`, `user_id` | A user joined voice |
| `user_left_voice` | `channel_id`, `user_id` | A user left voice |

### Reactions

| Type | Fields | Description |
|------|--------|-------------|
| `reaction_added` | `message_id`, `user_id`, `emoji` | A reaction was added |
| `reaction_removed` | `message_id`, `user_id`, `emoji` | A reaction was removed |

### Read Receipts

| Type | Fields | Description |
|------|--------|-------------|
| `read_receipt` | `channel_id`, `user_id`, `message_id`, `timestamp` | A user read up to a message |

### Pinned Messages

| Type | Fields | Description |
|------|--------|-------------|
| `message_pinned` | `message_id`, `channel_id`, `pinned_by` | A message was pinned |
| `message_unpinned` | `message_id`, `channel_id` | A message was unpinned |

### Channel Moderation

| Type | Fields | Description |
|------|--------|-------------|
| `member_kicked` | `channel_id`, `user_id` | A member was kicked |
| `member_banned` | `channel_id`, `user_id` | A member was banned |
| `member_role_updated` | `channel_id`, `user_id`, `role` | A member's role changed |

### DM Notifications

| Type | Fields | Description |
|------|--------|-------------|
| `new_dm_channel` | `channel object` | A new DM channel was created with you |

### Sender Keys (Group E2E)

| Type | Fields | Description |
|------|--------|-------------|
| `sender_key_updated` | `channel_id`, `user_id`, `distribution` | A member uploaded a new sender key |
| `sender_key_rotation_required` | `channel_id` | All members must rotate their sender keys |
| `keys_low` | `remaining: u32` | One-time prekey count is low; upload more |

### System

| Type | Fields | Description |
|------|--------|-------------|
| `pong` | `timestamp: i64` | Heartbeat response (echoes the client's timestamp) |

---

## Message Flow Examples

### Sending a Message

```
Client:          {"type": "send_message", "channel_id": "...", "ciphertext": "base64...", "nonce": "base64..."}
Server -> Self:  {"type": "message_sent", "message_id": "...", "created_at": "2024-..."}
Server -> Others: {"type": "new_message", "message": {...}}
```

### Typing Indicator

- Typing events are deduplicated: only one `typing` event per user per channel per 3 seconds
- `stop_typing` is sent when the user clears the input or sends a message

### Voice Call Signaling

```
1. Client sends `join_voice`
   -> Server broadcasts `user_joined_voice` to others in the channel

2. Existing participants send `rtc_offer` to the new participant

3. New participant responds with `rtc_answer`

4. ICE candidates are exchanged via `rtc_ice_candidate`

5. On leave: Client sends `leave_voice`
   -> Server broadcasts `user_left_voice`
```

---

## Error Codes

| Code | Meaning |
|------|---------|
| `400` | Bad request / malformed message |
| `401` | Authentication failed |
| `403` | Permission denied |
| `404` | Resource not found |
| `429` | Rate limited |

---

## Subscription Model

Clients must explicitly subscribe to channels to receive events. On connection, the client typically subscribes to all channels the user is a member of. Channel events are broadcast via `tokio::sync::broadcast` channels with a **256-message buffer**. If a subscriber falls behind, it receives a `Lagged` error and may miss messages.

---

## Next Step

- [API Reference](./api-reference.md) -- Full REST API endpoint reference
- [Authentication](./authentication.md) -- JWT, refresh tokens, and password hashing
- [Architecture](./architecture.md) -- System design and component overview
