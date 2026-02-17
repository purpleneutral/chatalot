# Voice and Video Troubleshooting

> **Status: Complete**

Solutions for common voice, video, and screen sharing issues in Chatalot.

## Browser Microphone and Camera Permissions

### "Microphone access denied" Error

Your browser blocked Chatalot from accessing your microphone. To fix this:

1. Click the **lock/site settings** icon in your browser's address bar.
2. Find "Microphone" and change it to **Allow**.
3. Reload the page and try joining the call again.

### "No microphone found" Error

Your browser cannot detect any audio input device. Check that:

- A microphone is physically connected to your computer.
- The microphone is not disabled in your operating system's sound settings.
- No other application has exclusive control of the microphone.

### Camera Not Working

If toggling the camera button does nothing or shows an error:

1. Check browser permissions (same steps as microphone above, but for "Camera").
2. Verify the camera is not in use by another application.
3. Try closing other tabs or applications that might be using the camera.

## No Sound from Other Participants

If you joined a call but cannot hear anyone:

1. **Check output device** -- Go to **Settings > Voice > Output Device** and verify the correct speaker is selected.
2. **Check output volume** -- Make sure the output volume slider is not at 0%.
3. **Check per-user volume** -- Right-click on a participant's tile and verify their volume is not set to 0%.
4. **Check browser tab** -- Some browsers mute audio on background tabs. Click on the Chatalot tab to bring it to the foreground.
5. **Check system volume** -- Verify your operating system's volume is not muted.

## Others Cannot Hear You

If other participants report that they cannot hear you:

1. **Check mute status** -- Verify you are not muted (the microphone icon should not have a red slash).
2. **Check input device** -- Go to **Settings > Voice** and use the **Test Microphone** button. You should see the level meter move when you speak.
3. **Check input volume** -- Make sure the input volume slider is not at 0%.
4. **Check noise suppression** -- If you are very quiet, noise suppression might be filtering your voice. Try setting it to "Off" temporarily to test.

## Echo and Feedback Issues

If you or others hear echo during a call:

1. **Use headphones** -- This is the most effective solution. Echo occurs when your speakers' audio feeds back into your microphone.
2. **Enable Echo Cancellation** -- Go to **Settings > Voice > Advanced** and ensure Echo Cancellation is turned on (it is enabled by default).
3. **Lower speaker volume** -- Reducing your output volume decreases the amount of audio that can feed back into the microphone.
4. **Increase distance** -- Move your microphone further from your speakers.

## Firewall and NAT Issues with WebRTC

Chatalot uses WebRTC for peer-to-peer connections. If you can join a call but cannot hear or see other participants, your network may be blocking the peer connection.

### How WebRTC Connections Work

1. **STUN** -- Chatalot uses Google's public STUN servers (`stun.l.google.com:19302`) to discover your public IP address and NAT type.
2. **Direct connection** -- If both peers can reach each other directly, a peer-to-peer connection is established.
3. **NAT traversal** -- STUN helps peers behind different NATs find paths to each other.

### Common Network Issues

| Problem | Likely Cause | Solution |
|---------|-------------|----------|
| Can hear some people but not others | Symmetric NAT or strict firewall on one side | The blocked participant should try a different network |
| Cannot connect to anyone | Corporate firewall blocking UDP | Try from a different network, or ask your network admin to allow UDP traffic |
| Connection drops after a few seconds | Firewall closing idle UDP connections | This is uncommon; check for aggressive firewall timeout settings |

> **Note:** Chatalot does not currently include a TURN server, which means connections between two peers both behind symmetric NATs may fail. Self-hosted deployments can add a TURN server by modifying the ICE server configuration.

### Checking Connection State

If a peer connection enters the "disconnected" state:

- Chatalot waits **10 seconds** for the connection to recover.
- If it does not recover, the peer is cleaned up automatically.
- If a connection enters the "failed" state, cleanup happens immediately.

## Screen Sharing Issues

### "No audio" Indicator

If your screen share shows "No audio":

- Share a **browser tab** instead of an entire screen or window for the best audio capture support.
- On Linux, ensure PipeWire or PulseAudio is running -- Chatalot looks for audio monitor devices.
- Some browsers do not support system audio capture for window or screen sharing.

### Screen Share Not Appearing for Others

If you started sharing but others do not see it:

1. Verify the screen share icon in your call controls is highlighted (active).
2. Check your browser's sharing indicator -- it should show that sharing is active.
3. Try stopping and restarting the share.
4. If the issue persists, leave the call and rejoin, then start sharing again.

## Supported Browsers

Chatalot's voice and video features require a modern browser with WebRTC support:

| Browser | Support Level |
|---------|--------------|
| **Chrome / Chromium** (90+) | Full support, including system audio capture for screen sharing |
| **Firefox** (85+) | Full support; system audio capture for screen sharing may be limited |
| **Safari** (15+) | Basic support; screen sharing audio and output device selection may not work |
| **Edge** (90+) | Full support (Chromium-based) |
| **Brave** | Full support (Chromium-based); ensure WebRTC is not blocked in Brave's shields |

### Feature Availability by Browser

| Feature | Chrome | Firefox | Safari |
|---------|--------|---------|--------|
| Voice calls | Yes | Yes | Yes |
| Video calls | Yes | Yes | Yes |
| Screen sharing | Yes | Yes | Yes |
| Screen share audio | Yes | Limited | No |
| Output device selection | Yes | Yes | No |
| Noise suppression (WASM) | Yes | Yes | Yes |

## Performance Issues

### High CPU Usage

- **Lower noise suppression** -- "Maximum" (RNNoise) uses more CPU than "Standard" (Speex) or "Noise Gate."
- **Turn off video** -- Camera encoding is CPU-intensive, especially with many participants.
- **Close unnecessary tabs** -- Each tab competes for CPU resources.

### High Bandwidth Usage

With full-mesh WebRTC, bandwidth scales with the number of participants. Each participant sends their stream to every other participant and receives a stream from each.

For a call with N participants, each person sends N-1 streams and receives N-1 streams. In a 10-person call with video, this can use significant bandwidth. Chatalot mitigates this by automatically reducing video resolution and frame rate as participants increase (see [Video Calls](./video-calls.md#performance-considerations)).

## Related Pages

- [Joining Voice](./joining-voice.md) -- how to join and leave calls
- [Voice Controls](./voice-controls.md) -- mute, volume, and noise suppression
- [Video Calls](./video-calls.md) -- camera and screen sharing
