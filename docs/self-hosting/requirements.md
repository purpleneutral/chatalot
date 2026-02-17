# Requirements

> **Status: Complete**

Hardware and software prerequisites for running a Chatalot instance.

## Hardware Requirements

### Minimum

| Resource | Minimum | Notes |
|----------|---------|-------|
| **CPU** | 1 core | Any modern x86_64 or ARM64 processor |
| **RAM** | 1 GB | For running the pre-built image. Building from source requires 2 GB+ |
| **Disk** | 2 GB | Base install. Add more for file uploads and database growth |

### Recommended (25-100 users)

| Resource | Recommended | Notes |
|----------|-------------|-------|
| **CPU** | 2 cores | Handles concurrent WebSocket connections and voice signaling |
| **RAM** | 2 GB | Comfortable headroom for PostgreSQL and the Rust server |
| **Disk** | 20 GB SSD | Fast storage improves database query times and file upload throughput |

### Build Requirements

If you are building the Docker image from source (rather than using a pre-built image), the Rust compilation step needs approximately **2 GB of RAM**. On low-memory VPS instances, you may need to add swap space:

```bash
# Add 2 GB swap (if needed for builds)
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Make permanent
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

## Software Requirements

### Docker (Recommended)

This is the fastest path to a running instance.

| Software | Minimum Version | Install Guide |
|----------|----------------|---------------|
| **Docker Engine** | 24.0+ | [docs.docker.com/engine/install](https://docs.docker.com/engine/install/) |
| **Docker Compose** | v2.0+ | Included with Docker Engine on most platforms |
| **OpenSSL** | 1.1+ | Pre-installed on most Linux distributions |
| **Git** | 2.0+ | For cloning the repository and receiving updates |

Verify your installation:

```bash
docker --version          # Docker version 24.0+
docker compose version    # Docker Compose version v2.x
openssl version           # OpenSSL 1.1+
git --version             # git version 2.x
```

### Manual Build (Without Docker)

If you prefer to build and run without Docker, you need:

| Software | Version | Purpose |
|----------|---------|---------|
| **Rust** | 1.85+ (2024 edition) | Server compilation |
| **Node.js** | 22+ | Web client build |
| **npm** | 10+ | Web client dependencies |
| **PostgreSQL** | 15+ | Database |
| **OpenSSL** | 1.1+ | JWT key generation |
| **wasm-pack** | 0.13+ | WASM crypto module build |

## Infrastructure

### Domain Name

A domain name is **recommended but not required**. Without one, you can access Chatalot via IP address (`http://192.168.1.100:8080`) or use a Cloudflare Quick Tunnel for a temporary public URL.

A domain is required if you want:
- TLS certificates from Let's Encrypt
- A persistent public URL
- Proper WebRTC connectivity for voice/video calls

### TLS Certificate

TLS (HTTPS) is **strongly recommended** for any instance accessible beyond your local network.

- Required for secure WebSocket connections (`wss://`)
- Required for WebRTC voice/video calls to work reliably in browsers
- Free certificates are available via [Let's Encrypt](https://letsencrypt.org/)
- Cloudflare Tunnel provides free TLS automatically

> **Warning:** Without TLS, browsers will block microphone/camera access (required for voice calls) and WebSocket connections may be unreliable. For local testing, `localhost` is exempt from this restriction.

### Network

| Port | Protocol | Purpose |
|------|----------|---------|
| **8080** | TCP | HTTP API, WebSocket, and static files (configurable) |

If using a reverse proxy (nginx, Caddy, Traefik), only the proxy's ports (typically 80 and 443) need to be exposed publicly. The Chatalot port can remain internal.

## Supported Platforms

### Server

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux (Debian/Ubuntu) | x86_64 | Fully supported |
| Linux (Debian/Ubuntu) | ARM64 | Supported (pre-built images available) |
| Linux (Fedora/RHEL) | x86_64 | Supported |
| Linux (Arch) | x86_64 | Supported |
| macOS | ARM64 (Apple Silicon) | Development only |
| Windows | x86_64 | Not recommended for production |

### Client (Web Browser)

The Chatalot web client works in any modern browser:
- Chrome/Chromium 100+
- Firefox 100+
- Safari 16+
- Edge 100+

## Next Step

Ready to get started? Continue to [Quick Start](./quick-start.md).
