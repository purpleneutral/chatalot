# Voice Controls

> **Status: Complete**

A guide to all the audio controls available during a voice call in Chatalot.

## Call Controls Bar

When you are in a voice call, a control bar appears with the following buttons (from left to right):

| Button | Function |
|--------|----------|
| Microphone | Toggle mute/unmute |
| Shield (with badge) | Cycle noise suppression level |
| Camera | Toggle video on/off |
| Monitor | Toggle screen sharing |
| Phone (red) | Leave call |

> **Tip:** The noise suppression and screen sharing buttons are hidden on small screens to save space. Access these features from **Settings > Voice** on mobile.

## Mute / Unmute Microphone

Click the **microphone** button to toggle your microphone on or off.

- When muted, your microphone icon shows a red slash and your audio track is disabled (no audio data is sent to other participants).
- Other participants see "(muted)" next to your name in the voice grid.
- Muting is instant and does not require renegotiating the peer connection.

## Per-User Volume Control

You can adjust the playback volume of any other participant independently:

1. **Right-click** on a participant's tile in the voice grid.
2. A volume slider appears with a range of **0% to 500%**.
3. Drag the slider to adjust their volume.
4. Click **Reset to 100%** to restore the default.

The volume percentage is displayed next to the participant's name when it differs from 100%.

> **Tip:** Boosting above 100% amplifies quiet speakers. Use this if someone's microphone is too quiet even at full volume.

### Microphone Volume (Self)

Right-clicking your own tile opens the **Mic Volume** control:

- This adjusts your **input gain** (what others hear from you), from 0% to 200%.
- The default is 100% (no amplification).
- Changes are reflected in real time and saved to your preferences.

## Noise Suppression

Chatalot includes built-in noise suppression that runs entirely on your device using WebAssembly. There are four levels:

| Level | Technology | CPU Usage | Description |
|-------|-----------|-----------|-------------|
| **Off** | None | -- | No noise processing |
| **Noise Gate** | Volume threshold | Minimal | Silences audio below a set volume level. Good for eliminating constant low-level hum. |
| **Standard** | Speex DSP | Low | Traditional DSP-based noise reduction. Removes steady-state noise like fans and air conditioning. |
| **Maximum** | RNNoise ML | Moderate | Machine-learning-powered noise removal. Handles variable noise like keyboard typing and background chatter. |

### Changing Noise Suppression During a Call

- **From the call controls:** Click the shield icon to cycle through levels (Off, Noise Gate, Standard, Maximum). A badge on the icon shows the current level (G, S, or M).
- **From Settings:** Go to **Settings > Voice > Noise Suppression** and select a level. If you are in an active call, the change applies immediately without needing to rejoin.

> **Tip:** Start with "Standard" for most situations. Switch to "Maximum" if you are in a noisy environment, but note it uses more CPU.

## Voice Settings Page

The full set of audio options is available under **Settings > Voice**. This includes:

### Input Device

- **Microphone selection** -- Choose from detected input devices, or use "System Default."
- **Mic Test** -- Click "Test Microphone" to see a real-time level meter showing your input volume. The meter turns yellow when levels are very high.
- **Input Volume** -- Slider from 0% to 200%. Controls your microphone gain (what others hear).

### Output Device

- **Speaker selection** -- Choose from detected output devices (requires browser support for `setSinkId`). If your browser does not support output device selection, use your operating system's audio settings instead.
- **Output Volume** -- Slider from 0% to 200%.

### Advanced Settings

| Setting | Default | Description |
|---------|---------|-------------|
| Echo Cancellation | On | Removes echo from speakers feeding back into your microphone |
| Auto Gain Control | On | Automatically adjusts microphone sensitivity |

These settings take effect on your next call or when switching input devices mid-call.

### Stream Focus

| Setting | Default | Description |
|---------|---------|-------------|
| Auto-focus streams | Off | Automatically hides participant tiles when someone shares their screen, showing only the shared content |

## Speaking Indicator

Chatalot monitors audio levels in real time. When a participant is speaking, their tile in the voice grid shows a green glowing ring. This uses local audio analysis -- no speaking data is sent to the server.

## Keyboard Reference

There are currently no dedicated keyboard shortcuts for voice controls. All interactions use the on-screen buttons.

## Related Pages

- [Joining Voice](./joining-voice.md) -- how to join and leave calls
- [Video Calls](./video-calls.md) -- camera and screen sharing
- [Troubleshooting](./troubleshooting.md) -- fixing audio problems
