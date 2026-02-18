# Frequently Asked Questions

Common questions about Chatalot and their answers.

---

## General

### Is Chatalot free?

Yes. Chatalot is free and open source software licensed under GPL-3.0. There are no paid tiers, premium features, or subscriptions. You host it yourself on your own hardware.

### How is Chatalot different from other chat platforms?

Chatalot is **self-hosted** -- you run the server on your own infrastructure. Your data never leaves your control. Key differences:

- **No data harvesting** -- your messages, files, and metadata stay on your server
- **No tracking or analytics** -- no telemetry is sent anywhere
- **End-to-end encryption** -- DMs use the Signal protocol; group encryption is in progress
- **No algorithmic feeds** -- conversations are shown chronologically, nothing more
- **No feature gates** -- every feature is available to every user
- **You own the server** -- you set the rules, control access, and manage the data

### Can I migrate from another chat platform?

Not yet. An import tool is planned but not currently available. You would need to manually recreate your server structure and invite your members.

### Is Chatalot federated?

No. Chatalot follows a single-instance model: one server, one community. This is intentional -- federation adds complexity and makes end-to-end encryption significantly harder to implement correctly. Each Chatalot instance is fully independent.

### What license is Chatalot under?

GPL-3.0-only. You are free to use, modify, and distribute it, provided derivative works are also released under GPL-3.0.

## Privacy and Security

### Is voice/video encrypted?

Voice and video calls use WebRTC, which encrypts all media in transit using SRTP (Secure Real-time Transport Protocol). This means your calls are encrypted between your browser and each peer. However, this is **not** end-to-end encrypted in the same way as text messages -- the encryption is at the transport layer, not the application layer. In the current mesh architecture (peer-to-peer), there is no server in the media path, so the server never sees your call data.

### Are my messages end-to-end encrypted?

Direct messages use the Signal protocol (X3DH key agreement + Double Ratchet) compiled to WebAssembly and running client-side. This means the server cannot read your DM content.

Group encryption uses the Sender Keys protocol and is implemented but currently in beta -- the WASM integration for groups is still being finalized.

Text channels within communities are **not** end-to-end encrypted. Messages are encrypted in transit (TLS) and at rest if your server's disk encryption is enabled, but the server can read them.

### Does the server admin see my messages?

For regular channel messages, yes -- the server stores them in plaintext in the database. For E2E-encrypted DMs, no -- the server only stores ciphertext that it cannot decrypt.

This is why you should only use Chatalot instances run by people you trust, or run your own.

### What data does Chatalot collect?

Chatalot does not phone home or send telemetry anywhere. The only data stored is on your server:

- Account information (username, email, hashed password)
- Messages and files you send
- Session metadata (IP address, user agent) for the audit log
- Encryption keys (identity keys are stored server-side; private keys stay on-device)

## Usage

### What browsers are supported?

Chatalot supports modern versions of:

- **Google Chrome** (and Chromium-based browsers)
- **Mozilla Firefox**
- **Apple Safari**
- **Microsoft Edge**

The web client requires JavaScript and WebSocket support. For voice/video calls, WebRTC support is required.

### Can I use Chatalot on mobile?

Yes, through PWA (Progressive Web App) support. Open your Chatalot instance in a mobile browser and use the "Add to Home Screen" or "Install App" option. The interface is fully responsive and optimized for touch screens.

A dedicated native mobile app is planned for the future.

### Is there a mobile app?

Not yet. The PWA provides a near-native experience on mobile devices, including push notifications (where supported) and offline indicators. A native mobile app is on the roadmap.

### Is there a desktop app?

Yes. Chatalot has a desktop app built with Tauri 2.0, available for Linux (AppImage, .deb) and Windows (NSIS installer). See the [Desktop App](../desktop-app/README.md) section for details.

### How many users can Chatalot handle?

It depends on your instance's hardware resources. The Rust server is efficient -- a modest VPS (2 CPU cores, 2 GB RAM) can comfortably handle hundreds of concurrent users for text chat. Voice/video calls are peer-to-peer (WebRTC mesh), so they scale with participants' bandwidth rather than server resources, though the mesh topology works best with up to 25 participants per call.

For larger deployments, increase PostgreSQL connection pool size and consider placing the server behind a reverse proxy with connection limits.

### Can I run multiple communities on one instance?

Yes. A single Chatalot instance supports multiple communities. Each community has its own groups, channels, roles, and settings. Users can be members of multiple communities on the same instance.

### How do I invite people to my server?

Generate invite codes from the admin panel or community settings. Invite codes can have optional usage limits and expiration times (1 hour to 1 year). Share the invite link, and recipients can join by clicking it or pasting the code in the join dialog.

## Self-Hosting

### What do I need to run Chatalot?

- A Linux server (x86_64 or ARM64)
- Docker and Docker Compose
- A domain name (optional but recommended for HTTPS)

The minimum hardware is 1 CPU core and 1 GB RAM, though 2 cores and 2 GB is recommended. See the [Self-Hosting](../self-hosting/README.md) section for full details.

### How do I update my instance?

Pull the latest Docker images and restart:

```bash
docker compose pull
docker compose up -d
```

Database migrations run automatically on startup. See [Updating](../self-hosting/updating.md) for more details.

### Can I use Chatalot without Docker?

Yes. You can build the Rust server from source and run it directly. You will need PostgreSQL installed separately. See [Manual Deployment](../self-hosting/manual-deployment.md) for instructions.

### Is HTTPS required?

HTTPS is strongly recommended but not strictly required for local/development use. For production, HTTPS is essential for security -- especially for WebRTC (which requires secure contexts in browsers) and to protect authentication tokens in transit. Use a reverse proxy like Caddy or nginx with Let's Encrypt certificates. See [TLS and Reverse Proxy](../self-hosting/tls-and-reverse-proxy.md).

---

[Back to Appendix](README.md) | [Back to Documentation](../README.md)
