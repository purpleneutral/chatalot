# Video Calls

> **Status: Complete**

How to use camera video and screen sharing during voice calls in Chatalot.

## Enabling Your Camera

There are two ways to start video:

1. **Join with video** -- Click the **Video** button when joining a voice channel. This acquires your camera alongside your microphone from the start.
2. **Toggle mid-call** -- Click the **camera** button in the call controls to turn your camera on or off at any time.

When you toggle video mid-call:

- **Turning on:** Your browser requests camera access (if not already granted), adds the video track to all peer connections, and renegotiates with each participant.
- **Turning off:** The video track is removed from all peer connections, stopped, and renegotiation occurs. Other participants see your avatar instead.

## Screen Sharing

To share your screen:

1. Click the **monitor** button in the call controls (hidden on small screens).
2. Your browser shows a system picker where you can choose to share:
   - Your entire screen
   - A specific application window
   - A browser tab
3. Once selected, the screen share stream is added to all peer connections.

To stop sharing:

- Click the **monitor** button again, or
- Click the **Stop sharing** button in your browser's native sharing indicator.

### Screen Share Audio

Chatalot attempts to capture system audio alongside your screen share:

- **Browser tab sharing** -- Audio is typically captured automatically.
- **Application window / entire screen** -- Audio capture depends on your operating system and browser:
  - **Linux (PipeWire/PulseAudio):** Chatalot looks for audio monitor devices and captures system output through them.
  - **Chrome 105+:** The `systemAudio: 'include'` option is used when available.
  - If no audio source is available, a "No audio" indicator appears on the screen share.

> **Tip:** For the most reliable audio sharing, share a **browser tab** rather than an entire screen or window.

### Screen Share Volume Control

Other participants can control the volume of your screen share independently:

1. **Right-click** on the screen share area.
2. A panel appears with a volume slider (0% to 100%) and a mute toggle.
3. An indicator shows the current volume or "Audio muted" status.

### Focus Mode

When someone is sharing their screen, the video layout switches to show the shared content prominently. You can toggle between two viewing modes:

- **Tiled mode** (default) -- The screen share occupies the main area, with participant tiles stacked on the right side.
- **Focus mode** -- The screen share fills the full width, with participant avatars shown in a compact strip below. Click the **expand** icon (top right of the video area) to enter focus mode, or the **grid** icon to return to tiled mode.

> **Tip:** Enable **Settings > Voice > Auto-focus streams** to automatically enter focus mode whenever someone starts sharing their screen.

## Video Layout

The video grid adapts based on the number of participants and whether a screen share is active.

### Standard Grid (No Screen Share)

Participants are displayed in a responsive grid:

| Participants | Grid |
|-------------|------|
| 1 | 1 column |
| 2--4 | 1--2 columns |
| 5--9 | 2--3 columns |
| 10--16 | 3--4 columns |
| 17--25 | 4--5 columns |

Each tile shows the participant's avatar (when video is off) or camera feed (when video is on), with their display name overlaid in the corner. A green ring around the tile indicates the participant is currently speaking.

### With Screen Share

When a screen share is active, the layout switches to a master-detail arrangement:

- **Master pane (left):** The screen share(s) fill the available space.
- **Detail pane (right):** Participant tiles are stacked vertically at 176px wide.

Multiple simultaneous screen shares are stacked vertically in the master pane.

## Voice Background

Chatalot supports customizable backgrounds for your voice tile (visible when your camera is off):

- **Settings > Voice > Voice Background** lets you choose from:
  - **None** -- Default dark tile
  - **Solid color** -- A single color
  - **Gradient** -- Two colors with a configurable angle
  - **Preset** -- Six built-in presets: Fireplace, Aurora, Rain, Sunset, Space, Cozy
  - **Custom image** -- Upload your own background image (max 10 MB)

Custom image backgrounds are stored on the server and visible to other call participants. When you upload a custom image, other users in the call see it as your tile background. Presets, solid colors, and gradients are applied locally and are only visible on your own screen.

## Performance Considerations

Chatalot automatically adjusts video quality based on the number of participants to balance quality and bandwidth:

| Participants | Resolution | Frame Rate |
|-------------|-----------|------------|
| 1--4 | 640x480 | 30 fps |
| 5--8 | 480x360 | 24 fps |
| 9--15 | 320x240 | 20 fps |
| 16--25 | 240x180 | 15 fps |

These adjustments happen automatically when participants join or leave. Audio quality is not affected by the number of participants.

> **Tip:** If you experience performance issues in large calls, consider turning off your camera. Audio-only participation uses significantly less CPU and bandwidth.

## Related Pages

- [Joining Voice](./joining-voice.md) -- how to join and leave calls
- [Voice Controls](./voice-controls.md) -- mute, volume, and noise suppression
- [Troubleshooting](./troubleshooting.md) -- fixing video and screen sharing problems
