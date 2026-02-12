interface SoundPreferences {
	dmMessage: boolean;
	channelMessage: boolean;
	mentionMessage: boolean;
	voiceJoin: boolean;
	voiceLeave: boolean;
	volume: number;
}

const DEFAULTS: SoundPreferences = {
	dmMessage: true,
	channelMessage: false,
	mentionMessage: true,
	voiceJoin: true,
	voiceLeave: true,
	volume: 0.5
};

class SoundStore {
	private context: AudioContext | null = null;
	preferences = $state<SoundPreferences>({ ...DEFAULTS });

	constructor() {
		if (typeof localStorage !== 'undefined') {
			const saved = localStorage.getItem('sound_preferences');
			if (saved) {
				try {
					this.preferences = { ...DEFAULTS, ...JSON.parse(saved) };
				} catch {
					// ignore corrupt data
				}
			}
		}
	}

	save() {
		localStorage.setItem('sound_preferences', JSON.stringify(this.preferences));
	}

	private getContext(): AudioContext {
		if (!this.context) this.context = new AudioContext();
		return this.context;
	}

	private playTone(frequency: number, duration: number, type: OscillatorType = 'sine') {
		try {
			const ctx = this.getContext();
			if (ctx.state === 'suspended') ctx.resume();
			const osc = ctx.createOscillator();
			const gain = ctx.createGain();
			osc.type = type;
			osc.frequency.value = frequency;
			gain.gain.value = this.preferences.volume;
			gain.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + duration);
			osc.connect(gain).connect(ctx.destination);
			osc.start();
			osc.stop(ctx.currentTime + duration);
		} catch {
			// AudioContext may not be available
		}
	}

	playDmNotification() {
		if (!this.preferences.dmMessage) return;
		this.playTone(800, 0.15);
		setTimeout(() => this.playTone(1000, 0.15), 150);
	}

	playChannelNotification() {
		if (!this.preferences.channelMessage) return;
		this.playTone(600, 0.1);
	}

	playMentionNotification() {
		if (!this.preferences.mentionMessage) return;
		this.playTone(700, 0.12);
		setTimeout(() => this.playTone(900, 0.12), 120);
		setTimeout(() => this.playTone(1100, 0.15), 240);
	}

	playVoiceJoin() {
		if (!this.preferences.voiceJoin) return;
		this.playTone(500, 0.1);
		setTimeout(() => this.playTone(700, 0.15), 100);
	}

	playVoiceLeave() {
		if (!this.preferences.voiceLeave) return;
		this.playTone(700, 0.1);
		setTimeout(() => this.playTone(500, 0.15), 100);
	}
}

export const soundStore = new SoundStore();
