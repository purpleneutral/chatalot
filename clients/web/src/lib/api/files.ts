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
	const formData = new FormData();
	formData.append('file', file);
	formData.append('name', file.name);
	if (channelId) {
		formData.append('channel_id', channelId);
	}

	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const response = await fetch(`${apiBase()}/files/upload`, {
		method: 'POST',
		headers,
		body: formData
	});

	if (!response.ok) {
		const body = await response.json().catch(() => null);
		throw new Error(body?.error?.message || `Upload failed: ${response.status}`);
	}

	return response.json();
}

export async function getFileMetadata(fileId: string): Promise<FileMetadata> {
	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const response = await fetch(`${apiBase()}/files/${fileId}/meta`, { headers });
	if (!response.ok) throw new Error(`Failed to get file metadata: ${response.status}`);
	return response.json();
}

export function getFileDownloadUrl(fileId: string): string {
	return `${apiBase()}/files/${fileId}`;
}

const blobUrlCache = new Map<string, string>();

export async function getAuthenticatedBlobUrl(fileId: string): Promise<string> {
	const cached = blobUrlCache.get(fileId);
	if (cached) return cached;

	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const resp = await fetch(`${apiBase()}/files/${fileId}`, { headers });
	if (!resp.ok) throw new Error(`Failed to fetch file: ${resp.status}`);

	const blob = await resp.blob();
	const url = URL.createObjectURL(blob);
	blobUrlCache.set(fileId, url);
	return url;
}
