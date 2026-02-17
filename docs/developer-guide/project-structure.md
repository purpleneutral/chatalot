# Project Structure

## Workspace Layout

Chatalot uses a Cargo workspace with four crates, plus a Svelte web client and a Tauri desktop client.

```
chatalot/
+-- Cargo.toml                 Workspace root (edition 2024, version 0.17.0)
+-- Cargo.lock
+-- Dockerfile                 Multi-stage Docker build
+-- docker-compose.yml         Server + PostgreSQL + Cloudflare tunnel
+-- migrations/                SQL migration files (001-039)
+-- secrets/                   JWT Ed25519 PEM key pair (not committed)
+-- scripts/                   Deployment and utility scripts
+-- crates/
|   +-- chatalot-server/       HTTP server, WS handler, route handlers
|   +-- chatalot-db/           Database models and repository functions
|   +-- chatalot-crypto/       Signal protocol cryptographic primitives
|   +-- chatalot-common/       Shared types (API types, WS messages, constants)
|   +-- chatalot-crypto-wasm/  WASM bindings for crypto (excluded from workspace)
+-- clients/
|   +-- web/                   Svelte 5 SPA (SvelteKit + Tailwind v4)
|   +-- desktop/               Tauri desktop wrapper
+-- docs/                      Documentation
```

## Crate Dependency Graph

```
chatalot-server
  +-- chatalot-db
  +-- chatalot-crypto
  +-- chatalot-common

chatalot-db
  +-- (no internal deps)

chatalot-crypto
  +-- (no internal deps)

chatalot-common
  +-- (no internal deps)

chatalot-crypto-wasm   (excluded from workspace, builds separately)
  +-- chatalot-crypto
```

## Crate Details

### `chatalot-server`

The main binary crate. Contains the Axum web server, route handlers, middleware, WebSocket handling, and background tasks.

```
crates/chatalot-server/
+-- Cargo.toml
+-- src/
    +-- main.rs                 Entry point: config, DB pool, background tasks, server
    +-- config.rs               Config struct populated from environment variables
    +-- app_state.rs            AppState: shared state passed to all handlers
    +-- error.rs                AppError enum -> HTTP status code mapping
    +-- permissions.rs          Role hierarchy and permission checks
    +-- routes/
    |   +-- mod.rs              Router builder, middleware stack, static file serving
    |   +-- auth.rs             POST /auth/register, login, refresh, recover; GET /auth/config
    |   +-- users.rs            GET /users/search, /users/{id}; POST block/unblock; reports
    |   +-- communities.rs      Community CRUD, members, roles, bans, invites, timeouts, emojis
    |   +-- groups.rs           Group CRUD, discover, join/leave, channels, invites, assets
    |   +-- channels.rs         Channel CRUD, join/leave, members, roles, kick/ban, ownership
    |   +-- messages.rs         GET messages, search, global search, threads, edit history, pins
    |   +-- dms.rs              GET/POST /dms (direct message channels)
    |   +-- files.rs            POST upload, GET download, DELETE, GET meta
    |   +-- keys.rs             E2E key bundle fetch, prekey upload, prekey count
    |   +-- sender_keys.rs      Upload/fetch sender key distributions
    |   +-- totp.rs             TOTP setup, verify, backup codes
    |   +-- admin.rs            Admin panel: users, invites, purge, files, audit, reports
    |   +-- webhooks.rs         Channel webhook CRUD, public execute endpoint
    |   +-- polls.rs            Create, vote, close polls
    |   +-- scheduled.rs        Schedule, list, cancel scheduled messages
    |   +-- bookmarks.rs        Add/remove/list message bookmarks
    |   +-- announcements.rs    List announcements, dismiss
    |   +-- feedback.rs         POST feedback (Forgejo issue integration)
    |   +-- gifs.rs             GIF search proxy (Giphy API)
    |   +-- link_preview.rs     Link preview metadata fetching
    |   +-- account.rs          GET /account/me, PATCH profile, change password, sessions
    |   +-- health.rs           GET /health (status, version, uptime, DB check)
    |   +-- legal.rs            GET /legal/terms, GET /legal/privacy
    +-- ws/
    |   +-- session.rs          WS upgrade, auth handshake (10s timeout)
    |   +-- handler.rs          Message dispatch, rate limiting, business logic
    |   +-- connection_manager.rs  Session registry, channel broadcast, typing state
    +-- middleware/
    |   +-- auth.rs             JWT extraction from Authorization header
    |   +-- community_gate.rs   Verifies community membership for gated routes
    |   +-- rate_limit.rs       Token bucket rate limiter (in-memory)
    |   +-- security.rs         Security response headers (CSP, HSTS, etc.)
    +-- services/
        +-- auth_service.rs     Password hashing, JWT issuance, lockout, recovery codes
        +-- file_security.rs    File type validation via magic bytes
        +-- css_sanitizer.rs    Sanitizes custom community theme CSS
```

