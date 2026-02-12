export type Theme = 'dark' | 'light';

class ThemeStore {
	current = $state<Theme>('dark');

	constructor() {
		if (typeof window !== 'undefined') {
			const saved = localStorage.getItem('chatalot-theme') as Theme | null;
			if (saved === 'light' || saved === 'dark') {
				this.current = saved;
			}
			this.apply();
		}
	}

	toggle() {
		this.current = this.current === 'dark' ? 'light' : 'dark';
		this.apply();
	}

	set(theme: Theme) {
		this.current = theme;
		this.apply();
	}

	private apply() {
		if (typeof document === 'undefined') return;
		document.documentElement.setAttribute('data-theme', this.current);
		localStorage.setItem('chatalot-theme', this.current);
	}
}

export const themeStore = new ThemeStore();
