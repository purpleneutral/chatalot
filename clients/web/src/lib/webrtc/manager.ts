import { voiceStore } from '$lib/stores/voice.svelte';
import { authStore } from '$lib/stores/auth.svelte';
import { preferencesStore, type NoiseSuppression } from '$lib/stores/preferences.svelte';
import { audioDeviceStore } from '$lib/stores/audioDevices.svelte';
import { wsClient } from '$lib/ws/connection';
import { getServerConfig } from '$lib/api/auth';
import { randomUUID } from '$lib/env';
import {
	applyNoiseSuppression,
	removeNoiseSuppression,
	changeSuppressionLevel
} from './noise-suppression';

const DEFAULT_ICE_SERVERS: RTCIceServer[] = [
	{ urls: 'stun:stun.l.google.com:19302' },
	{ urls: 'stun:stun1.l.google.com:19302' }
];

const SPEAKING_THRESHOLD = 15; // RMS level (0-255) above which user is "speaking"
const SPEAKING_CHECK_INTERVAL = 100; // ms between audio level checks

const VIDEO_QUALITY_TIERS = [
	{ maxParticipants: 4,  width: 640, height: 480, frameRate: 30 },
	{ maxParticipants: 8,  width: 480, height: 360, frameRate: 24 },
	{ maxParticipants: 15, width: 320, height: 240, frameRate: 20 },
	{ maxParticipants: 25, width: 240, height: 180, frameRate: 15 },
];
const MAX_PARTICIPANTS = 25;

/// Manages WebRTC peer connections for voice/video calls.
/// Uses full-mesh topology: each participant connects to every other participant.
/// Uses "polite peer" pattern: the peer with the lower user ID is the polite peer
/// (yields on offer collision). This prevents both sides from creating offers simultaneously.
class WebRTCManager {
	private peers = new Map<string, RTCPeerConnection>();
	private pendingCandidates = new Map<string, RTCIceCandidateInit[]>();
	private sessionId: string | null = null;
	private channelId: string | null = null;

	// Track which stream IDs are the "main" stream per user (audio/camera)
	private mainStreamIds = new Map<string, string>();

	// Raw microphone stream (before noise suppression), kept for hot-swap
	private rawStream: MediaStream | null = null;

	// Mic input gain node (inserted after noise suppression)
	private micGainNode: GainNode | null = null;

	// Timeouts for cleaning up peers stuck in 'disconnected' state
	private disconnectTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

	// Timeouts for offer answers that never arrive
	private answerTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

	// ICE servers fetched from server config (STUN/TURN)
	private iceServers: RTCIceServer[] = DEFAULT_ICE_SERVERS;

	// Guard against concurrent audio pipeline rebuilds
	private rebuildingPipeline = false;


	// Audio level monitoring
	private audioContext: AudioContext | null = null;
	private analysers = new Map<string, { analyser: AnalyserNode; source: MediaStreamAudioSourceNode }>();
	private levelCheckInterval: ReturnType<typeof setInterval> | null = null;

	private clearDisconnectTimeout(userId: string): void {
		const timeout = this.disconnectTimeouts.get(userId);
		if (timeout) {
			clearTimeout(timeout);
			this.disconnectTimeouts.delete(userId);
		}
	}

	private clearAllDisconnectTimeouts(): void {
		for (const timeout of this.disconnectTimeouts.values()) {
			clearTimeout(timeout);
		}
		this.disconnectTimeouts.clear();
	}

	private clearAnswerTimeout(userId: string): void {
		const timeout = this.answerTimeouts.get(userId);
		if (timeout) {
			clearTimeout(timeout);
			this.answerTimeouts.delete(userId);
		}
	}

	private clearAllAnswerTimeouts(): void {
		for (const timeout of this.answerTimeouts.values()) {
			clearTimeout(timeout);
		}
		this.answerTimeouts.clear();
	}

	/// Rejoin voice channel after WebSocket reconnection.
	/// Only cleans up dead/failed peer connections — healthy WebRTC connections
	/// survive WebSocket reconnects since they use independent transport.
	rejoinAfterReconnect(): void {
		if (!voiceStore.isInCall || !this.channelId) return;

		// Only close failed/closed connections; keep healthy ones alive
		let cleaned = 0;
		for (const [userId, pc] of this.peers) {
			const state = pc.connectionState ?? pc.iceConnectionState;
			if (state === 'failed' || state === 'closed') {
				pc.close();
				this.stopMonitoringStream(userId);
				voiceStore.removeRemoteStream(userId);
				voiceStore.removeRemoteScreenStream(userId);
				this.peers.delete(userId);
				this.pendingCandidates.delete(userId);
				this.mainStreamIds.delete(userId);
				cleaned++;
			}
		}
		this.clearAllDisconnectTimeouts();
		this.clearAllAnswerTimeouts();

		console.info(`WebSocket reconnected in voice call — rejoining (${this.peers.size} healthy peers kept, ${cleaned} cleaned)`);

		// Re-send join_voice — server upserts (idempotent) and broadcasts VoiceStateUpdate
		wsClient.send({ type: 'join_voice', channel_id: this.channelId });
	}

