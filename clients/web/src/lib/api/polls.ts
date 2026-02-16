import { api } from './client';

export interface PollOptionVotes {
	option_index: number;
	count: number;
	voter_ids: string[];
}

export interface Poll {
	id: string;
	channel_id: string;
	created_by: string;
	question: string;
	options: string[];
	multi_select: boolean;
	anonymous: boolean;
	closed: boolean;
	expires_at: string | null;
	created_at: string;
	votes: PollOptionVotes[];
}

export interface CreatePollRequest {
	question: string;
	options: string[];
	multi_select?: boolean;
	anonymous?: boolean;
	expires_in_minutes?: number;
}

export async function createPoll(channelId: string, req: CreatePollRequest): Promise<Poll> {
	return api.post(`/channels/${channelId}/polls`, req);
}

export async function listPolls(channelId: string): Promise<Poll[]> {
	return api.get(`/channels/${channelId}/polls`);
}

export async function getPoll(pollId: string): Promise<Poll> {
	return api.get(`/polls/${pollId}`);
}

export async function votePoll(pollId: string, optionIndex: number): Promise<void> {
	return api.post(`/polls/${pollId}/vote`, { option_index: optionIndex });
}

export async function removeVote(pollId: string, optionIndex: number): Promise<void> {
	return api.delete(`/polls/${pollId}/vote/${optionIndex}`);
}

export async function closePoll(pollId: string): Promise<void> {
	return api.post(`/polls/${pollId}/close`, {});
}
