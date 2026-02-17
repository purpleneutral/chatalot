# Architecture

## System Overview

Chatalot is a self-hosted, end-to-end encrypted chat platform. The architecture consists of a Rust server, a Svelte 5 single-page application, and an optional Tauri desktop wrapper.

```
                    +------------------+
                    |   Cloudflare     |
                    |   Tunnel (opt.)  |
                    +--------+---------+
                             |
                    +--------+---------+
                    |   Axum Server    |
                    |   (Rust, :8080)  |
                    +--+-----+-----+--+
                       |     |     |
            +----------+  +--+--+  +----------+
            | REST API |  | WS  |  | Static   |
            | /api/*   |  | /ws |  | SPA      |
            +----+-----+  +--+--+  +----------+
                 |            |
            +----+------------+----+
            |    AppState          |
            | +------------------+ |
            | | PgPool           | |
            | | JWT keys         | |
            | | ConnectionMgr   | |
            | | Config           | |
            | | HTTP client      | |
            | +------------------+ |
            +----------+-----------+
                       |
              +--------+--------+
              |   PostgreSQL    |
              |   (port 5432)   |
              +-----------------+
```

## Component Architecture

### Server (`crates/chatalot-server`)

The server is the central hub. It handles HTTP REST requests, WebSocket connections, and serves the SPA static files. Built on Axum 0.8 with Tokio as the async runtime.

```
chatalot-server
  +-- main.rs           Entry point, background tasks, graceful shutdown
  +-- config.rs         Environment variable configuration
  +-- app_state.rs      Shared state (DB pool, JWT keys, ConnectionManager)
  +-- error.rs          Unified error types -> HTTP status codes
  +-- routes/           HTTP route handlers (24 modules)
  +-- ws/               WebSocket handling
  |   +-- session.rs    WS upgrade + auth handshake
  |   +-- handler.rs    Message routing + rate limiting
  |   +-- connection_manager.rs  Session registry + broadcast channels
  +-- middleware/        Request pipeline
  |   +-- auth.rs       JWT extraction + validation
  |   +-- community_gate.rs  Community membership gate
  |   +-- rate_limit.rs Token bucket rate limiter
  |   +-- security.rs   Security response headers
  +-- services/          Business logic
  |   +-- auth_service.rs    Password hashing, token issuance, lockout
  |   +-- file_security.rs   Magic byte validation, MIME detection
  |   +-- css_sanitizer.rs   Custom theme CSS sanitization
  +-- permissions.rs     Role-based permission checks
```

### Database (`crates/chatalot-db`)

Database access layer with sqlx runtime queries (not compile-time macros). Contains models, repository functions, and pool/migration management.

```
chatalot-db
  +-- pool.rs           Connection pool creation (50 max, 2 min)
  +-- models/           Rust structs mapped from DB rows
  +-- repos/            Query functions grouped by domain
```

All SQL queries use `sqlx::query!` or `sqlx::query_as!` at runtime with `SQLX_OFFLINE=true` for CI builds.

### Cryptography (`crates/chatalot-crypto`)

Pure Rust implementation of the Signal protocol cryptographic primitives:

```
chatalot-crypto
  +-- x3dh.rs           X3DH key agreement (session establishment)
  +-- double_ratchet.rs Double Ratchet (1:1 message encryption)
  +-- sender_keys.rs    Sender Keys (group message encryption)
  +-- aead.rs           ChaCha20-Poly1305 encrypt/decrypt
  +-- identity.rs       Ed25519 key generation, fingerprints
  +-- types.rs          SecretKey (zeroize-on-drop), Fingerprint
```

There is also a `crates/chatalot-crypto-wasm` crate that compiles the crypto library to WebAssembly for use in the web client.

### Common (`crates/chatalot-common`)

Shared types between server and (potentially) client crates:

```
chatalot-common
  +-- ws_messages.rs    ClientMessage / ServerMessage enums
  +-- api_types.rs      Request/response DTOs for REST API
  +-- constants.rs      Token lifetimes, limits, thresholds
```

## Request Flow

### REST API Request

