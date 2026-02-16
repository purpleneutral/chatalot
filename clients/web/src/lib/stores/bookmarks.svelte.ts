import type { Bookmark } from '$lib/api/bookmarks';

class BookmarkStore {
	bookmarks = $state<Bookmark[]>([]);
	// Set of message IDs that are bookmarked (for fast lookup)
	private bookmarkedMessageIds = $state<Set<string>>(new Set());

	setBookmarks(bookmarks: Bookmark[]) {
		this.bookmarks = bookmarks;
		this.bookmarkedMessageIds = new Set(bookmarks.map(b => b.message_id));
	}

	addBookmark(bookmark: Bookmark) {
		// Deduplicate: skip if this message is already bookmarked
		if (this.bookmarkedMessageIds.has(bookmark.message_id)) return;
		this.bookmarks = [...this.bookmarks, bookmark];
		const next = new Set(this.bookmarkedMessageIds);
		next.add(bookmark.message_id);
		this.bookmarkedMessageIds = next;
	}

	removeBookmark(id: string) {
		const bookmark = this.bookmarks.find(b => b.id === id);
		this.bookmarks = this.bookmarks.filter(b => b.id !== id);
		if (bookmark) {
			const next = new Set(this.bookmarkedMessageIds);
			next.delete(bookmark.message_id);
			this.bookmarkedMessageIds = next;
		}
	}

	isBookmarked(messageId: string): boolean {
		return this.bookmarkedMessageIds.has(messageId);
	}

	getByMessageId(messageId: string): Bookmark | undefined {
		return this.bookmarks.find(b => b.message_id === messageId);
	}
}

export const bookmarkStore = new BookmarkStore();
