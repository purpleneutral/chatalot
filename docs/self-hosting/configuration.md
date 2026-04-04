# Configuration

> **Status: Complete**

All environment variables and settings for the Chatalot server.

## Environment Variables

Chatalot is configured entirely through environment variables. When using Docker, these are set in the `.env` file in the project root. The `docker-compose.yml` passes them to the container.

### Required Variables

These must be set for the server to start.

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://chatalot:password@postgres:5432/chatalot` |

### Server

| Variable | Description | Default |
|----------|-------------|---------|
| `LISTEN_ADDR` | Address and port the server binds to | `0.0.0.0:8080` |
| `STATIC_FILES_PATH` | Path to the built Svelte SPA files | `./static` (Docker: `/app/static`) |
| `PUBLIC_URL` | Public-facing URL of your instance (used in links, invites) | *(none)* |
| `RUST_LOG` | Log level filter ([tracing-subscriber](https://docs.rs/tracing-subscriber) syntax) | `chatalot_server=info,tower_http=info` |

### Authentication

| Variable | Description | Default |
|----------|-------------|---------|
| `JWT_PRIVATE_KEY_PATH` | Path to Ed25519 private key PEM file | `./secrets/jwt_private.pem` (Docker: `/run/secrets/jwt_private_key`) |
| `JWT_PUBLIC_KEY_PATH` | Path to Ed25519 public key PEM file | `./secrets/jwt_public.pem` (Docker: `/run/secrets/jwt_public_key`) |
| `TOTP_ENCRYPTION_KEY` | 32-byte hex key for encrypting TOTP secrets at rest | *(none -- 2FA setup requires this)* |

Access tokens are valid for **15 minutes**. Refresh tokens are valid for **30 days**. These values are compiled into the binary and are not configurable at runtime.

### Registration

| Variable | Description | Default |
|----------|-------------|---------|
| `REGISTRATION_MODE` | Controls who can register: `open`, `invite_only`, or `closed` | `invite_only` |
| `ADMIN_USERNAME` | Username to automatically grant admin privileges on startup | *(none)* |
| `COMMUNITY_CREATION_MODE` | Who can create communities: `anyone` or `admin_only` | `admin_only` |

### File Uploads

| Variable | Description | Default |
|----------|-------------|---------|
| `FILE_STORAGE_PATH` | Directory where uploaded files are stored | `./data/files` (Docker: `/app/data/files`) |
| `MAX_FILE_SIZE_MB` | Maximum file upload size in megabytes | `100` |
| `UPLOAD_QUOTA_MB` | Per-user upload quota in megabytes (0 = unlimited) | `500` |

### OIDC / SSO

Chatalot supports single sign-on via any OpenID Connect provider (Authentik, Keycloak, Authelia, Zitadel, etc.). When configured, a **Log in with SSO** button appears on the login page alongside the standard email/password form.

| Variable | Description | Default |
|----------|-------------|----------|
| `OIDC_ISSUER_URL` | OIDC provider discovery URL. Must expose a `/.well-known/openid-configuration` endpoint. | *(none -- SSO disabled)* |
| `OIDC_CLIENT_ID` | Client ID registered with your identity provider | *(none)* |
| `OIDC_CLIENT_SECRET` | Client secret from your identity provider | *(none)* |
| `OIDC_REDIRECT_URI` | The callback URL the provider redirects to after authentication. Must match the redirect URI configured in your provider. | *(none)* |
| `OIDC_DISABLE_PASSWORD_LOGIN` | Set to `true` to hide the email/password form and require all users to log in via SSO | `false` |

SSO is enabled when `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, and `OIDC_CLIENT_SECRET` are all set. If any of the three is missing, OIDC is silently disabled.

#### Quick Example: Authentik

1. In Authentik, create a new **OAuth2/OpenID Provider**:
   - **Name**: `chatalot`
   - **Authorization flow**: implicit consent (or explicit, depending on your preference)
   - **Redirect URI**: `https://chat.example.com/auth/oidc/callback`
   - **Scopes**: `openid`, `email`, `profile`

2. Create an **Application** linked to the provider, with the slug `chatalot`.

3. Set the environment variables:
   ```bash
   OIDC_ISSUER_URL=https://auth.example.com/application/o/chatalot/
   OIDC_CLIENT_ID=<client-id-from-authentik>
   OIDC_CLIENT_SECRET=<client-secret-from-authentik>
   OIDC_REDIRECT_URI=https://chat.example.com/auth/oidc/callback
   ```

4. Restart Chatalot:
   ```bash
   docker compose up -d
   ```

#### User Provisioning

When a user logs in via OIDC for the first time, Chatalot automatically creates a local account using the `email` and `preferred_username` claims from the identity provider. If the username is already taken, a numeric suffix is appended.

OIDC users can still set a local password later from Settings > Security if you want to allow fallback authentication. If `OIDC_DISABLE_PASSWORD_LOGIN=true`, users who were provisioned via SSO will only be able to log in through the identity provider.

#### Notes

- The provider must support the **Authorization Code** flow.
- The `openid`, `email`, and `profile` scopes are required.
- OIDC login respects the same rate limiting and account lockout rules as password login.
- Invite-only mode still applies: if `REGISTRATION_MODE=invite_only`, first-time SSO users will need a valid invite code unless an admin has pre-created their account.

### Integrations (Optional)

| Variable | Description | Default |
|----------|-------------|---------|
| `GIPHY_API_KEY` | API key for GIF search via Giphy | *(none -- GIF search disabled)* |
| `GITHUB_API_TOKEN` | GitHub personal access token for feedback issue creation | *(none)* |
| `GITHUB_REPO_OWNER` | GitHub repository owner for feedback issues | *(none)* |
| `GITHUB_REPO_NAME` | GitHub repository name for feedback issues | *(none)* |

### Web Push Notifications (Optional)

| Variable | Description | Default |
|----------|-------------|---------|
| `VAPID_PRIVATE_KEY` | Base64-encoded ECDSA P-256 private key for web push | *(none -- push disabled)* |
| `VAPID_PUBLIC_KEY` | Base64-encoded ECDSA P-256 public key for web push | *(none -- push disabled)* |

If both keys are set, users can enable push notifications in Settings to receive DM alerts when the tab is closed. If omitted, the push feature is silently hidden.

Generate VAPID keys with:
```bash
npx web-push generate-vapid-keys
```

### Cloudflare Tunnel (Optional)

| Variable | Description | Default |
|----------|-------------|---------|
| `CLOUDFLARE_TUNNEL_TOKEN` | Token for a named Cloudflare Tunnel | *(none)* |

### Docker Compose Only

These variables are used by `docker-compose.yml` but not by the server binary directly:

| Variable | Description | Default |
|----------|-------------|---------|
| `DB_PASSWORD` | PostgreSQL password (used to construct `DATABASE_URL`) | *(auto-generated by setup scripts)* |

## Example .env File

```bash
# Database
DATABASE_URL=postgres://chatalot:your_secure_password@postgres:5432/chatalot
DB_PASSWORD=your_secure_password

# JWT signing keys (Docker paths)
JWT_PRIVATE_KEY_PATH=/run/secrets/jwt_private_key
JWT_PUBLIC_KEY_PATH=/run/secrets/jwt_public_key

# 2FA encryption key (generate with: openssl rand -hex 32)
TOTP_ENCRYPTION_KEY=a1b2c3d4e5f6...

# Server
LISTEN_ADDR=0.0.0.0:8080
RUST_LOG=chatalot_server=info,tower_http=info

# Files
FILE_STORAGE_PATH=/app/data/files
MAX_FILE_SIZE_MB=100
UPLOAD_QUOTA_MB=500

# Registration
REGISTRATION_MODE=invite_only
ADMIN_USERNAME=alice
COMMUNITY_CREATION_MODE=admin_only

# Public URL (if behind a reverse proxy)
PUBLIC_URL=https://chat.example.com

# OIDC / SSO (optional)
OIDC_ISSUER_URL=https://auth.example.com/application/o/chatalot/
OIDC_CLIENT_ID=
OIDC_CLIENT_SECRET=
OIDC_REDIRECT_URI=https://chat.example.com/auth/oidc/callback
OIDC_DISABLE_PASSWORD_LOGIN=false

# Web push notifications (optional -- generate with: npx web-push generate-vapid-keys)
VAPID_PRIVATE_KEY=
VAPID_PUBLIC_KEY=

# Cloudflare Tunnel (optional)
CLOUDFLARE_TUNNEL_TOKEN=

# GIF search (optional)
GIPHY_API_KEY=
```

## Log Levels

The `RUST_LOG` variable controls logging verbosity using `tracing-subscriber` filter syntax:

```bash
# Production (default)
RUST_LOG=chatalot_server=info,tower_http=info

# Debug all server logs
RUST_LOG=chatalot_server=debug,tower_http=debug

# Debug specific modules
RUST_LOG=chatalot_server::ws=debug,chatalot_server::routes::auth=debug

# Verbose everything (generates a lot of output)
RUST_LOG=debug
```

After changing `RUST_LOG`, restart the container:

```bash
docker compose up -d
```

## CORS Configuration

CORS is configured permissively in the server (`allow_origin: Any`, `allow_methods: Any`, `allow_headers: Any`). This is intentional because:

- The Tauri desktop client makes cross-origin requests from a `file://` or `tauri://` origin
- All API endpoints (except `/api/health` and `/api/auth/*`) require a valid JWT, which is the actual access gate

If you need to restrict CORS for your deployment, this would require modifying the source code in `crates/chatalot-server/src/routes/mod.rs`.

## Security Headers

The server automatically sets the following security headers on all responses:

| Header | Value |
|--------|-------|
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `X-XSS-Protection` | `1; mode=block` |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Permissions-Policy` | `camera=(self), microphone=(self), geolocation=()` |
| `Content-Security-Policy` | Restricts scripts, styles, connections, and media sources |

## Rate Limiting

The server includes built-in rate limiting:

| Scope | Rate | Burst |
|-------|------|-------|
| **General API** | 20 requests/second per IP | 50 |
| **Auth endpoints** (login, register) | 5 requests/second per IP | 10 |

Rate limited requests receive HTTP 429 with a JSON error body.

## Next Step

For database-specific configuration, see [Database Setup](./database-setup.md).
