import { api } from './client';
import { apiBase } from '$lib/env';

export async function getVapidKey(): Promise<string | null> {
	try {
		const base = apiBase();
		const res = await fetch(`${base}/push/vapid-key`);
		if (!res.ok) return null;
		const data = await res.json();
		return data.public_key ?? null;
	} catch {
		return null;
	}
}

export async function subscribePush(
	endpoint: string,
	p256dhKey: string,
	authKey: string
): Promise<void> {
	await api.post('/push/subscribe', {
		endpoint,
		p256dh_key: p256dhKey,
		auth_key: authKey
	});
}

export async function unsubscribePush(endpoint: string): Promise<void> {
	await api.post('/push/unsubscribe', { endpoint });
}
