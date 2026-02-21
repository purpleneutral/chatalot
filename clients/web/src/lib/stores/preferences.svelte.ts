import { getPreferences, updatePreferences } from '$lib/api/preferences';
import { themeStore, type Theme } from '$lib/stores/theme.svelte';

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
export type SidebarLayout = 'expanded' | 'compact';
export type VoiceActivationMode = 'open-mic' | 'push-to-talk' | 'toggle-mute';
export type VoiceBackgroundType = 'none' | 'solid' | 'gradient' | 'preset' | 'custom';

export interface VoiceBackground {
	type: VoiceBackgroundType;
	color?: string;
	gradientFrom?: string;
	gradientTo?: string;
	gradientAngle?: number;
	presetId?: string;
	customUrl?: string;
}

export const VOICE_BG_PRESETS: Record<string, { label: string; css: string }> = {
	fireplace: {
		label: 'Fireplace',
		css: 'linear-gradient(135deg, #ff6b35 0%, #d62828 40%, #f77f00 60%, #d62828 100%)'
	},
	aurora: {
		label: 'Aurora',
		css: 'linear-gradient(135deg, #0d7377 0%, #14ffec 30%, #0d7377 60%, #32e0c4 100%)'
	},
	rain: {
		label: 'Rain',
		css: 'linear-gradient(180deg, #0c1445 0%, #1a237e 50%, #283593 100%)'
	},
	sunset: {
		label: 'Sunset',
		css: 'linear-gradient(135deg, #ff6b2b 0%, #ee5a24 30%, #9b59b6 70%, #6c3483 100%)'
	},
	space: {
		label: 'Space',
		css: 'radial-gradient(ellipse at 20% 50%, #1a1a2e 0%, #0a0a14 50%, #000 100%)'
	},
	cozy: {
		label: 'Cozy',
		css: 'linear-gradient(135deg, #8b6914 0%, #a0522d 40%, #6b4226 70%, #3e2723 100%)'
	}
};

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
	voiceBackground: VoiceBackground;
	sendReadReceipts: boolean;
	sidebarLayout: SidebarLayout;
	voiceActivationMode: VoiceActivationMode;
	pttKey: string;
	toggleMuteKey: string;
	theme: Theme;
}

const DEFAULT_CUSTOM_COLORS: CustomThemeColors = {
	bgPrimary: '#1a1816',
	bgSecondary: '#221f1c',
	bgTertiary: '#2d2926',
	textPrimary: '#ede8e2',
	textSecondary: '#8a8078',
	accent: '#e07a4f',
	accentHover: '#eb9670'
};

const DEFAULTS: UserPreferences = {
	accentColor: 'orange',
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
	customThemeColors: { ...DEFAULT_CUSTOM_COLORS },
	voiceBackground: { type: 'none' },
	sendReadReceipts: true,
	sidebarLayout: 'expanded',
	voiceActivationMode: 'open-mic',
	pttKey: ' ',
	toggleMuteKey: 'm',
	theme: 'system'
};

