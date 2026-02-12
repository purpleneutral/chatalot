export interface MentionDetectionResult {
	isMentioned: boolean;
	hasEveryoneMention: boolean;
	hasHereMention: boolean;
	hasChannelMention: boolean;
	hasDirectMention: boolean;
}

/**
 * Scan message content for @mentions relevant to the current user.
 * Uses the same /@(\w+)/g regex as renderMarkdown for consistency.
 */
export function detectMentions(content: string, username: string): MentionDetectionResult {
	const mentionRegex = /@(\w+)/g;
	const mentions = new Set<string>();
	let match: RegExpExecArray | null;
	while ((match = mentionRegex.exec(content)) !== null) {
		mentions.add(match[1].toLowerCase());
	}

	const hasEveryoneMention = mentions.has('everyone');
	const hasHereMention = mentions.has('here');
	const hasChannelMention = mentions.has('channel');
	const hasDirectMention = mentions.has(username.toLowerCase());

	return {
		isMentioned: hasDirectMention || hasEveryoneMention || hasHereMention || hasChannelMention,
		hasEveryoneMention,
		hasHereMention,
		hasChannelMention,
		hasDirectMention
	};
}
