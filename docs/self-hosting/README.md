# Self-Hosting

> **Status: Complete**

Run your own Chatalot instance. Self-hosting is a first-class feature of Chatalot -- not an afterthought.

## Why Self-Host?

Chatalot is designed from day one to be self-hosted. When you run your own instance, you get:

- **Complete data ownership.** Your messages, files, and metadata live on your server. No third party ever has access.
- **End-to-end encryption you can verify.** Because you control the server code, you can audit the cryptographic pipeline yourself.
- **No usage limits.** No artificial caps on file sizes, message history, or number of users.
- **Privacy by architecture.** No telemetry, no tracking, no ads.

## Pages

| # | Page | Description |
|---|------|-------------|
| 1 | [Requirements](./requirements.md) | Hardware, software, and infrastructure prerequisites |
| 2 | [Quick Start](./quick-start.md) | Get a working instance in 5 minutes with Docker |
| 3 | [Configuration](./configuration.md) | All environment variables and settings explained |
| 4 | [Database Setup](./database-setup.md) | PostgreSQL installation, migrations, and connection tuning |
| 5 | [TLS and Reverse Proxy](./tls-and-reverse-proxy.md) | Nginx, Caddy, and Let's Encrypt configuration |
| 6 | [Docker Deployment](./docker-deployment.md) | Full Docker Compose reference and customization |
| 7 | [Manual Deployment](./manual-deployment.md) | Building from source without Docker |
| 8 | [Backup and Restore](./backup-and-restore.md) | Database dumps, file backups, and recovery procedures |
| 9 | [Updating](./updating.md) | Pulling updates, running migrations, and rollback |
| 10 | [Monitoring](./monitoring.md) | Health checks, logs, and resource monitoring |
| 11 | [Security Hardening](./security-hardening.md) | Production security checklist and best practices |
| 12 | [Troubleshooting](./troubleshooting.md) | Common issues and how to resolve them |
| 13 | [Cloudflare Tunnel](./cloudflare-tunnel.md) | Zero-config public access with Cloudflare Tunnel |

## Deployment Paths

There are several ways to deploy Chatalot. Pick the one that fits your situation:

| Path | Best For | Difficulty |
|------|----------|------------|
| **Docker + Quick Start** | Most users, fastest setup | Easy |
| **Docker + Cloudflare Tunnel** | Public-facing instance without port forwarding | Easy |
| **Docker + Reverse Proxy** | Production with custom domain and TLS | Moderate |
| **Manual Build** | Developers, custom environments, non-Docker hosts | Advanced |

## Quick Reference

```bash
# Interactive setup (recommended for first-time users)
./scripts/install.sh

# Or manual setup
./scripts/generate-secrets.sh
docker compose up -d

# Check status
curl http://localhost:8080/api/health

# View logs
docker compose logs -f chatalot
```
