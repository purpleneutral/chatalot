import { api } from './client';

export interface TotpSetup {
	otpauth_url: string;
	secret: string;
}

export async function setupTotp(): Promise<TotpSetup> {
	return api.post<TotpSetup>('/totp/setup', {});
}

export async function verifyTotp(code: string): Promise<{ enabled: boolean }> {
	return api.post<{ enabled: boolean }>('/totp/verify', { code });
}

export async function disableTotp(code: string): Promise<{ enabled: boolean }> {
	return api.post<{ enabled: boolean }>('/totp/disable', { code });
}