### `chatalot-db`

Library crate for all database interactions. Uses sqlx with runtime queries against PostgreSQL.

```
crates/chatalot-db/
+-- Cargo.toml
+-- src/
    +-- lib.rs
    +-- pool.rs                 Pool creation (50 max, 2 min conns, 5s timeout) + migration runner
    +-- models/                 Rust structs matching DB rows
    |   +-- user.rs
    |   +-- channel.rs          ChannelType enum (Text, Voice, Dm)
    |   +-- message.rs
    |   +-- file.rs
    |   +-- group.rs
    |   +-- community.rs
    |   +-- ...                 (one per domain entity)
    +-- repos/                  Query functions grouped by domain
        +-- user_repo.rs        User CRUD, search, admin ops, refresh tokens, audit log
        +-- channel_repo.rs     Channel CRUD, membership, roles, bans
        +-- message_repo.rs     Message CRUD, search, soft-delete, GC, expiry
        +-- group_repo.rs       Group CRUD, membership, channels
        +-- community_repo.rs   Community CRUD, membership, roles
        +-- file_repo.rs        File record CRUD, quota tracking
        +-- key_repo.rs         Identity keys, signed prekeys, one-time prekeys
        +-- sender_key_repo.rs  Sender key distribution storage
        +-- dm_repo.rs          DM pair management
        +-- reaction_repo.rs    Message reaction CRUD
        +-- pin_repo.rs         Pinned message CRUD
        +-- unread_repo.rs      Unread count tracking, read cursors
        +-- invite_repo.rs      Group invite CRUD
        +-- report_repo.rs      User/content report CRUD
        +-- block_repo.rs       User blocking
        +-- ...                 (additional repos for each feature)
```

### `chatalot-crypto`

Library crate containing the Signal protocol cryptographic primitives. Pure Rust, no database dependencies.

```
crates/chatalot-crypto/
+-- Cargo.toml
+-- src/
    +-- lib.rs
    +-- x3dh.rs                 X3DH key agreement protocol
    +-- double_ratchet.rs       Double Ratchet for 1:1 messaging
    +-- sender_keys.rs          Sender Keys for group messaging
    +-- aead.rs                 ChaCha20-Poly1305 AEAD (encrypt/decrypt/key gen)
    +-- identity.rs             Ed25519 identity key generation, fingerprints, safety numbers
    +-- types.rs                SecretKey (zeroize on drop), Fingerprint
```

### `chatalot-common`

Library crate with types shared between the server and other crates.

```
crates/chatalot-common/
+-- Cargo.toml
+-- src/
    +-- lib.rs
    +-- ws_messages.rs          ClientMessage and ServerMessage enums (serde tagged)
    +-- api_types.rs            All REST API request/response structs
    +-- constants.rs            Token lifetimes, message size limits, prekey thresholds
```

## Web Client

