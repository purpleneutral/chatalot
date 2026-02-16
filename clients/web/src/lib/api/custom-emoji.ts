import { api } from './client';

export interface CustomEmoji {
	id: string;
	community_id: string;
	shortcode: string;
	url: string;
	content_type: string;
	uploaded_by: string;
	created_at: string;
}

export async function listCommunityEmojis(communityId: string): Promise<CustomEmoji[]> {
	return api.get(`/communities/${communityId}/emojis`);
}

export async function uploadEmoji(communityId: string, shortcode: string, file: File): Promise<CustomEmoji> {
	return api.upload(`/communities/${communityId}/emojis`, 'file', file, { shortcode });
}

export async function deleteEmoji(communityId: string, emojiId: string): Promise<void> {
	return api.delete(`/communities/${communityId}/emojis/${emojiId}`);
}
