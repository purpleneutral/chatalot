# Glossary

Definitions of terms used throughout the Chatalot documentation and application.

---

## A

### Accent Color
The primary highlight color used throughout the UI for buttons, links, active states, and indicators. Configurable per user (8 options) and per community (theme editor).

### Admin
An instance-level administrator who manages the Chatalot server via the admin panel. Admins can manage users, generate invite codes, review reports, and post announcements. Distinct from community-level admin roles.

### Announcement
A server-wide banner message created by an admin. Announcements appear as dismissible banners for all users and are delivered in real-time via WebSocket.

### AppImage
A portable Linux application format. The Chatalot desktop app is distributed as an AppImage that runs without installation.

### Argon2id
The password hashing algorithm used by Chatalot. A memory-hard function designed to resist GPU and ASIC attacks. Configured with 64 MiB memory, 3 iterations, and 4 lanes.

### Audit Log
A record of security-relevant events on the server, including login attempts, session management, and destructive administrative actions. Logs include IP address and user agent.

## B

### Ban
The permanent removal of a user from a community. Banned users cannot rejoin via invite codes. Admins and moderators can ban users.

### Backup Codes
Eight single-use codes generated when enabling TOTP two-factor authentication. Each code can be used once to log in if the authenticator app is unavailable.

### Bookmark
A saved message reference. Users can bookmark any message for later reference, optionally with a personal note. Bookmarks are browsable in the "Saved Items" panel.

## C

### Channel
A conversation space within a group. Channels can be text (messaging) or voice (WebRTC calls). Each channel has its own topic, permissions, and settings.

### ChaCha20-Poly1305
The authenticated encryption algorithm used for encrypting message content in E2E encrypted conversations. Provides both confidentiality and integrity.

### Community
The top-level organizational unit in Chatalot. A community contains groups, which contain channels. Each community has its own settings, roles, invite codes, and optional theme customization.

### CSP (Content Security Policy)
A security header that restricts which resources the browser can load. Chatalot sets CSP headers to prevent cross-site scripting (XSS) and other injection attacks.

### Custom Emoji
User-uploaded emoji specific to a community. Uploaded as PNG, GIF, or WebP (max 256 KB, 50 per community). Used with `:shortcode:` syntax in messages.

## D

### DM (Direct Message)
A private 1:1 conversation between two users. DMs use end-to-end encryption via the Signal protocol. DMs exist outside the community/group/channel hierarchy.

### Double Ratchet
The core messaging protocol used for DM encryption. After the initial key exchange (X3DH), the Double Ratchet provides forward secrecy and break-in recovery by deriving new encryption keys for every message.

## E

### E2E Encryption (End-to-End Encryption)
Encryption where only the communicating parties can read the messages. The server stores ciphertext it cannot decrypt. Chatalot implements E2E encryption for DMs using the Signal protocol.

## G

### Group
A sub-organization within a community. Groups contain channels and have their own membership, roles (owner, admin, member), and visibility settings. Groups can be public (anyone in the community can join) or private (invite required).

## H

### HKDF (HMAC-based Key Derivation Function)
A key derivation function used in the Signal protocol implementation to derive symmetric encryption keys from shared secrets established during X3DH.

## I

### Identity Key
A long-term Ed25519 key pair that uniquely identifies a user for E2E encryption purposes. The public identity key is uploaded to the server; the private key remains on-device.

### Instance
A single deployment of the Chatalot server. Each instance is independent -- there is no federation between instances. Users create accounts on specific instances and connect to them directly.

### Invite Code
An alphanumeric code (12 characters) used to join a community or group. Codes can have optional usage limits and expiration times. Shareable as links or plain codes.

## J

### JWT (JSON Web Token)
The authentication token format used by Chatalot. Access tokens are Ed25519-signed, expire after 15 minutes, and are paired with longer-lived refresh tokens for session continuity.

## K

### Kick
The removal of a user from a community or voice call. Unlike a ban, a kicked user can rejoin if they have a valid invite code.

### Keychain / Keyring
The OS-level secure credential store used by the desktop app. On Linux: GNOME Keyring or KWallet (via Secret Service API). On Windows: Credential Manager.

