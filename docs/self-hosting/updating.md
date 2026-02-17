# Updating

> **Status: Complete**

How to update your Chatalot instance to the latest version.

## Before You Update

1. **Read the changelog** for breaking changes between your current version and the target version.
2. **Back up your instance** before updating. See [Backup and Restore](./backup-and-restore.md).
3. **Check your current version:**

```bash
curl -s http://localhost:8080/api/health | python3 -c "import sys,json; print(json.load(sys.stdin)['version'])"
```

## Docker Update

### Pull and Rebuild

```bash
cd /path/to/chatalot

# Pull latest code
git pull

# Rebuild and restart
docker compose up -d --build
```

The build will only recompile changed source files thanks to Docker's cache mounts. Migrations run automatically on startup.

### Verify the Update

```bash
# Wait for health check
sleep 10
curl http://localhost:8080/api/health
```

The `version` field in the health response should reflect the new version.

## Using the Deploy Script

If you are deploying to a remote server, the `scripts/deploy.sh` script handles the full workflow:

```bash
# Pull latest on the server and rebuild
DEPLOY_HOST=user@your-server \
DEPLOY_DIR=/srv/chatalot \
DEPLOY_GIT_URL=ssh://git@github.com/you/chatalot.git \
./scripts/deploy.sh --pull-only
```

The deploy script:

1. SSHs to the server
2. Pulls the latest code from git
3. Generates secrets if they do not exist
4. Runs `docker compose up -d --build`
5. Waits for the health check to pass (up to 60 seconds)
6. Reports success or failure

## Manual Deployment Update

If you are running without Docker:

```bash
cd /path/to/chatalot

# Pull latest code
git pull

# Rebuild the server
export SQLX_OFFLINE=true
cargo build --release

# Rebuild WASM crypto module (if crates/chatalot-crypto* changed)
./scripts/build-wasm.sh

# Rebuild the web client
cd clients/web && npm ci && npm run build && cd ../..

# Copy updated files to the install location
sudo cp target/release/chatalot-server /opt/chatalot/bin/
sudo cp -r clients/web/build/* /opt/chatalot/static/
sudo cp -r migrations/* /opt/chatalot/migrations/
sudo chown -R chatalot:chatalot /opt/chatalot

# Restart the service (migrations run automatically)
sudo systemctl restart chatalot
```

## Database Migrations

Migrations are applied **automatically** every time the server starts. The server calls `sqlx::migrate!()` which:

1. Checks which migrations have already been applied
2. Runs any pending migrations in order
3. Records each migration in the `_sqlx_migrations` table

You do not need to run migrations manually. If a migration fails, the server will log the error and exit. Fix the issue and restart.

## Zero-Downtime Considerations

Chatalot does not currently support zero-downtime updates. During the update process:

- Users with active WebSocket connections will be disconnected
- The web client will attempt to reconnect automatically
- Pending requests will fail with connection errors

To minimize disruption:

1. Schedule updates during low-activity periods
2. The restart itself is fast (seconds) -- most of the update time is the Docker build
3. Build the image first, then swap:

```bash
# Build without restarting
docker compose build

# Quick swap (seconds of downtime)
docker compose up -d
```

## Rollback

If an update causes issues, you can roll back to a previous version:

### Git Rollback

```bash
# Find the previous commit
git log --oneline -10

# Roll back to a specific commit
git checkout <commit-hash>

# Rebuild and restart
docker compose up -d --build
```

### Database Rollback

> **Warning:** Rolling back database migrations is not automatically supported. If a new migration altered the schema, rolling back the code without reverting the migration may cause errors.

If you need to undo a migration:

1. Check what the migration did:
   ```bash
   cat migrations/039_threads.sql
   ```

2. Manually reverse it:
   ```bash
   docker exec -it chatalot-db psql -U chatalot
   ```
   ```sql
   -- Example: if the migration added a table
   DROP TABLE IF EXISTS threads;
   -- Remove the migration record
   DELETE FROM _sqlx_migrations WHERE version = 39;
   ```

3. Roll back the code and restart:
   ```bash
   git checkout <previous-commit>
   docker compose up -d --build
   ```

### Full Restore from Backup

If the rollback is complex, it may be easier to restore from a backup:

```bash
# Stop everything
docker compose down

# Roll back the code
git checkout <previous-commit>

# Restore database
docker compose up -d postgres
sleep 5
docker exec chatalot-db psql -U chatalot -c "DROP DATABASE chatalot;"
docker exec chatalot-db psql -U chatalot -c "CREATE DATABASE chatalot OWNER chatalot;"
docker exec -i chatalot-db pg_restore -U chatalot -d chatalot < backup.dump

# Restore files
docker run --rm \
    -v chatalot_file_storage:/data \
    -v $(pwd):/backup \
    alpine sh -c "rm -rf /data/* && tar xzf /backup/files_backup.tar.gz -C /data"

# Start everything
docker compose up -d --build
```

## Checking for Updates

```bash
# Check remote for new commits
cd /path/to/chatalot
git fetch
git log HEAD..origin/master --oneline
```

## Next Step

For monitoring your running instance, see [Monitoring](./monitoring.md).
