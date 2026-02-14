import { api } from './client';

export interface GifResult {
	id: string;
	title: string;
	preview_url: string;
	url: string;
	width: number;
	height: number;
}

export interface GifSearchResponse {
	results: GifResult[];
	next: string | null;
}

export async function searchGifs(query: string, limit = 20): Promise<GifSearchResponse> {
	return api.get(`/gifs/search?q=${encodeURIComponent(query)}&limit=${limit}`);
}

export async function getTrendingGifs(limit = 20): Promise<GifSearchResponse> {
	return api.get(`/gifs/trending?limit=${limit}`);
}
