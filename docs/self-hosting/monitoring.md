# Monitoring

> **Status: Complete**

Health checks, logging, and resource monitoring for your Chatalot instance.

## Health Check Endpoint

The server exposes a health check at `GET /api/health` that verifies both the server and database are operational:

```bash
curl http://localhost:8080/api/health
```

Response:

```json
{
  "status": "ok",
  "version": "0.17.0",
  "uptime_secs": 86400,
  "db_healthy": true
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | `"ok"` if everything is healthy, `"degraded"` if the database is unreachable |
| `version` | string | Server version (from `Cargo.toml`) |
| `uptime_secs` | number | Seconds since the server started |
| `db_healthy` | boolean | Whether a `SELECT 1` query to PostgreSQL succeeded |

### Using the Health Check

**Simple uptime monitoring:**

```bash
# Cron job: check every 5 minutes, alert on failure
*/5 * * * * curl -sf http://localhost:8080/api/health > /dev/null || echo "Chatalot is down" | mail -s "Alert" admin@example.com
```

**With monitoring tools:**

| Tool | Configuration |
|------|--------------|
| UptimeRobot | HTTP check on `https://chat.example.com/api/health` |
| Uptime Kuma | HTTP(s) monitor, expected status code 200 |
| Prometheus | Use a JSON exporter or blackbox exporter |
| Healthchecks.io | Ping from cron after successful health check |

### Docker Health Check

The Chatalot container has a built-in Docker health check that queries `/api/health` every 30 seconds:

```bash
# Check container health status
docker inspect --format='{{.State.Health.Status}}' chatalot-server

# View recent health check results
docker inspect --format='{{json .State.Health}}' chatalot-server | python3 -m json.tool
```

## Server Logs

### Docker

```bash
# Follow all logs
docker compose logs -f

# Follow only the server
docker compose logs -f chatalot

# Follow only the database
docker compose logs -f postgres

# View last 100 lines
docker compose logs --tail=100 chatalot

# View logs since a specific time
docker compose logs --since="2026-02-16T10:00:00" chatalot
```

### systemd (Manual Deployment)

```bash
# Follow logs
sudo journalctl -u chatalot -f

# View recent logs
sudo journalctl -u chatalot --since "1 hour ago"

# View logs from last boot
sudo journalctl -u chatalot -b
```

### Log Levels

Configure verbosity with the `RUST_LOG` environment variable:

```bash
# Production (default)
RUST_LOG=chatalot_server=info,tower_http=info

# Debug WebSocket issues
RUST_LOG=chatalot_server::ws=debug,chatalot_server=info,tower_http=info

# Debug authentication issues
RUST_LOG=chatalot_server::routes::auth=debug,chatalot_server::middleware::auth=debug,chatalot_server=info

# Debug database queries (very verbose)
RUST_LOG=sqlx=debug,chatalot_server=info

# Everything (generates a lot of output)
RUST_LOG=debug
```

### What the Server Logs

At the `info` level, the server logs:

- Startup sequence (database connection, migrations, listening address)
- Admin user promotion
- Periodic cleanup operations:
  - Expired refresh token removal (hourly)
  - Used prekey cleanup (hourly)
  - Audit log pruning, entries older than 90 days (hourly)
  - Orphaned voice session cleanup (hourly)
  - Soft-deleted message garbage collection (daily)
  - Orphan file cleanup, disk files with no DB record (daily)
  - Expired message TTL enforcement (every 5 minutes)
- Graceful shutdown signal handling

At the `debug` level, you additionally see:
- Individual HTTP requests (via tower-http tracing)
- Broadcast channel cleanup
- Cache eviction (GIF and link preview caches)

## Database Monitoring

### Connection Count

```bash
docker exec chatalot-db psql -U chatalot -c \
    "SELECT count(*) AS active_connections FROM pg_stat_activity WHERE datname = 'chatalot';"
```

The server's connection pool allows up to 50 connections with a minimum of 2 kept alive.

### Database Size

```bash
docker exec chatalot-db psql -U chatalot -c \
    "SELECT pg_size_pretty(pg_database_size('chatalot')) AS db_size;"
```

### Table Sizes

```bash
docker exec chatalot-db psql -U chatalot -c \
    "SELECT relname AS table, pg_size_pretty(pg_total_relation_size(relid)) AS size
     FROM pg_catalog.pg_statio_user_tables
     ORDER BY pg_total_relation_size(relid) DESC
     LIMIT 10;"
```

### Slow Queries

```bash
docker exec chatalot-db psql -U chatalot -c \
    "SELECT query, calls, mean_exec_time::numeric(10,2) AS avg_ms, total_exec_time::numeric(10,2) AS total_ms
     FROM pg_stat_statements
     ORDER BY mean_exec_time DESC
     LIMIT 10;"
```

> **Note:** The `pg_stat_statements` extension must be enabled for slow query tracking. Add `shared_preload_libraries = 'pg_stat_statements'` to `postgresql.conf`.

## Disk Usage Monitoring

### Docker Volumes

```bash
# Overall Docker disk usage
docker system df

# Detailed volume sizes
docker system df -v | grep -A5 "VOLUME NAME"
```

### File Storage

```bash
# Size of uploaded files
docker run --rm -v chatalot_file_storage:/data alpine du -sh /data

# Number of uploaded files
docker run --rm -v chatalot_file_storage:/data alpine find /data -type f | wc -l
```

### Database Volume

```bash
docker run --rm -v chatalot_postgres_data:/data alpine du -sh /data
```

## Resource Usage

### Container Stats

```bash
# Live resource usage
docker stats chatalot-server chatalot-db

# One-time snapshot
docker stats --no-stream chatalot-server chatalot-db
```

### Expected Resource Usage

For a small instance (10-50 users):

| Container | CPU | RAM |
|-----------|-----|-----|
| `chatalot-server` | < 5% idle, spikes during file uploads | 50-150 MB |
| `chatalot-db` | < 2% idle | 100-300 MB |

## Setting Up Alerts

### Simple Script-Based Alerting

Create a monitoring script:

```bash
#!/usr/bin/env bash
# check-chatalot.sh

HEALTH_URL="http://localhost:8080/api/health"
ALERT_EMAIL="admin@example.com"

# Check health endpoint
RESPONSE=$(curl -sf -w '%{http_code}' "$HEALTH_URL" 2>/dev/null)
HTTP_CODE="${RESPONSE: -3}"

if [ "$HTTP_CODE" != "200" ]; then
    echo "Chatalot health check failed (HTTP $HTTP_CODE)" | \
        mail -s "[ALERT] Chatalot down" "$ALERT_EMAIL"
    exit 1
fi

# Check disk usage (alert at 90%)
DISK_USAGE=$(df -h /var/lib/docker | awk 'NR==2 {print $5}' | tr -d '%')
if [ "$DISK_USAGE" -gt 90 ]; then
    echo "Disk usage at ${DISK_USAGE}%" | \
        mail -s "[WARNING] Chatalot disk usage high" "$ALERT_EMAIL"
fi
```

Add to cron:

```bash
*/5 * * * * /path/to/check-chatalot.sh
```

## Next Step

For production security best practices, see [Security Hardening](./security-hardening.md).
