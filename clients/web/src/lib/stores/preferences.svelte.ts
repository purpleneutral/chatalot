import { getPreferences, updatePreferences } from '$lib/api/preferences';

export type AccentColor =
	| 'blue'
	| 'purple'
	| 'green'
	| 'orange'
	| 'red'
	| 'pink'
	| 'teal'
	| 'cyan';
export type MessageDensity = 'cozy' | 'compact';
export type TimeFormat = '12h' | '24h';
export type FontSize = 'small' | 'medium' | 'large';
export type SendBehavior = 'enter' | 'ctrl-enter';
export type NoiseSuppression = 'off' | 'noise-gate' | 'standard' | 'maximum';

export interface UserPreferences {
	accentColor: AccentColor;
	messageDensity: MessageDensity;
	timeFormat: TimeFormat;
	fontSize: FontSize;
	showLinkPreviews: boolean;
	sendBehavior: SendBehavior;
	showFormattingToolbar: boolean;
	desktopNotifyDm: boolean;
	desktopNotifyMention: boolean;
	desktopNotifyChannel: boolean;
	noiseSuppression: NoiseSuppression;
	inputGain: number;
	outputVolume: number;
	echoCancellation: boolean;
	autoGainControl: boolean;
	autoHideParticipantsOnStream: boolean;
}

const DEFAULTS: UserPreferences = {
	accentColor: 'purple',
	messageDensity: 'cozy',
	timeFormat: '12h',
	fontSize: 'medium',
	showLinkPreviews: true,
	sendBehavior: 'enter',
	showFormattingToolbar: true,
	desktopNotifyDm: true,
	desktopNotifyMention: true,
	desktopNotifyChannel: false,
	noiseSuppression: 'standard',
	inputGain: 100,
	outputVolume: 100,
	echoCancellation: true,
	autoGainControl: true,
	autoHideParticipantsOnStream: false
};

export const ACCENT_COLORS: Record<
	AccentColor,
	{ main: string; hover: string; mainLight: string; hoverLight: string }
> = {
	blue: { main: '#2563ff', hover: '#5b8aff', mainLight: '#1d4ed8', hoverLight: '#2563ff' },
	purple: { main: '#7c3aed', hover: '#9d5cff', mainLight: '#6d28d9', hoverLight: '#7c3aed' },
	green: { main: '#00d68a', hover: '#33ffaa', mainLight: '#059669', hoverLight: '#00d68a' },
	orange: { main: '#ff6b2b', hover: '#ff8f5c', mainLight: '#ea580c', hoverLight: '#ff6b2b' },
	red: { main: '#ff3366', hover: '#ff6690', mainLight: '#dc2660', hoverLight: '#ff3366' },
	pink: { main: '#ff2d9b', hover: '#ff66b8', mainLight: '#d6238a', hoverLight: '#ff2d9b' },
	teal: { main: '#00d4aa', hover: '#33ffd4', mainLight: '#0d9488', hoverLight: '#00d4aa' },
	cyan: { main: '#00bbff', hover: '#44d4ff', mainLight: '#0284c7', hoverLight: '#00bbff' }
};

export const FONT_SIZES: Record<FontSize, string> = {
	small: '13px',
	medium: '14px',
	large: '16px'
};

const STORAGE_KEY = 'chatalot:preferences';

class PreferencesStore {
	preferences = $state<UserPreferences>({ ...DEFAULTS });
	private syncTimer: ReturnType<typeof setTimeout> | null = null;

	constructor() {
		if (typeof localStorage !== 'undefined') {
			const saved = localStorage.getItem(STORAGE_KEY);
			if (saved) {
				try {
					this.preferences = { ...DEFAULTS, ...JSON.parse(saved) };
				} catch {
					/* ignore corrupt data */
				}
			}
		}
		this.applyToDOM();
	}

	/** Update a single preference and persist. */
	set<K extends keyof UserPreferences>(key: K, value: UserPreferences[K]) {
		this.preferences = { ...this.preferences, [key]: value };
		this.saveLocal();
		this.applyToDOM();
		this.debounceSyncToServer();
	}

	/** Replace all preferences (e.g. on login sync). */
	setAll(prefs: Partial<UserPreferences>) {
		this.preferences = { ...DEFAULTS, ...prefs };
		this.saveLocal();
		this.applyToDOM();
	}

	/** Load from server and merge with local. */
	async loadFromServer() {
		try {
			const serverPrefs = await getPreferences();
			if (serverPrefs && typeof serverPrefs === 'object') {
				this.preferences = { ...DEFAULTS, ...serverPrefs } as UserPreferences;
				this.saveLocal();
				this.applyToDOM();
			}
		} catch {
			// Server might not have prefs yet, use local
		}
	}

	/** Push current preferences to server. */
	async syncToServer() {
		try {
			await updatePreferences(this.preferences as unknown as Record<string, unknown>);
		} catch {
			// Silently fail — preferences are non-critical
		}
	}

	applyToDOM() {
		if (typeof document === 'undefined') return;
		const el = document.documentElement;
		const theme = el.getAttribute('data-theme') || 'dark';
		const colors = ACCENT_COLORS[this.preferences.accentColor];

		if (theme === 'light') {
			el.style.setProperty('--accent', colors.mainLight);
			el.style.setProperty('--accent-hover', colors.hoverLight);
		} else {
			el.style.setProperty('--accent', colors.main);
			el.style.setProperty('--accent-hover', colors.hover);
		}

		el.style.setProperty('--font-size-base', FONT_SIZES[this.preferences.fontSize]);
	}

	private saveLocal() {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(STORAGE_KEY, JSON.stringify(this.preferences));
		}
	}

	private debounceSyncToServer() {
		if (this.syncTimer) clearTimeout(this.syncTimer);
		this.syncTimer = setTimeout(() => this.syncToServer(), 2000);
	}
}

export const preferencesStore = new PreferencesStore();
