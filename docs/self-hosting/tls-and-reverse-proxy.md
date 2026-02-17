# TLS and Reverse Proxy

> **Status: Complete**

How to set up HTTPS and reverse proxy for a production Chatalot instance.

## Why TLS Matters

TLS (HTTPS) is essential for any Chatalot instance accessible beyond localhost:

- **WebSocket security.** Browsers require `wss://` (secure WebSocket) on HTTPS pages. Without TLS, real-time messaging will fail.
- **WebRTC voice/video.** Browsers block microphone and camera access on insecure origins. Voice and video calls will not work without TLS.
- **End-to-end encryption integrity.** While messages are E2E encrypted, TLS protects authentication tokens, metadata, and the key exchange from network-level attacks.
- **Browser warnings.** Modern browsers display prominent security warnings on HTTP sites.

> **Note:** `localhost` and `127.0.0.1` are exempt from these restrictions and work fine without TLS during development.

## Architecture

In a typical production setup:

```
Client --> Reverse Proxy (TLS on 443) --> Chatalot Server (HTTP on 8080)
```

The reverse proxy handles TLS termination and forwards requests to the Chatalot container. Both HTTP API requests and WebSocket connections at `/ws` must be proxied.

## Nginx

### Installation

```bash
# Debian/Ubuntu
sudo apt install nginx

# Fedora/RHEL
sudo dnf install nginx

# Arch Linux
sudo pacman -S nginx
```

### Configuration

Create `/etc/nginx/sites-available/chatalot`:

```nginx
server {
    listen 80;
    server_name chat.example.com;

    # Redirect HTTP to HTTPS
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name chat.example.com;

    ssl_certificate     /etc/letsencrypt/live/chat.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/chat.example.com/privkey.pem;

    # TLS settings
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Max upload size (match MAX_FILE_SIZE_MB + overhead)
    client_max_body_size 110M;

    # API and static files
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket endpoint
    location /ws {
        proxy_pass http://127.0.0.1:8080/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

        # Keep WebSocket connections alive
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }
}
```

Enable the site:

```bash
sudo ln -s /etc/nginx/sites-available/chatalot /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

> **Important:** The `proxy_read_timeout 86400s` on the `/ws` location is critical. Without it, nginx will close idle WebSocket connections after 60 seconds, causing constant reconnections.

## Caddy

Caddy is an excellent choice because it handles TLS certificates automatically via Let's Encrypt.

### Installation

```bash
# Debian/Ubuntu
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

### Configuration

Edit `/etc/caddy/Caddyfile`:

```
chat.example.com {
    reverse_proxy localhost:8080

    # Request body limit for file uploads
    request_body {
        max_size 110MB
    }
}
```

Caddy automatically:
- Obtains and renews Let's Encrypt certificates
- Redirects HTTP to HTTPS
- Handles WebSocket upgrades (no special configuration needed)

Start Caddy:

```bash
sudo systemctl enable --now caddy
```

That is all. Caddy's WebSocket handling works out of the box with no additional configuration.

## Traefik

If you are using Traefik as a Docker-based reverse proxy, create `docker-compose.override.yml`:

```yaml
services:
  chatalot:
    labels:
      - "traefik.enable=true"
      - "traefik.docker.network=web"
      - "traefik.http.routers.chatalot.rule=Host(`chat.example.com`)"
      - "traefik.http.routers.chatalot.entrypoints=websecure"
      - "traefik.http.routers.chatalot.tls=true"
      - "traefik.http.services.chatalot.loadbalancer.server.port=8080"
    networks:
      - web

networks:
  web:
    external: true
```

The deploy script (`scripts/deploy.sh`) can generate this automatically if you set `DEPLOY_DOMAIN`:

```bash
export DEPLOY_DOMAIN="chat.example.com"
export DEPLOY_NETWORK="web"
```

## Let's Encrypt

### With Certbot (for nginx)

```bash
# Install certbot
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d chat.example.com

# Auto-renewal is configured automatically
# Verify with:
sudo certbot renew --dry-run
```

### With Caddy

Caddy handles Let's Encrypt automatically. No additional setup is needed.

### With acme.sh

```bash
# Install acme.sh
curl https://get.acme.sh | sh

# Issue certificate
acme.sh --issue -d chat.example.com --webroot /var/www/html

# Install certificate
acme.sh --install-cert -d chat.example.com \
    --key-file /etc/ssl/private/chatalot.key \
    --fullchain-file /etc/ssl/certs/chatalot.pem \
    --reloadcmd "systemctl reload nginx"
```

## Self-Signed Certificates (Testing Only)

For local testing or development, you can generate a self-signed certificate:

```bash
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout /etc/ssl/private/chatalot-selfsigned.key \
    -out /etc/ssl/certs/chatalot-selfsigned.crt \
    -subj "/CN=chat.local"
```

> **Warning:** Self-signed certificates will trigger browser security warnings. They should never be used in production. For testing, consider using [mkcert](https://github.com/FiloSottile/mkcert) which installs a local CA that browsers trust.

## Verifying Your Setup

After configuring TLS, verify everything works:

```bash
# Check TLS certificate
curl -I https://chat.example.com

# Check health endpoint
curl https://chat.example.com/api/health

# Test WebSocket (requires wscat: npm install -g wscat)
wscat -c wss://chat.example.com/ws
```

## Next Step

For the full Docker Compose reference, see [Docker Deployment](./docker-deployment.md).
