import { api } from './client';

export interface TotpSetup {
	otpauth_url: string;
	secret: string;
}

export interface TotpEnableResponse {
	enabled: boolean;
	backup_codes: string[];
}

export async function setupTotp(): Promise<TotpSetup> {
	return api.post<TotpSetup>('/totp/setup', {});
}

export async function verifyTotp(code: string): Promise<TotpEnableResponse> {
	return api.post<TotpEnableResponse>('/totp/verify', { code });
}

export async function disableTotp(code: string): Promise<{ enabled: boolean }> {
	return api.post<{ enabled: boolean }>('/totp/disable', { code });
}

export async function regenerateBackupCodes(code: string): Promise<{ backup_codes: string[] }> {
	return api.post('/totp/regenerate-backup-codes', { code });
}
