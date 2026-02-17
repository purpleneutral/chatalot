# Joining Voice

> **Status: Complete**

How to join, monitor, and leave voice channels in Chatalot.

## Joining a Voice Channel

There are two ways to join a voice call:

1. **Voice only** -- Click the **Join Voice** button next to a voice channel. This acquires your microphone and connects you to the call with audio only.
2. **Voice with video** -- Click the **Video** button next to a voice channel. This acquires both your microphone and camera before connecting.

When you join, your browser will prompt you for microphone (and optionally camera) permission if it has not already been granted.

> **Tip:** If you have a preferred microphone configured in **Settings > Voice**, Chatalot will use that device. If the saved device is unavailable, it falls back to the system default automatically.

## Voice Channel Indicator

Even when you are not in a call, you can see who is currently connected:

- A green pulsing dot appears next to voice channels that have active participants.
- The number of participants is displayed (e.g., "3 in call").
- The participant list updates in real time as users join and leave.

## Joining an Active Call

If other people are already in a voice channel, you will see the participant count and the **Join Voice** / **Video** buttons. Clicking either button adds you to the existing call. WebRTC peer connections are established automatically with all current participants.

## Leaving a Voice Call

To leave a call, click the red **Leave call** button (phone icon with a slash) in the call controls bar. This:

1. Stops your local audio and video streams.
2. Notifies the server, which broadcasts your departure to other participants.
3. Closes all peer connections.

## Auto-Disconnect Behavior

Chatalot automatically handles disconnections in several scenarios:

- **Browser tab closed or navigated away** -- The WebSocket disconnects, and the server removes you from any active voice sessions. Other participants see you leave.
- **Network interruption** -- If a peer connection enters the "disconnected" state, Chatalot waits 10 seconds for recovery. If the connection does not recover, it is cleaned up automatically.
- **Connection failure** -- If a peer connection enters the "failed" state, it is cleaned up immediately.

> **Tip:** If you experience a temporary network glitch, the call may recover on its own within a few seconds. If it does not, leave and rejoin the channel.

## Rejoining After Disconnect

If your WebSocket reconnects (e.g., after a brief network outage), Chatalot automatically re-subscribes to channels and will show the current voice state. However, you will need to manually rejoin the voice call -- the server cleans up voice sessions on disconnect to prevent ghost participants.

## Participant Limit

Each voice channel supports up to **25 simultaneous participants**. If a channel is full, you will see an error message when attempting to join.

## Related Pages

- [Voice Controls](./voice-controls.md) -- mute, volume, and noise suppression
- [Video Calls](./video-calls.md) -- camera and screen sharing
- [Troubleshooting](./troubleshooting.md) -- fixing connection issues