```
Client
  |
  v
[Security Headers]  middleware/security.rs
  |
  v
[CORS]              tower_http CorsLayer (permissive for Tauri)
  |
  v
[Compression]        tower_http CompressionLayer (gzip)
  |
  v
[Rate Limit]         middleware/rate_limit.rs (token bucket)
  |
  v
[Body Limit]         110 MB max
  |
  v
[Route Match]        /api/* -> API routes, /ws -> WS, /* -> SPA fallback
  |
  v
[Auth Middleware]     middleware/auth.rs (Bearer JWT -> AccessClaims)
  |                   (skipped for public routes: /auth/*, /health, /legal)
  v
[Community Gate]      middleware/community_gate.rs (optional, for /communities/{cid}/*)
  |
  v
[Handler]             routes/*.rs -> business logic -> DB query -> response
```

### WebSocket Connection

```
Client
  |
  v
[WS Upgrade]          GET /ws -> 101 Switching Protocols
  |
  v
[Auth Handshake]      Client sends: {"type":"authenticate","token":"<JWT>"}
  |                   Server replies: {"type":"authenticated","user_id":"..."}
  |                   10-second timeout, no auth = disconnect
  v
[Session Setup]       ConnectionManager.add_session() (max 8 per user)
  |
  v
[Message Loop]        Client sends ClientMessage, server responds with ServerMessage
  |                   Rate limited: 10 msg/s burst, 5/s refill
  |                   Max WS frame: 1 MB
  v
[Channel Pub/Sub]     Subscribe/Unsubscribe to channels via broadcast::channel (256 buffer)
  |
  v
[Disconnect]          ConnectionManager.remove_session(), broadcast PresenceUpdate
```

## Concurrency Model

The server uses Tokio for async I/O. Key concurrency patterns:

- **ConnectionManager** uses `DashMap` (concurrent hash map) for lock-free session registry and channel subscriptions.
- **Channel broadcast** uses `tokio::sync::broadcast` with a buffer of 256 messages per channel. Subscribers that fall behind receive a `Lagged` error.
- **Per-user messaging** uses `tokio::sync::mpsc::UnboundedSender` for direct server-to-client delivery.
- **Typing state** is tracked in a `DashMap<(channel_id, user_id), Instant>` with periodic cleanup.
- **Account lockout** uses an in-memory `DashMap` (resets on server restart).

## Background Tasks

The server spawns several background tasks on startup:

| Task | Interval | Purpose |
|------|----------|---------|
| Typing timeout | 5s | Expire stale typing indicators (>10s) |
| Channel cleanup | 5min | Remove broadcast channels with zero subscribers |
| Data cleanup | 1h | Delete expired refresh tokens (>7d), used prekeys (>30d), old audit logs (>90d), orphaned voice sessions |
| Message GC | 24h | Hard-delete messages soft-deleted >30 days ago |
| Orphan file cleanup | 24h | Remove disk files with no DB record |
| Scheduled messages | 30s | Deliver messages whose `scheduled_for` time has passed |
| Message expiry | 5min | Delete messages past their TTL |
| Timeout cleanup | 5min | Remove expired user timeouts |
| Cache cleanup | 10min | Evict stale GIF and link preview cache entries |

## Deployment Architecture

```
+-------------------------------------------+
|  Docker Compose                           |
|                                           |
|  +-------------+    +------------------+  |
|  | chatalot    |    | postgres:17      |  |
|  | (Rust+SPA)  |--->| (chatalot-net)   |  |
|  | :8080       |    |                  |  |
|  +------+------+    +------------------+  |
|         |                                 |
|  +------+------+                          |
|  | cloudflared |  (optional, production)  |
|  | tunnel      |                          |
|  +-------------+                          |
+-------------------------------------------+
```

The Docker image uses a multi-stage build:

1. **Rust builder** -- compiles the server binary (with dependency caching)
2. **WASM builder** -- compiles `chatalot-crypto-wasm` to WebAssembly via `wasm-pack`
3. **Node builder** -- builds the Svelte SPA (with WASM crypto module)
4. **Runtime** -- Debian slim with the server binary, SPA static files, and migrations

Secrets (JWT Ed25519 key pair) are mounted via Docker secrets from `./secrets/`.