	/// Join a voice channel: acquire media, tell server, set up peers.
	async joinCall(channelId: string, withVideo: boolean = false): Promise<void> {
		if (voiceStore.isInCall) {
			await this.leaveCall();
		}

		// Check participant count before joining
		const currentParticipants = voiceStore.getChannelParticipants(channelId);
		if (currentParticipants.length >= MAX_PARTICIPANTS) {
			throw new Error(`Voice channel is full (max ${MAX_PARTICIPANTS} participants)`);
		}

		this.channelId = channelId;
		this.sessionId = randomUUID();

		// Fetch ICE servers from server config (includes TURN if configured)
		try {
			const config = await getServerConfig();
			if (config.ice_servers && config.ice_servers.length > 0) {
				this.iceServers = config.ice_servers;
			}
		} catch {
			// Fall back to default STUN servers
		}

		// Enumerate devices first so we can validate the saved selection
		await audioDeviceStore.enumerateDevices();

		// Acquire local media — disable browser's built-in noise suppression
		// so it doesn't conflict with our AudioWorklet pipeline
		const audioConstraints: MediaTrackConstraints = {
			noiseSuppression: false,
			echoCancellation: preferencesStore.preferences.echoCancellation,
			autoGainControl: preferencesStore.preferences.autoGainControl,
		};
		const selectedInput = audioDeviceStore.selectedInputId;
		if (selectedInput) {
			audioConstraints.deviceId = { exact: selectedInput };
		}
		const constraints: MediaStreamConstraints = {
			audio: audioConstraints,
			video: withVideo ? { width: 640, height: 480 } : false
		};

		let rawStream: MediaStream;
		try {
			rawStream = await navigator.mediaDevices.getUserMedia(constraints);
		} catch (err) {
			// If the saved device is unavailable, fall back to system default
			if (selectedInput) {
				console.warn(`Saved audio device ${selectedInput} unavailable, using default`);
				audioDeviceStore.setInputDevice('');
				delete (audioConstraints as Record<string, unknown>).deviceId;
				try {
					rawStream = await navigator.mediaDevices.getUserMedia({
						audio: audioConstraints,
						video: withVideo ? { width: 640, height: 480 } : false
					});
				} catch (fallbackErr) {
					console.error('Failed to access media devices:', fallbackErr);
					throw fallbackErr;
				}
			} else {
				console.error('Failed to access media devices:', err);
				throw err;
			}
		}

		this.rawStream = rawStream;

		// Re-enumerate now that we have permission (labels become available)
		audioDeviceStore.enumerateDevices();

		// Start audio level monitoring first (creates AudioContext)
		this.audioContext = new AudioContext({ sampleRate: 48000 });

		// Create mic gain node
		this.micGainNode = this.audioContext.createGain();
		this.micGainNode.gain.value = preferencesStore.preferences.inputGain / 100;

		// Apply noise suppression + gain pipeline
		let localStream: MediaStream;
		localStream = await this.buildAudioPipeline(rawStream);

		voiceStore.setCallState({
			channelId,
			participants: [authStore.user?.id ?? ''],
			localStream,
			screenStream: null,
			audioEnabled: true,
			videoEnabled: withVideo,
			screenSharing: false
		});

		// Start audio level monitoring (uses existing audioContext)
		this.startAudioLevelMonitoring();

		// Tell the server we're joining
		wsClient.send({ type: 'join_voice', channel_id: channelId });
	}

	/// Build the audio pipeline: rawStream → [suppression] → gainNode → destination
	/// Returns the final processed MediaStream.
	private async buildAudioPipeline(rawStream: MediaStream): Promise<MediaStream> {
		const level = preferencesStore.preferences.noiseSuppression;

		if (level !== 'off' && this.audioContext && this.micGainNode) {
			try {
				return await applyNoiseSuppression(this.audioContext, rawStream, level, this.micGainNode);
			} catch (err) {
				console.warn('Noise suppression failed, using gain-only pipeline:', err);
			}
		}

		// 'off' or fallback: source → gainNode → destination
		if (this.audioContext && this.micGainNode) {
			const source = this.audioContext.createMediaStreamSource(rawStream);
			const destination = this.audioContext.createMediaStreamDestination();
			source.connect(this.micGainNode);
			this.micGainNode.connect(destination);
			const stream = destination.stream;
			for (const vt of rawStream.getVideoTracks()) {
				stream.addTrack(vt);
			}
			return stream;
		}

		return rawStream;
	}

	/// Leave the current call and clean up all peer connections.
	async leaveCall(): Promise<void> {
		if (!this.channelId) return;

		// Stop audio monitoring
		this.stopAudioLevelMonitoring();

		// Clean up noise suppression and gain node
		removeNoiseSuppression();
		if (this.micGainNode) {
			this.micGainNode.disconnect();
			this.micGainNode = null;
		}

		// Stop raw stream tracks
		this.rawStream?.getTracks().forEach(t => t.stop());
		this.rawStream = null;

		// Stop system audio stream (captured for screen share)
		this.systemAudioStream?.getTracks().forEach(t => t.stop());
		this.systemAudioStream = null;

		// Tell server we're leaving
		wsClient.send({ type: 'leave_voice', channel_id: this.channelId });

		// Close all peer connections
		for (const [userId, pc] of this.peers) {
			pc.close();
			voiceStore.removeRemoteStream(userId);
			voiceStore.removeRemoteScreenStream(userId);
		}
		this.peers.clear();
		this.pendingCandidates.clear();
		this.mainStreamIds.clear();
		this.clearAllDisconnectTimeouts();
		this.clearAllAnswerTimeouts();

		voiceStore.clearCall();
		this.channelId = null;
		this.sessionId = null;
	}

