import { api } from './client';
import type { Channel } from './channels';
import type { UserPublic } from './users';

export interface DmChannel {
	channel: Channel;
	other_user: UserPublic;
}

export async function listDms(): Promise<DmChannel[]> {
	return api.get<DmChannel[]>('/dms');
}

export async function createDm(targetUserId: string): Promise<DmChannel> {
	return api.post<DmChannel>('/dms', { target_user_id: targetUserId });
}
