# Features Reference

Complete reference of Chatalot's features, organized by category.

## Messaging

### Core
- **Real-time messaging** — WebSocket-based delivery with optimistic UI updates
- **Markdown formatting** — Bold, italic, strikethrough, inline code, code blocks, links, lists, blockquotes
- **Message editing** — Edit your own messages; "(edited)" indicator with hover tooltip showing edit timestamp
- **Message deletion** — Delete your own messages or others' (admin/owner); styled confirmation dialog
- **Message forwarding** — Forward any message to another channel with attributed quote
- **Reply threads** — Reply to specific messages with inline quote preview
- **Mentions** — `@username`, `@everyone`, `@here`, `@channel` with autocomplete popup

### Rich Content
- **Syntax-highlighted code blocks** — 15+ languages (JS, TS, Python, Rust, Go, Java, C++, SQL, YAML, Bash, CSS, HTML, JSON, Markdown); auto-detection for unlabeled blocks
- **Code block copy button** — One-click copy with language label badge
- **Inline image preview** — Images embedded in messages render inline with lightbox viewer
- **Image lightbox** — Click any image to open a full-screen overlay with backdrop blur; close with Escape, click outside, or X button
- **Link previews** — URL embeds with title, description, site name, and thumbnail (toggleable per user)
- **Inline video player** — Video files play with native controls
- **Inline audio player** — Audio files (MP3, WAV, FLAC, AAC, OGG, M4A) with playback controls and download link
- **GIF search** — Built-in Tenor-powered picker with search and trending, auto-sends on selection
- **Emoji autocomplete** — Type `:shortcode` to search 250+ emoji with prefix-first ranking

### Polls
- **Channel polls** — Create polls with 2-10 options, multi-select, anonymous voting, and optional expiry (15 min to 1 week)
- **Real-time voting** — Votes broadcast instantly via WebSocket with optimistic UI updates and animated progress bars
- **Poll management** — Creators and admins can close polls; vote percentages and counts shown per option

### Webhooks
- **Webhook management** — Admins can create, toggle, and delete webhooks per channel from channel settings
- **Webhook execution** — External services POST to `/api/webhooks/execute/{token}` to send messages as the webhook
- **Copy webhook URL** — One-click copy of the full webhook endpoint URL

### Organization
- **Message grouping** — Consecutive messages from the same sender within 5 minutes collapse (hidden avatar/name, tighter spacing, hover timestamp in gutter)
- **Pinned messages** — Pin important messages (admin/owner only, max 50 per channel); slide-out panel to browse pins; real-time pin/unpin updates via WebSocket
- **Message search** — Full-text search within a channel with highlighted matching terms
- **Unread tracking** — Per-channel unread counts with badge display
- **Unread separator** — "NEW MESSAGES" divider when entering a channel with unread messages
- **Date separators** — Visual dividers between messages on different days
- **Smart auto-scroll** — Only auto-scrolls to new messages if already near bottom; doesn't interrupt reading history
- **Scroll-to-bottom button** — Appears when scrolled up, shows unread count badge

## Channels & Communities

- **Communities** — Top-level organization containing groups and channels with invite codes
- **Groups** — Sub-organization within communities for channel grouping
- **Personal groups** — Moderators+ can create a group assigned to a specific member, giving them their own isolated space with owner-level control (create channels, edit settings); moderators retain override access and can manage or remove the group
- **Text channels** — Standard messaging channels with topic support
- **Voice channels** — Peer-to-peer WebRTC calls (up to 5 participants) with video grid
- **Direct messages** — Private 1:1 conversations with E2E encryption
- **Channel topic editing** — Inline editing in the channel header
- **Channel roles** — Owner, admin, and member roles with graduated permissions
- **Invite system** — Generate invite codes with optional usage limits and expiration dates
- **Privacy controls** — Communities, groups, and channels each have `visibility` (public/private) and `discoverable` toggles; personal groups default to private and non-discoverable
- **Allow invites toggle** — Moderators can control whether a personal group's assigned member can invite others

## User Experience

### Profiles & Presence
- **User profile cards** — Click any username to see avatar, display name, username, custom status, role badge, community nickname, join date, and action buttons (Send Message, Copy ID)
- **Online/offline presence** — Real-time status indicators (online, idle, do not disturb, invisible)
- **Custom status** — Set a custom status message visible on your profile
- **Typing indicators** — Shows who is currently typing by display name
- **Avatars** — User-uploaded profile pictures with automatic serving and caching

### Member Management
- **Member sidebar** — Collapsible panel showing channel members grouped into Online and Offline sections with count headers
- **Member search** — Filter members by name in the sidebar
- **Role management** — Promote/demote members (owner/admin only)
- **Kick and ban** — Remove members from channels