export const PRESET_THEMES: Record<PresetTheme, { label: string; colors: { dark: CustomThemeColors; light: CustomThemeColors } }> = {
	default: {
		label: 'Default',
		colors: {
			dark: { bgPrimary: '#1a1816', bgSecondary: '#221f1c', bgTertiary: '#2d2926', textPrimary: '#ede8e2', textSecondary: '#8a8078', accent: '#e07a4f', accentHover: '#eb9670' },
			light: { bgPrimary: '#f8f5f0', bgSecondary: '#ffffff', bgTertiary: '#f0ebe4', textPrimary: '#2c2520', textSecondary: '#7a6e63', accent: '#c75d35', accentHover: '#e07a4f' }
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
	blue: { main: '#4e8fda', hover: '#6ba3e8', mainLight: '#3574c2', hoverLight: '#4e8fda' },
	purple: { main: '#9c7ae0', hover: '#b394ea', mainLight: '#7c5ec4', hoverLight: '#9c7ae0' },
	green: { main: '#57ab5a', hover: '#73c176', mainLight: '#3d8c40', hoverLight: '#57ab5a' },
	orange: { main: '#e07a4f', hover: '#eb9670', mainLight: '#c75d35', hoverLight: '#e07a4f' },
	red: { main: '#e5534b', hover: '#f08078', mainLight: '#cc3d35', hoverLight: '#e5534b' },
	pink: { main: '#d96ba0', hover: '#e88dbb', mainLight: '#c44e87', hoverLight: '#d96ba0' },
	teal: { main: '#4ecdc4', hover: '#70dbd4', mainLight: '#2eb5ac', hoverLight: '#4ecdc4' },
	cyan: { main: '#56b6c2', hover: '#78c8d2', mainLight: '#3a9eab', hoverLight: '#56b6c2' }
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
		// Sync saved theme preference to ThemeStore
		if (this.preferences.theme && this.preferences.theme !== 'system') {
			themeStore.set(this.preferences.theme);
		}
		this.applyToDOM();

		// Re-apply when dark/light mode changes
		themeStore.onThemeChange = () => this.applyToDOM();
	}

	/** Update a single preference and persist. */
	set<K extends keyof UserPreferences>(key: K, value: UserPreferences[K]) {
		this.preferences = { ...this.preferences, [key]: value };
		// Keep ThemeStore in sync when theme preference changes
		if (key === 'theme') themeStore.set(value as Theme);
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

	/** Load from server and merge with local (local values preserved for keys server doesn't have). */
	async loadFromServer() {
		try {
			const serverPrefs = await getPreferences();
			if (serverPrefs && typeof serverPrefs === 'object') {
				this.preferences = { ...this.preferences, ...serverPrefs } as UserPreferences;
				this.saveLocal();
				// Sync theme preference to ThemeStore (server is source of truth)
				if (this.preferences.theme) {
					themeStore.set(this.preferences.theme);
				}
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

	/** Cancel pending sync timer (call on logout). */
	cancelPendingSync() {
		if (this.syncTimer) {
			clearTimeout(this.syncTimer);
			this.syncTimer = null;
		}
	}
}

export const preferencesStore = new PreferencesStore();

/** Sanitize a string for safe use inside a CSS url() value. */
function safeCssUrl(url: string): string | null {
	// Block characters that can break out of url("...") or inject CSS
	if (/[;'"\\(){}]/.test(url)) return null;
	return url;
}

/** Sanitize a CSS color value (hex, rgb, hsl, named color). */
function safeCssColor(val: string): string | null {
	// Allow hex, rgb(), hsl(), oklch(), named colors — block injection chars
	if (/[;'"\\(){}]/.test(val.replace(/\([^)]*\)/g, ''))) return null;
	return val;
}

/** Compute inline CSS for a voice background config. Returns empty string for 'none'. */
export function voiceBackgroundStyle(bg: VoiceBackground): string {
	switch (bg.type) {
		case 'solid': {
			const c = bg.color ? safeCssColor(bg.color) : null;
			return c ? `background: ${c};` : '';
		}
		case 'gradient': {
			const from = bg.gradientFrom ? safeCssColor(bg.gradientFrom) : null;
			const to = bg.gradientTo ? safeCssColor(bg.gradientTo) : null;
			return from && to
				? `background: linear-gradient(${bg.gradientAngle ?? 135}deg, ${from}, ${to});`
				: '';
		}
		case 'preset': {
			const preset = bg.presetId ? VOICE_BG_PRESETS[bg.presetId] : null;
			return preset ? `background: ${preset.css};` : '';
		}
		case 'custom': {
			const url = bg.customUrl ? safeCssUrl(bg.customUrl) : null;
			return url ? `background: url("${url}") center/cover no-repeat;` : '';
		}
		default:
			return '';
	}
}
