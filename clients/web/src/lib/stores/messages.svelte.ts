/// Decrypted message for display.
/// In Phase 2, this is populated by decrypting the ciphertext client-side.
/// For now, we store the raw ciphertext and display a placeholder.
export interface ChatMessage {
	id: string;
	channelId: string;
	senderId: string;
	content: string;        // Decrypted plaintext (or placeholder)
	messageType: string;
	replyToId: string | null;
	editedAt: string | null;
	createdAt: string;
	pending?: boolean;      // Optimistic send, not yet confirmed
	reactions?: Map<string, Set<string>>; // emoji -> set of user IDs
}

export interface UnreadCount {
	channelId: string;
	count: number;
}

class MessageStore {
	// channelId -> messages (sorted by time ascending)
	private messagesByChannel = $state<Map<string, ChatMessage[]>>(new Map());
	private loadingChannels = $state<Set<string>>(new Set());
	private fetchedChannels = new Set<string>(); // channels whose history has been loaded via REST
	private unreadCounts = $state<Map<string, number>>(new Map());
	private noMoreMessages = new Set<string>(); // channels where we've loaded all history

	getMessages(channelId: string): ChatMessage[] {
		return this.messagesByChannel.get(channelId) ?? [];
	}

	isLoading(channelId: string): boolean {
		return this.loadingChannels.has(channelId);
	}

	hasLoadedHistory(channelId: string): boolean {
		return this.fetchedChannels.has(channelId);
	}

	setLoading(channelId: string, loading: boolean) {
		const next = new Set(this.loadingChannels);
		if (loading) next.add(channelId);
		else next.delete(channelId);
		this.loadingChannels = next;
	}

	setMessages(channelId: string, messages: ChatMessage[], fetchedLimit?: number) {
		this.fetchedChannels.add(channelId);
		// If we got fewer messages than requested, there are no more to load
		if (fetchedLimit !== undefined && messages.length < fetchedLimit) {
			this.noMoreMessages.add(channelId);
		}
		// Merge with any real-time messages that arrived before history was fetched
		const existing = this.messagesByChannel.get(channelId) ?? [];
		const historyIds = new Set(messages.map(m => m.id));
		const realtimeOnly = existing.filter(m => !historyIds.has(m.id));
		const next = new Map(this.messagesByChannel);
		next.set(channelId, [...messages, ...realtimeOnly]);
		this.messagesByChannel = next;
	}

	prependMessages(channelId: string, olderMessages: ChatMessage[], fetchedLimit: number) {
		if (olderMessages.length < fetchedLimit) {
			this.noMoreMessages.add(channelId);
		}
		const existing = this.messagesByChannel.get(channelId) ?? [];
		const existingIds = new Set(existing.map(m => m.id));
		const newMsgs = olderMessages.filter(m => !existingIds.has(m.id));
		if (newMsgs.length === 0) return;
		const next = new Map(this.messagesByChannel);
		next.set(channelId, [...newMsgs, ...existing]);
		this.messagesByChannel = next;
	}

	hasMore(channelId: string): boolean {
		return !this.noMoreMessages.has(channelId);
	}

	addMessage(channelId: string, message: ChatMessage) {
		const existing = this.messagesByChannel.get(channelId) ?? [];
		// Don't add duplicates
		if (existing.some(m => m.id === message.id)) return;
		const next = new Map(this.messagesByChannel);
		next.set(channelId, [...existing, message]);
		this.messagesByChannel = next;
	}

	// Confirm a pending message (replace temp ID)
	confirmMessage(channelId: string, tempId: string, realId: string, createdAt: string) {
		const messages = this.messagesByChannel.get(channelId);
		if (!messages) return;
		const next = new Map(this.messagesByChannel);
		next.set(
			channelId,
			messages.map(m =>
				m.id === tempId ? { ...m, id: realId, createdAt, pending: false } : m
			)
		);
		this.messagesByChannel = next;
	}

	// Edit a message
	editMessage(messageId: string, content: string, editedAt: string) {
		const next = new Map(this.messagesByChannel);
		for (const [channelId, messages] of next) {
			const idx = messages.findIndex(m => m.id === messageId);
			if (idx !== -1) {
				next.set(
					channelId,
					messages.map(m =>
						m.id === messageId ? { ...m, content, editedAt } : m
					)
				);
				break;
			}
		}
		this.messagesByChannel = next;
	}

	// Delete a message
	deleteMessage(messageId: string) {
		const next = new Map(this.messagesByChannel);
		for (const [channelId, messages] of next) {
			const idx = messages.findIndex(m => m.id === messageId);
			if (idx !== -1) {
				next.set(
					channelId,
					messages.filter(m => m.id !== messageId)
				);
				break;
			}
		}
		this.messagesByChannel = next;
	}

	// Add a reaction
	addReaction(messageId: string, userId: string, emoji: string) {
		const next = new Map(this.messagesByChannel);
		for (const [channelId, messages] of next) {
			const idx = messages.findIndex(m => m.id === messageId);
			if (idx !== -1) {
				next.set(
					channelId,
					messages.map(m => {
						if (m.id !== messageId) return m;
						const reactions = new Map(m.reactions ?? new Map());
						const users = new Set(reactions.get(emoji) ?? new Set());
						users.add(userId);
						reactions.set(emoji, users);
						return { ...m, reactions };
					})
				);
				break;
			}
		}
		this.messagesByChannel = next;
	}

	// Remove a reaction
	removeReaction(messageId: string, userId: string, emoji: string) {
		const next = new Map(this.messagesByChannel);
		for (const [channelId, messages] of next) {
			const idx = messages.findIndex(m => m.id === messageId);
			if (idx !== -1) {
				next.set(
					channelId,
					messages.map(m => {
						if (m.id !== messageId) return m;
						const reactions = new Map(m.reactions ?? new Map());
						const users = new Set(reactions.get(emoji) ?? new Set());
						users.delete(userId);
						if (users.size === 0) reactions.delete(emoji);
						else reactions.set(emoji, users);
						return { ...m, reactions };
					})
				);
				break;
			}
		}
		this.messagesByChannel = next;
	}

	// Unread tracking
	getUnreadCount(channelId: string): number {
		return this.unreadCounts.get(channelId) ?? 0;
	}

	setUnreadCounts(counts: { channel_id: string; unread_count: number }[]) {
		const next = new Map<string, number>();
		for (const c of counts) {
			next.set(c.channel_id, c.unread_count);
		}
		this.unreadCounts = next;
	}

	incrementUnread(channelId: string) {
		const next = new Map(this.unreadCounts);
		next.set(channelId, (next.get(channelId) ?? 0) + 1);
		this.unreadCounts = next;
	}

	clearUnread(channelId: string) {
		const next = new Map(this.unreadCounts);
		next.set(channelId, 0);
		this.unreadCounts = next;
	}
}

export const messageStore = new MessageStore();
