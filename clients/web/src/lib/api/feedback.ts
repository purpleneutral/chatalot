import { api } from './client';

export interface FeedbackRequest {
	title: string;
	description: string;
	category: string;
}

export interface FeedbackResponse {
	success: boolean;
	issue_number?: number;
	message: string;
}

export async function submitFeedback(req: FeedbackRequest): Promise<FeedbackResponse> {
	return api.post('/feedback', req);
}
