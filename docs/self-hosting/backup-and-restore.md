# Backup and Restore

> **Status: Complete**

How to back up your Chatalot instance and restore from backups.

## What to Back Up

A complete Chatalot backup includes three components:

| Component | Location (Docker) | Location (Manual) | Contains |
|-----------|--------------------|-------------------|----------|
| **Database** | `postgres_data` volume | PostgreSQL data directory | Users, messages, channels, communities, all metadata |
| **Uploaded files** | `file_storage` volume | `./data/files` | All file uploads (encrypted on disk) |
| **Secrets** | `./secrets/` directory | `./secrets/` directory | JWT signing keys, `.env` file |

> **Warning:** If you lose the JWT private key, all existing access and refresh tokens become invalid. Every user will be forced to log in again. Keep backups of `secrets/jwt_private.pem` and `secrets/jwt_public.pem` secure.

## Database Backup

### Using pg_dump (Recommended)

```bash
# Docker: dump to a SQL file
docker exec chatalot-db pg_dump -U chatalot --format=custom chatalot > backup_$(date +%Y%m%d_%H%M%S).dump

# Manual: dump from local PostgreSQL
pg_dump -U chatalot --format=custom chatalot > backup_$(date +%Y%m%d_%H%M%S).dump
```

The `--format=custom` flag produces a compressed binary format that supports selective restore. For a plain SQL dump:

```bash
docker exec chatalot-db pg_dump -U chatalot chatalot > backup_$(date +%Y%m%d_%H%M%S).sql
```

### Verify the Dump

```bash
# Check the dump file is valid (custom format)
pg_restore --list backup_20260216_120000.dump | head -20

# Check file size (should be non-zero)
ls -lh backup_*.dump
```

## File Backup

### Docker

```bash
# Copy files from the Docker volume to a local tar archive
docker run --rm \
    -v chatalot_file_storage:/data:ro \
    -v $(pwd):/backup \
    alpine tar czf /backup/files_$(date +%Y%m%d_%H%M%S).tar.gz -C /data .
```

### Manual

```bash
tar czf files_$(date +%Y%m%d_%H%M%S).tar.gz -C /opt/chatalot/data/files .
```

## Secrets Backup

```bash
# Back up JWT keys and environment file
tar czf secrets_$(date +%Y%m%d_%H%M%S).tar.gz secrets/ .env
```

> **Warning:** The `.env` file contains the database password and TOTP encryption key. Store secret backups in an encrypted location (e.g., an encrypted USB drive, a password manager, or an encrypted cloud storage bucket).

## Full Backup Script

Create `scripts/backup.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

BACKUP_DIR="${1:-./backups}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

echo "Backing up database..."
docker exec chatalot-db pg_dump -U chatalot --format=custom chatalot \
    > "$BACKUP_DIR/db_${TIMESTAMP}.dump"

echo "Backing up uploaded files..."
docker run --rm \
    -v chatalot_file_storage:/data:ro \
    -v "$(pwd)/$BACKUP_DIR":/backup \
    alpine tar czf "/backup/files_${TIMESTAMP}.tar.gz" -C /data .

echo "Backing up secrets..."
tar czf "$BACKUP_DIR/secrets_${TIMESTAMP}.tar.gz" secrets/ .env

echo "Backup complete:"
ls -lh "$BACKUP_DIR"/*_${TIMESTAMP}*
```

```bash
chmod +x scripts/backup.sh
./scripts/backup.sh ./backups
```

## Automated Backups with Cron

Add a cron job to run backups daily:

```bash
# Edit crontab
crontab -e

# Add: run backup at 3 AM daily, keep 30 days of backups
0 3 * * * cd /srv/chatalot && ./scripts/backup.sh ./backups && find ./backups -mtime +30 -delete
```

## Restore Procedure

### Restore the Database

```bash
# Stop the application (keep PostgreSQL running)
docker compose stop chatalot

# Drop and recreate the database
docker exec chatalot-db psql -U chatalot -c "DROP DATABASE chatalot;"
docker exec chatalot-db psql -U chatalot -c "CREATE DATABASE chatalot OWNER chatalot;"

# Restore from custom format dump
docker exec -i chatalot-db pg_restore -U chatalot -d chatalot < backup_20260216_120000.dump

# Or restore from plain SQL
docker exec -i chatalot-db psql -U chatalot chatalot < backup_20260216_120000.sql

# Start the application
docker compose start chatalot
```

### Restore Uploaded Files

```bash
# Stop the application
docker compose stop chatalot

# Clear existing files and restore
docker run --rm \
    -v chatalot_file_storage:/data \
    -v $(pwd):/backup \
    alpine sh -c "rm -rf /data/* && tar xzf /backup/files_20260216_120000.tar.gz -C /data"

# Start the application
docker compose start chatalot
```

### Restore Secrets

```bash
# Stop the application
docker compose down

# Restore secrets
tar xzf secrets_20260216_120000.tar.gz

# Verify file permissions
chmod 600 secrets/jwt_private.pem
chmod 644 secrets/jwt_public.pem
chmod 600 .env

# Start the application
docker compose up -d
```

## Testing Backups

Periodically test your backups by restoring to a separate environment:

```bash
# Create a test directory
mkdir /tmp/chatalot-restore-test
cd /tmp/chatalot-restore-test

# Start a temporary PostgreSQL instance
docker run -d --name restore-test-db \
    -e POSTGRES_DB=chatalot \
    -e POSTGRES_USER=chatalot \
    -e POSTGRES_PASSWORD=test \
    postgres:17

# Wait for it to start
sleep 5

# Restore the dump
docker exec -i restore-test-db pg_restore -U chatalot -d chatalot < /path/to/backup.dump

# Verify the data
docker exec restore-test-db psql -U chatalot -c "SELECT count(*) FROM users;"
docker exec restore-test-db psql -U chatalot -c "SELECT count(*) FROM messages;"

# Clean up
docker rm -f restore-test-db
```

## Backup Storage Recommendations

| Method | Suitable For |
|--------|-------------|
| Local directory | Development, small instances |
| External drive | Small to medium instances |
| Remote server (rsync/scp) | Production instances |
| S3-compatible storage | Large instances, long-term retention |
| Encrypted cloud storage | Sensitive data, compliance requirements |

Example rsync to a remote server:

```bash
rsync -az ./backups/ backup-server:/backups/chatalot/
```

## Next Step

For update procedures, see [Updating](./updating.md).