	/// Toggle audio mute.
	toggleAudio(): void {
		if (!voiceStore.activeCall) return;
		voiceStore.setAudioEnabled(!voiceStore.activeCall.audioEnabled);
	}

	/// Change noise suppression level mid-call.
	async setNoiseSuppressionLevel(level: NoiseSuppression): Promise<void> {
		if (!voiceStore.activeCall || !this.rawStream || !this.audioContext || this.rebuildingPipeline) return;
		this.rebuildingPipeline = true;
		try { await this._setNoiseSuppressionLevel(level); } finally { this.rebuildingPipeline = false; }
	}

	private async _setNoiseSuppressionLevel(level: NoiseSuppression): Promise<void> {
		if (!voiceStore.activeCall || !this.rawStream || !this.audioContext) return;

		// Disconnect gain node before pipeline rebuild (will be reconnected inside)
		if (this.micGainNode) {
			this.micGainNode.disconnect();
		}

		// Rebuild the full pipeline with new suppression level
		removeNoiseSuppression();
		const pipelineStream = await this.buildAudioPipeline(this.rawStream);
		const newAudioTrack = pipelineStream.getAudioTracks()[0];
		if (!newAudioTrack) return;

		// Swap audio track in existing local stream (preserves stream ID for peers)
		const localStream = voiceStore.activeCall.localStream;
		let prevMicTrack: MediaStreamTrack | undefined;
		if (localStream) {
			prevMicTrack = localStream.getAudioTracks()[0];
			if (prevMicTrack) localStream.removeTrack(prevMicTrack);
			localStream.addTrack(newAudioTrack);
			// Preserve mute state
			if (!voiceStore.activeCall.audioEnabled) {
				newAudioTrack.enabled = false;
			}
		}

		// Replace only the mic audio sender on all peers (skip screen share audio senders)
		for (const pc of this.peers.values()) {
			for (const sender of pc.getSenders()) {
				if (sender.track?.kind === 'audio' && sender.track === prevMicTrack) {
					await sender.replaceTrack(newAudioTrack);
				}
			}
		}

		// Stop the old processed track to release AudioContext resources
		if (prevMicTrack) prevMicTrack.stop();

		// Re-monitor with existing stream
		const myId = authStore.user?.id;
		if (myId && localStream) this.monitorStream(myId, localStream);
	}

	/// Set microphone input gain (0-200, where 100 = normal).
	setMicGain(gain: number): void {
		const clamped = Math.max(0, Math.min(200, gain));
		if (this.micGainNode) {
			this.micGainNode.gain.value = clamped / 100;
		}
		preferencesStore.set('inputGain', clamped);
	}

	/// Switch input device mid-call.
	async switchInputDevice(deviceId: string): Promise<void> {
		if (!voiceStore.activeCall || !this.audioContext || this.rebuildingPipeline) return;
		this.rebuildingPipeline = true;
		try { await this._switchInputDevice(deviceId); } finally { this.rebuildingPipeline = false; }
	}

	private async _switchInputDevice(deviceId: string): Promise<void> {
		if (!voiceStore.activeCall || !this.audioContext) return;

		audioDeviceStore.setInputDevice(deviceId);

		const audioConstraints: MediaTrackConstraints = {
			noiseSuppression: false,
			echoCancellation: preferencesStore.preferences.echoCancellation,
			autoGainControl: preferencesStore.preferences.autoGainControl,
		};
		if (deviceId) {
			audioConstraints.deviceId = { exact: deviceId };
		}

		const newRawStream = await navigator.mediaDevices.getUserMedia({ audio: audioConstraints });

		// Stop old raw stream's audio tracks
		this.rawStream?.getAudioTracks().forEach(t => t.stop());
		this.rawStream = newRawStream;

		// Disconnect gain node before pipeline rebuild
		if (this.micGainNode) {
			this.micGainNode.disconnect();
		}

		// Rebuild pipeline with new raw stream
		removeNoiseSuppression();
		const pipelineStream = await this.buildAudioPipeline(newRawStream);
		const newAudioTrack = pipelineStream.getAudioTracks()[0];
		if (!newAudioTrack) return;

		// Swap audio track in existing local stream (preserves stream ID for peers)
		const localStream = voiceStore.activeCall.localStream;
		let prevMicTrack: MediaStreamTrack | undefined;
		if (localStream) {
			prevMicTrack = localStream.getAudioTracks()[0];
			if (prevMicTrack) localStream.removeTrack(prevMicTrack);
			localStream.addTrack(newAudioTrack);
			// Preserve mute state
			if (!voiceStore.activeCall.audioEnabled) {
				newAudioTrack.enabled = false;
			}
		}

		// Replace only the mic audio sender on all peers (skip screen share audio senders)
		for (const pc of this.peers.values()) {
			for (const sender of pc.getSenders()) {
				if (sender.track?.kind === 'audio' && sender.track === prevMicTrack) {
					await sender.replaceTrack(newAudioTrack);
				}
			}
		}

		// Stop the old processed track to release AudioContext resources
		if (prevMicTrack) prevMicTrack.stop();

		const myId = authStore.user?.id;
		if (myId && localStream) this.monitorStream(myId, localStream);
	}

