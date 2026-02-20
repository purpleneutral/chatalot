import type { Community } from '$lib/api/communities';

const ACTIVE_KEY = 'chatalot:activeCommunity';

const THEME_CSS_VARS: Record<string, string> = {
	accent: '--accent',
	accentHover: '--accent-hover',
	bgPrimary: '--bg-primary',
	bgSecondary: '--bg-secondary',
	bgTertiary: '--bg-tertiary',
	textPrimary: '--text-primary',
	textSecondary: '--text-secondary'
};

class CommunityStore {
	communities = $state<Community[]>([]);
	activeCommunityId = $state<string | null>(null);

	get activeCommunity(): Community | undefined {
		return this.communities.find((c) => c.id === this.activeCommunityId);
	}

	constructor() {
		if (typeof window !== 'undefined') {
			this.activeCommunityId = localStorage.getItem(ACTIVE_KEY);
		}
	}

	setCommunities(communities: Community[]) {
		this.communities = communities;
		// If active community no longer exists, pick the first one
		if (this.activeCommunityId && !communities.find((c) => c.id === this.activeCommunityId)) {
			this.activeCommunityId = communities[0]?.id ?? null;
			this.persist();
		}
		// Apply theme for active community
		if (this.activeCommunityId) {
			const active = communities.find((c) => c.id === this.activeCommunityId);
			this.applyCommunityTheme(active?.community_theme ?? null);
		}
	}

	addCommunity(community: Community) {
		if (!this.communities.find((c) => c.id === community.id)) {
			this.communities = [...this.communities, community];
		}
	}

	updateCommunity(id: string, updates: Partial<Community>) {
		this.communities = this.communities.map((c) => (c.id === id ? { ...c, ...updates } : c));
	}

	removeCommunity(id: string) {
		this.communities = this.communities.filter((c) => c.id !== id);
		if (this.activeCommunityId === id) {
			this.activeCommunityId = this.communities[0]?.id ?? null;
			this.persist();
		}
	}

	setActive(communityId: string | null) {
		this.activeCommunityId = communityId;
		this.persist();
		// Apply theme for new active community
		const community = communityId
			? this.communities.find((c) => c.id === communityId)
			: undefined;
		this.applyCommunityTheme(community?.community_theme ?? null);
	}

	applyCommunityTheme(theme: Record<string, string> | null) {
		if (typeof window === 'undefined') return;
		const root = document.documentElement;

		// Remove any previous community overrides
		for (const cssVar of Object.values(THEME_CSS_VARS)) {
			root.style.removeProperty(cssVar);
		}
		// Remove custom CSS
		document.getElementById('community-custom-css')?.remove();

		if (!theme) return;

		// Apply color overrides
		for (const [key, cssVar] of Object.entries(THEME_CSS_VARS)) {
			if (theme[key]) {
				root.style.setProperty(cssVar, theme[key]);
			}
		}

		// Inject custom CSS (defense-in-depth: server already sanitizes, but reject
		// obviously dangerous patterns client-side too)
		if (theme.customCss) {
			const css = theme.customCss;
			const dangerous = /url\s*\(|@import|expression\s*\(|javascript:|behavior\s*:|binding\s*:|moz-binding/i;
			if (!dangerous.test(css)) {
				const style = document.createElement('style');
				style.id = 'community-custom-css';
				style.textContent = css;
				document.head.appendChild(style);
			}
		}
	}

	clear() {
		this.communities = [];
		this.activeCommunityId = null;
		localStorage.removeItem(ACTIVE_KEY);
		this.applyCommunityTheme(null);
	}

	private persist() {
		if (this.activeCommunityId) {
			localStorage.setItem(ACTIVE_KEY, this.activeCommunityId);
		} else {
			localStorage.removeItem(ACTIVE_KEY);
		}
	}
}

export const communityStore = new CommunityStore();
