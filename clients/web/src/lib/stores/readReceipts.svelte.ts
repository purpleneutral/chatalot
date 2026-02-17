interface ReadPosition {
	messageId: string;
	timestamp: string;
}

class ReadReceiptStore {
	// channelId -> userId -> { messageId, timestamp }
	private cursors = $state<Map<string, Map<string, ReadPosition>>>(new Map());

	/** Set the read position for a user in a channel. */
	setReadPosition(channelId: string, userId: string, messageId: string, timestamp: string) {
		const channelMap = new Map(this.cursors.get(channelId) ?? new Map());
		channelMap.set(userId, { messageId, timestamp });
		const next = new Map(this.cursors);
		next.set(channelId, channelMap);
		this.cursors = next;
	}

	/** Bulk-set read positions (from REST on channel load). */
	setChannelCursors(channelId: string, cursors: Array<{ user_id: string; last_read_message_id: string | null; last_read_at: string }>) {
		const channelMap = new Map<string, ReadPosition>();
		for (const c of cursors) {
			if (c.last_read_message_id) {
				channelMap.set(c.user_id, { messageId: c.last_read_message_id, timestamp: c.last_read_at });
			}
		}
		const next = new Map(this.cursors);
		next.set(channelId, channelMap);
		this.cursors = next;
	}

	/** Get all user IDs who have read up to or past a specific message.
	 *  Uses message list ordering to determine "read up to". */
	getReadersAtMessage(channelId: string, messageId: string, messages: Array<{ id: string }>, myUserId: string): string[] {
		const channelMap = this.cursors.get(channelId);
		if (!channelMap) return [];

		const msgIndex = messages.findIndex(m => m.id === messageId);
		if (msgIndex === -1) return [];

		const readers: string[] = [];
		for (const [userId, cursor] of channelMap) {
			if (userId === myUserId) continue;
			const cursorIndex = messages.findIndex(m => m.id === cursor.messageId);
			if (cursorIndex >= msgIndex) {
				readers.push(userId);
			}
		}
		return readers;
	}

	/** For DMs: get the furthest message the other user has read. */
	getLastReadMessageId(channelId: string, userId: string): string | null {
		return this.cursors.get(channelId)?.get(userId)?.messageId ?? null;
	}

	/** Get the timestamp when a user last read. */
	getLastReadTimestamp(channelId: string, userId: string): string | null {
		return this.cursors.get(channelId)?.get(userId)?.timestamp ?? null;
	}

	/** Clean up data for a channel. */
	clearChannel(channelId: string) {
		const next = new Map(this.cursors);
		next.delete(channelId);
		this.cursors = next;
	}

	clear() {
		this.cursors = new Map();
	}
}

export const readReceiptStore = new ReadReceiptStore();
