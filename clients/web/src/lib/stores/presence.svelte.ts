class PresenceStore {
	// userId -> status
	private statuses = $state<Map<string, string>>(new Map());
	// channelId -> Set of user IDs currently typing
	private typingUsers = $state<Map<string, Set<string>>>(new Map());

	getStatus(userId: string): string {
		return this.statuses.get(userId) ?? 'offline';
	}

	setStatus(userId: string, status: string) {
		const next = new Map(this.statuses);
		next.set(userId, status);
		this.statuses = next;
	}

	getTypingUsers(channelId: string): string[] {
		return Array.from(this.typingUsers.get(channelId) ?? []);
	}

	setTyping(channelId: string, userId: string) {
		const next = new Map(this.typingUsers);
		const users = new Set(next.get(channelId) ?? []);
		users.add(userId);
		next.set(channelId, users);
		this.typingUsers = next;

		// Auto-clear after 5 seconds
		setTimeout(() => this.clearTyping(channelId, userId), 5000);
	}

	clearTyping(channelId: string, userId: string) {
		const users = this.typingUsers.get(channelId);
		if (!users?.has(userId)) return;
		const next = new Map(this.typingUsers);
		const updated = new Set(users);
		updated.delete(userId);
		next.set(channelId, updated);
		this.typingUsers = next;
	}
}

export const presenceStore = new PresenceStore();
