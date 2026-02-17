# Cloudflare Tunnel

> **Status: Complete**

Expose your Chatalot instance to the internet without port forwarding, using Cloudflare Tunnel.

## What Is Cloudflare Tunnel?

Cloudflare Tunnel (formerly Argo Tunnel) creates an encrypted outbound-only connection from your server to Cloudflare's edge network. Traffic flows like this:

```
User --> Cloudflare Edge (TLS) --> Tunnel --> Your Server (HTTP on 8080)
```

Your server makes an outbound connection to Cloudflare -- no inbound ports need to be opened on your firewall or router. This is especially useful for:

- **Home servers** behind NAT/CGNAT where port forwarding is impossible or undesirable
- **VPS instances** where you want to avoid exposing ports directly
- **Quick demos** where you need a temporary public URL in seconds

## Benefits

| Benefit | Description |
|---------|-------------|
| **No port forwarding** | Works behind NAT, CGNAT, and restrictive firewalls |
| **Free TLS** | Cloudflare handles TLS termination with a valid certificate |
| **DDoS protection** | Cloudflare's network absorbs volumetric attacks |
| **WebSocket support** | Full WebSocket proxy support (required for Chatalot's `/ws` endpoint) |
| **Free tier** | Cloudflare Tunnel is free for unlimited bandwidth |
| **No public IP needed** | Your server's IP address is never exposed |

## Limitations

| Limitation | Description |
|------------|-------------|
| **Cloudflare dependency** | Your instance is unreachable if Cloudflare has an outage |
| **Terms of Service** | Cloudflare's TOS prohibits using the free tier primarily for video streaming; chat and file sharing are fine |
| **Added latency** | Requests route through Cloudflare's edge, adding a few milliseconds |
| **Upload limits** | Cloudflare's free tier has a 100 MB upload limit per request (matches the default `MAX_FILE_SIZE_MB`) |
| **Account required** | Named tunnels require a free Cloudflare account and a domain on Cloudflare |

## Option 1: Quick Tunnel (No Account Needed)

A Quick Tunnel gives you a temporary public URL with zero configuration. The URL changes every time the tunnel restarts.

```bash
docker compose --profile quick-tunnel up -d
```

Find your public URL:

```bash
docker compose logs cloudflared-quick 2>&1 | grep trycloudflare.com
```

You will see something like:

```
https://random-words-here.trycloudflare.com
```

This URL is publicly accessible immediately. Share it with anyone you want to access your instance.

> **Note:** Quick Tunnel URLs change on every restart and cannot be customized. For a persistent URL, use a Named Tunnel.

### Stop the Quick Tunnel

```bash
docker compose --profile quick-tunnel down
```

## Option 2: Named Tunnel (Persistent Domain)

A Named Tunnel gives you a permanent subdomain (e.g., `chat.example.com`) that persists across restarts.

### Prerequisites

1. A free Cloudflare account
2. A domain added to Cloudflare (DNS managed by Cloudflare)

### Step 1: Create a Tunnel

1. Go to the [Cloudflare Zero Trust dashboard](https://one.dash.cloudflare.com/)
2. Navigate to **Networks** > **Tunnels**
3. Click **Create a tunnel**
4. Choose **Cloudflared** as the connector
5. Name your tunnel (e.g., "chatalot")
6. Copy the tunnel token (a long `eyJ...` string)

### Step 2: Configure the Public Hostname

In the tunnel configuration page:

1. Click **Public Hostname**
2. Set:
   - **Subdomain:** `chat` (or whatever you prefer)
   - **Domain:** `example.com` (your Cloudflare domain)
   - **Service Type:** `HTTP`
   - **URL:** `chatalot:8080`

> **Important:** The URL uses the Docker service name `chatalot` (not `localhost`) because the `cloudflared` container communicates with the Chatalot container over the Docker network.

### Step 3: Add the Token to Your Environment

Edit `.env`:

```bash
CLOUDFLARE_TUNNEL_TOKEN=eyJhIjoiYWJj...your_token_here
```

### Step 4: Start with the Production Profile

```bash
docker compose --profile production up -d
```

### Step 5: Verify

```bash
# Check tunnel is running
docker compose logs cloudflared

# Check your public URL
curl https://chat.example.com/api/health
```

### WebSocket Configuration

Cloudflare automatically proxies WebSocket connections. No additional configuration is needed in the tunnel settings. The Chatalot client connects to `wss://chat.example.com/ws` and Cloudflare handles the upgrade.

## Docker Compose Configuration

The `docker-compose.yml` includes two Cloudflare Tunnel services behind profiles:

```yaml
# Named Tunnel (requires token)
cloudflared:
  image: cloudflare/cloudflared:latest
  container_name: chatalot-tunnel
  restart: unless-stopped
  command: tunnel --no-autoupdate run
  environment:
    TUNNEL_TOKEN: ${CLOUDFLARE_TUNNEL_TOKEN}
  networks:
    - chatalot-net
  depends_on:
    - chatalot
  profiles:
    - production

# Quick Tunnel (no configuration needed)
cloudflared-quick:
  image: cloudflare/cloudflared:latest
  container_name: chatalot-tunnel-quick
  restart: unless-stopped
  command: tunnel --no-autoupdate --url http://chatalot:8080
  networks:
    - chatalot-net
  depends_on:
    - chatalot
  profiles:
    - quick-tunnel
```

Both services:
- Are on the same Docker network as the Chatalot server
- Only start when their profile is explicitly activated
- Connect to the Chatalot server using the Docker internal hostname `chatalot`

## Cloudflare Settings (Recommended)

In the Cloudflare dashboard for your domain, consider these settings:

### SSL/TLS

- **Mode:** Full (strict) -- Cloudflare to your tunnel is already encrypted
- **Always Use HTTPS:** On
- **Minimum TLS Version:** 1.2

### Speed

- **Brotli compression:** On (reduces bandwidth)

### Security

- **WAF:** Enable the free managed rules for basic protection
- **Bot Fight Mode:** On (reduces automated attacks)

### Network

- **WebSockets:** Enabled (should be enabled by default)

## Troubleshooting

### Tunnel Not Connecting

```bash
# Check tunnel logs
docker compose logs cloudflared

# Common issues:
# - Invalid token: regenerate in the Cloudflare dashboard
# - DNS not resolving: check your internet connection
# - Container not starting: check docker compose ps
```

### 502 Bad Gateway

**Cause:** Cloudflare can reach the tunnel, but the tunnel cannot reach the Chatalot server.

**Fix:**

```bash
# Verify the Chatalot server is running
docker compose ps chatalot

# Check the health endpoint from inside the network
docker exec chatalot-tunnel curl -f http://chatalot:8080/api/health
```

### WebSocket Disconnections

**Cause:** Cloudflare has a 100-second idle timeout for WebSocket connections.

The Chatalot server sends periodic keepalive frames which should prevent idle timeouts. If you still experience disconnections, check:

```bash
# Verify the tunnel is using the correct URL
docker compose logs cloudflared | grep "chatalot:8080"
```

### Slow File Uploads

**Cause:** Large uploads route through Cloudflare's network, which adds latency.

**Mitigation:**
- Cloudflare's free tier allows uploads up to 100 MB (matching the default `MAX_FILE_SIZE_MB`)
- For larger files, you would need a Cloudflare Pro plan (500 MB limit) or use a direct connection

## Removing the Tunnel

```bash
# Stop the tunnel container
docker compose --profile production down

# Remove the tunnel from Cloudflare
# Go to Zero Trust Dashboard > Networks > Tunnels > Delete
```

The DNS records created by the tunnel will be cleaned up automatically when the tunnel is deleted.
