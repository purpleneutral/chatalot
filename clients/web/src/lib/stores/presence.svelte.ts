export type PresenceStatus = 'online' | 'idle' | 'dnd' | 'invisible' | 'offline';

class PresenceStore {
	// userId -> status
	private statuses = $state<Map<string, PresenceStatus>>(new Map());
	// channelId -> Set of user IDs currently typing
	private typingUsers = $state<Map<string, Set<string>>>(new Map());
	// Track typing timeouts to cancel them (key: "channelId:userId")
	private typingTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

	getStatus(userId: string): PresenceStatus {
		return this.statuses.get(userId) ?? 'offline';
	}

	setStatus(userId: string, status: PresenceStatus) {
		if (this.statuses.get(userId) === status) return;
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

		// Cancel existing timeout for this user/channel and set a new one
		const key = `${channelId}:${userId}`;
		const existing = this.typingTimeouts.get(key);
		if (existing) clearTimeout(existing);
		this.typingTimeouts.set(key, setTimeout(() => {
			this.clearTyping(channelId, userId);
			this.typingTimeouts.delete(key);
		}, 5000));
	}

	clearTyping(channelId: string, userId: string) {
		const users = this.typingUsers.get(channelId);
		if (!users?.has(userId)) return;
		const next = new Map(this.typingUsers);
		const updated = new Set(users);
		updated.delete(userId);
		next.set(channelId, updated);
		this.typingUsers = next;

		// Clean up the timeout
		const key = `${channelId}:${userId}`;
		const timeout = this.typingTimeouts.get(key);
		if (timeout) {
			clearTimeout(timeout);
			this.typingTimeouts.delete(key);
		}
	}

	clearChannel(channelId: string) {
		const users = this.typingUsers.get(channelId);
		if (!users) return;
		// Clean up timeouts for this channel
		for (const userId of users) {
			const key = `${channelId}:${userId}`;
			const timeout = this.typingTimeouts.get(key);
			if (timeout) {
				clearTimeout(timeout);
				this.typingTimeouts.delete(key);
			}
		}
		const next = new Map(this.typingUsers);
		next.delete(channelId);
		this.typingUsers = next;
	}

	/** Clear all state (call on logout). */
	reset() {
		for (const timeout of this.typingTimeouts.values()) clearTimeout(timeout);
		this.typingTimeouts.clear();
		this.typingUsers = new Map();
		this.statuses = new Map();
	}
}

export const presenceStore = new PresenceStore();
