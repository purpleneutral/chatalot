export type Theme = 'dark' | 'light' | 'system';

class ThemeStore {
	current = $state<Theme>('system');
	private systemDark = $state(true);

	/** The actual dark/light value after resolving 'system'. */
	get resolved(): 'dark' | 'light' {
		if (this.current === 'system') {
			return this.systemDark ? 'dark' : 'light';
		}
		return this.current;
	}

	constructor() {
		if (typeof window !== 'undefined') {
			// Detect system preference
			const mq = window.matchMedia('(prefers-color-scheme: dark)');
			this.systemDark = mq.matches;
			mq.addEventListener('change', (e) => {
				this.systemDark = e.matches;
				if (this.current === 'system') {
					this.apply();
				}
			});

			// Load saved preference
			const saved = localStorage.getItem('chatalot-theme') as Theme | null;
			if (saved === 'light' || saved === 'dark' || saved === 'system') {
				this.current = saved;
			}
			this.apply();
		}
	}

	toggle() {
		this.current = this.resolved === 'dark' ? 'light' : 'dark';
		this.apply();
	}

	set(theme: Theme) {
		this.current = theme;
		this.apply();
	}

	private apply() {
		if (typeof document === 'undefined') return;
		document.documentElement.setAttribute('data-theme', this.resolved);
		localStorage.setItem('chatalot-theme', this.current);
		// Re-apply preferences so preset theme colors update for new dark/light mode
		this.onThemeChange?.();
	}

	/** Callback to re-apply preferences when theme changes. Set by PreferencesStore. */
	onThemeChange: (() => void) | null = null;
}

export const themeStore = new ThemeStore();
