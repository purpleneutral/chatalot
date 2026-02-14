import { voiceStore } from '$lib/stores/voice.svelte';
import { authStore } from '$lib/stores/auth.svelte';
import { wsClient } from '$lib/ws/connection';

const ICE_SERVERS: RTCIceServer[] = [
	{ urls: 'stun:stun.l.google.com:19302' },
	{ urls: 'stun:stun1.l.google.com:19302' }
];

const SPEAKING_THRESHOLD = 15; // RMS level (0-255) above which user is "speaking"
const SPEAKING_CHECK_INTERVAL = 100; // ms between audio level checks

/// Manages WebRTC peer connections for voice/video calls.
/// Uses full-mesh topology: each participant connects to every other participant.
/// Uses "polite peer" pattern: the peer with the lower user ID is the polite peer
/// (yields on offer collision). This prevents both sides from creating offers simultaneously.
class WebRTCManager {
	private peers = new Map<string, RTCPeerConnection>();
	private pendingCandidates = new Map<string, RTCIceCandidateInit[]>();
	private sessionId: string | null = null;
	private channelId: string | null = null;

	// Audio level monitoring
	private audioContext: AudioContext | null = null;
	private analysers = new Map<string, { analyser: AnalyserNode; source: MediaStreamAudioSourceNode }>();
	private levelCheckInterval: ReturnType<typeof setInterval> | null = null;

	/// Join a voice channel: acquire media, tell server, set up peers.
	async joinCall(channelId: string, withVideo: boolean = false): Promise<void> {
		if (voiceStore.isInCall) {
			await this.leaveCall();
		}

		this.channelId = channelId;
		this.sessionId = crypto.randomUUID();

		// Acquire local media
		const constraints: MediaStreamConstraints = {
			audio: true,
			video: withVideo ? { width: 640, height: 480 } : false
		};

		let localStream: MediaStream;
		try {
			localStream = await navigator.mediaDevices.getUserMedia(constraints);
		} catch (err) {
			console.error('Failed to access media devices:', err);
			throw err;
		}

		voiceStore.setCallState({
			channelId,
			participants: [authStore.user?.id ?? ''],
			localStream,
			screenStream: null,
			audioEnabled: true,
			videoEnabled: withVideo,
			screenSharing: false
		});

		// Start audio level monitoring
		this.startAudioLevelMonitoring();

		// Tell the server we're joining
		wsClient.send({ type: 'join_voice', channel_id: channelId });
	}

	/// Leave the current call and clean up all peer connections.
	async leaveCall(): Promise<void> {
		if (!this.channelId) return;

		// Stop audio monitoring
		this.stopAudioLevelMonitoring();

		// Tell server we're leaving
		wsClient.send({ type: 'leave_voice', channel_id: this.channelId });

		// Close all peer connections
		for (const [userId, pc] of this.peers) {
			pc.close();
			voiceStore.removeRemoteStream(userId);
		}
		this.peers.clear();
		this.pendingCandidates.clear();

		voiceStore.clearCall();
		this.channelId = null;
		this.sessionId = null;
	}

	/// Toggle audio mute.
	toggleAudio(): void {
		if (!voiceStore.activeCall) return;
		voiceStore.setAudioEnabled(!voiceStore.activeCall.audioEnabled);
	}

	/// Toggle video.
	async toggleVideo(): Promise<void> {
		if (!voiceStore.activeCall) return;
		const stream = voiceStore.activeCall.localStream;
		if (!stream) return;

		if (voiceStore.activeCall.videoEnabled) {
			// Turn off video
			stream.getVideoTracks().forEach(t => {
				t.stop();
				stream.removeTrack(t);
			});
			voiceStore.setVideoEnabled(false);
		} else {
			// Turn on video
			try {
				const videoStream = await navigator.mediaDevices.getUserMedia({
					video: { width: 640, height: 480 }
				});
				const videoTrack = videoStream.getVideoTracks()[0];
				stream.addTrack(videoTrack);
				voiceStore.setVideoEnabled(true);

				// Add the video track to all existing peer connections
				for (const pc of this.peers.values()) {
					pc.addTrack(videoTrack, stream);
				}
			} catch (err) {
				console.error('Failed to enable video:', err);
			}
		}
	}

	/// Toggle screen sharing.
	async toggleScreenShare(): Promise<void> {
		if (!voiceStore.activeCall) return;

		if (voiceStore.activeCall.screenSharing) {
			voiceStore.activeCall.screenStream?.getTracks().forEach(t => t.stop());
			voiceStore.setScreenSharing(false, null);
		} else {
			try {
				const screenStream = await navigator.mediaDevices.getDisplayMedia({
					video: true,
					audio: false
				});
				voiceStore.setScreenSharing(true, screenStream);

				// When user stops sharing via browser UI
				screenStream.getVideoTracks()[0].onended = () => {
					voiceStore.setScreenSharing(false, null);
				};

				// Add screen track to all peers
				const screenTrack = screenStream.getVideoTracks()[0];
				for (const pc of this.peers.values()) {
					pc.addTrack(screenTrack, screenStream);
				}
			} catch (err) {
				console.error('Failed to share screen:', err);
			}
		}
	}

