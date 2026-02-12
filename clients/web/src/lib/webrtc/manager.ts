import { voiceStore } from '$lib/stores/voice.svelte';
import { authStore } from '$lib/stores/auth.svelte';
import { wsClient } from '$lib/ws/connection';

const ICE_SERVERS: RTCIceServer[] = [
	{ urls: 'stun:stun.l.google.com:19302' },
	{ urls: 'stun:stun1.l.google.com:19302' }
];

/// Manages WebRTC peer connections for voice/video calls.
/// Uses full-mesh topology: each participant connects to every other participant.
class WebRTCManager {
	private peers = new Map<string, RTCPeerConnection>();
	private sessionId: string | null = null;
	private channelId: string | null = null;

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

		// Tell the server we're joining
		wsClient.send({ type: 'join_voice', channel_id: channelId });
	}

	/// Leave the current call and clean up all peer connections.
	async leaveCall(): Promise<void> {
		if (!this.channelId) return;

		// Tell server we're leaving
		wsClient.send({ type: 'leave_voice', channel_id: this.channelId });

		// Close all peer connections
		for (const [userId, pc] of this.peers) {
			pc.close();
			voiceStore.removeRemoteStream(userId);
		}
		this.peers.clear();

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

	/// Called when a new user joins the voice channel — initiate a connection as the offerer.
	async onUserJoined(userId: string): Promise<void> {
		if (!this.sessionId || !voiceStore.activeCall?.localStream) return;
		if (userId === authStore.user?.id) return;

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

	/// Called when a user leaves the voice channel.
	onUserLeft(userId: string): void {
		const pc = this.peers.get(userId);
		if (pc) {
			pc.close();
			this.peers.delete(userId);
			voiceStore.removeRemoteStream(userId);
		}
	}

	/// Handle an incoming RTC offer.
	async handleOffer(fromUserId: string, sessionId: string, sdpJson: string): Promise<void> {
		if (!voiceStore.activeCall?.localStream) return;

		const pc = this.createPeerConnection(fromUserId);

		// Add our local tracks
		for (const track of voiceStore.activeCall.localStream.getTracks()) {
			pc.addTrack(track, voiceStore.activeCall.localStream);
		}

		const offer = JSON.parse(sdpJson) as RTCSessionDescriptionInit;
		await pc.setRemoteDescription(offer);

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
		if (!pc) return;

		const answer = JSON.parse(sdpJson) as RTCSessionDescriptionInit;
		await pc.setRemoteDescription(answer);
	}

	/// Handle an incoming ICE candidate.
	async handleIceCandidate(fromUserId: string, candidateJson: string): Promise<void> {
		const pc = this.peers.get(fromUserId);
		if (!pc) return;

		const candidate = JSON.parse(candidateJson) as RTCIceCandidateInit;
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
					await this.onUserJoined(userId);
				}
			}
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
			}
		};

		// Connection state monitoring
		pc.onconnectionstatechange = () => {
			if (pc.connectionState === 'failed' || pc.connectionState === 'disconnected') {
				console.warn(`Peer connection to ${userId} ${pc.connectionState}`);
			}
		};

		return pc;
	}
}

export const webrtcManager = new WebRTCManager();
