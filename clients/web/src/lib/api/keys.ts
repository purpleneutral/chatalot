import { api } from './client';

export interface KeyBundleResponse {
	identity_key: number[];
	signed_prekey: {
		key_id: number;
		public_key: number[];
		signature: number[];
	};
	one_time_prekey: {
		key_id: number;
		public_key: number[];
	} | null;
}

export async function getKeyBundle(userId: string): Promise<KeyBundleResponse> {
	return api.get<KeyBundleResponse>(`/keys/${userId}/bundle`);
}

export async function uploadSignedPrekey(prekey: {
	key_id: number;
	public_key: number[];
	signature: number[];
}): Promise<void> {
	await api.post('/keys/prekeys/signed', prekey);
}

export async function uploadOneTimePrekeys(prekeys: {
	key_id: number;
	public_key: number[];
}[]): Promise<void> {
	await api.post('/keys/prekeys/one-time', prekeys);
}

export async function getPrekeyCount(): Promise<number> {
	const result = await api.get<{ count: number }>('/keys/prekeys/count');
	return result.count;
}

export async function registerKeys(data: {
	identity_key: number[];
	signed_prekey: { key_id: number; public_key: number[]; signature: number[] };
	one_time_prekeys: { key_id: number; public_key: number[] }[];
}): Promise<void> {
	await api.post('/keys/register', data);
}