	/// Toggle video.
	async toggleVideo(): Promise<void> {
		if (!voiceStore.activeCall) return;
		const stream = voiceStore.activeCall.localStream;
		if (!stream) return;

		if (voiceStore.activeCall.videoEnabled) {
			// Turn off video — remove tracks from peers, renegotiate, THEN stop
			const videoTracks = stream.getVideoTracks();
			for (const pc of this.peers.values()) {
				for (const sender of pc.getSenders()) {
					if (sender.track && sender.track.kind === 'video'
						&& videoTracks.includes(sender.track)) {
						pc.removeTrack(sender);
					}
				}
			}
			for (const t of videoTracks) {
				stream.removeTrack(t);
			}
			voiceStore.setVideoEnabled(false);
			await this.renegotiateAll();
			// Stop tracks after renegotiation so the removal is properly signaled
			videoTracks.forEach(t => t.stop());
		} else {
			// Turn on video
			let videoStream: MediaStream | null = null;
			try {
				videoStream = await navigator.mediaDevices.getUserMedia({
					video: { width: 640, height: 480 }
				});
				const videoTrack = videoStream.getVideoTracks()[0];
				stream.addTrack(videoTrack);
				voiceStore.setVideoEnabled(true);

				// Add the video track to all existing peer connections
				for (const pc of this.peers.values()) {
					pc.addTrack(videoTrack, stream);
				}
				await this.renegotiateAll();
			} catch (err) {
				console.error('Failed to enable video:', err);
				// Clean up any acquired tracks to release camera hardware
				videoStream?.getTracks().forEach(t => t.stop());
			}
		}
	}

	/// Toggle screen sharing.
	async toggleScreenShare(): Promise<void> {
		if (!voiceStore.activeCall) return;

		if (voiceStore.activeCall.screenSharing) {
			// Stop sharing — remove screen tracks from peers, renegotiate, THEN stop tracks
			const screenStream = voiceStore.activeCall.screenStream;
			if (screenStream) {
				for (const pc of this.peers.values()) {
					for (const sender of pc.getSenders()) {
						if (sender.track && screenStream.getTracks().includes(sender.track)) {
							pc.removeTrack(sender);
						}
					}
				}
			}
			voiceStore.setScreenSharing(false, null);
			await this.renegotiateAll();
			// Stop tracks after renegotiation so the removal is properly signaled
			screenStream?.getTracks().forEach(t => t.stop());
			this.systemAudioStream?.getTracks().forEach(t => t.stop());
			this.systemAudioStream = null;
		} else {
			try {
				const screenStream = await navigator.mediaDevices.getDisplayMedia({
					video: true,
					audio: true,
					// Request system audio capture (Chrome 105+, works with PipeWire on Linux)
					systemAudio: 'include',
				} as DisplayMediaStreamOptions & { systemAudio: string });

				// If getDisplayMedia didn't capture audio, try system audio via monitor device
				if (screenStream.getAudioTracks().length === 0) {
					const systemTrack = await this.captureSystemAudio();
					if (systemTrack) {
						screenStream.addTrack(systemTrack);
					}
				}

				voiceStore.setScreenSharing(true, screenStream);

				// When user stops sharing via browser UI
				screenStream.getVideoTracks()[0].onended = () => {
					this.stopScreenShare();
				};

				// Add all screen tracks (video + audio) to all peers and renegotiate
				for (const pc of this.peers.values()) {
					for (const track of screenStream.getTracks()) {
						pc.addTrack(track, screenStream);
					}
				}
				await this.renegotiateAll();
			} catch (err) {
				console.error('Failed to share screen:', err);
			}
		}
	}

	/// Try to capture system audio via a PipeWire/PulseAudio monitor device.
	/// These devices mirror audio output and show up as audio inputs on Linux.
	private systemAudioStream: MediaStream | null = null;

	private async captureSystemAudio(): Promise<MediaStreamTrack | null> {
		try {
			const devices = await navigator.mediaDevices.enumerateDevices();
			const monitors = devices.filter(d =>
				d.kind === 'audioinput' &&
				d.label.toLowerCase().includes('monitor')
			);

			if (monitors.length === 0) return null;

			// Capture from the first monitor device (system audio output)
			this.systemAudioStream = await navigator.mediaDevices.getUserMedia({
				audio: {
					deviceId: { exact: monitors[0].deviceId },
					// Disable processing — we want raw system audio
					echoCancellation: false,
					noiseSuppression: false,
					autoGainControl: false,
				}
			});

			return this.systemAudioStream.getAudioTracks()[0] ?? null;
		} catch {
			return null;
		}
	}

	/// Stop screen sharing (called by browser "stop sharing" button).
	private async stopScreenShare(): Promise<void> {
		if (!voiceStore.activeCall?.screenSharing) return;
		const screenStream = voiceStore.activeCall.screenStream;
		if (screenStream) {
			for (const pc of this.peers.values()) {
				for (const sender of pc.getSenders()) {
					if (sender.track && screenStream.getTracks().includes(sender.track)) {
						pc.removeTrack(sender);
					}
				}
			}
		}
		voiceStore.setScreenSharing(false, null);
		await this.renegotiateAll();
		// Stop tracks after renegotiation so the removal is properly signaled
		screenStream?.getTracks().forEach(t => t.stop());
		this.systemAudioStream?.getTracks().forEach(t => t.stop());
		this.systemAudioStream = null;
	}