	/// Start monitoring audio levels for all streams.
	private startAudioLevelMonitoring(): void {
		if (this.levelCheckInterval) return;

		this.audioContext = new AudioContext();

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
			}
		}, SPEAKING_CHECK_INTERVAL);
	}

	/// Monitor audio level for a specific stream.
	private monitorStream(userId: string, stream: MediaStream): void {
		if (!this.audioContext || stream.getAudioTracks().length === 0) return;

		// Clean up existing monitor for this user
		this.stopMonitoringStream(userId);

		const source = this.audioContext.createMediaStreamSource(stream);
		const analyser = this.audioContext.createAnalyser();
		analyser.fftSize = 256;
		analyser.smoothingTimeConstant = 0.3;
		source.connect(analyser);

		this.analysers.set(userId, { analyser, source });
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

		// Only the impolite peer (higher ID) initiates the offer to avoid collisions.
		if (this.isPolite(userId)) return;

		await this.createAndSendOffer(userId);
	}

	/// Called when a user leaves the voice channel.
	onUserLeft(userId: string): void {
		this.stopMonitoringStream(userId);
		voiceStore.setRemoteVideo(userId, false);
		const pc = this.peers.get(userId);
		if (pc) {
			pc.close();
			this.peers.delete(userId);
			this.pendingCandidates.delete(userId);
			voiceStore.removeRemoteStream(userId);
		}
	}

	/// Handle an incoming RTC offer.
	async handleOffer(fromUserId: string, sessionId: string, sdpJson: string): Promise<void> {
		if (!voiceStore.activeCall?.localStream) return;

		// If we already have a connection and we're the impolite peer, ignore this offer (we sent ours first).
		const existing = this.peers.get(fromUserId);
		if (existing && !this.isPolite(fromUserId)) {
			return;
		}

		const pc = this.createPeerConnection(fromUserId);

		// Add our local tracks
		for (const track of voiceStore.activeCall.localStream.getTracks()) {
			pc.addTrack(track, voiceStore.activeCall.localStream);
		}

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
	}

	/// Handle an incoming RTC answer.
	async handleAnswer(fromUserId: string, sdpJson: string): Promise<void> {
		const pc = this.peers.get(fromUserId);
		if (!pc) {
			console.warn(`Received answer from ${fromUserId} but no peer connection exists`);
			return;
		}

		if (pc.signalingState !== 'have-local-offer') {
			console.warn(`Received answer in unexpected state: ${pc.signalingState}`);
			return;
		}

		const answer = JSON.parse(sdpJson) as RTCSessionDescriptionInit;
		await pc.setRemoteDescription(answer);

		// Flush any ICE candidates that arrived before the answer
		await this.flushPendingCandidates(fromUserId, pc);
	}

	/// Handle an incoming ICE candidate.
	async handleIceCandidate(fromUserId: string, candidateJson: string): Promise<void> {
		const candidate = JSON.parse(candidateJson) as RTCIceCandidateInit;
		const pc = this.peers.get(fromUserId);

		// Queue candidates if peer connection doesn't exist yet or remote description isn't set
		if (!pc || !pc.remoteDescription) {
			const pending = this.pendingCandidates.get(fromUserId) ?? [];
			pending.push(candidate);
			this.pendingCandidates.set(fromUserId, pending);
			return;
		}

		await pc.addIceCandidate(candidate);
	}

	/// Handle voice state update (list of current participants).
	async onVoiceStateUpdate(channelId: string, participants: string[]): Promise<void> {
		voiceStore.setChannelParticipants(channelId, participants);

		// If we're in this call, establish connections with existing participants
		if (voiceStore.activeCall?.channelId === channelId) {
			const myId = authStore.user?.id;
			for (const userId of participants) {
				if (userId !== myId && !this.peers.has(userId)) {
					// Only the impolite peer (higher ID) creates offers
					if (!this.isPolite(userId)) {
						await this.createAndSendOffer(userId);
					}
				}
			}
		}
	}

	/// Create a peer connection, add tracks, create offer, and send it.
	private async createAndSendOffer(userId: string): Promise<void> {
		if (!this.sessionId || !voiceStore.activeCall?.localStream) return;

		const pc = this.createPeerConnection(userId);

		// Add our local tracks
		for (const track of voiceStore.activeCall.localStream.getTracks()) {
			pc.addTrack(track, voiceStore.activeCall.localStream);
		}

		// Create and send offer
		const offer = await pc.createOffer();
		await pc.setLocalDescription(offer);

		wsClient.send({
			type: 'rtc_offer',
			target_user_id: userId,
			session_id: this.sessionId,
			sdp: JSON.stringify(offer)
		});
	}

	/// Flush queued ICE candidates after remote description is set.
	private async flushPendingCandidates(userId: string, pc: RTCPeerConnection): Promise<void> {
		const pending = this.pendingCandidates.get(userId);
		if (pending) {
			for (const candidate of pending) {
				await pc.addIceCandidate(candidate);
			}
			this.pendingCandidates.delete(userId);
		}
	}

	private createPeerConnection(userId: string): RTCPeerConnection {
		// Close existing connection if any
		this.peers.get(userId)?.close();

		const pc = new RTCPeerConnection({ iceServers: ICE_SERVERS });
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

		// Remote track handling
		pc.ontrack = (event) => {
			const [stream] = event.streams;
			if (stream) {
				voiceStore.addRemoteStream(userId, stream);
				this.monitorStream(userId, stream);

				// Track remote video state
				if (event.track.kind === 'video') {
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
				this.stopMonitoringStream(userId);
				pc.close();
				this.peers.delete(userId);
				this.pendingCandidates.delete(userId);
				voiceStore.removeRemoteStream(userId);
			} else if (pc.connectionState === 'disconnected') {
				console.warn(`Peer connection to ${userId} disconnected`);
			}
		};

		return pc;
	}
}

export const webrtcManager = new WebRTCManager();
