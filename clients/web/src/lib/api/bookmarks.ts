import { api } from './client';

export interface Bookmark {
	id: string;
	message_id: string;
	note: string | null;
	created_at: string;
}

export async function addBookmark(messageId: string, note?: string): Promise<Bookmark> {
	return api.post('/bookmarks', { message_id: messageId, note: note ?? null });
}

export async function listBookmarks(): Promise<Bookmark[]> {
	return api.get('/bookmarks');
}

export async function removeBookmark(id: string): Promise<void> {
	return api.delete(`/bookmarks/${id}`);
}
