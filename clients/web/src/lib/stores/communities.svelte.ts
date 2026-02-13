import type { Community } from '$lib/api/communities';

const ACTIVE_KEY = 'chatalot:activeCommunity';

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
