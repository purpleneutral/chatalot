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
	desktopNotifyChannel: false
};

export const ACCENT_COLORS: Record<
	AccentColor,
	{ main: string; hover: string; mainLight: string; hoverLight: string }
> = {
	blue: { main: '#3b82f6', hover: '#60a5fa', mainLight: '#2563eb', hoverLight: '#3b82f6' },
	purple: { main: '#6366f1', hover: '#818cf8', mainLight: '#4f46e5', hoverLight: '#6366f1' },
	green: { main: '#22c55e', hover: '#4ade80', mainLight: '#16a34a', hoverLight: '#22c55e' },
	orange: { main: '#f97316', hover: '#fb923c', mainLight: '#ea580c', hoverLight: '#f97316' },
	red: { main: '#ef4444', hover: '#f87171', mainLight: '#dc2626', hoverLight: '#ef4444' },
	pink: { main: '#ec4899', hover: '#f472b6', mainLight: '#db2777', hoverLight: '#ec4899' },
	teal: { main: '#14b8a6', hover: '#2dd4bf', mainLight: '#0d9488', hoverLight: '#14b8a6' },
	cyan: { main: '#06b6d4', hover: '#22d3ee', mainLight: '#0891b2', hoverLight: '#06b6d4' }
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
