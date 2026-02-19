# Security Hardening

> **Status: Complete**

Production security checklist and best practices for self-hosted Chatalot instances.

## Security Checklist

Use this checklist before exposing your instance to the internet:

- [ ] TLS enabled (HTTPS) with a valid certificate
- [ ] JWT keys generated with strong randomness (`openssl genpkey -algorithm Ed25519`)
- [ ] JWT private key file permissions set to `600`
- [ ] `.env` file permissions set to `600`
- [ ] Database password is randomly generated (not a default)
- [ ] TOTP encryption key is set (required for 2FA)
- [ ] Database port (5432) is not exposed publicly
- [ ] Registration mode set to `invite_only` or `closed`
- [ ] Admin username configured
- [ ] Firewall rules in place
- [ ] Backups configured and tested
- [ ] Server and dependencies kept up to date

## JWT Key Management

Chatalot uses Ed25519 key pairs for signing JWT tokens. These keys are the root of your authentication system.

### Key Generation

```bash
# Using the provided script
./scripts/generate-keys.sh

# Or manually
openssl genpkey -algorithm Ed25519 -out secrets/jwt_private.pem
openssl pkey -in secrets/jwt_private.pem -pubout -out secrets/jwt_public.pem
```

### Key Security

```bash
# Set strict permissions
chmod 600 secrets/jwt_private.pem  # Owner read/write only
chmod 644 secrets/jwt_public.pem   # Owner read/write, others read

# Verify permissions
ls -la secrets/
# -rw-------  1 chatalot chatalot  119 Feb 16 12:00 jwt_private.pem
# -rw-r--r--  1 chatalot chatalot   93 Feb 16 12:00 jwt_public.pem
```

> **Warning:** If the private key is compromised, an attacker can forge authentication tokens for any user. Rotate keys immediately if you suspect a breach (see Key Rotation below).

### Key Rotation

To rotate JWT keys:

1. Generate new keys
2. Replace the old key files
3. Restart the server

All existing access tokens (15-minute lifetime) and refresh tokens (30-day lifetime) will be invalidated. Every user will need to log in again.

```bash
# Back up old keys
cp secrets/jwt_private.pem secrets/jwt_private.pem.bak
cp secrets/jwt_public.pem secrets/jwt_public.pem.bak

# Generate new keys
openssl genpkey -algorithm Ed25519 -out secrets/jwt_private.pem
openssl pkey -in secrets/jwt_private.pem -pubout -out secrets/jwt_public.pem
chmod 600 secrets/jwt_private.pem

# Restart
docker compose restart chatalot
```

### Docker Secrets

In the Docker setup, JWT keys are mounted as Docker secrets (read-only files at `/run/secrets/`), which is more secure than environment variables because:

- They are not visible in `docker inspect`
- They are not passed through the process environment
- They exist only as temporary files inside the container

## Database Security

### Password Strength

The `generate-secrets.sh` and `install.sh` scripts generate a 32-character random password. If you set the password manually, use at least 24 random characters:

```bash
openssl rand -base64 32 | tr -d '/+=' | head -c 32
```

### Network Isolation

By default, PostgreSQL is **not exposed** to the host network. It is only accessible within the Docker bridge network (`chatalot-net`). Keep it this way.

If you need external access for administration, use SSH tunneling instead of exposing the port:

```bash
# SSH tunnel from your local machine
ssh -L 5432:localhost:5432 user@your-server

# Then connect locally
psql -h localhost -U chatalot
```

### Connection Security

For external PostgreSQL servers, enable SSL:

```bash
DATABASE_URL=postgres://chatalot:password@db.example.com:5432/chatalot?sslmode=require
```

## Firewall Rules

### UFW (Ubuntu/Debian)

```bash
# Allow SSH
sudo ufw allow 22/tcp

# Allow HTTPS (if using a reverse proxy)
sudo ufw allow 443/tcp

# Allow HTTP (for Let's Encrypt challenges, redirect to HTTPS)
sudo ufw allow 80/tcp

# If NOT using a reverse proxy, allow 8080 directly
# sudo ufw allow 8080/tcp

# Enable the firewall
sudo ufw enable
```

### iptables

