# Security

Chatalot takes security seriously as a self-hosted, end-to-end encrypted chat platform. This section covers the security features available to every user.

## Sections

| Page | Description |
|------|-------------|
| [Account Security](./account-security.md) | Passwords, 2FA, sessions, recovery codes, and account deletion |
| [Encryption Status](./encryption-status.md) | What is encrypted, what is not, and the current E2E status |

## Quick Overview

- **Passwords** are hashed with Argon2id and enforced with strict complexity requirements.
- **Two-factor authentication (2FA)** is available using any TOTP-compatible authenticator app.
- **Active sessions** can be viewed and individually revoked from the Security tab.
- **Recovery codes** allow password reset without admin intervention.
- **End-to-end encryption** uses the Signal Protocol (X3DH + Double Ratchet for DMs, Sender Keys for groups).

All security settings are accessible from **Settings > Security** and **Settings > Account**.
