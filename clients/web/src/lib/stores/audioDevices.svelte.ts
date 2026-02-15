/// Audio device enumeration and selection store.
/// Device IDs are browser-specific, so they're stored in localStorage only (not synced to server).

const INPUT_KEY = 'chatalot:inputDeviceId';
const OUTPUT_KEY = 'chatalot:outputDeviceId';

export interface AudioDevice {
	deviceId: string;
	label: string;
	kind: 'audioinput' | 'audiooutput';
}

class AudioDeviceStore {
	inputDevices = $state<AudioDevice[]>([]);
	outputDevices = $state<AudioDevice[]>([]);
	selectedInputId = $state<string>('');
	selectedOutputId = $state<string>('');

	private _boundDeviceChange = this.enumerateDevices.bind(this);

	constructor() {
		if (typeof localStorage !== 'undefined') {
			this.selectedInputId = localStorage.getItem(INPUT_KEY) ?? '';
			this.selectedOutputId = localStorage.getItem(OUTPUT_KEY) ?? '';
		}
		if (typeof navigator !== 'undefined' && navigator.mediaDevices) {
			// Use a bound handler so we can remove it if the module is re-executed (HMR)
			navigator.mediaDevices.removeEventListener('devicechange', this._boundDeviceChange);
			navigator.mediaDevices.addEventListener('devicechange', this._boundDeviceChange);
		}
	}

	async enumerateDevices(): Promise<void> {
		if (typeof navigator === 'undefined' || !navigator.mediaDevices) return;
		try {
			const devices = await navigator.mediaDevices.enumerateDevices();
			this.inputDevices = devices
				.filter((d): d is MediaDeviceInfo & { kind: 'audioinput' } => d.kind === 'audioinput')
				.map((d, i) => ({
					deviceId: d.deviceId,
					label: d.label || `Microphone ${i + 1}`,
					kind: 'audioinput' as const
				}));
			this.outputDevices = devices
				.filter((d): d is MediaDeviceInfo & { kind: 'audiooutput' } => d.kind === 'audiooutput')
				.map((d, i) => ({
					deviceId: d.deviceId,
					label: d.label || `Speaker ${i + 1}`,
					kind: 'audiooutput' as const
				}));

			// If selected device is gone, fall back to default
			if (this.selectedInputId && !this.inputDevices.some(d => d.deviceId === this.selectedInputId)) {
				this.setInputDevice('');
			}
			if (this.selectedOutputId && !this.outputDevices.some(d => d.deviceId === this.selectedOutputId)) {
				this.setOutputDevice('');
			}
		} catch (err) {
			console.warn('Failed to enumerate devices:', err);
		}
	}

	setInputDevice(deviceId: string): void {
		this.selectedInputId = deviceId;
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(INPUT_KEY, deviceId);
		}
	}

	setOutputDevice(deviceId: string): void {
		this.selectedOutputId = deviceId;
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(OUTPUT_KEY, deviceId);
		}
	}

	/** Whether the browser supports output device selection. */
	get supportsOutputSelection(): boolean {
		if (typeof HTMLAudioElement === 'undefined') return false;
		return 'setSinkId' in HTMLAudioElement.prototype;
	}
}

export const audioDeviceStore = new AudioDeviceStore();
