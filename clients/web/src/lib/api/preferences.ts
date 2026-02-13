import { api } from './client';

export interface PreferencesResponse {
	preferences: Record<string, unknown>;
}

export async function getPreferences(): Promise<Record<string, unknown>> {
	const res = await api.get<PreferencesResponse>('/account/preferences');
	return res.preferences;
}

export async function updatePreferences(
	partial: Record<string, unknown>
): Promise<Record<string, unknown>> {
	const res = await api.put<PreferencesResponse>('/account/preferences', {
		preferences: partial
	});
	return res.preferences;
}