	/// Get video constraints based on current participant count.
	private getVideoConstraints(): { width: number; height: number; frameRate: number } {
		const count = this.peers.size + 1; // +1 for self
		const tier = VIDEO_QUALITY_TIERS.find(t => count <= t.maxParticipants)
			?? VIDEO_QUALITY_TIERS[VIDEO_QUALITY_TIERS.length - 1];
		return tier;
	}

	/// Adjust local video quality based on current participant count.
	/// Called when participants join or leave to scale resolution/framerate.
	private async adjustVideoQuality(): Promise<void> {
		if (!voiceStore.activeCall?.videoEnabled) return;

		const { width, height, frameRate } = this.getVideoConstraints();
		const videoTrack = voiceStore.activeCall.localStream?.getVideoTracks()[0];
		if (!videoTrack) return;

		try {
			await videoTrack.applyConstraints({ width, height, frameRate });
		} catch (err) {
			console.warn('Failed to adjust video quality:', err);
		}
	}

	/// Renegotiate all peer connections (after adding/removing tracks).
	/// Peers not in 'stable' state are queued and retried after a short delay
	/// to handle cases where tracks are removed during an in-flight offer/answer.
	private async renegotiateAll(): Promise<void> {
		const deferred: string[] = [];
		for (const [userId, pc] of this.peers) {
			if (pc.signalingState !== 'stable') {
				deferred.push(userId);
				continue;
			}
			try {
				const offer = await pc.createOffer();
				await pc.setLocalDescription(offer);
				wsClient.send({
					type: 'rtc_offer',
					target_user_id: userId,
					session_id: this.sessionId!,
					sdp: JSON.stringify(offer)
				});
			} catch (err) {
				console.error(`Renegotiation with ${userId} failed:`, err);
			}
		}
		// Retry deferred peers after a short delay (wait for in-flight offer/answer to settle)
		if (deferred.length > 0) {
			setTimeout(async () => {
				for (const userId of deferred) {
					const pc = this.peers.get(userId);
					if (!pc || pc.signalingState !== 'stable') continue;
					try {
						const offer = await pc.createOffer();
						await pc.setLocalDescription(offer);
						wsClient.send({
							type: 'rtc_offer',
							target_user_id: userId,
							session_id: this.sessionId!,
							sdp: JSON.stringify(offer)
						});
					} catch (err) {
						console.error(`Deferred renegotiation with ${userId} failed:`, err);
					}
				}
			}, 2000);
		}
	}

	/// Check if a remote screen share stream has gone stale (all tracks ended/muted)
	/// and clean it up. Called after renegotiation since track events are unreliable.
	private cleanupStaleScreenShares(userId: string): void {
		const screenStream = voiceStore.remoteScreenStreams.get(userId);
		if (!screenStream) return;

		const activeTracks = screenStream.getTracks().filter(
			t => t.readyState === 'live' && !t.muted
		);
		if (activeTracks.length === 0) {
			voiceStore.removeRemoteScreenStream(userId);
		}
	}

	/// Start monitoring audio levels for all streams.
	private startAudioLevelMonitoring(): void {
		if (this.levelCheckInterval) return;

		// AudioContext is created in joinCall() (shared with noise suppression)
		if (!this.audioContext) this.audioContext = new AudioContext({ sampleRate: 48000 });

		// Monitor local stream
		const localStream = voiceStore.activeCall?.localStream;
		const myId = authStore.user?.id;
		if (localStream && myId) {
			this.monitorStream(myId, localStream);
		}

		// Poll audio levels periodically
		this.levelCheckInterval = setInterval(() => {
			const buffer = new Uint8Array(64);
			for (const [userId, { analyser }] of this.analysers) {
				analyser.getByteTimeDomainData(buffer);
				// Calculate RMS
				let sum = 0;
				for (let i = 0; i < buffer.length; i++) {
					const val = buffer[i] - 128;
					sum += val * val;
				}
				const rms = Math.sqrt(sum / buffer.length);
				voiceStore.setSpeaking(userId, rms > SPEAKING_THRESHOLD);

				// Expose numeric level for the local user's in-call meter
				if (userId === myId) {
					voiceStore.setLocalAudioLevel(Math.min(100, Math.round(rms * 3)));
				}
			}
		}, SPEAKING_CHECK_INTERVAL);
	}

	/// Monitor audio level for a specific stream.
	private monitorStream(userId: string, stream: MediaStream): void {
		if (!this.audioContext || stream.getAudioTracks().length === 0) return;

		// Ensure AudioContext is running (may be suspended by browser autoplay policy)
		if (this.audioContext.state === 'suspended') {
			this.audioContext.resume().catch(() => {});
		}

		// Clean up existing monitor for this user
		this.stopMonitoringStream(userId);

		const source = this.audioContext.createMediaStreamSource(stream);
		const analyser = this.audioContext.createAnalyser();
		analyser.fftSize = 256;
		analyser.smoothingTimeConstant = 0.3;
		source.connect(analyser);

		this.analysers.set(userId, { analyser, source });

		// Auto-cleanup when all audio tracks end (prevents orphaned analyser nodes)
		const cleanup = () => {
			if (stream.getAudioTracks().every(t => t.readyState === 'ended')) {
				this.stopMonitoringStream(userId);
			}
		};
		for (const track of stream.getAudioTracks()) {
			track.addEventListener('ended', cleanup);
		}
	}

