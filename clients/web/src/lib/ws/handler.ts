import { messageStore, type ChatMessage } from '$lib/stores/messages.svelte';
import { memberStore } from '$lib/stores/members.svelte';
import { presenceStore } from '$lib/stores/presence.svelte';
import { channelStore } from '$lib/stores/channels.svelte';
import { authStore } from '$lib/stores/auth.svelte';
import { toastStore } from '$lib/stores/toast.svelte';
import { voiceStore } from '$lib/stores/voice.svelte';
import { webrtcManager } from '$lib/webrtc/manager';
import { soundStore } from '$lib/stores/sound.svelte';
import { notificationStore } from '$lib/stores/notification.svelte';
import { userStore } from '$lib/stores/users.svelte';
import { groupStore } from '$lib/stores/groups.svelte';
import { communityStore } from '$lib/stores/communities.svelte';
import { readReceiptStore } from '$lib/stores/readReceipts.svelte';
import { detectMentions } from '$lib/utils/mentions';
import { getUser } from '$lib/api/users';
import { wsClient } from './connection';
import type { ServerMessage } from './types';
import { initCrypto, getSessionManager, getKeyManager } from '$lib/crypto';
import { decryptMessage } from '$lib/crypto/decrypt';

// Debounced mark-read for incoming messages while viewing a channel
let markReadTimer: ReturnType<typeof setTimeout> | null = null;
function debouncedMarkRead(channelId: string, messageId: string) {
	if (markReadTimer) clearTimeout(markReadTimer);
	markReadTimer = setTimeout(() => {
		wsClient.send({ type: 'mark_read', channel_id: channelId, message_id: messageId });
	}, 1000);
}

/** Cancel pending mark-read timer (call on logout). */
export function clearMarkReadTimer() {
	if (markReadTimer) {
		clearTimeout(markReadTimer);
		markReadTimer = null;
	}
}

/** Fetch and cache user info if not already in the store. */
async function ensureUser(userId: string) {
	if (userStore.getUser(userId)) return;
	try {
		const user = await getUser(userId);
		userStore.setUser(user);
	} catch {
		// User lookup failed — display will fall back to truncated ID
	}
}

