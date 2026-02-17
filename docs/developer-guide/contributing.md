# Contributing

> **Status: Complete**

Thank you for your interest in contributing to Chatalot. This guide covers code conventions, project structure, and the process for submitting changes.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/chatalot.git`
3. Follow the [Building from Source](./building-from-source.md) guide to set up your development environment
4. Create a feature branch: `git checkout -b feature/your-feature`

## Project Structure

Chatalot is a Cargo workspace with four Rust crates and a Svelte web client:

```
chatalot/
├── crates/
│   ├── chatalot-server/    # Axum HTTP + WebSocket server
│   ├── chatalot-db/        # PostgreSQL models and repositories
│   ├── chatalot-crypto/    # E2E encryption (X3DH, Double Ratchet, Sender Keys)
│   └── chatalot-common/    # Shared types (API types, WS messages)
├── clients/
│   ├── web/                # Svelte 5 SPA (SvelteKit + Vite)
│   └── desktop/            # Tauri desktop wrapper
├── migrations/             # PostgreSQL migrations (sqlx)
├── scripts/                # Build and deployment scripts
└── docs/                   # This documentation
```

## Code Conventions

### Rust

- **Edition:** 2024 (set in workspace Cargo.toml)
- **Formatting:** Use `rustfmt` defaults
- **Linting:** Code must pass `cargo clippy -- -W clippy::all` (3 `too_many_arguments` warnings on DB repo functions are acceptable)
- **Dependencies:** Define versions at the workspace level in root `Cargo.toml` and reference them in crate `Cargo.toml` files
- **Error handling:** Use `thiserror` for error type definitions. Use `anyhow` sparingly (only in main/entrypoint code)
- **Sensitive data:** Types holding secrets must derive `Zeroize` for secure memory cleanup
- **Database queries:** Use sqlx runtime-checked queries (not compile-time macros). Set `SQLX_OFFLINE=true` for offline builds

### Svelte / TypeScript

- **Framework:** Svelte 5 with runes (`$state`, `$derived`) -- do not use legacy `$:` reactive statements
- **Styling:** Tailwind CSS + CSS custom properties for theming (e.g., `var(--bg-primary)`)
- **Stores:** Class-based stores using Svelte 5 runes. Create new references (new Map, new Array) for reactivity
- **API client:** Use the shared API client that handles JWT injection and automatic 401 refresh

### Commit Messages

- Use imperative mood: "Add feature", "Fix bug", "Update docs"
- Keep the first line concise (under 72 characters)
- Reference issue numbers where applicable: "Fix login timeout (#42)"

## Adding Features

### New REST API Endpoint

1. Create handler in `crates/chatalot-server/src/routes/`
2. Register the route in `routes/mod.rs` (protected or public)
3. Add request/response types to `chatalot-common/src/api_types.rs`
4. Handler signature: `async fn handler(State(state): State<AppState>, Extension(claims): Extension<Claims>) -> Result<Json<Response>, AppError>`

### New WebSocket Message Type

1. Add variant to `ClientMessage` or `ServerMessage` enum in `chatalot-common/src/ws_messages.rs`
2. Handle in `crates/chatalot-server/src/ws/handler.rs`
3. Mirror the type in `clients/web/src/lib/ws/types.ts`
4. Handle in `clients/web/src/lib/ws/handler.ts`

### New Database Table

1. Create a migration file: `migrations/NNN_description.sql` (sequential numbering, currently at 039)
2. Add model in `chatalot-db/src/models/` with `#[derive(Debug, sqlx::FromRow)]`
3. Add repository functions in `chatalot-db/src/repos/`
4. Register in `models/mod.rs` and `repos/mod.rs`

## Testing

Before submitting a pull request:

```bash
# Run all Rust tests (23 crypto tests + server tests)
cargo test

# Run clippy
cargo clippy -- -W clippy::all

# Check Svelte types
cd clients/web && npm run check

# Build web client (0 warnings expected)
cd clients/web && npm run build
```

See [Testing](./testing.md) for detailed test information.

## Pull Request Process

1. Ensure all tests pass and linting is clean
2. Write a clear PR description explaining what changed and why
3. Reference related issues
4. Keep PRs focused -- one feature or fix per PR when possible
5. The maintainer will review and may request changes

## Security

If you discover a security vulnerability, please report it responsibly. Do not open a public issue. Contact the maintainers directly.

## Related Pages

- [Building from Source](./building-from-source.md) -- Development environment setup
- [Architecture](./architecture.md) -- System architecture overview
- [Project Structure](./project-structure.md) -- Detailed file layout
- [Testing](./testing.md) -- Test suite details
