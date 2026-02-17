# Voice and Video

Everything you need to know about real-time voice calls, video, and screen sharing in Chatalot.

Chatalot uses peer-to-peer WebRTC connections for voice and video, meaning audio and video flow directly between participants without passing through the server. The server handles signaling (connecting peers) and tracks who is in each call, but the actual media streams are direct.

## In This Section

| Page | Description |
|------|-------------|
| [Joining Voice](./joining-voice.md) | How to join and leave voice channels |
| [Voice Controls](./voice-controls.md) | Mute, deafen, volume, and noise suppression |
| [Video Calls](./video-calls.md) | Camera, screen sharing, and video layout |
| [Troubleshooting](./troubleshooting.md) | Fixing common voice and video problems |

## How It Works

Chatalot uses a **full-mesh** WebRTC topology. Each participant establishes a direct peer connection with every other participant. This gives excellent latency and privacy for small to medium groups but means bandwidth usage scales with participant count.

- **Maximum participants:** 25 per voice channel
- **Signaling:** WebSocket messages relay SDP offers/answers and ICE candidates through the server
- **STUN servers:** Google's public STUN servers are used for NAT traversal
- **Audio processing:** Noise suppression runs entirely on your device using WebAssembly (no audio is sent to any third-party service)

## Related Sections

- [Channels](../channels/README.md) -- voice channels are a channel type within Groups
- [Notifications](../notifications/README.md) -- sound notifications for voice join/leave events
