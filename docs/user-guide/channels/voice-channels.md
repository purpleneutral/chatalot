# Voice Channels

> **Status: Complete**

Voice channels provide real-time audio and video communication within a Group. They use a peer-to-peer WebRTC mesh architecture, meaning audio and video streams flow directly between participants without passing through the server.

## Creating a Voice Channel

1. Expand the Group in the sidebar.
2. Click the **+** button next to the Group name.
3. Enter a channel name and select **Voice** as the channel type.
4. Click **Create**.

> **Permission:** Only Group owners and admins can create voice channels.

Voice channels appear in the sidebar with a speaker icon instead of the `#` hash symbol used by text channels.

## Joining a Voice Channel

Click on a voice channel in the sidebar to join the call. When you join:

- Your microphone is enabled by default (you can mute immediately).
- The video grid appears in the main content area, showing all participants.
- Other participants in the channel see you join in real time.
- A **call controls** bar appears with buttons for microphone, camera, screen share, and leave.

### Participant Limit

Voice channels support up to **25 concurrent participants**. If a channel is full, you will see an error message and cannot join until someone leaves.

## Call Controls

The call controls bar provides the following options:

| Control | Description |
|---------|-------------|
| Microphone | Toggle your microphone on/off |
| Camera | Toggle your camera on/off |
| Screen Share | Share your screen with other participants |
| Leave | Leave the voice channel |

## Voice Channel Features

- **Participant list:** All connected users are shown in the video grid.
- **Voice activity indicator:** Visual feedback shows who is currently speaking.
- **Background image:** Admins can set a custom background image for the voice channel via [Channel Settings](./channel-settings.md).
- **Collapse chat:** During a voice call, you can collapse the text chat area to give the video grid more space.

## Kicking Users from Voice

Group owners and admins can kick participants from a voice channel:

1. Right-click on a participant in the video grid.
2. Select **Kick from Voice**.
3. Confirm the action.

The kicked user is immediately removed from the call.

> **Permission:** Only users with a higher role level than the target can kick. Owners can kick admins and members; admins can kick members.

## WebRTC Mesh Architecture

Chatalot uses a **mesh** topology for voice calls. Each participant establishes a direct peer-to-peer connection with every other participant. This means:

- Audio and video data does not pass through the Chatalot server.
- Latency is typically lower than server-relayed architectures.
- Bandwidth usage increases with more participants (each participant sends their stream to every other participant).

The server is only involved in signaling (exchanging connection offers, answers, and ICE candidates between peers) and tracking who is in the call.

## Persistence Across Navigation

Voice calls remain active when you navigate to other pages (such as Settings or Admin). When you return to the channels page, the call resumes automatically.

## Related Pages

- [Voice and Video](../voice-and-video/README.md) -- Detailed guide on audio/video features
- [Text Channels](./text-channels.md)
- [Channel Settings](./channel-settings.md)
