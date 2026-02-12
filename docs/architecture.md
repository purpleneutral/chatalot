# Architecture

## Overview

Chatalot follows a client-server architecture where the server acts as an **untrusted relay**. All message content is encrypted client-side before transmission; the server stores and routes ciphertext without access to plaintext.

```
┌──────────────┐     HTTPS/WSS      ┌─────────────────┐     SQL      ┌────────────┐
│  Web Client  │ ◄────────────────► │  Rust Server     │ ◄──────────► │ PostgreSQL │
│  (Svelte 5)  │                    │  (axum + tokio)  │              │    17      │
└──────────────┘                    └─────────────────┘              └────────────┘
                                           ▲
┌──────────────┐     HTTPS/WSS             │
│ Desktop App  │ ◄─────────────────────────┘
│  (Tauri 2.0) │
└──────────────┘
```

## Server Architecture

### Crate Layout

The server is a Cargo workspace with four crates, each with a specific responsibility:

```
chatalot-server  ──depends on──►  chatalot-db
       │                              │
       ├──depends on──►  chatalot-common  ◄──depends on──┘
       │
       └──depends on──►  chatalot-crypto
```

**chatalot-server** (binary) — The HTTP/WebSocket server. Contains routes, middleware, WebSocket session management, and the connection manager. This is the only crate that produces a binary.

**chatalot-db** (library) — Database access layer using the repository pattern. Each entity (users, channels, messages, etc.) has its own repository module with pure query functions. Models are plain Rust structs derived from `sqlx::FromRow`.

**chatalot-common** (library) — Shared types used by both server and client. Includes API request/response DTOs and WebSocket message type definitions. This prevents type drift between the server's Rust code and the client's TypeScript types.

**chatalot-crypto** (library) — The E2E encryption library. Implements X3DH, Double Ratchet, Sender Keys, and ChaCha20-Poly1305 AEAD. Designed to compile to both native Rust and WASM (via wasm-pack) so the same crypto code runs on the server (for key validation) and in the browser.

### Request Flow

```
Client Request
    │
    ▼
┌─────────────────┐
│  Security Headers│  (HSTS, CSP, X-Frame-Options, etc.)
├─────────────────┤
│  Rate Limiting   │  (token-bucket per IP, stricter for auth)
├─────────────────┤
│  Compression     │  (gzip)
├─────────────────┤
│  Tracing         │  (request/response logging)
├─────────────────┤
│  JWT Auth        │  (extracts claims, injects into extensions)
├─────────────────┤
│  Route Handler   │  (business logic)
├─────────────────┤
│  Repository      │  (database queries via sqlx)
└─────────────────┘
```

Middleware is applied as tower layers in this order (outermost first):
1. `TraceLayer` — HTTP request/response tracing
2. `CompressionLayer` — gzip response compression
3. `security_headers` — inject security response headers
4. `rate_limit_middleware` — global rate limiting (20 req/s per IP)
5. `auth_rate_limit_middleware` — stricter rate limiting on auth routes (5 req/s per IP)
6. `auth_middleware` — JWT verification, applied only to protected routes

### WebSocket Architecture

The WebSocket system has three components:

**Session** (`ws/session.rs`) — Handles the upgrade from HTTP to WebSocket. Each connection spawns two tasks: a reader task that processes incoming messages, and a writer task that sends outgoing messages. The reader task calls into the handler for business logic.

**Handler** (`ws/handler.rs`) — Contains the message routing logic. Each incoming `ClientMessage` is matched and dispatched: messages are stored and broadcast, typing indicators are forwarded, reactions are persisted, WebRTC signals are relayed. The handler verifies channel membership before processing most operations.

**ConnectionManager** (`ws/connection_manager.rs`) — Manages all active WebSocket sessions. Uses `DashMap<UserId, Vec<SessionHandle>>` for per-user session tracking and `DashMap<ChannelId, broadcast::Sender>` for per-channel pub/sub. When a message is sent to a channel, the connection manager broadcasts it to all subscribers.

```
             ┌──────────────────────────────────────┐
             │         ConnectionManager             │
             │                                      │
  subscribe  │  channels: {                         │
  ─────────► │    "ch-1": broadcast::Sender ───────►│──► subscriber A
             │    "ch-2": broadcast::Sender ───────►│──► subscriber B, C
             │  }                                   │
             │                                      │
             │  sessions: {                         │
             │    "user-1": [SessionHandle, ...]    │
             │    "user-2": [SessionHandle, ...]    │
             │  }                                   │
             └──────────────────────────────────────┘
```

**Heartbeat**: The server sends a WebSocket ping every 30 seconds. If a client doesn't respond, the connection is considered stale and cleaned up.

### Authentication Flow

```
Register                              Login
   │                                     │
   ▼                                     ▼
Validate inputs                    Verify Argon2id hash
   │                                     │
   ▼                                     ▼
Hash password (Argon2id)           Check TOTP (if enabled)
   │                                     │
   ▼                                     ▼
Store user + identity key          Generate access JWT (15 min)
   │                                     │
   ▼                                     ▼
Generate tokens                    Generate refresh token (30 day)
   │                                     │
   ▼                                     ▼
Return tokens + user info          Store refresh hash + audit log
```

**Token refresh**: The client sends the refresh token to `/api/auth/refresh`. The server verifies it against the stored SHA-256 hash, revokes the old token, generates new access + refresh tokens, and returns them. This rotation ensures that a leaked refresh token can only be used once.

