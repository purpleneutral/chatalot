# Development Guide

## Setting Up the Development Environment

### System Dependencies

**Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Requires Rust 1.84+ (edition 2024)
```

**Node.js:**
```bash
# Using nvm (recommended)
nvm install 22
nvm use 22
```

**PostgreSQL** (or use Docker):
```bash
# Start PostgreSQL via Docker (no need to install locally)
docker compose up postgres -d
```

**Tauri dependencies** (for desktop app, Linux):
```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Arch Linux
sudo pacman -S webkit2gtk-4.1 base-devel curl wget openssl gtk3 \
  libayatana-appindicator librsvg
```

### First-Time Setup

```bash
# Clone the repository
git clone <repo-url> chatalot
cd chatalot

# Generate secrets (JWT keys + .env)
./scripts/generate-secrets.sh

# Edit .env for local development
# Change DATABASE_URL to point to localhost if running PostgreSQL directly
# For Docker PostgreSQL, the default config works as-is

# Install web client dependencies
cd clients/web && npm install && cd ../..

# Start PostgreSQL
docker compose up postgres -d

# Verify everything builds
cargo build
cd clients/web && npm run build && cd ../..
```

### Running Locally

You'll typically run the server and web client in separate terminals:

**Terminal 1 — Server:**
```bash
# Run with local .env
cargo run
# Server starts on http://localhost:8080
```

**Terminal 2 — Web client (dev mode with hot reload):**
```bash
cd clients/web
npm run dev
# Dev server starts on http://localhost:5173
# Proxies API requests to the Rust server on :8080
```

When developing the web client, use the Vite dev server (port 5173) for hot module replacement. API calls are proxied to the Rust server.

For production-like testing, build the web client and let the Rust server serve it:
```bash
cd clients/web && npm run build
# The Rust server serves files from ./static (or STATIC_FILES_PATH)
# Copy build output: cp -r clients/web/build ./static
```

## Code Organization

### Server (Rust)

#### Adding a new API endpoint

1. Create or edit a route file in `crates/chatalot-server/src/routes/`:

```rust
use axum::{extract::State, Json, Router};
use axum::routing::get;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/my-endpoint", get(my_handler))
}

async fn my_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,  // JWT-authenticated user
) -> Result<Json<MyResponse>, AppError> {
    // Use state.db for database access
    // Use claims.sub for the authenticated user's ID
    Ok(Json(MyResponse { ... }))
}
```

2. Register the routes in `routes/mod.rs`:
```rust
// Add to protected_routes (requires auth) or public_routes
let protected_routes = Router::new()
    .merge(my_module::routes())
    // ...
