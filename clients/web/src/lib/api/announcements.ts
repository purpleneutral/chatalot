import { api } from './client';

export interface Announcement {
	id: string;
	title: string;
	body: string;
	created_by: string;
	created_at: string;
}

export async function listUndismissed(): Promise<Announcement[]> {
	return api.get('/announcements');
}

export async function dismissAnnouncement(id: string): Promise<void> {
	return api.post(`/announcements/${id}/dismiss`, {});
}

export async function createAnnouncement(title: string, body: string): Promise<Announcement> {
	return api.post('/admin/announcements', { title, body });
}

export async function listAllAnnouncements(): Promise<Announcement[]> {
	return api.get('/admin/announcements');
}