## Client Architecture

### Svelte 5 State Management

The web client uses Svelte 5 runes for reactive state management. Each domain has its own store:

| Store | State | Purpose |
|-------|-------|---------|
| `authStore` | tokens, user info | Auth state, persisted to localStorage |
| `channelStore` | channel list, active channel | Channel navigation |
| `messageStore` | messages by channel, unread counts | Message display + caching |
| `presenceStore` | online status, typing indicators | Real-time presence |
| `voiceStore` | voice participants, local media state | Voice/video calls |
| `themeStore` | dark/light | Theme preference, persisted to localStorage |
| `toastStore` | notification queue | Transient UI notifications |

Stores use `$state` for reactive properties and expose methods for mutations. Components access store state directly (it's reactive via Svelte 5's fine-grained reactivity).

### WebSocket Client

The WebSocket client (`ws/connection.ts`) handles:
- Connection lifecycle with automatic reconnection (exponential backoff + jitter)
- Authentication on connect (sends access token as first message)
- Message serialization/deserialization
- Ping/pong keepalive
- Listener registration for components

Incoming messages are routed through `ws/handler.ts` which dispatches to the appropriate store.

### WebRTC Manager

The WebRTC manager (`webrtc/manager.ts`) handles voice and video calls:
- Mesh topology: each participant connects directly to every other participant
- Signaling goes through the existing WebSocket connection (no separate server)
- Manages local media streams (microphone, camera, screen share)
- Handles ICE candidate exchange and SDP offer/answer negotiation

## Database Design

### Key Decisions

**UUIDv7 primary keys**: All tables use UUIDv7 which embeds a timestamp, providing time-ordered uniqueness without a sequence. This allows distributed ID generation and efficient B-tree indexing.

**Ciphertext storage**: Messages are stored as `BYTEA` (binary). The server never decrypts message content. Nonces are stored alongside ciphertext for the recipient to use during decryption.

**Repository pattern**: Each entity has a repository module with standalone async functions (not traits). This keeps the data layer simple and testable without requiring mocking frameworks.

**Soft deletion**: Refresh tokens use `revoked_at` for soft revocation rather than hard deletion, enabling audit trails.

### Entity Relationships

```
users ──1:N──► channels (created_by)
users ──M:N──► channels (via channel_members, with role)
users ──1:N──► messages (sender_id)
channels ──1:N──► messages
messages ──0:1──► messages (reply_to, self-referential)
messages ──1:N──► reactions
users ──1:N──► reactions
users ──1:1──► identity_keys
users ──1:N──► signed_prekeys
users ──1:N──► one_time_prekeys
users ──M:N──► users (via dm_pairs → channel)
channels ──1:N──► voice_sessions
users ──M:N──► voice_sessions (via participants)
users ──1:N──► read_cursors (per channel)
```

## Encryption Design

### DM Encryption (X3DH + Double Ratchet)

1. **Key generation**: Each user generates a long-lived Ed25519 identity key pair. The private key never leaves the device.

2. **Prekey bundles**: Each user uploads to the server:
   - 1 signed prekey (X25519, rotated weekly, signed with identity key)
   - 100 one-time prekeys (X25519, consumed on use)

3. **Session establishment (X3DH)**:
   - Alice fetches Bob's prekey bundle from the server
   - Alice performs 3-4 Diffie-Hellman operations to derive a shared secret
   - Alice initializes a Double Ratchet session with this shared secret
   - Alice sends her first message along with her ephemeral public key

4. **Ongoing messaging (Double Ratchet)**:
   - Each message advances the symmetric chain (hash ratchet)
   - Each reply triggers a DH ratchet step (new ephemeral key pair)
   - This provides forward secrecy (past messages can't be decrypted if current keys are compromised) and break-in recovery (future messages are secure even after a compromise)

5. **Cipher**: ChaCha20-Poly1305 with a per-message derived key and random nonce.

### Group Encryption (Sender Keys)

For channels with multiple participants:

1. Each member generates a Sender Key (random symmetric key + hash chain)
2. Sender Keys are distributed to group members via existing pairwise Double Ratchet sessions (so the server never sees them)
3. Messages are encrypted once with the sender's key; all recipients can decrypt
4. When a member is removed, all remaining members regenerate and redistribute their Sender Keys

### Key Verification

Users can verify each other's identity keys using safety numbers:
```
SHA-256(sort(IdentityKey_A || IdentityKey_B))
```
Displayed as numeric blocks for manual comparison or QR code scanning.

## Docker Build

The Dockerfile uses a multi-stage build:

```
Stage 1: rust:1.84-bookworm (builder)
├── Copy Cargo manifests + create dummy sources
├── Build dependencies only (cached layer)
├── Copy real source + rebuild (only changed code recompiles)
└── Output: /build/target/release/chatalot-server

Stage 2: node:22-bookworm (web-builder)
├── npm ci (cached layer)
├── Copy source + build
└── Output: /build/build/ (static SPA files)

Stage 3: debian:bookworm-slim (runtime)
├── Install ca-certificates + curl (for health checks)
├── Copy binary from stage 1
├── Copy static files from stage 2
├── Copy migrations
├── Create non-root user
└── Output: ~15 MB binary + static files
```

The dependency caching strategy means that changing only Rust source code doesn't re-download or recompile any dependencies, and changing only Svelte source doesn't rebuild the Rust binary.