### Navigation & Input
- **Keyboard shortcuts** — Press `?` to see all shortcuts; configurable send behavior (Enter vs Ctrl+Enter)
- **Formatting toolbar** — Quick-access buttons for bold, italic, code, and links (toggleable)
- **Slash commands** — `/shrug`, `/tableflip`, `/unflip`, `/lenny`
- **Context menu** — Right-click messages for reply, edit, delete, copy text, pin, forward
- **Paste image upload** — Ctrl+V to paste images directly as file messages
- **Drag-and-drop file upload** — Drop files onto the chat area

### Notifications
- **Desktop notifications** — Browser notification API with permission prompt on first message
- **Notification sounds** — Per-type sound effects with volume control
- **Per-channel notification levels** — All messages, only @mentions, or nothing
- **Notification permission prompt** — Polished slide-up prompt after first message, dismissable and persisted

### Reactions
- **Emoji reactions** — React to messages with emoji; quick picker and full emoji panel
- **Reaction tooltips** — Hover a reaction to see who reacted
- **Toggle reactions** — Click to add/remove your own reaction

## Customization

All preferences sync to the server and persist across devices.

| Setting | Options | Default |
|---------|---------|---------|
| **Theme** | Dark / Light | Dark |
| **Accent color** | Blue, Purple, Green, Orange, Red, Pink, Teal, Cyan | Purple |
| **Message density** | Cozy (with avatars) / Compact (tight spacing) | Cozy |
| **Time format** | 12-hour / 24-hour | 12-hour |
| **Font size** | Small (13px) / Medium (14px) / Large (16px) | Medium |
| **Send behavior** | Enter / Ctrl+Enter | Enter |
| **Link previews** | Show / Hide | Show |
| **Formatting toolbar** | Show / Hide | Show |

### Profile Customization
- **Profile banners** — Upload a banner image (max 5 MB) displayed on profile cards; supports PNG, JPEG, WebP, and GIF
- **GIF avatars** — Animated avatar support

### Community Theming
- **Community theme editor** — Admins can override 7 CSS color variables (accent, accent hover, 3 backgrounds, 2 text colors) per community
- **Custom CSS** — Server-side sanitized CSS injection for advanced community styling; allowlisted properties only, no XSS vectors
- **Community icon and banner** — Upload custom icon (2 MB) and banner (5 MB) images
- **Welcome message** — Configurable splash screen shown to members on their first visit; tracked via localStorage dismissal
- **Live theme preview** — Color pickers show a mock chat preview before saving

### Group Customization
- **Group icon and banner** — Upload custom icon and banner images per group
- **Accent color** — Per-group accent color displayed on group names in the sidebar

### Voice Call Backgrounds
- **Per-user backgrounds** — Choose a background for your video tile when camera is off: solid color, gradient, preset, or custom image upload
- **6 animated presets** — Fireplace, Aurora, Rain, Sunset, Space, and Cozy — CSS-only animations with no image assets
- **Per-channel ambiance** — Admins can set a shared background image for voice channel grids

- **Custom scrollbars** — Themed scrollbars matching the active theme (WebKit and Firefox)
- **PWA support** — Installable as a progressive web app

## File Sharing

- **Encrypted uploads** — Files are uploaded via the REST API with authentication
- **Drag-and-drop** — Drop files into the chat area
- **Clipboard paste** — Paste images from clipboard
- **File type detection** — Automatic rendering for images, video, and audio
- **Download links** — Direct download with original filename
- **Max upload size** — Configurable (default 100 MB)

## Security

- **End-to-end encryption** — Signal protocol (X3DH + Double Ratchet + ChaCha20-Poly1305) for DMs, compiled to WASM
- **Argon2id password hashing** — 64 MiB memory, 3 iterations, 4 lanes
- **Ed25519-signed JWTs** — 15-minute access tokens with refresh token rotation
- **TOTP two-factor authentication** — RFC 6238 with any authenticator app
- **Rate limiting** — Token-bucket per IP (20 req/s general, 5 req/s auth)
- **Security headers** — HSTS, CSP, X-Frame-Options, Permissions-Policy
- **SSRF protection** — Link preview proxy blocks private/internal IPs
- **Channel authorization** — All message, file, and typing operations are permission-checked
- **Audit logging** — Auth events logged with IP and user agent
- **Session management** — View and revoke active sessions; logout all devices

## Administration

- **Admin panel** — In-browser user management, invite code generation, system feedback
- **Invite-only registration** — Default mode; open and closed modes also available
- **User management** — View, promote, and manage all users
- **Server admin badge** — Distinguished in profile cards

## Technical Details

- **Server**: Rust (axum + tokio), single binary serving API + WebSocket + static files
- **Database**: PostgreSQL 17 with sqlx (34 migrations)
- **Web client**: Svelte 5 SPA with Tailwind CSS (15 stores, 18 API modules, 9 components)
- **Desktop client**: Tauri 2.0
- **Deployment**: Single `docker compose up -d` — two containers (app + PostgreSQL)
- **GIF proxy**: Server-side Tenor API proxy with DashMap caching (5-min TTL, max 200 entries)
