import { api } from './client';

export interface SenderKeyDistributionResponse {
	id: string;
	channel_id: string;
	user_id: string;
	chain_id: number;
	distribution: object;
	created_at: string;
}

export async function uploadSenderKey(
	channelId: string,
	chainId: number,
	distribution: object,
): Promise<SenderKeyDistributionResponse> {
	return api.post<SenderKeyDistributionResponse>(
		`/channels/${channelId}/sender-keys`,
		{ chain_id: chainId, distribution },
	);
}

export async function getSenderKeys(
	channelId: string,
): Promise<SenderKeyDistributionResponse[]> {
	return api.get<SenderKeyDistributionResponse[]>(
		`/channels/${channelId}/sender-keys`,
	);
}
