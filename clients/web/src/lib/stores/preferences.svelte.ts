import { getPreferences, updatePreferences } from '$lib/api/preferences';
import { themeStore } from '$lib/stores/theme.svelte';

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
export type ChatBubbleStyle = 'flat' | 'bubbles';
export type PresetTheme = 'default' | 'monokai' | 'dracula' | 'nord' | 'solarized' | 'amoled' | 'catppuccin' | 'custom';

export interface CustomThemeColors {
	bgPrimary: string;
	bgSecondary: string;
	bgTertiary: string;
	textPrimary: string;
	textSecondary: string;
	accent: string;
	accentHover: string;
}

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
	chatBubbleStyle: ChatBubbleStyle;
	animatedAccent: boolean;
	relativeTimestamps: boolean;
	reduceMotion: boolean;
	presetTheme: PresetTheme;
	customThemeColors: CustomThemeColors;
}

const DEFAULT_CUSTOM_COLORS: CustomThemeColors = {
	bgPrimary: '#08080f',
	bgSecondary: '#0f0f1a',
	bgTertiary: '#1a1a2e',
	textPrimary: '#e2e0f0',
	textSecondary: '#7a7a98',
	accent: '#7c3aed',
	accentHover: '#9d5cff'
};

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
	autoHideParticipantsOnStream: false,
	chatBubbleStyle: 'flat',
	animatedAccent: false,
	relativeTimestamps: false,
	reduceMotion: false,
	presetTheme: 'default',
	customThemeColors: { ...DEFAULT_CUSTOM_COLORS }
};