```
clients/web/
+-- package.json               Svelte 5, SvelteKit, Tailwind v4, Vite 6
+-- svelte.config.js           Static adapter configuration
+-- vite.config.ts             Vite build configuration
+-- tsconfig.json
+-- src/
    +-- app.html               HTML shell
    +-- app.css                 Tailwind imports + CSS custom properties
    +-- routes/                 SvelteKit file-based routing
    +-- lib/
        +-- components/        Reusable Svelte components
        +-- stores/            Svelte stores (auth, channels, messages, etc.)
        +-- api/               REST API client functions
        +-- ws/                WebSocket client
        +-- crypto/            Crypto module (WASM bindings)
        +-- utils/             Utility functions
```

## Migrations

SQL migration files live in `migrations/` at the workspace root. They are run automatically on server startup via `sqlx::migrate!()`. Files are numbered sequentially:

```
migrations/
+-- 001_users.sql              User table, email, password hash
+-- 002_identity_keys.sql      Ed25519 identity keys per user
+-- 003_refresh_tokens.sql     JWT refresh token storage
+-- 004_audit_log.sql          Audit log for admin actions
+-- 005_prekeys.sql            Signed + one-time prekeys for X3DH
+-- 006_channels.sql           Text/voice channels with membership
+-- 007_messages.sql           Encrypted messages (ciphertext + nonce)
+-- 008_files.sql              Uploaded file records
+-- 009_dm_pairs.sql           Direct message pair tracking
+-- 010_voice_sessions.sql     Voice/video session tracking
+-- 011_reactions.sql          Message reactions (emoji)
+-- 012_unread_tracking.sql    Per-user unread message counts
+-- 013_channel_bans.sql       Channel-level bans
+-- 014_groups.sql             Group containers for channels
+-- 015_group_invites.sql      Group invite codes
+-- 016_account_deletion_fks.sql  Foreign key cascade for account deletion
+-- 017_admin_role.sql         Admin flag on users
+-- 018_registration_invites.sql  Admin-generated registration invite codes
+-- 019_communities.sql        Community containers for groups
+-- 020_user_preferences.sql   User preference storage
+-- 021_pinned_messages.sql    Pinned messages per channel
+-- 022_sender_key_distributions.sql  Sender key storage for group E2E
+-- 023_instance_owner.sql     Instance owner flag
+-- 024_permissions_enhancements.sql  Fine-grained permission fields
+-- 025_performance_indexes.sql  Performance indexes
+-- 026_security_suite.sql     Suspension, TOTP, file quarantine
+-- 027_lockout_and_quota.sql  Upload quotas
+-- 028_blocking_and_reports.sql  User blocking and reporting
+-- 029_new_features.sql       Webhooks, polls, scheduled messages, bookmarks
+-- 030_discoverable.sql       Discoverable channels
+-- 031_group_discoverable.sql Group discoverability
+-- 032_personal_groups.sql    Personal (per-user) groups within communities
+-- 033_channel_archiving.sql  Channel archive flag
+-- 034_customization.sql      Community themes, emojis, user customization
+-- 035_more_indexes.sql       Additional performance indexes
+-- 036_community_members_index.sql  Community membership index
+-- 037_recovery_codes.sql     Account recovery codes
+-- 038_message_edits.sql      Message edit history tracking
+-- 039_threads.sql            Thread support for messages
```

## Key Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace definition, shared dependencies, version |
| `Dockerfile` | Multi-stage build (Rust + WASM + Node -> runtime) |
| `docker-compose.yml` | Server + Postgres + optional Cloudflare tunnel |
| `clients/web/package.json` | Web client dependencies |
| `clients/web/svelte.config.js` | SvelteKit static adapter config |
| `secrets/jwt_private.pem` | Ed25519 private key for JWT signing (not committed) |
| `secrets/jwt_public.pem` | Ed25519 public key for JWT verification (not committed) |