	/// Stop monitoring a specific stream.
	private stopMonitoringStream(userId: string): void {
		const existing = this.analysers.get(userId);
		if (existing) {
			existing.source.disconnect();
			this.analysers.delete(userId);
		}
		voiceStore.setSpeaking(userId, false);
	}

	/// Stop all audio level monitoring.
	private stopAudioLevelMonitoring(): void {
		if (this.levelCheckInterval) {
			clearInterval(this.levelCheckInterval);
			this.levelCheckInterval = null;
		}
		for (const [userId] of this.analysers) {
			this.stopMonitoringStream(userId);
		}
		this.analysers.clear();
		if (this.audioContext) {
			this.audioContext.close();
			this.audioContext = null;
		}
		voiceStore.clearActiveSpeakers();
	}

	/// Determine if we are the "polite" peer (lower user ID yields on collision).
	private isPolite(remoteUserId: string): boolean {
		const myId = authStore.user?.id ?? '';
		return myId < remoteUserId;
	}

	/// Called when a new user joins the voice channel — only the impolite peer creates offers.
	async onUserJoined(userId: string): Promise<void> {
		if (!this.sessionId || !voiceStore.activeCall?.localStream) return;
		if (userId === authStore.user?.id) return;

		// Skip if we already have a healthy peer connection (prevents double
		// connection creation when both onUserJoined and onVoiceStateUpdate fire)
		const existingPc = this.peers.get(userId);
		if (existingPc) {
			const state = existingPc.connectionState ?? existingPc.iceConnectionState;
			if (state !== 'failed' && state !== 'closed') return;
		}

		// Only the impolite peer (higher ID) initiates the offer to avoid collisions.
		if (this.isPolite(userId)) return;

		await this.createAndSendOffer(userId);
	}

	/// Called when a user leaves the voice channel.
	onUserLeft(userId: string): void {
		this.clearDisconnectTimeout(userId);
		this.clearAnswerTimeout(userId);
		this.stopMonitoringStream(userId);
		voiceStore.setRemoteVideo(userId, false);
		voiceStore.removeRemoteScreenStream(userId);
		this.mainStreamIds.delete(userId);
		const pc = this.peers.get(userId);
		if (pc) {
			pc.close();
			this.peers.delete(userId);
			this.pendingCandidates.delete(userId);
			voiceStore.removeRemoteStream(userId);
		}
	}

	/// Handle an incoming RTC offer (initial or renegotiation).
	async handleOffer(fromUserId: string, sessionId: string, sdpJson: string): Promise<void> {
		console.info(`[VOICE] handleOffer from=${fromUserId.slice(0,8)} localStream=${!!voiceStore.activeCall?.localStream}`);
		if (!voiceStore.activeCall?.localStream) {
			console.warn(`[VOICE] handleOffer DROPPED: no activeCall/localStream`);
			return;
		}

		let pc = this.peers.get(fromUserId);
		const isRenegotiation = !!pc;

		if (pc) {
			// Existing connection — this is a renegotiation.
			// Only reject if there's an actual offer collision: we're the impolite
			// peer AND we already sent our own offer (have-local-offer).
			if (!this.isPolite(fromUserId) && pc.signalingState === 'have-local-offer') {
				console.info(`Offer collision with ${fromUserId} — dropping (we are impolite peer)`);
				return;
			}
			// Accept their offer. Roll back our pending offer if needed (polite peer yields).
			if (pc.signalingState === 'have-local-offer') {
				await pc.setLocalDescription({ type: 'rollback' });
			}
		} else {
			// New connection
			pc = this.createPeerConnection(fromUserId);

			// Add our local tracks
			for (const track of voiceStore.activeCall.localStream.getTracks()) {
				pc.addTrack(track, voiceStore.activeCall.localStream);
			}

			// Also add screen share tracks if we're currently sharing
			if (voiceStore.activeCall.screenSharing && voiceStore.activeCall.screenStream) {
				for (const track of voiceStore.activeCall.screenStream.getTracks()) {
					pc.addTrack(track, voiceStore.activeCall.screenStream);
				}
			}
		}

		try {
			const offer = JSON.parse(sdpJson) as RTCSessionDescriptionInit;
			await pc.setRemoteDescription(offer);

			// Flush any ICE candidates that arrived before the offer
			await this.flushPendingCandidates(fromUserId, pc);

			const answer = await pc.createAnswer();
			await pc.setLocalDescription(answer);

			wsClient.send({
				type: 'rtc_answer',
				target_user_id: fromUserId,
				session_id: sessionId,
				sdp: JSON.stringify(answer)
			});
		} catch (err) {
			console.error(`Failed to handle offer from ${fromUserId}:`, err);
			this.onUserLeft(fromUserId);
			return;
		}

		// After renegotiation, check for stale screen shares. Track events
		// (onended/onmute) don't fire reliably across browsers when the sender
		// removes tracks, so we actively check after a short delay.
		if (isRenegotiation) {
			setTimeout(() => {
				this.cleanupStaleScreenShares(fromUserId);
			}, 1000);
		}
	}

