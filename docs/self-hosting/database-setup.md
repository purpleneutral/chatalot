# Database Setup

> **Status: Complete**

Chatalot uses PostgreSQL as its database. This page covers installation, configuration, and migration management.

## Docker Setup (Automatic)

If you are using Docker Compose (the recommended setup), PostgreSQL is included as the `postgres` service and requires no manual setup. The database, user, and password are configured automatically through the `.env` file.

The `docker-compose.yml` defines the PostgreSQL service:

```yaml
postgres:
  image: postgres:17
  container_name: chatalot-db
  restart: unless-stopped
  environment:
    POSTGRES_DB: chatalot
    POSTGRES_USER: chatalot
    POSTGRES_PASSWORD: ${DB_PASSWORD}
  volumes:
    - postgres_data:/var/lib/postgresql/data
  healthcheck:
    test: ["CMD-SHELL", "pg_isready -U chatalot"]
    interval: 10s
    timeout: 5s
    retries: 5
```

The Chatalot server container waits for the PostgreSQL health check to pass before starting.

## Manual PostgreSQL Setup

If you are running without Docker or want to use an external PostgreSQL server:

### Install PostgreSQL

```bash
# Debian/Ubuntu
sudo apt install postgresql postgresql-contrib

# Fedora/RHEL
sudo dnf install postgresql-server postgresql-contrib
sudo postgresql-setup --initdb
sudo systemctl enable --now postgresql

# Arch Linux
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
sudo systemctl enable --now postgresql
```

### Create Database and User

```bash
sudo -u postgres psql
```

```sql
CREATE USER chatalot WITH PASSWORD 'your_secure_password';
CREATE DATABASE chatalot OWNER chatalot;
GRANT ALL PRIVILEGES ON DATABASE chatalot TO chatalot;

-- Required for extensions and schema management
\c chatalot
GRANT ALL ON SCHEMA public TO chatalot;
```

### Connection String Format

The `DATABASE_URL` environment variable uses the standard PostgreSQL connection string format:

```
postgres://username:password@host:port/database
```

Examples:

```bash
# Local PostgreSQL
DATABASE_URL=postgres://chatalot:your_password@localhost:5432/chatalot

# Docker internal network
DATABASE_URL=postgres://chatalot:your_password@postgres:5432/chatalot

# Remote server
DATABASE_URL=postgres://chatalot:your_password@db.example.com:5432/chatalot

# With SSL
DATABASE_URL=postgres://chatalot:your_password@db.example.com:5432/chatalot?sslmode=require
```

## Migrations

### Automatic Migrations

Chatalot runs database migrations **automatically on every server startup**. The server calls `sqlx::migrate!()` which applies any pending migrations from the `migrations/` directory. There is no need to run migrations manually.

When the server starts, you will see log output like:

```
INFO chatalot_server: Database connected
INFO chatalot_server: Migrations applied
```

### Migration Files

Migrations are stored in the `migrations/` directory at the project root. As of the current version, there are 39 migration files covering:

| Range | Description |
|-------|-------------|
| `001`-`005` | Users, identity keys, refresh tokens, audit log, prekeys |
| `006`-`010` | Channels, messages, files, DM pairs, voice sessions |
| `011`-`015` | Reactions, unread tracking, channel bans, groups, group invites |
| `016`-`020` | Account deletion, admin role, registration invites, communities, user preferences |
| `021`-`025` | Pinned messages, sender key distributions, instance owner, permissions, performance indexes |
| `026`-`030` | Security suite, lockout/quota, blocking/reports, new features, discoverable |
| `031`-`035` | Group discoverable, personal groups, channel archiving, customization, more indexes |
| `036`-`039` | Community members index, recovery codes, message edits, threads |

### Running Migrations Manually (Without the Server)

If you need to run migrations without starting the full server (e.g., during maintenance), you can use the `sqlx` CLI:

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run pending migrations
DATABASE_URL=postgres://chatalot:password@localhost:5432/chatalot sqlx migrate run

# Check migration status
DATABASE_URL=postgres://chatalot:password@localhost:5432/chatalot sqlx migrate info
```

## Connection Pool Settings

The server configures its connection pool with these settings:

| Setting | Value |
|---------|-------|
| Maximum connections | 50 |
| Minimum connections | 2 |
| Acquire timeout | 5 seconds |

These values are compiled into the binary. For most deployments (up to a few hundred concurrent users), the defaults are appropriate.

## Database Permissions

The Chatalot database user needs the following permissions:

- `CREATE TABLE` -- Migrations create tables
- `ALTER TABLE` -- Migrations modify tables
- `CREATE INDEX` -- Migrations add indexes
- `INSERT`, `UPDATE`, `DELETE`, `SELECT` -- Normal operations
- `CREATE TYPE` -- Some migrations create custom enum types
- `USAGE` on the `public` schema

The simplest approach is to make the Chatalot user the **owner** of the database, which grants all needed permissions:

```sql
CREATE DATABASE chatalot OWNER chatalot;
```

## PostgreSQL Tuning (Optional)

For larger instances (100+ users), consider tuning these PostgreSQL settings in `postgresql.conf`:

```ini
# Memory
shared_buffers = 256MB          # 25% of available RAM (default: 128MB)
effective_cache_size = 768MB    # 75% of available RAM
work_mem = 4MB                  # Per-operation memory (default: 4MB)

# Write-ahead log
wal_buffers = 16MB
checkpoint_completion_target = 0.9

# Connections
max_connections = 100           # Chatalot uses up to 50
```

## Accessing the Database

### Docker

```bash
# Interactive psql session
docker exec -it chatalot-db psql -U chatalot

# Run a single query
docker exec chatalot-db psql -U chatalot -c "SELECT count(*) FROM users;"

# Check database size
docker exec chatalot-db psql -U chatalot -c "SELECT pg_size_pretty(pg_database_size('chatalot'));"
```

### External Connection

If you need to connect to the Docker PostgreSQL from the host (for tools like pgAdmin), add a port mapping to `docker-compose.override.yml`:

```yaml
services:
  postgres:
    ports:
      - "5432:5432"
```

> **Warning:** Do not expose the database port to the public internet. Use this only for local administration or tunnel through SSH.

## Next Step

For TLS and reverse proxy configuration, see [TLS and Reverse Proxy](./tls-and-reverse-proxy.md).