## M

### Mesh (WebRTC Mesh)
The network topology used for voice/video calls. Each participant connects directly to every other participant (peer-to-peer). No media passes through the server. Scales well up to approximately 25 participants.

### Moderator
A community-level role that can issue warnings, timeouts, kicks, and bans. Moderators can also create personal groups and manage group settings.

## N

### Noise Suppression
Audio processing to reduce background noise during voice calls. Chatalot offers three tiers: Noise Gate (basic), Speex (moderate), and RNNoise (AI-powered).

### NSIS
Nullsoft Scriptable Install System. The Windows installer format used for the Chatalot desktop app. Installs per-user without requiring administrator privileges.

## P

### Personal Group
A group assigned to a specific member by a moderator+. The assigned member has owner-level control within their group (create channels, edit settings) while moderators retain override access.

### Poll
An in-channel voting feature. Polls support 2-10 options, multi-select voting, anonymous voting, and optional expiry (15 minutes to 1 week). Votes are broadcast in real-time.

### Prekey (Pre-Key Bundle)
A set of public keys uploaded to the server that allows other users to initiate an E2E encrypted session without the recipient being online. Part of the X3DH key agreement protocol.

### PWA (Progressive Web App)
A web application that can be installed on a device and behaves like a native app. Chatalot's web client is a PWA, allowing it to be installed from the browser on mobile and desktop.

## R

### Recovery Code
A code provided at registration that allows self-service password reset without admin intervention. Separate from 2FA backup codes.

### Refresh Token
A long-lived token used to obtain new access tokens without re-entering credentials. Tokens are rotated on each use (the old token is invalidated when a new one is issued).

## S

### Sender Keys
The group encryption protocol used for E2E-encrypted group messages. Each participant maintains a sending key that encrypts messages for the group. More efficient than pairwise encryption for multi-party conversations.

### Signed Prekey
A medium-term key pair signed by the identity key, used as part of the X3DH key agreement. Rotated periodically.

### Signaling
The process of exchanging connection metadata (SDP offers/answers, ICE candidates) between WebRTC peers. Chatalot uses WebSocket messages for signaling.

### Slow Mode
A channel setting that rate-limits how often users can send messages. Configurable from 5 seconds to 10 minutes between messages. Admins and moderators are exempt.

### SRTP (Secure Real-time Transport Protocol)
The encryption protocol used by WebRTC for voice and video media. Encrypts audio/video in transit between peers.

## T

### Tauri
The framework used to build the Chatalot desktop app. Tauri wraps the web client in a native window using the system WebView, resulting in a lightweight application.

### Thread
A focused conversation branching from a specific message. Threads appear in a side panel and have their own message composer. Messages in threads do not appear in the main channel flow.

### Timeout
A temporary mute applied to a user. During a timeout, the user cannot send messages in the community. Timeouts have a configurable duration and expire automatically.

### TOTP (Time-based One-Time Password)
The two-factor authentication method supported by Chatalot. Compatible with any authenticator app (Google Authenticator, Authy, etc.). Follows RFC 6238.

## W

### Warning
A moderation action that formally notifies a user of a rule violation. Warnings are tracked and visible to moderators. Unlike timeouts, warnings do not restrict the user's ability to send messages.

### WebRTC (Web Real-Time Communication)
The browser API used for voice and video calls. Enables peer-to-peer audio, video, and screen sharing without plugins. Chatalot uses a mesh topology where each participant connects directly to every other participant.

### WebSocket
A persistent, full-duplex communication protocol used for real-time features: message delivery, typing indicators, presence updates, poll votes, and voice signaling. The client authenticates via the first WebSocket message (not headers).

### Webhook
An HTTP endpoint that allows external services to post messages into a channel. Each webhook has a unique token and posts as a named bot user. Managed by admins in channel settings.

## X

### X3DH (Extended Triple Diffie-Hellman)
The initial key agreement protocol used to establish an E2E encrypted session between two users. Uses identity keys, signed prekeys, and one-time prekeys to derive a shared secret, even if one party is offline.

---

[Back to Appendix](README.md) | [Back to Documentation](../README.md)
