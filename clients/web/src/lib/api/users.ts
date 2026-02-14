import { api } from './client';

export interface UserPublic {
	id: string;
	username: string;
	display_name: string;
	avatar_url: string | null;
	status: string;
	custom_status: string | null;
	is_admin?: boolean;
	is_owner?: boolean;
	created_at?: string;
}

export async function searchUsers(query: string): Promise<UserPublic[]> {
	return api.get<UserPublic[]>(`/users/search?q=${encodeURIComponent(query)}`);
}

export async function getUser(userId: string): Promise<UserPublic> {
	return api.get<UserPublic>(`/users/${userId}`);
}
