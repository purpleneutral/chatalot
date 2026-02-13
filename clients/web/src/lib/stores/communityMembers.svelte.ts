import type { CommunityMember } from '$lib/api/communities';

class CommunityMemberStore {
	private membersByCommunity = $state<Map<string, CommunityMember[]>>(new Map());

	getMembers(communityId: string): CommunityMember[] {
		return this.membersByCommunity.get(communityId) ?? [];
	}

	getMember(communityId: string, userId: string): CommunityMember | undefined {
		return this.getMembers(communityId).find((m) => m.user_id === userId);
	}

	getNickname(communityId: string, userId: string): string | null {
		return this.getMember(communityId, userId)?.nickname ?? null;
	}

	setMembers(communityId: string, members: CommunityMember[]) {
		const next = new Map(this.membersByCommunity);
		next.set(communityId, members);
		this.membersByCommunity = next;
	}

	updateNickname(communityId: string, userId: string, nickname: string | null) {
		const members = this.getMembers(communityId);
		const next = new Map(this.membersByCommunity);
		next.set(
			communityId,
			members.map((m) => (m.user_id === userId ? { ...m, nickname } : m))
		);
		this.membersByCommunity = next;
	}

	clear() {
		this.membersByCommunity = new Map();
	}
}

export const communityMemberStore = new CommunityMemberStore();