	/// Handle an incoming RTC answer.
	async handleAnswer(fromUserId: string, sdpJson: string): Promise<void> {
		console.info(`[VOICE] handleAnswer from=${fromUserId.slice(0,8)}`);
		this.clearAnswerTimeout(fromUserId);

		const pc = this.peers.get(fromUserId);
		if (!pc) {
			console.warn(`[VOICE] handleAnswer DROPPED: no peer connection for ${fromUserId.slice(0,8)}`);
			return;
		}

		if (pc.signalingState !== 'have-local-offer') {
			console.warn(`Received answer in unexpected state: ${pc.signalingState}`);
			return;
		}

		try {
			const answer = JSON.parse(sdpJson) as RTCSessionDescriptionInit;
			await pc.setRemoteDescription(answer);

			// Flush any ICE candidates that arrived before the answer
			await this.flushPendingCandidates(fromUserId, pc);
		} catch (err) {
			console.error(`Failed to handle answer from ${fromUserId}:`, err);
			this.onUserLeft(fromUserId);
		}
	}

	/// Handle an incoming ICE candidate.
	async handleIceCandidate(fromUserId: string, sessionId: string, candidateJson: string): Promise<void> {
		// Discard candidates from a different session (e.g. stale messages after rejoin)
		if (sessionId !== this.sessionId) return;

		const candidate = JSON.parse(candidateJson) as RTCIceCandidateInit;
		const pc = this.peers.get(fromUserId);

		// Queue candidates if peer connection doesn't exist yet or remote description isn't set
		if (!pc || !pc.remoteDescription) {
			const pending = this.pendingCandidates.get(fromUserId) ?? [];
			pending.push(candidate);
			this.pendingCandidates.set(fromUserId, pending);
			return;
		}

		if (pc.connectionState === 'closed') return;

		try {
			await pc.addIceCandidate(candidate);
		} catch (err) {
			console.warn(`Failed to add ICE candidate from ${fromUserId}:`, err);
		}
	}

	/// Handle voice state update (list of current participants).
	async onVoiceStateUpdate(channelId: string, participants: string[]): Promise<void> {
		voiceStore.setChannelParticipants(channelId, participants);

		const myId = authStore.user?.id;
		const inCall = voiceStore.activeCall?.channelId === channelId;
		console.info(`[VOICE] onVoiceStateUpdate ch=${channelId.slice(0,8)} participants=[${participants.map(p => p.slice(0,8)).join(',')}] myId=${myId?.slice(0,8)} inCall=${inCall} activeCallCh=${voiceStore.activeCall?.channelId?.slice(0,8)}`);

		// If we're in this call, reconcile peer connections with authoritative list
		if (inCall) {
			const participantSet = new Set(participants);

			// Remove peers who are no longer in the participant list
			const toRemove = [...this.peers.keys()].filter(id => !participantSet.has(id));
			for (const userId of toRemove) {
				this.onUserLeft(userId);
			}

			// Establish connections with new participants (or reconnect stale ones)
			for (const userId of participants) {
				if (userId === myId) continue;

				// Check for stale peer connections (failed/closed only).
				// 'disconnected' is a transient state that often self-recovers —
				// the onconnectionstatechange handler has a 10-second grace period for it.
				const existingPc = this.peers.get(userId);
				if (existingPc) {
					const state = existingPc.connectionState ?? existingPc.iceConnectionState;
					if (state === 'failed' || state === 'closed') {
						this.onUserLeft(userId);
					} else {
						continue; // healthy or recovering connection, skip
					}
				}

				const amImpolite = !this.isPolite(userId);

				// Only the impolite peer (higher ID) creates offers
				if (amImpolite) {
					try {
						await this.createAndSendOffer(userId);
					} catch (err) {
						console.error(`[VOICE] Failed to create offer for ${userId}:`, err);
					}
				}
			}

			// Adjust video quality based on new participant count
			await this.adjustVideoQuality();
		}
	}

	/// Create a peer connection, add tracks, create offer, and send it.
	private async createAndSendOffer(userId: string, isRetry = false): Promise<void> {
		if (!this.sessionId || !voiceStore.activeCall?.localStream) {
			console.warn(`[VOICE] createAndSendOffer(${userId.slice(0,8)}) ABORTED: sessionId=${!!this.sessionId} localStream=${!!voiceStore.activeCall?.localStream}`);
			return;
		}

		const pc = this.createPeerConnection(userId);

		// Add our local tracks
		for (const track of voiceStore.activeCall.localStream.getTracks()) {
			pc.addTrack(track, voiceStore.activeCall.localStream);
		}

		// Also add screen share tracks if we're currently sharing
		if (voiceStore.activeCall.screenSharing && voiceStore.activeCall.screenStream) {
			for (const track of voiceStore.activeCall.screenStream.getTracks()) {
				pc.addTrack(track, voiceStore.activeCall.screenStream);
			}
		}

		try {
			// Create and send offer
			const offer = await pc.createOffer();
			await pc.setLocalDescription(offer);

			wsClient.send({
				type: 'rtc_offer',
				target_user_id: userId,
				session_id: this.sessionId,
				sdp: JSON.stringify(offer)
			});

			// Set timeout for answer — if no answer arrives, clean up and retry once
			this.clearAnswerTimeout(userId);
			this.answerTimeouts.set(userId, setTimeout(() => {
				this.answerTimeouts.delete(userId);
				const current = this.peers.get(userId);
				if (current?.signalingState === 'have-local-offer') {
					console.warn(`No answer from ${userId} after 15s${isRetry ? ' (retry)' : ''}, cleaning up`);
					this.onUserLeft(userId);
					if (!isRetry) {
						this.createAndSendOffer(userId, true).catch(() => {});
					}
				}
			}, 15_000));
		} catch (err) {
			console.error(`Failed to create offer for ${userId}:`, err);
			this.onUserLeft(userId);
		}
	}

