import { api } from './client';

export interface AuthResponse {
	access_token: string;
	refresh_token: string;
	user: UserPublic;
}

export interface UserPublic {
	id: string;
	username: string;
	display_name: string;
	avatar_url: string | null;
	status: string;
	custom_status: string | null;
	is_admin?: boolean;
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
}

export async function getServerConfig(): Promise<ServerConfig> {
	return api.get<ServerConfig>('/auth/config');
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
