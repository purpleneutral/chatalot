# Developer Guide

Technical reference documentation for contributors and developers working on the Chatalot codebase.

## Contents

| Document | Description |
|----------|-------------|
| [Architecture](architecture.md) | System architecture, component interactions, data flow |
| [Project Structure](project-structure.md) | Workspace layout, crate purposes, directory map |
| [API Reference](api-reference.md) | All REST API endpoints grouped by resource |
| [WebSocket Protocol](websocket-protocol.md) | WS connection lifecycle, authentication, message types |
| [Database Schema](database-schema.md) | Tables, relationships, indexes, migrations |
| [Authentication](authentication.md) | Auth flow, JWT, refresh tokens, TOTP, account lockout |
| [Crypto Implementation](crypto-implementation.md) | X3DH, Double Ratchet, Sender Keys deep dive |
| [Building from Source](building-from-source.md) | Prerequisites, build commands, environment setup |
| [Testing](testing.md) | Test suite, linting, build verification |
| [Contributing](contributing.md) | How to contribute, code style, patterns |

## Quick Links

- **Server entry point:** `crates/chatalot-server/src/main.rs`
- **Web client:** `clients/web/`
- **Desktop client:** `clients/desktop/`
- **Migrations:** `migrations/`
- **Docker:** `Dockerfile`, `docker-compose.yml`

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Server | Rust (edition 2024), Axum 0.8, Tokio |
| Database | PostgreSQL 17, sqlx 0.8 (runtime queries) |
| Web Client | Svelte 5, SvelteKit, Tailwind CSS v4, Vite 6 |
| Desktop Client | Tauri |
| Cryptography | ed25519-dalek, x25519-dalek, chacha20poly1305, hkdf, argon2 |
| Auth | Ed25519-signed JWT, Argon2id password hashing |
| Real-time | WebSocket (axum::ws), WebRTC (peer-to-peer) |
| Deployment | Docker, Cloudflare Tunnels |
