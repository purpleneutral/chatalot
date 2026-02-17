# Troubleshooting

> **Status: Complete**

Common issues and solutions for self-hosted Chatalot instances.

## Server Won't Start

### "DATABASE_URL must be set"

The server cannot find the database connection string.

**Cause:** The `.env` file is missing or does not contain `DATABASE_URL`.

**Fix:**

```bash
# Generate .env from scratch
./scripts/generate-secrets.sh

# Or create it manually
cp .env.example .env
# Edit .env and set DATABASE_URL
```

### "failed to connect to database" / "connection refused"

The server cannot reach PostgreSQL.

**Cause:** PostgreSQL is not running, not ready yet, or the connection string is wrong.

**Fix:**

```bash
# Check if PostgreSQL is running
docker compose ps postgres
# Should show "Up (healthy)"

# Check PostgreSQL logs
docker compose logs postgres

# Verify the connection string in .env
# Docker internal: postgres://chatalot:password@postgres:5432/chatalot
# External:       postgres://chatalot:password@localhost:5432/chatalot
```

If PostgreSQL is healthy but the connection still fails, check that `DB_PASSWORD` in `.env` matches the password in `DATABASE_URL`.

### "No such file or directory" for JWT Keys

**Cause:** The JWT key files do not exist at the configured paths.

**Fix:**

```bash
# Generate keys
./scripts/generate-keys.sh

# Verify they exist
ls -la secrets/jwt_private.pem secrets/jwt_public.pem
```

For Docker, ensure the paths in `.env` point to the Docker secret paths:

```bash
JWT_PRIVATE_KEY_PATH=/run/secrets/jwt_private_key
JWT_PUBLIC_KEY_PATH=/run/secrets/jwt_public_key
```

For manual deployments, use the actual file paths:

```bash
JWT_PRIVATE_KEY_PATH=./secrets/jwt_private.pem
JWT_PUBLIC_KEY_PATH=./secrets/jwt_public.pem
```

### "InvalidKeyFormat" or "JWT Key Error"

**Cause:** The key files are corrupted or in the wrong format.

**Fix:** Regenerate the keys:

```bash
rm -f secrets/jwt_private.pem secrets/jwt_public.pem
./scripts/generate-keys.sh
docker compose restart chatalot
```

## Database Issues

### Migration Errors

**Cause:** A migration failed to apply, possibly due to a schema conflict.

**Fix:**

```bash
# Check the server logs for the specific error
docker compose logs chatalot | grep -i "migration\|error"

# Migrations are idempotent -- restarting usually resolves transient issues
docker compose restart chatalot
```

If a migration fails persistently, check the migration file and the current database state:

```bash
# See which migrations have been applied
docker exec chatalot-db psql -U chatalot -c \
    "SELECT version, description, success FROM _sqlx_migrations ORDER BY version;"
```

### "too many connections"

**Cause:** The connection pool is exhausted, or multiple server instances are running.

**Fix:**

```bash
# Check active connections
docker exec chatalot-db psql -U chatalot -c \
    "SELECT count(*) FROM pg_stat_activity WHERE datname = 'chatalot';"

# Check for multiple server instances
docker ps | grep chatalot

# If there are stale connections, restart PostgreSQL
docker compose restart postgres
```

### Database Is Full

**Cause:** The disk is full or the PostgreSQL volume has grown too large.

**Fix:**

```bash
# Check disk usage
df -h

# Check database size
docker exec chatalot-db psql -U chatalot -c \
    "SELECT pg_size_pretty(pg_database_size('chatalot'));"

# Check which tables are largest
docker exec chatalot-db psql -U chatalot -c \
    "SELECT relname, pg_size_pretty(pg_total_relation_size(relid))
     FROM pg_catalog.pg_statio_user_tables
     ORDER BY pg_total_relation_size(relid) DESC LIMIT 10;"

# Run PostgreSQL vacuum to reclaim space
docker exec chatalot-db psql -U chatalot -c "VACUUM FULL;"
```

## WebSocket Connection Issues

### WebSocket Fails to Connect

**Symptoms:** The web client shows "Connecting..." indefinitely or "Connection lost" errors.

**Common causes and fixes:**

1. **Reverse proxy not configured for WebSocket:**

   The `/ws` endpoint requires WebSocket upgrade support. See [TLS and Reverse Proxy](./tls-and-reverse-proxy.md) for configuration examples.

   For nginx, ensure:
   ```nginx
   location /ws {
       proxy_http_version 1.1;
       proxy_set_header Upgrade $http_upgrade;
       proxy_set_header Connection "upgrade";
       proxy_read_timeout 86400s;
   }
   ```

2. **TLS required:**

   If the page is served over HTTPS, the WebSocket must use `wss://`. Mixed content (HTTPS page with `ws://` WebSocket) is blocked by browsers.