	/// Flush queued ICE candidates after remote description is set.
	private async flushPendingCandidates(userId: string, pc: RTCPeerConnection): Promise<void> {
		const pending = this.pendingCandidates.get(userId);
		if (pending) {
			for (const candidate of pending) {
				try {
					await pc.addIceCandidate(candidate);
				} catch (err) {
					console.warn(`Failed to add buffered ICE candidate for ${userId}:`, err);
				}
			}
			this.pendingCandidates.delete(userId);
		}
	}

	private createPeerConnection(userId: string): RTCPeerConnection {
		// Close existing connection if any
		this.peers.get(userId)?.close();

		const pc = new RTCPeerConnection({ iceServers: this.iceServers });
		this.peers.set(userId, pc);

		// ICE candidate handling
		pc.onicecandidate = (event) => {
			if (event.candidate && this.sessionId) {
				wsClient.send({
					type: 'rtc_ice_candidate',
					target_user_id: userId,
					session_id: this.sessionId,
					candidate: JSON.stringify(event.candidate.toJSON())
				});
			}
		};

		// Remote track handling — distinguish main stream from screen share
		pc.ontrack = (event) => {
			const [stream] = event.streams;
			if (!stream) return;

			const knownMainId = this.mainStreamIds.get(userId);

			if (!knownMainId) {
				// First stream from this user = main stream (audio/camera)
				this.mainStreamIds.set(userId, stream.id);
				voiceStore.addRemoteStream(userId, stream);
				// Audio playback is handled by PersistentAudio.svelte (per-user volume via GainNode)
				this.monitorStream(userId, stream);
			} else if (stream.id === knownMainId) {
				// Additional track on the main stream (e.g., camera toggled on)
				voiceStore.addRemoteStream(userId, stream);
			} else {
				// Different stream = screen share
				voiceStore.addRemoteScreenStream(userId, stream);

				// Clean up when screen share stops — multiple mechanisms because
				// browsers vary in which events they fire on renegotiation
				const cleanupScreen = () => {
					voiceStore.removeRemoteScreenStream(userId);
				};

				// Direct track end (e.g., sender stopped the track)
				event.track.onended = cleanupScreen;

				// Track muted = sender removed it during renegotiation
				event.track.onmute = () => {
					// Verify after a brief delay (avoids false positives from network glitches)
					setTimeout(() => {
						if (event.track.readyState === 'ended' || event.track.muted) {
							cleanupScreen();
						}
					}, 500);
				};

				// Track removed from the stream entirely
				stream.onremovetrack = () => {
					if (stream.getTracks().length === 0) {
						cleanupScreen();
					}
				};
			}

			// Track remote video state (only for main stream)
			if (event.track.kind === 'video' && stream.id === (this.mainStreamIds.get(userId) ?? stream.id)) {
				if (stream.id === this.mainStreamIds.get(userId)) {
					voiceStore.setRemoteVideo(userId, true);
					event.track.onended = () => {
						voiceStore.setRemoteVideo(userId, false);
					};
					event.track.onmute = () => {
						voiceStore.setRemoteVideo(userId, false);
					};
					event.track.onunmute = () => {
						voiceStore.setRemoteVideo(userId, true);
					};
				}
			}
		};

		// Connection state monitoring
		pc.onconnectionstatechange = () => {
			if (pc.connectionState === 'failed') {
				console.warn(`Peer connection to ${userId} failed, cleaning up`);
				this.clearDisconnectTimeout(userId);
				this.onUserLeft(userId);
			} else if (pc.connectionState === 'disconnected') {
				console.warn(`Peer connection to ${userId} disconnected, will clean up in 10s if not recovered`);
				this.clearDisconnectTimeout(userId);
				this.disconnectTimeouts.set(userId, setTimeout(() => {
					this.disconnectTimeouts.delete(userId);
					const current = this.peers.get(userId);
					if (current && (current.connectionState === 'disconnected' || current.connectionState === 'failed')) {
						console.warn(`Peer connection to ${userId} still disconnected after 10s, cleaning up`);
						this.onUserLeft(userId);
					}
				}, 10_000));
			} else if (pc.connectionState === 'connected') {
				this.clearDisconnectTimeout(userId);
			}
		};

		// ICE connection state monitoring (catches DTLS failures that
		// onconnectionstatechange may not report on all browsers)
		pc.oniceconnectionstatechange = () => {
			if (pc.iceConnectionState === 'failed') {
				console.warn(`ICE connection to ${userId} failed (DTLS/ICE failure), cleaning up`);
				this.clearDisconnectTimeout(userId);
				this.onUserLeft(userId);
			}
		};

		return pc;
	}
}

export const webrtcManager = new WebRTCManager();