```bash
# Allow established connections
sudo iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# Allow SSH
sudo iptables -A INPUT -p tcp --dport 22 -j ACCEPT

# Allow HTTPS
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Allow HTTP (for Let's Encrypt)
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT

# Drop everything else
sudo iptables -A INPUT -j DROP
```

### What NOT to Expose

| Port | Service | Should Be Public? |
|------|---------|-------------------|
| 8080 | Chatalot (direct) | No -- use a reverse proxy |
| 5432 | PostgreSQL | No -- never |

## Rate Limiting

Chatalot includes built-in rate limiting:

| Endpoint | Rate | Burst |
|----------|------|-------|
| General API | 20 req/sec per IP | 50 |
| Auth endpoints (login, register) | 5 req/sec per IP | 10 |

Additionally, the server implements:
- **Account lockout:** 10 failed login attempts triggers a 15-minute lockout
- **Per-IP tracking:** Rate limits use `cf-connecting-ip` or `x-forwarded-for` headers when behind a proxy

### Reverse Proxy Rate Limiting

For additional protection, add rate limiting at the reverse proxy level:

**Nginx:**

```nginx
# Define rate limit zones
limit_req_zone $binary_remote_addr zone=api:10m rate=20r/s;
limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/s;

server {
    # Apply to API
    location /api/ {
        limit_req zone=api burst=50 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }

    # Stricter limit on auth
    location /api/auth/ {
        limit_req zone=auth burst=10 nodelay;
        proxy_pass http://127.0.0.1:8080;
    }
}
```

## CORS Configuration

The server uses permissive CORS (`allow_origin: Any`) to support the Tauri desktop client. All protected endpoints require JWT authentication, which is the actual access control mechanism.

If your instance is web-only (no desktop client), you can restrict CORS by modifying `crates/chatalot-server/src/routes/mod.rs`.

## File Upload Restrictions

| Setting | Default | Description |
|---------|---------|-------------|
| `MAX_FILE_SIZE_MB` | 100 | Maximum single file upload size |
| `UPLOAD_QUOTA_MB` | 500 | Per-user total upload quota |

The server also enforces:
- A `DefaultBodyLimit` of 110 MB on all requests
- Community/group icon uploads capped at 10 MB
- Community/group banner uploads capped at 10 MB
- Maximum 50 custom emojis per community

Uploaded files are stored in a sharded directory structure under `FILE_STORAGE_PATH`.

## Security Headers

The server automatically sets security headers on all responses:

| Header | Value | Purpose |
|--------|-------|---------|
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | Forces HTTPS |
| `X-Content-Type-Options` | `nosniff` | Prevents MIME sniffing |
| `X-Frame-Options` | `DENY` | Prevents clickjacking |
| `X-XSS-Protection` | `1; mode=block` | XSS filter |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Controls referrer information |
| `Permissions-Policy` | `camera=(self), microphone=(self), geolocation=()` | Restricts browser features |
| `Content-Security-Policy` | *(see below)* | Controls resource loading |

The CSP policy restricts:
- Scripts to `self`, inline, and WASM
- Connections to `self` and `wss:`
- Images to `self`, data/blob URIs, and Giphy CDN
- Frames to `none`
- Form actions to `self`

## TOTP Encryption Key

The `TOTP_ENCRYPTION_KEY` encrypts TOTP secrets at rest in the database. Without this key, 2FA setup will not work.

```bash
# Generate a 32-byte hex key
openssl rand -hex 32
```

> **Warning:** If you lose this key, all users with 2FA enabled will be locked out. Include it in your backup strategy.

## Keeping Dependencies Updated

### System Packages

```bash
# Debian/Ubuntu
sudo apt update && sudo apt upgrade

# Fedora/RHEL
sudo dnf update
```

### Docker Images

```bash
# Pull latest PostgreSQL
docker compose pull postgres

# Rebuild the Chatalot image
docker compose up -d --build
```

### Monitoring for Vulnerabilities

Check the Chatalot repository for security advisories and update promptly when patches are released.

## Container Security

The Dockerfile follows security best practices:
- Runs as a non-root user (`chatalot:chatalot`)
- Uses a minimal base image (`debian:bookworm-slim`)
- Only installs necessary runtime dependencies (`ca-certificates`, `curl`)
- JWT keys are mounted as read-only Docker secrets

## Next Step

For common issues and solutions, see [Troubleshooting](./troubleshooting.md).
