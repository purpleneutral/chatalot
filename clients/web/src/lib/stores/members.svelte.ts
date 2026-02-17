import type { ChannelMember } from '$lib/api/channels';

class MemberStore {
	private membersByChannel = $state<Map<string, ChannelMember[]>>(new Map());

	getMembers(channelId: string): ChannelMember[] {
		return this.membersByChannel.get(channelId) ?? [];
	}

	setMembers(channelId: string, members: ChannelMember[]) {
		const next = new Map(this.membersByChannel);
		next.set(channelId, members);
		this.membersByChannel = next;
	}

	getMyRole(channelId: string, userId: string): string {
		const members = this.membersByChannel.get(channelId) ?? [];
		return members.find((m) => m.user_id === userId)?.role ?? 'member';
	}

	updateMemberRole(channelId: string, userId: string, newRole: string) {
		const members = this.membersByChannel.get(channelId);
		if (!members) return;
		const next = new Map(this.membersByChannel);
		next.set(
			channelId,
			members.map((m) => (m.user_id === userId ? { ...m, role: newRole } : m))
		);
		this.membersByChannel = next;
	}

	removeMember(channelId: string, userId: string) {
		const members = this.membersByChannel.get(channelId);
		if (!members) return;
		const next = new Map(this.membersByChannel);
		next.set(
			channelId,
			members.filter((m) => m.user_id !== userId)
		);
		this.membersByChannel = next;
	}

	addMember(channelId: string, member: ChannelMember) {
		const members = this.membersByChannel.get(channelId) ?? [];
		if (members.some((m) => m.user_id === member.user_id)) return;
		const next = new Map(this.membersByChannel);
		next.set(channelId, [...members, member]);
		this.membersByChannel = next;
	}

	clearChannel(channelId: string) {
		if (!this.membersByChannel.has(channelId)) return;
		const next = new Map(this.membersByChannel);
		next.delete(channelId);
		this.membersByChannel = next;
	}

	clear() {
		this.membersByChannel = new Map();
	}
}

export const memberStore = new MemberStore();