```

3. Add the API types to `chatalot-common/src/api_types.rs`:
```rust
#[derive(Serialize, Deserialize)]
pub struct MyResponse {
    pub field: String,
}
```

#### Adding a new database entity

1. Create the migration in `migrations/`:
```sql
-- migrations/013_my_table.sql
CREATE TABLE my_table (
    id UUID PRIMARY KEY,
    ...
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

2. Create the model in `chatalot-db/src/models/`:
```rust
#[derive(Debug, sqlx::FromRow)]
pub struct MyEntity {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

3. Create the repository in `chatalot-db/src/repos/`:
```rust
pub async fn create(pool: &PgPool, id: Uuid) -> Result<MyEntity, sqlx::Error> {
    sqlx::query_as::<_, MyEntity>(
        "INSERT INTO my_table (id) VALUES ($1) RETURNING *"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}
```

4. Register in `models/mod.rs` and `repos/mod.rs`.

#### Adding a new WebSocket message type

1. Add the message variants to `chatalot-common/src/ws_messages.rs`:
```rust
// Client → Server
pub enum ClientMessage {
    MyAction { data: String },
    // ...
}

// Server → Client
pub enum ServerMessage {
    MyEvent { data: String },
    // ...
}
```

2. Handle the client message in `ws/handler.rs`:
```rust
ClientMessage::MyAction { data } => {
    // Process the action
    // Broadcast to relevant users via connection_manager
}
```

3. Mirror the types in `clients/web/src/lib/ws/types.ts`:
```typescript
export type ClientMessage =
    | { type: 'my_action'; data: string }
    // ...

export type ServerMessage =
    | { type: 'my_event'; data: string }
    // ...
```

4. Handle the server message in `clients/web/src/lib/ws/handler.ts`:
```typescript
case 'my_event': {
    // Update the appropriate store
    break;
}
```

### Web Client (Svelte 5)

#### State management with runes

Stores use Svelte 5 runes (`$state`, `$derived`):

```typescript
class MyStore {
    // Reactive state
    private items = $state<Map<string, Item>>(new Map());

    // Read access
    getItem(id: string): Item | undefined {
        return this.items.get(id);
    }

    // Mutations (create new references for reactivity)
    addItem(item: Item) {
        const next = new Map(this.items);
        next.set(item.id, item);
        this.items = next;
    }
}

export const myStore = new MyStore();
```

**Important**: Svelte 5 runes require creating new references (new Map, new Array) for reactivity to trigger. Mutating in place won't cause re-renders.

#### Adding a new page

1. Create `clients/web/src/routes/my-page/+page.svelte`:

```svelte
<script lang="ts">
    import { goto } from '$app/navigation';
    import { authStore } from '$lib/stores/auth.svelte';
    import { onMount } from 'svelte';

    onMount(() => {
        if (!authStore.isAuthenticated) {
            goto('/login');
        }
    });
</script>

{#if authStore.isAuthenticated}
    <div class="min-h-screen bg-[var(--bg-primary)]">
        <!-- Page content using CSS variables for theming -->
    </div>
{/if}
```

#### API client functions

Add API calls to `clients/web/src/lib/api/`:

```typescript
import { apiClient } from './client';

export async function myApiCall(data: string): Promise<MyResponse> {
    return apiClient.post('/api/my-endpoint', { data });
}
```

The `apiClient` handles JWT token injection and automatic refresh on 401.

## Testing

### Rust Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p chatalot-crypto

# Run a specific test
cargo test test_x3dh_initiator_responder_agree

# Run with output
cargo test -- --nocapture
```

### Linting

```bash
# Rust
cargo clippy -- -W clippy::all

# Svelte type checking
cd clients/web && npm run check

# Svelte build (catches template errors)
cd clients/web && npm run build
```

### Manual Testing Checklist

1. Register two users in separate browsers/incognito windows
2. Create a channel with user A
3. Join the channel with user B
4. Send messages between A and B (verify real-time delivery)
5. Edit a message (verify the edit appears for both users)
6. Delete a message (verify removal for both users)
7. Add/remove reactions
8. Test typing indicators
9. Upload a file and verify it appears as a file message
10. Test voice call (join from both users)
11. Enable 2FA on one account and verify login requires the code
12. Switch themes (light/dark) and verify all pages render correctly
13. Test on mobile viewport (sidebar should collapse)
14. Open multiple tabs — messages should appear in all tabs

## Code Style

### Rust

- Follow standard Rust conventions (`rustfmt` defaults)
- Use `thiserror` for error types, `anyhow` for ad-hoc errors in main
- Repository functions are standalone `async fn` (not trait methods)
- All sensitive data types derive `Zeroize` for secure cleanup
- Use `sqlx::query_as::<_, Type>(...)` for database queries (runtime-checked, not compile-time)

### Svelte/TypeScript

- Svelte 5 runes — no legacy `$:` reactive statements or writable stores
- Tailwind CSS for styling — use CSS variables (`var(--bg-primary)`) for theme-aware colors
- Components in `src/lib/components/`, stores in `src/lib/stores/`
- API functions in `src/lib/api/`, one file per domain

### Commit Messages

Use concise, imperative commit messages:
```
Add reaction support to messages
Fix WebSocket reconnection on token refresh
Update rate limiting to use token bucket
```