3. **Proxy timeout too short:**

   Nginx's default `proxy_read_timeout` is 60 seconds. Set it to `86400s` (24 hours) for WebSocket connections to prevent disconnections.

### Frequent Disconnections

**Cause:** A proxy or load balancer is closing idle connections.

**Fix:** Increase timeouts on your reverse proxy. The Chatalot server sends periodic ping frames to keep connections alive, but the proxy must allow long-lived connections.

## File Upload Issues

### "File too large" Error

**Cause:** The file exceeds `MAX_FILE_SIZE_MB` (default: 100 MB).

**Fix:** Increase the limit in `.env`:

```bash
MAX_FILE_SIZE_MB=250
```

If using a reverse proxy, also update its body size limit:

**Nginx:**
```nginx
client_max_body_size 250M;
```

**Caddy:**
```
request_body {
    max_size 250MB
}
```

Then restart:

```bash
docker compose up -d
```

### "Upload quota exceeded"

**Cause:** The user has reached their per-user upload quota (`UPLOAD_QUOTA_MB`, default: 500 MB).

**Fix:** Increase the quota in `.env`:

```bash
UPLOAD_QUOTA_MB=1000  # 1 GB per user
# Or
UPLOAD_QUOTA_MB=0     # Unlimited
```

### Files Not Persisted After Restart

**Cause:** The file storage volume is not mounted correctly.

**Fix:** Verify the Docker volume:

```bash
# Check that the volume exists
docker volume ls | grep file_storage

# Check that it is mounted
docker inspect chatalot-server | grep -A5 "file_storage"
```

## Docker Issues

### Build Fails: Out of Memory

**Cause:** Rust compilation requires approximately 2 GB of RAM.

**Fix:**

```bash
# Add swap space
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

Or use a pre-built image instead of building from source (the `install.sh` script offers this option on ARM64).

### Build Fails: Cargo.lock Conflict

**Cause:** The lock file has conflicts from a git merge.

**Fix:**

```bash
# Regenerate the lock file
rm Cargo.lock
cargo generate-lockfile
docker compose up -d --build
```

### Container Keeps Restarting

**Cause:** The server is crashing on startup.

**Fix:**

```bash
# Check the exit code
docker inspect chatalot-server --format='{{.State.ExitCode}}'

# Check logs for the crash reason
docker compose logs --tail=50 chatalot
```

Common crash causes:
- Missing or invalid `.env` file
- Missing JWT key files
- Database not reachable
- Port already in use

### "Address already in use"

**Cause:** Port 8080 is already occupied by another process.

**Fix:**

```bash
# Find what is using port 8080
sudo lsof -i :8080
# or
sudo ss -tlnp | grep 8080

# Either stop the other process or change the Chatalot port
# In docker-compose.override.yml:
# ports:
#   - "3000:8080"
```

## Docker Networking

### Containers Can't Communicate

**Cause:** Services are not on the same Docker network.

**Fix:**

```bash
# Check that all services are on chatalot-net
docker network inspect chatalot_chatalot-net

# If using docker-compose.override.yml with custom networks,
# ensure chatalot and postgres share a network
```

### DNS Resolution Fails Inside Container

**Cause:** Docker's internal DNS is not resolving service names.

**Fix:**

```bash
# Test DNS from inside the container
docker exec chatalot-server sh -c "getent hosts postgres"

# If it fails, recreate the network
docker compose down
docker compose up -d
```

## Authentication Issues

### "token expired" After Server Restart

**Cause:** If JWT keys were regenerated, all existing tokens are invalid.

**Fix:** Users need to log in again. This is expected behavior after key rotation.

### 2FA Not Working

**Cause:** The `TOTP_ENCRYPTION_KEY` is missing or was changed.

**Fix:** Check that `TOTP_ENCRYPTION_KEY` is set in `.env`. If the key was changed, all existing 2FA secrets are unrecoverable -- affected users will need to use their backup codes or have an admin disable their 2FA.

## Log Analysis

### Finding Errors

```bash
# Search for errors in server logs
docker compose logs chatalot 2>&1 | grep -i "error\|panic\|fatal"

# Search for warnings
docker compose logs chatalot 2>&1 | grep -i "warn"

# View the most recent errors
docker compose logs --tail=500 chatalot 2>&1 | grep -i "error" | tail -20
```

### Enabling Debug Logging

```bash
# Temporarily enable debug logging
# In .env:
RUST_LOG=chatalot_server=debug,tower_http=debug

# Restart to apply
docker compose up -d

# Remember to set back to info after debugging
RUST_LOG=chatalot_server=info,tower_http=info
```

## Getting Help

If you cannot resolve an issue:

1. Check the server logs (`docker compose logs chatalot`)
2. Check the health endpoint (`curl http://localhost:8080/api/health`)
3. Search existing issues on the repository
4. Open a new issue with:
   - Server version (`curl /api/health`)
   - Relevant log output
   - Steps to reproduce
   - Your deployment method (Docker, manual, etc.)
