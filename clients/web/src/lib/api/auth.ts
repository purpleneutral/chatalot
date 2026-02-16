import { api } from './client';

export interface AuthResponse {
	access_token: string;
	refresh_token: string;
	user: UserPublic;
	recovery_code?: string;
}

export interface UserPublic {
	id: string;
	username: string;
	display_name: string;
	avatar_url: string | null;
	banner_url: string | null;
	status: string;
	custom_status: string | null;
	is_admin?: boolean;
	is_owner?: boolean;
}

export interface RegisterParams {
	username: string;
	email: string;
	password: string;
	display_name: string;
	identity_key: number[];
	signed_prekey: {
		key_id: number;
		public_key: number[];
		signature: number[];
	};
	one_time_prekeys: {
		key_id: number;
		public_key: number[];
	}[];
	invite_code?: string;
}

export interface ServerConfig {
	registration_mode: string;
	public_url?: string;
}

/** Cached public URL from server config, populated on first getServerConfig() call. */
let _publicUrl: string | undefined;

/** Get the public base URL for generating shareable links. Falls back to window.location.origin. */
export function getPublicUrl(): string {
	return _publicUrl ?? (typeof window !== 'undefined' ? window.location.origin : '');
}

export async function getServerConfig(): Promise<ServerConfig> {
	const config = await api.get<ServerConfig>('/auth/config');
	if (config.public_url) {
		_publicUrl = config.public_url.replace(/\/+$/, '');
	}
	return config;
}

export async function register(params: RegisterParams): Promise<AuthResponse> {
	return api.post<AuthResponse>('/auth/register', params);
}

export async function login(username: string, password: string, totpCode?: string): Promise<AuthResponse> {
	return api.post<AuthResponse>('/auth/login', {
		username,
		password,
		totp_code: totpCode ?? null
	});
}

export async function refreshToken(token: string): Promise<{ access_token: string; refresh_token: string }> {
	return api.post('/auth/refresh', { refresh_token: token });
}

export async function recoverAccount(
	username: string,
	recoveryCode: string,
	newPassword: string
): Promise<{ recovery_code: string }> {
	return api.post('/auth/recover', {
		username,
		recovery_code: recoveryCode,
		new_password: newPassword
	});
}
