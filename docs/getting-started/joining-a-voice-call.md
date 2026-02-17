# Joining a Voice Call

> **Status: Complete**

Chatalot supports voice and video calls through WebRTC. This page explains how to join, control, and leave a call.

## Finding a Voice Channel

Voice channels are listed alongside text channels in the navigation sidebar. They are marked with a speaker icon instead of the `#` symbol. In the navigation dropdown, they appear as part of their group (e.g., "General Lounge > voice").

If other users are already in a voice channel, you will see a green pulsing indicator and a count of participants (e.g., "3 in call").

## Joining a Call

1. Open the navigation sidebar and click on a voice channel.
2. Two buttons appear in the channel header:
   - **Join Voice** -- Joins the call with audio only (microphone).
   - **Video** -- Joins the call with both audio and video (microphone + camera).
3. Your browser will prompt you for microphone (and camera, if applicable) permission. Click **Allow**.

Once connected, the video grid appears in the main area showing the participants.

## Call Controls

When you are in a call, a control bar appears in the channel header with the following buttons:

| Button | Action |
|--------|--------|
| **Microphone** | Toggle your microphone on/off. The icon turns red with a strikethrough when muted. |
| **Noise Suppression** | Cycle through noise suppression modes: Off, Noise Gate (minimal CPU), Standard/Speex (low CPU), and Maximum/RNNoise (moderate CPU). A badge shows the active level. |
| **Camera** | Toggle your camera on/off. |
| **Screen Share** | Start or stop sharing your screen. |
| **Leave** (red phone icon) | Disconnect from the call. |

> **Tip:** Noise suppression is also available on desktop. You can configure it in advance from Settings > Voice, along with echo cancellation and auto gain control.

## User Menu Voice Controls

When you are in a call, your **user menu** (click your avatar in the top-right corner) also shows voice controls. This is especially useful for quick mute/unmute without scrolling to the channel header.

The user menu shows:
- The name of the voice channel you are connected to
- A miniature set of microphone, camera, screen share, and disconnect buttons

## Volume and Participant Controls

Right-click on a participant in the voice grid to access per-user controls:

- **Volume slider** -- Adjust the volume of that user's audio from 0% to 500%.
- **Screen share audio** -- Separate volume and mute controls for a user's screen share audio.

## Leaving a Call

Click the **red phone icon** (Leave) in the call controls, or use the disconnect button in the user menu. Your audio and video streams are stopped immediately.

If you close the browser tab or navigate away, you are automatically disconnected from the call.

## Troubleshooting

### Browser asks for permission every time

Make sure you click **Allow** (not "Block") when the browser prompts for microphone/camera access. In most browsers, you can also click the lock/site-info icon in the address bar to manage permissions for the site.

### No audio / others cannot hear me

1. Check that your microphone is not muted (the microphone icon should not have a red strikethrough).
2. Go to **Settings > Voice** and use the **Test Microphone** feature to verify your mic is working. The level meter should respond when you speak.
3. Make sure the correct input device is selected in Settings > Voice. If you have multiple microphones, the wrong one may be active.
4. Check your operating system's audio settings to ensure the microphone is not muted or disabled at the OS level.

### No video / camera not working

1. Make sure you clicked **Video** (not just "Join Voice") when joining the call, or toggle the camera on using the camera button after joining.
2. Check that no other application (e.g., Zoom, Teams) is using the camera.
3. Verify browser permissions for camera access (same as microphone -- check the lock icon in the address bar).

### Echo or feedback

Enable **Echo Cancellation** in Settings > Voice. This is on by default, but if it was disabled, re-enabling it should resolve the issue.

### Audio quality is poor

Try increasing the **Noise Suppression** level by clicking the shield icon in the call controls. The "Standard" (Speex) and "Maximum" (RNNoise) levels actively filter background noise at the cost of slightly more CPU usage.

## Voice Settings

For advanced configuration, go to **Settings > Voice**:

- **Input device** -- Select which microphone to use.
- **Output device** -- Select which speaker or headphones to use.
- **Echo cancellation** -- Toggle on/off.
- **Auto gain control** -- Automatically adjusts microphone sensitivity.
- **Noise suppression** -- Choose between Off, Noise Gate, Standard (Speex), or Maximum (RNNoise).
- **Test microphone** -- Verify your mic is working with a live level meter.
- **Voice call background** -- Customize the visual background of the voice call area (solid color, gradient, preset image, or custom upload).

## Next Steps

That covers the essentials of getting started with Chatalot. Here are some areas to explore next:

- **Theming** -- Customize your experience in Settings > Appearance (dark/light mode, color palettes, accent colors, font sizes, message density).
- **Security** -- Enable two-factor authentication (TOTP) in Settings > Security.
- **Notifications** -- Fine-tune desktop and push notification settings in Settings > Notifications.
- **Keyboard shortcuts** -- Learn the full set of shortcuts to navigate Chatalot faster.
