/// Voice/video call state management.

export interface VoiceState {
	channelId: string;
	participants: string[];
	localStream: MediaStream | null;
	screenStream: MediaStream | null;
	audioEnabled: boolean;
	videoEnabled: boolean;
	screenSharing: boolean;
}

class VoiceStore {
	// Current active call state (null if not in a call)
	activeCall = $state<VoiceState | null>(null);

	// Remote streams keyed by user ID
	remoteStreams = $state<Map<string, MediaStream>>(new Map());

	// Remote screen share streams keyed by user ID
	remoteScreenStreams = $state<Map<string, MediaStream>>(new Map());

	// Voice participants per channel (for showing who's in a call even if we aren't)
	channelVoiceParticipants = $state<Map<string, string[]>>(new Map());

	// Currently speaking users (based on audio level detection)
	activeSpeakers = $state<Set<string>>(new Set());

	// Which remote users have video enabled
	remoteVideoEnabled = $state<Set<string>>(new Set());

	// Per-user volume (0-200, default 100)
	userVolumes = $state<Map<string, number>>(new Map());

	get isInCall(): boolean {
		return this.activeCall !== null;
	}

	get currentChannelId(): string | null {
		return this.activeCall?.channelId ?? null;
	}

	setCallState(state: VoiceState) {
		this.activeCall = state;
	}

	clearCall() {
		// Stop all local streams
		this.activeCall?.localStream?.getTracks().forEach(t => t.stop());
		this.activeCall?.screenStream?.getTracks().forEach(t => t.stop());
		this.activeCall = null;
		this.remoteStreams = new Map();
		this.remoteScreenStreams = new Map();
	}

	setAudioEnabled(enabled: boolean) {
		if (!this.activeCall) return;
		this.activeCall = { ...this.activeCall, audioEnabled: enabled };
		this.activeCall.localStream?.getAudioTracks().forEach(t => {
			t.enabled = enabled;
		});
	}

	setVideoEnabled(enabled: boolean) {
		if (!this.activeCall) return;
		this.activeCall = { ...this.activeCall, videoEnabled: enabled };
		this.activeCall.localStream?.getVideoTracks().forEach(t => {
			t.enabled = enabled;
		});
	}

	setScreenSharing(sharing: boolean, stream: MediaStream | null) {
		if (!this.activeCall) return;
		this.activeCall = { ...this.activeCall, screenSharing: sharing, screenStream: stream };
	}

	addRemoteStream(userId: string, stream: MediaStream) {
		const next = new Map(this.remoteStreams);
		next.set(userId, stream);
		this.remoteStreams = next;
	}

	removeRemoteStream(userId: string) {
		const next = new Map(this.remoteStreams);
		next.delete(userId);
		this.remoteStreams = next;
	}

	addRemoteScreenStream(userId: string, stream: MediaStream) {
		const next = new Map(this.remoteScreenStreams);
		next.set(userId, stream);
		this.remoteScreenStreams = next;
	}

	removeRemoteScreenStream(userId: string) {
		const next = new Map(this.remoteScreenStreams);
		next.delete(userId);
		this.remoteScreenStreams = next;
	}

	setChannelParticipants(channelId: string, participants: string[]) {
		const next = new Map(this.channelVoiceParticipants);
		next.set(channelId, participants);
		this.channelVoiceParticipants = next;
	}

	addChannelParticipant(channelId: string, userId: string) {
		const current = this.channelVoiceParticipants.get(channelId) ?? [];
		if (!current.includes(userId)) {
			const next = new Map(this.channelVoiceParticipants);
			next.set(channelId, [...current, userId]);
			this.channelVoiceParticipants = next;
		}
		// Also update active call participants if we're in this channel
		if (this.activeCall?.channelId === channelId) {
			const participants = [...this.activeCall.participants];
			if (!participants.includes(userId)) {
				participants.push(userId);
				this.activeCall = { ...this.activeCall, participants };
			}
		}
	}

	removeChannelParticipant(channelId: string, userId: string) {
		const current = this.channelVoiceParticipants.get(channelId) ?? [];
		const next = new Map(this.channelVoiceParticipants);
		next.set(channelId, current.filter(id => id !== userId));
		this.channelVoiceParticipants = next;

		if (this.activeCall?.channelId === channelId) {
			this.activeCall = {
				...this.activeCall,
				participants: this.activeCall.participants.filter(id => id !== userId)
			};
		}
	}

	getChannelParticipants(channelId: string): string[] {
		return this.channelVoiceParticipants.get(channelId) ?? [];
	}

	setSpeaking(userId: string, speaking: boolean) {
		const next = new Set(this.activeSpeakers);
		if (speaking) {
			next.add(userId);
		} else {
			next.delete(userId);
		}
		this.activeSpeakers = next;
	}

	isSpeaking(userId: string): boolean {
		return this.activeSpeakers.has(userId);
	}

	clearActiveSpeakers() {
		this.activeSpeakers = new Set();
	}

	setRemoteVideo(userId: string, hasVideo: boolean) {
		const next = new Set(this.remoteVideoEnabled);
		if (hasVideo) {
			next.add(userId);
		} else {
			next.delete(userId);
		}
		this.remoteVideoEnabled = next;
	}

	hasRemoteVideo(userId: string): boolean {
		return this.remoteVideoEnabled.has(userId);
	}

	getUserVolume(userId: string): number {
		return this.userVolumes.get(userId) ?? 100;
	}

	setUserVolume(userId: string, volume: number) {
		const next = new Map(this.userVolumes);
		next.set(userId, Math.max(0, Math.min(500, volume)));
		this.userVolumes = next;
	}
}

export const voiceStore = new VoiceStore();
