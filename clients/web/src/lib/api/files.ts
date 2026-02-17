import { api } from './client';
import { authStore } from '$lib/stores/auth.svelte';
import { apiBase } from '$lib/env';

export interface FileUploadResponse {
	id: string;
	size_bytes: number;
	created_at: string;
}

export interface FileMetadata {
	id: string;
	uploader_id: string;
	encrypted_name: string;
	size_bytes: number;
	content_type: string | null;
	checksum: string;
	created_at: string;
}

export async function uploadFile(
	file: File,
	channelId?: string
): Promise<FileUploadResponse> {
	const extra: Record<string, string> = { name: file.name };
	if (channelId) extra.channel_id = channelId;
	return api.upload('/files/upload', 'file', file, extra);
}

export async function getFileMetadata(fileId: string): Promise<FileMetadata> {
	return api.get(`/files/${fileId}/meta`);
}

export function getFileDownloadUrl(fileId: string): string {
	return `${apiBase()}/files/${fileId}`;
}

const MAX_BLOB_CACHE = 100;
const blobUrlCache = new Map<string, string>();
const blobUrlPending = new Map<string, Promise<string>>();

export async function getAuthenticatedBlobUrl(fileId: string): Promise<string> {
	const cached = blobUrlCache.get(fileId);
	if (cached) {
		// Move to end (most recently used)
		blobUrlCache.delete(fileId);
		blobUrlCache.set(fileId, cached);
		return cached;
	}

	// Deduplicate concurrent fetches for the same file
	const pending = blobUrlPending.get(fileId);
	if (pending) return pending;

	const promise = fetchBlobUrl(fileId);
	blobUrlPending.set(fileId, promise);
	try {
		return await promise;
	} finally {
		blobUrlPending.delete(fileId);
	}
}

async function fetchBlobUrl(fileId: string): Promise<string> {
	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const resp = await fetch(`${apiBase()}/files/${fileId}`, { headers });
	if (!resp.ok) throw new Error(`Failed to fetch file: ${resp.status}`);

	const blob = await resp.blob();
	const url = URL.createObjectURL(blob);

	// Evict oldest entry if cache is full
	if (blobUrlCache.size >= MAX_BLOB_CACHE) {
		const oldest = blobUrlCache.keys().next().value!;
		URL.revokeObjectURL(blobUrlCache.get(oldest)!);
		blobUrlCache.delete(oldest);
	}

	blobUrlCache.set(fileId, url);
	return url;
}