/// Handle incoming server WebSocket messages, updating the appropriate stores.
export async function handleServerMessage(msg: ServerMessage) {
	switch (msg.type) {
		case 'new_message': {
			const content = await decryptMessage(
				msg.channel_id,
				msg.sender_id,
				msg.ciphertext,
				msg.id,
			);
			ensureUser(msg.sender_id);

			const chatMsg: ChatMessage = {
				id: msg.id,
				channelId: msg.channel_id,
				senderId: msg.sender_id,
				content,
				messageType: msg.message_type,
				replyToId: msg.reply_to,
				editedAt: null,
				createdAt: msg.created_at,
				threadId: msg.thread_id ?? null,
			};
			messageStore.addMessage(msg.channel_id, chatMsg);

			// If this is a thread reply, increment the reply count on the root message
			if (msg.thread_id) {
				messageStore.incrementThreadReplyCount(msg.thread_id, msg.created_at);
				// Notify thread panel if open
				window.dispatchEvent(
					new CustomEvent('chatalot:thread-reply', {
						detail: { threadId: msg.thread_id, message: chatMsg }
					})
				);
			}

			// Check if user is actively viewing this channel
			const isViewingChannel = !notificationStore.pageHidden
				&& channelStore.activeChannelId === msg.channel_id;

			// Only increment unread if not viewing the channel and not own message
			if (!isViewingChannel && msg.sender_id !== authStore.user?.id) {
				messageStore.incrementUnread(msg.channel_id);
			}

			// Auto mark-read when viewing the channel (debounced)
			if (isViewingChannel && msg.sender_id !== authStore.user?.id) {
				debouncedMarkRead(msg.channel_id, msg.id);
			}

			// Notifications (skip own messages)
			if (msg.sender_id !== authStore.user?.id) {
				const channel = channelStore.channels.find(c => c.id === msg.channel_id);
				const isDm = channel?.channel_type === 'dm';
				const senderName = userStore.getDisplayName(msg.sender_id);
				const channelName = channel?.name ?? 'Direct Message';

				// Detect @mentions
				const mentions = authStore.user?.username
					? detectMentions(content, authStore.user.username)
					: { isMentioned: false };

				// Per-channel notification level (DMs always 'all')
				const level = isDm ? 'all' : notificationStore.getChannelLevel(msg.channel_id);

				const shouldNotify = level === 'all'
					|| (level === 'mentions' && mentions.isMentioned)
					|| isDm;

				if (shouldNotify) {
					// Play appropriate sound
					if (isDm) {
						soundStore.playDmNotification();
					} else if (mentions.isMentioned) {
						soundStore.playMentionNotification();
					} else {
						soundStore.playChannelNotification();
					}

					// Desktop notification (only when not viewing this channel)
					if (!isViewingChannel) {
						const preview = content.length > 100 ? content.slice(0, 100) + '...' : content;
						notificationStore.showDesktopNotification({
							title: isDm ? senderName : `${senderName} in #${channelName}`,
							body: preview,
							channelId: msg.channel_id
						});
					}
				}
			}
			break;
		}

		case 'message_sent': {
			// Confirm optimistic message — find the pending message in this channel
			const pending = messageStore
				.getMessages(msg.channel_id)
				.find((m) => m.pending);
			if (pending) {
				messageStore.confirmMessage(msg.channel_id, pending.id, msg.id, msg.created_at);
			}
			// Notify thread panel of confirmed thread messages
			if (msg.thread_id) {
				window.dispatchEvent(
					new CustomEvent('chatalot:thread-message-confirmed', {
						detail: { channelId: msg.channel_id, newId: msg.id, createdAt: msg.created_at, threadId: msg.thread_id }
					})
				);
			}
			break;
		}

		case 'message_edited': {
			const editedContent = await decryptMessage(
				msg.channel_id,
				msg.sender_id,
				msg.ciphertext,
				msg.message_id,
			);
			messageStore.editMessage(msg.message_id, editedContent, msg.edited_at);
			// Notify thread panel
			window.dispatchEvent(
				new CustomEvent('chatalot:thread-message-edited', {
					detail: { messageId: msg.message_id, content: editedContent, editedAt: msg.edited_at }
				})
			);
			break;
		}

		case 'message_deleted': {
			messageStore.deleteMessage(msg.message_id);
			// Notify thread panel
			window.dispatchEvent(
				new CustomEvent('chatalot:thread-message-deleted', {
					detail: { messageId: msg.message_id }
				})
			);
			break;
		}

		case 'reaction_added': {
			messageStore.addReaction(msg.message_id, msg.user_id, msg.emoji);
			// Notify thread panel
			window.dispatchEvent(
				new CustomEvent('chatalot:thread-reaction-updated', {
					detail: { messageId: msg.message_id, userId: msg.user_id, emoji: msg.emoji, action: 'add' }
				})
			);
			break;
		}

		case 'reaction_removed': {
			messageStore.removeReaction(msg.message_id, msg.user_id, msg.emoji);
			// Notify thread panel
			window.dispatchEvent(
				new CustomEvent('chatalot:thread-reaction-updated', {
					detail: { messageId: msg.message_id, userId: msg.user_id, emoji: msg.emoji, action: 'remove' }
				})
			);
			break;
		}

		case 'presence_update': {
			presenceStore.setStatus(msg.user_id, msg.status);
			break;
		}

		case 'user_typing': {
			presenceStore.setTyping(msg.channel_id, msg.user_id);
			break;
		}

		case 'user_stopped_typing': {
			presenceStore.clearTyping(msg.channel_id, msg.user_id);
			break;
		}

		// Voice/Video
		case 'voice_state_update': {
			// If the server shows us as a participant but we have no active call
			// (e.g. page was refreshed), leave the stale session immediately
			const myId = authStore.user?.id;
			console.info(`[VOICE-WS] voice_state_update ch=${msg.channel_id.slice(0,8)} participants=[${msg.participants.map((p: string) => p.slice(0,8)).join(',')}] isInCall=${voiceStore.isInCall} myInList=${msg.participants.includes(myId ?? '')}`);
			if (myId && msg.participants.includes(myId) && !voiceStore.isInCall) {
				console.warn(`[VOICE-WS] Auto-leaving stale voice session (not in call but listed as participant)`);
				wsClient.send({ type: 'leave_voice', channel_id: msg.channel_id });
				break;
			}
			webrtcManager.onVoiceStateUpdate(msg.channel_id, msg.participants);
			for (const uid of msg.participants) {
				ensureUser(uid);
			}
			break;
		}

		case 'user_joined_voice': {
			// Suppress join sound if user was already in the list (reconnect)
			const alreadyIn = voiceStore.getChannelParticipants(msg.channel_id).includes(msg.user_id);
			voiceStore.addChannelParticipant(msg.channel_id, msg.user_id);
			// Peer connections are established solely via voice_state_update
			// to avoid race conditions with concurrent offer creation.
			if (msg.user_id !== authStore.user?.id) {
				if (!alreadyIn) soundStore.playVoiceJoin();
				ensureUser(msg.user_id);
			}
			break;
		}

		case 'user_left_voice': {
			voiceStore.removeChannelParticipant(msg.channel_id, msg.user_id);
			webrtcManager.onUserLeft(msg.user_id);
			if (msg.user_id !== authStore.user?.id) {
				soundStore.playVoiceLeave();
			}
			break;
		}

		case 'kicked_from_voice': {
			voiceStore.removeChannelParticipant(msg.channel_id, msg.user_id);
			webrtcManager.onUserLeft(msg.user_id);
			if (msg.user_id === authStore.user?.id) {
				webrtcManager.leaveCall();
				toastStore.error('You were kicked from voice');
			}
			break;
		}

		// WebRTC signaling
		case 'rtc_offer': {
			console.info(`[VOICE-WS] rtc_offer from=${msg.from_user_id.slice(0,8)}`);
			webrtcManager.handleOffer(msg.from_user_id, msg.session_id, msg.sdp);
			break;
		}

		case 'rtc_answer': {
			console.info(`[VOICE-WS] rtc_answer from=${msg.from_user_id.slice(0,8)}`);
			webrtcManager.handleAnswer(msg.from_user_id, msg.sdp);
			break;
		}

		case 'rtc_ice_candidate': {
			console.info(`[VOICE-WS] rtc_ice_candidate from=${msg.from_user_id.slice(0,8)}`);
			webrtcManager.handleIceCandidate(msg.from_user_id, msg.candidate);
			break;
		}

		case 'member_kicked': {
			memberStore.removeMember(msg.channel_id, msg.user_id);
			if (msg.user_id === authStore.user?.id) {
				toastStore.error('You were kicked from the channel');
				channelStore.setActive(null);
			}
			break;
		}

		case 'member_banned': {
			memberStore.removeMember(msg.channel_id, msg.user_id);
			if (msg.user_id === authStore.user?.id) {
				toastStore.error('You were banned from this channel');
				channelStore.setActive(null);
			}
			break;
		}

		case 'member_role_updated': {
			memberStore.updateMemberRole(msg.channel_id, msg.user_id, msg.role);
			break;
		}

		case 'user_timed_out': {
			if (msg.user_id === authStore.user?.id) {
				const until = new Date(msg.expires_at);
				const reason = msg.reason ? `: ${msg.reason}` : '';
				toastStore.error(`You have been timed out until ${until.toLocaleTimeString()}${reason}`);
			}
			break;
		}

		case 'new_dm_channel': {
			// Subscribe to the new DM channel so we receive messages
			wsClient.send({ type: 'subscribe', channel_ids: [msg.channel_id] });

			// Add the other user to the user cache
			userStore.setUser({
				id: msg.other_user_id,
				username: msg.other_user_username,
				display_name: msg.other_user_display_name ?? msg.other_user_username,
				avatar_url: msg.other_user_avatar_url,
				banner_url: null,
				status: 'online',
				custom_status: null
			});

			// Notify the UI to add this DM to the sidebar
			window.dispatchEvent(
				new CustomEvent('chatalot:new-dm-channel', {
					detail: {
						channel: {
							id: msg.channel_id,
							name: msg.channel_name,
							channel_type: 'dm',
							topic: null,
							created_by: msg.other_user_id,
							created_at: msg.created_at,
							group_id: null
						},
						other_user: {
							id: msg.other_user_id,
							username: msg.other_user_username,
							display_name: msg.other_user_display_name,
							avatar_url: msg.other_user_avatar_url,
							banner_url: null,
							status: 'online',
							custom_status: null
						}
					}
				})
			);
			break;
		}

		case 'read_receipt': {
			readReceiptStore.setReadPosition(msg.channel_id, msg.user_id, msg.message_id, msg.timestamp);
			break;
		}

		case 'message_pinned': {
			messageStore.addPinned(msg.channel_id, msg.message_id);
			break;
		}

		case 'message_unpinned': {
			messageStore.removePinned(msg.channel_id, msg.message_id);
			break;
		}

		// Polls
		case 'poll_created': {
			window.dispatchEvent(
				new CustomEvent('chatalot:poll-created', {
					detail: { pollId: msg.poll_id, channelId: msg.channel_id, createdBy: msg.created_by, question: msg.question }
				})
			);
			break;
		}

		case 'poll_voted': {
			window.dispatchEvent(
				new CustomEvent('chatalot:poll-voted', {
					detail: { pollId: msg.poll_id, channelId: msg.channel_id, optionIndex: msg.option_index, voterId: msg.voter_id }
				})
			);
			break;
		}

		case 'poll_closed': {
			window.dispatchEvent(
				new CustomEvent('chatalot:poll-closed', {
					detail: { pollId: msg.poll_id, channelId: msg.channel_id }
				})
			);
			break;
		}

		// Moderation
		case 'user_warned': {
			if (msg.user_id === authStore.user?.id) {
				toastStore.error(`You received a warning: ${msg.reason} (${msg.warning_count} total)`);
			}
			break;
		}

		// Channel/group settings changes
		case 'channel_updated': {
			const ch = channelStore.channels.find(c => c.id === msg.channel_id);
			if (ch) {
				channelStore.updateChannel({
					...ch,
					name: msg.name,
					topic: msg.topic,
					read_only: msg.read_only,
					slow_mode_seconds: msg.slow_mode_seconds,
					archived: msg.archived,
					voice_background: msg.voice_background,
				});
			}
			break;
		}

		case 'group_updated': {
			groupStore.updateGroup(msg.group_id, {
				name: msg.name,
				description: msg.description,
				icon_url: msg.icon_url,
				banner_url: msg.banner_url,
				accent_color: msg.accent_color,
				visibility: msg.visibility,
			});
			break;
		}

		case 'community_updated': {
			communityStore.updateCommunity(msg.community_id, {
				name: msg.name,
				description: msg.description,
				icon_url: msg.icon_url,
				banner_url: msg.banner_url,
				community_theme: msg.community_theme,
				welcome_message: msg.welcome_message,
			});
			break;
		}

		case 'channel_deleted': {
			channelStore.removeChannel(msg.channel_id);
			break;
		}

		case 'group_deleted': {
			channelStore.removeChannelsForGroup(msg.group_id);
			groupStore.removeGroup(msg.group_id);
			break;
		}

		// User profile changes
		case 'user_profile_updated': {
			// Update user cache so display names / avatars refresh everywhere
			const existing = userStore.getUser(msg.user_id);
			if (existing) {
				userStore.setUser({
					...existing,
					display_name: msg.display_name,
					avatar_url: msg.avatar_url,
					banner_url: msg.banner_url,
					custom_status: msg.custom_status,
					bio: msg.bio,
					pronouns: msg.pronouns,
				});
			}
			// If this is the current user (e.g. profile updated from another session),
			// keep authStore in sync
			if (msg.user_id === authStore.user?.id) {
				authStore.updateUser({
					display_name: msg.display_name,
					avatar_url: msg.avatar_url,
					banner_url: msg.banner_url,
					custom_status: msg.custom_status,
					bio: msg.bio,
					pronouns: msg.pronouns,
				});
			}
			break;
		}

		// Announcements
		case 'announcement': {
			toastStore.info(`Announcement: ${msg.title}`);
			window.dispatchEvent(
				new CustomEvent('chatalot:announcement', {
					detail: { id: msg.id, title: msg.title, body: msg.body, createdBy: msg.created_by, createdAt: msg.created_at }
				})
			);
			break;
		}

		case 'sender_key_updated': {
			// Another user uploaded/rotated their sender key
			if (msg.user_id !== authStore.user?.id) {
				try {
					await initCrypto();
					const sm = getSessionManager();
					await sm.processSenderKeyDistribution(
						msg.channel_id,
						msg.user_id,
						JSON.stringify(msg.distribution),
					);
				} catch (err) {
					console.error('Failed to process sender key distribution:', err);
				}
			}
			break;
		}

		case 'sender_key_rotation_required': {
			// A member was removed — rotate our sender key
			try {
				await initCrypto();
				const sm = getSessionManager();
				await sm.rotateSenderKeys(msg.channel_id);
			} catch (err) {
				console.error('Failed to rotate sender keys:', err);
			}
			break;
		}

		case 'error': {
			console.error(`Server error: [${msg.code}] ${msg.message}`);
			if (msg.code === 'slow_mode') {
				const match = msg.message.match(/wait (\d+)/);
				const seconds = match ? parseInt(match[1], 10) : 5;
				window.dispatchEvent(new CustomEvent('chatalot:slow-mode', { detail: { seconds } }));
			} else if (msg.code === 'timed_out') {
				toastStore.error(msg.message);
			}
			break;
		}

		case 'keys_low': {
			console.warn(`One-time prekeys running low: ${msg.remaining} remaining`);
			initCrypto()
				.then(() => getKeyManager().replenishPrekeys())
				.catch((err) => console.error('Failed to replenish prekeys:', err));
			break;
		}

		case 'pong':
		case 'authenticated':
			break;
	}
}
