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
import { detectMentions } from '$lib/utils/mentions';
import { getUser } from '$lib/api/users';
import { wsClient } from './connection';
import type { ServerMessage } from './types';
import { initCrypto, getSessionManager, getKeyManager } from '$lib/crypto';
import { decryptMessage } from '$lib/crypto/decrypt';

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
				createdAt: msg.created_at
			};
			messageStore.addMessage(msg.channel_id, chatMsg);

			// Check if user is actively viewing this channel
			const isViewingChannel = !notificationStore.pageHidden
				&& channelStore.activeChannelId === msg.channel_id;

			// Only increment unread if not viewing the channel
			if (!isViewingChannel) {
				messageStore.incrementUnread(msg.channel_id);
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
			break;
		}

		case 'message_deleted': {
			messageStore.deleteMessage(msg.message_id);
			break;
		}

		case 'reaction_added': {
			messageStore.addReaction(msg.message_id, msg.user_id, msg.emoji);
			break;
		}

		case 'reaction_removed': {
			messageStore.removeReaction(msg.message_id, msg.user_id, msg.emoji);
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
			webrtcManager.onVoiceStateUpdate(msg.channel_id, msg.participants);
			for (const uid of msg.participants) {
				ensureUser(uid);
			}
			break;
		}

		case 'user_joined_voice': {
			voiceStore.addChannelParticipant(msg.channel_id, msg.user_id);
			webrtcManager.onUserJoined(msg.user_id);
			if (msg.user_id !== authStore.user?.id) {
				soundStore.playVoiceJoin();
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

		// WebRTC signaling
		case 'rtc_offer': {
			webrtcManager.handleOffer(msg.from_user_id, msg.session_id, msg.sdp);
			break;
		}

		case 'rtc_answer': {
			webrtcManager.handleAnswer(msg.from_user_id, msg.sdp);
			break;
		}

		case 'rtc_ice_candidate': {
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

		case 'new_dm_channel': {
			// Subscribe to the new DM channel so we receive messages
			wsClient.send({ type: 'subscribe', channel_ids: [msg.channel_id] });

			// Add the other user to the user cache
			userStore.setUser({
				id: msg.other_user_id,
				username: msg.other_user_username,
				display_name: msg.other_user_display_name,
				avatar_url: msg.other_user_avatar_url,
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
							status: 'online',
							custom_status: null
						}
					}
				})
			);
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