export const PRESET_THEMES: Record<PresetTheme, { label: string; colors: { dark: CustomThemeColors; light: CustomThemeColors } }> = {
	default: {
		label: 'Default',
		colors: {
			dark: { bgPrimary: '#08080f', bgSecondary: '#0f0f1a', bgTertiary: '#1a1a2e', textPrimary: '#e2e0f0', textSecondary: '#7a7a98', accent: '#7c3aed', accentHover: '#9d5cff' },
			light: { bgPrimary: '#f5f3ff', bgSecondary: '#ffffff', bgTertiary: '#ede9fe', textPrimary: '#1a1035', textSecondary: '#6b6190', accent: '#6d28d9', accentHover: '#7c3aed' }
		}
	},
	monokai: {
		label: 'Monokai',
		colors: {
			dark: { bgPrimary: '#272822', bgSecondary: '#1e1f1c', bgTertiary: '#3e3d32', textPrimary: '#f8f8f2', textSecondary: '#75715e', accent: '#a6e22e', accentHover: '#c4f060' },
			light: { bgPrimary: '#fafaf5', bgSecondary: '#ffffff', bgTertiary: '#f0f0e0', textPrimary: '#272822', textSecondary: '#75715e', accent: '#669900', accentHover: '#7ab300' }
		}
	},
	dracula: {
		label: 'Dracula',
		colors: {
			dark: { bgPrimary: '#282a36', bgSecondary: '#21222c', bgTertiary: '#343746', textPrimary: '#f8f8f2', textSecondary: '#6272a4', accent: '#bd93f9', accentHover: '#d4b0ff' },
			light: { bgPrimary: '#f8f8f2', bgSecondary: '#ffffff', bgTertiary: '#e8e8e0', textPrimary: '#282a36', textSecondary: '#6272a4', accent: '#7c3aed', accentHover: '#9d5cff' }
		}
	},
	nord: {
		label: 'Nord',
		colors: {
			dark: { bgPrimary: '#2e3440', bgSecondary: '#272c36', bgTertiary: '#3b4252', textPrimary: '#eceff4', textSecondary: '#7b88a1', accent: '#88c0d0', accentHover: '#a3d4e0' },
			light: { bgPrimary: '#eceff4', bgSecondary: '#ffffff', bgTertiary: '#d8dee9', textPrimary: '#2e3440', textSecondary: '#4c566a', accent: '#5e81ac', accentHover: '#7b9cc4' }
		}
	},
	solarized: {
		label: 'Solarized',
		colors: {
			dark: { bgPrimary: '#002b36', bgSecondary: '#001e27', bgTertiary: '#073642', textPrimary: '#fdf6e3', textSecondary: '#839496', accent: '#268bd2', accentHover: '#4ea3e0' },
			light: { bgPrimary: '#fdf6e3', bgSecondary: '#ffffff', bgTertiary: '#eee8d5', textPrimary: '#002b36', textSecondary: '#657b83', accent: '#268bd2', accentHover: '#4ea3e0' }
		}
	},
	amoled: {
		label: 'AMOLED',
		colors: {
			dark: { bgPrimary: '#000000', bgSecondary: '#0a0a0a', bgTertiary: '#141414', textPrimary: '#ffffff', textSecondary: '#666666', accent: '#7c3aed', accentHover: '#9d5cff' },
			light: { bgPrimary: '#f5f3ff', bgSecondary: '#ffffff', bgTertiary: '#ede9fe', textPrimary: '#1a1035', textSecondary: '#6b6190', accent: '#6d28d9', accentHover: '#7c3aed' }
		}
	},
	catppuccin: {
		label: 'Catppuccin',
		colors: {
			dark: { bgPrimary: '#1e1e2e', bgSecondary: '#181825', bgTertiary: '#313244', textPrimary: '#cdd6f4', textSecondary: '#6c7086', accent: '#cba6f7', accentHover: '#d9bcff' },
			light: { bgPrimary: '#eff1f5', bgSecondary: '#ffffff', bgTertiary: '#dce0e8', textPrimary: '#4c4f69', textSecondary: '#6c6f85', accent: '#8839ef', accentHover: '#a05cf5' }
		}
	},
	custom: {
		label: 'Custom',
		colors: {
			dark: { ...DEFAULT_CUSTOM_COLORS },
			light: { ...DEFAULT_CUSTOM_COLORS }
		}
	}
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

		// Re-apply when dark/light mode changes
		themeStore.onThemeChange = () => this.applyToDOM();
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
		const prefs = this.preferences;

		// Apply preset theme colors
		const preset = prefs.presetTheme;
		if (preset !== 'default') {
			const themeColors = preset === 'custom'
				? prefs.customThemeColors
				: PRESET_THEMES[preset]?.colors[theme as 'dark' | 'light'];
			if (themeColors) {
				el.style.setProperty('--bg-primary', themeColors.bgPrimary);
				el.style.setProperty('--bg-secondary', themeColors.bgSecondary);
				el.style.setProperty('--bg-tertiary', themeColors.bgTertiary);
				el.style.setProperty('--text-primary', themeColors.textPrimary);
				el.style.setProperty('--text-secondary', themeColors.textSecondary);
				el.style.setProperty('--accent', themeColors.accent);
				el.style.setProperty('--accent-hover', themeColors.accentHover);
			}
		} else {
			// Default theme: clear overrides so CSS takes over, then apply accent color
			el.style.removeProperty('--bg-primary');
			el.style.removeProperty('--bg-secondary');
			el.style.removeProperty('--bg-tertiary');
			el.style.removeProperty('--text-primary');
			el.style.removeProperty('--text-secondary');

			const colors = ACCENT_COLORS[prefs.accentColor];
			if (theme === 'light') {
				el.style.setProperty('--accent', colors.mainLight);
				el.style.setProperty('--accent-hover', colors.hoverLight);
			} else {
				el.style.setProperty('--accent', colors.main);
				el.style.setProperty('--accent-hover', colors.hover);
			}
		}

		el.style.setProperty('--font-size-base', FONT_SIZES[prefs.fontSize]);

		// Reduce motion
		el.classList.toggle('reduce-motion', prefs.reduceMotion);

		// Animated accent
		el.classList.toggle('animated-accent', prefs.animatedAccent);

		// Chat bubble style
		el.setAttribute('data-bubbles', prefs.chatBubbleStyle === 'bubbles' ? 'true' : 'false');
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
