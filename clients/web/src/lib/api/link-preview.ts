import { authStore } from '$lib/stores/auth.svelte';
import { apiBase } from '$lib/env';

export interface LinkPreview {
	url: string;
	title: string | null;
	description: string | null;
	image: string | null;
	site_name: string | null;
}

const MAX_PREVIEW_CACHE = 200;
const previewCache = new Map<string, Promise<LinkPreview | null>>();

export function fetchLinkPreview(url: string): Promise<LinkPreview | null> {
	const cached = previewCache.get(url);
	if (cached) return cached;

	const promise = doFetch(url);

	// Evict oldest entry if cache is full
	if (previewCache.size >= MAX_PREVIEW_CACHE) {
		const oldest = previewCache.keys().next().value!;
		previewCache.delete(oldest);
	}

	previewCache.set(url, promise);
	return promise;
}

async function doFetch(url: string): Promise<LinkPreview | null> {
	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	try {
		const response = await fetch(`${apiBase()}/link-preview?url=${encodeURIComponent(url)}`, { headers });
		if (!response.ok) return null;
		const data: LinkPreview = await response.json();
		if (data.title || data.description) return data;
		return null;
	} catch {
		return null;
	}
}
