<script lang="ts">
	import { goto } from '$app/navigation';
	import { listChannels, createChannel, getMessages, searchMessages, getChannelMembers, updateMemberRole, kickMember, banMember, type Channel, type Message } from '$lib/api/channels';
	import { listDms, createDm, type DmChannel } from '$lib/api/dms';
	import { searchUsers, type UserPublic } from '$lib/api/users';
	import { uploadFile, getAuthenticatedBlobUrl, type FileUploadResponse } from '$lib/api/files';
	import { fetchLinkPreview } from '$lib/api/link-preview';
	import { authStore } from '$lib/stores/auth.svelte';
	import { channelStore } from '$lib/stores/channels.svelte';
	import { messageStore, type ChatMessage } from '$lib/stores/messages.svelte';
	import { presenceStore } from '$lib/stores/presence.svelte';
	import { wsClient } from '$lib/ws/connection';
	import { handleServerMessage } from '$lib/ws/handler';
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { webrtcManager } from '$lib/webrtc/manager';
	import CallControls from '$lib/components/CallControls.svelte';
	import VideoGrid from '$lib/components/VideoGrid.svelte';
	import Avatar from '$lib/components/Avatar.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import { getMe } from '$lib/api/account';
	import { submitFeedback } from '$lib/api/feedback';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { memberStore } from '$lib/stores/members.svelte';
	import { userStore } from '$lib/stores/users.svelte';
	import { notificationStore, type NotificationLevel } from '$lib/stores/notification.svelte';
	import { listGroups, createGroup as apiCreateGroup, joinGroup, leaveGroup, deleteGroup, discoverGroups, listGroupChannels, createGroupChannel, updateChannel as apiUpdateChannel, deleteChannel as apiDeleteChannel, listGroupMembers, createInvite, acceptInvite, getInviteInfo, type Group, type GroupMember, type InviteInfo } from '$lib/api/groups';
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import { groupStore } from '$lib/stores/groups.svelte';
	import { onMount, onDestroy, tick } from 'svelte';
	import { fade, slide, fly, scale } from 'svelte/transition';

	let messageInput = $state('');
	let newChannelName = $state('');
	let showCreateChannel = $state(false);
	let messageListEl: HTMLDivElement | undefined = $state();
	let typingTimeout: ReturnType<typeof setTimeout> | null = null;
	let unsubWs: (() => void) | null = null;

	// DM state
	let dmChannels = $state<DmChannel[]>([]);
	let showNewDm = $state(false);
	let dmSearchQuery = $state('');
	let dmSearchResults = $state<UserPublic[]>([]);
	let dmSearchTimeout: ReturnType<typeof setTimeout> | null = null;

	// File upload state
	let fileInputEl: HTMLInputElement | undefined = $state();
	let uploading = $state(false);

	// Drag & drop state
	let dragOver = $state(false);
	let dragCounter = $state(0);

	// Sidebar tab (restore from localStorage)
	let sidebarTab = $state<'groups' | 'channels' | 'dms'>(
		(typeof localStorage !== 'undefined' && localStorage.getItem('chatalot:sidebarTab') as 'groups' | 'channels' | 'dms') || 'groups'
	);

	// Group state
	let showCreateGroup = $state(false);
	let newGroupName = $state('');
	let newGroupDescription = $state('');
	let expandedGroupIds = $state<Set<string>>(new Set());
	let groupChannelsMap = $state<Map<string, Channel[]>>(new Map());
	let showDiscoverGroups = $state(false);
	let discoverGroupsList = $state<Group[]>([]);
	let showGroupChannelCreate = $state<string | null>(null);
	let newGroupChannelName = $state('');
	let newGroupChannelType = $state('text');

	// Edit state
	let editingMessageId = $state<string | null>(null);
	let editInput = $state('');

	// Reaction picker state
	let reactionPickerMessageId = $state<string | null>(null);
	let fullEmojiPickerMessageId = $state<string | null>(null);

	// Context menu state
	let contextMenuMessageId = $state<string | null>(null);
	let contextMenuPos = $state({ x: 0, y: 0 });

	// Mobile sidebar state
	let sidebarOpen = $state(false);

	// Reply state
	let replyingTo = $state<ChatMessage | null>(null);

	// Mention autocomplete state
	let mentionQuery = $state('');
	let showMentionPopup = $state(false);
	let mentionIndex = $state(0);
	let messageInputEl: HTMLTextAreaElement | undefined = $state();

	// Invite state
	let showInviteCode = $state<string | null>(null);
	let inviteCode = $state('');
	let showJoinInvite = $state(false);
	let joinInviteCode = $state('');
	let invitePreview = $state<InviteInfo | null>(null);

	// Notification dropdown state
	let showNotifDropdown = $state(false);

	// Status picker state
	let showStatusPicker = $state(false);
	const statusOptions = [
		{ value: 'online', label: 'Online', desc: 'Available', color: 'bg-[var(--success)]' },
		{ value: 'idle', label: 'Away', desc: 'Be right back', color: 'bg-yellow-400' },
		{ value: 'dnd', label: 'Do Not Disturb', desc: 'Leave me alone', color: 'bg-[var(--danger)]' },
		{ value: 'invisible', label: 'Invisible', desc: 'Appear offline', color: 'bg-gray-500' }
	] as const;

	// Search state
	let showSearch = $state(false);
	let searchQuery = $state('');
	let searchResults = $state<ChatMessage[]>([]);
	let searching = $state(false);
	let searchTimeout: ReturnType<typeof setTimeout> | null = null;

	// Infinite scroll state
	let loadingOlder = $state(false);
	const FETCH_LIMIT = 50;

	// Member panel state
	let showMemberPanel = $state(false);
	let membersLoading = $state(false);

	// Feedback modal state
	let showFeedback = $state(false);
	let feedbackTitle = $state('');
	let feedbackDescription = $state('');
	let feedbackCategory = $state('bug');
	let feedbackSubmitting = $state(false);

	async function handleFeedbackSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!feedbackTitle.trim() || !feedbackDescription.trim()) return;
		feedbackSubmitting = true;
		try {
			const res = await submitFeedback({
				title: feedbackTitle.trim(),
				description: feedbackDescription.trim(),
				category: feedbackCategory
			});
			toastStore.success(res.message);
			showFeedback = false;
			feedbackTitle = '';
			feedbackDescription = '';
			feedbackCategory = 'bug';
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to submit feedback');
		} finally {
			feedbackSubmitting = false;
		}
	}

	// Member panel functions
	async function toggleMemberPanel() {
		showMemberPanel = !showMemberPanel;
		if (showMemberPanel && channelStore.activeChannelId) {
			membersLoading = true;
			try {
				const members = await getChannelMembers(channelStore.activeChannelId);
				memberStore.setMembers(channelStore.activeChannelId, members);
				// Populate user cache from member info
				userStore.setUsers(members.map(m => ({
					id: m.user_id,
					username: m.username,
					display_name: m.display_name,
					avatar_url: m.avatar_url,
					status: 'online',
					custom_status: null
				})));
			} catch (err) {
				console.error('Failed to load members:', err);
			} finally {
				membersLoading = false;
			}
		}
	}

	async function handleRoleChange(userId: string, newRole: string) {
		if (!channelStore.activeChannelId) return;
		try {
			await updateMemberRole(channelStore.activeChannelId, userId, newRole);
			memberStore.updateMemberRole(channelStore.activeChannelId, userId, newRole);
			toastStore.success(`Role updated to ${newRole}`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to update role');
		}
	}

	async function handleKick(userId: string, displayName: string) {
		if (!channelStore.activeChannelId) return;
		if (!confirm(`Kick ${displayName} from this channel?`)) return;
		try {
			await kickMember(channelStore.activeChannelId, userId);
			memberStore.removeMember(channelStore.activeChannelId, userId);
			toastStore.success(`${displayName} was kicked`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to kick member');
		}
	}

	async function handleBan(userId: string, displayName: string) {
		if (!channelStore.activeChannelId) return;
		const reason = prompt(`Ban ${displayName}? Enter an optional reason:`);
		if (reason === null) return; // cancelled
		try {
			await banMember(channelStore.activeChannelId, userId, reason || undefined);
			memberStore.removeMember(channelStore.activeChannelId, userId);
			toastStore.success(`${displayName} was banned`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to ban member');
		}
	}

	const QUICK_REACTIONS = ['👍', '❤️', '😂', '🎉', '😮', '😢'];

	// Derived state
	let activeChannel = $derived(channelStore.activeChannel);
	let messages = $derived(
		channelStore.activeChannelId
			? messageStore.getMessages(channelStore.activeChannelId)
			: []
	);
	let typingUsers = $derived(
		channelStore.activeChannelId
			? presenceStore.getTypingUsers(channelStore.activeChannelId)
			: []
	);
	let myRole = $derived(
		channelStore.activeChannelId && authStore.user?.id
			? memberStore.getMyRole(channelStore.activeChannelId, authStore.user.id)
			: 'member'
	);
	let channelMembers = $derived(
		channelStore.activeChannelId
			? memberStore.getMembers(channelStore.activeChannelId)
			: []
	);

	function subscribeToAllChannels() {
		const groupChannelIds = Array.from(groupChannelsMap.values()).flat().map(c => c.id);
		const allIds = [
			...channelStore.channels.map(c => c.id),
			...dmChannels.map(d => d.channel.id),
			...groupChannelIds
		];
		if (allIds.length > 0) {
			wsClient.send({
				type: 'subscribe',
				channel_ids: allIds
			});
		}

		// Restore saved presence status
		const savedStatus = localStorage.getItem('chatalot:status') as 'online' | 'idle' | 'dnd' | 'invisible' | null;
		if (savedStatus && savedStatus !== 'online') {
			wsClient.send({ type: 'update_presence', status: savedStatus });
		}
	}

	function setUserStatus(status: 'online' | 'idle' | 'dnd' | 'invisible') {
		wsClient.send({ type: 'update_presence', status });
		localStorage.setItem('chatalot:status', status);
		if (authStore.user) {
			presenceStore.setStatus(authStore.user.id, status);
		}
		showStatusPicker = false;
	}

	onMount(async () => {
		// Register emoji-picker web component (client-side only)
		import('emoji-picker-element');

		if (!authStore.isAuthenticated) {
			goto('/login');
			return;
		}

		// Refresh user data from server (keeps is_admin, avatar_url etc. current)
		try {
			const me = await getMe();
			authStore.updateUser(me);
		} catch {}

		// Populate user cache with current user
		if (authStore.user) {
			userStore.setUser(authStore.user as UserPublic);
		}

		// Connect WebSocket
		unsubWs = wsClient.onMessage(handleServerMessage);
		wsClient.onAuthenticated(subscribeToAllChannels);
		wsClient.connect();

		// Load channels + DMs + groups
		try {
			const [channels, dms, groups] = await Promise.all([
				listChannels(),
				listDms(),
				listGroups()
			]);

			channelStore.setChannels(channels);
			dmChannels = dms;
			groupStore.setGroups(groups);

			// Populate user cache from DM contacts
			userStore.setUsers(dms.map(d => d.other_user));

			// Load channels for each group
			const groupChannelPromises = groups.map(async (g) => {
				const chs = await listGroupChannels(g.id);
				return [g.id, chs] as [string, Channel[]];
			});
			const results = await Promise.all(groupChannelPromises);
			const newMap = new Map<string, Channel[]>();
			for (const [gid, chs] of results) {
				newMap.set(gid, chs);
				// Also add group channels to channel store so they can be selected
				for (const ch of chs) {
					channelStore.addChannel(ch);
				}
			}
			groupChannelsMap = newMap;

			// Restore previous session state or fall back to defaults
			const savedChannel = localStorage.getItem('chatalot:activeChannel');
			const savedExpanded = localStorage.getItem('chatalot:expandedGroups');

			// Restore expanded groups
			if (savedExpanded) {
				try {
					const ids = JSON.parse(savedExpanded) as string[];
					// Only restore groups that still exist
					const validIds = ids.filter(id => groups.some(g => g.id === id));
					if (validIds.length > 0) {
						expandedGroupIds = new Set(validIds);
					} else if (groups.length > 0) {
						expandedGroupIds = new Set([groups[0].id]);
					}
				} catch {
					if (groups.length > 0) expandedGroupIds = new Set([groups[0].id]);
				}
			} else if (groups.length > 0) {
				expandedGroupIds = new Set([groups[0].id]);
			}

			// Restore active channel or pick a default
			const allChannelIds = new Set([
				...channels.map(c => c.id),
				...dms.map(d => d.channel.id),
				...Array.from(newMap.values()).flat().map(c => c.id)
			]);

			if (savedChannel && allChannelIds.has(savedChannel)) {
				selectChannel(savedChannel);
			} else if (groups.length > 0) {
				const firstGroupChannels = newMap.get(groups[0].id);
				if (firstGroupChannels && firstGroupChannels.length > 0) {
					selectChannel(firstGroupChannels[0].id);
				}
			} else if (channels.length > 0) {
				sidebarTab = 'channels';
				selectChannel(channels[0].id);
			}

			// Subscribe to all channels + DMs via WebSocket
			subscribeToAllChannels();

			// Fetch unread counts
			try {
				const res = await fetch('/api/channels/unread', {
					headers: { 'Authorization': `Bearer ${authStore.accessToken}` }
				});
				if (res.ok) {
					const counts = await res.json();
					messageStore.setUnreadCounts(counts);
				}
			} catch { /* ignore */ }
		} catch (err) {
			console.error('Failed to load channels:', err);
		}

		// Close context menu on click outside
		document.addEventListener('click', closeContextMenu);

		// Notification click → navigate to channel
		window.addEventListener('chatalot:navigate-channel', handleNotifNavigate as EventListener);

		// New DM channel → add to sidebar
		window.addEventListener('chatalot:new-dm-channel', handleNewDmChannel as EventListener);
	});

	onDestroy(() => {
		unsubWs?.();
		webrtcManager.leaveCall();
		wsClient.disconnect();
		document.removeEventListener('click', closeContextMenu);
		window.removeEventListener('chatalot:navigate-channel', handleNotifNavigate as EventListener);
		window.removeEventListener('chatalot:new-dm-channel', handleNewDmChannel as EventListener);
	});

	function handleNotifNavigate(e: CustomEvent<string>) {
		selectChannel(e.detail);
	}

	function handleNewDmChannel(e: CustomEvent<DmChannel>) {
		const dm = e.detail;
		if (!dmChannels.some(d => d.channel.id === dm.channel.id)) {
			dmChannels = [dm, ...dmChannels];
			channelStore.addChannel({
				id: dm.channel.id,
				name: dm.channel.name,
				channel_type: 'dm',
				topic: dm.channel.topic,
				created_by: dm.channel.created_by,
				created_at: dm.channel.created_at,
				group_id: null
			});
		}
	}

	function closeContextMenu() {
		contextMenuMessageId = null;
		reactionPickerMessageId = null;
		fullEmojiPickerMessageId = null;
		showNotifDropdown = false;
		showStatusPicker = false;
	}

	function openFullEmojiPicker(messageId: string) {
		reactionPickerMessageId = null;
		fullEmojiPickerMessageId = messageId;
	}

	function bindEmojiPicker(node: HTMLElement, messageId: string) {
		const handler = (e: Event) => {
			const detail = (e as CustomEvent).detail;
			if (detail?.unicode) {
				toggleReaction(messageId, detail.unicode);
				fullEmojiPickerMessageId = null;
			}
		};
		node.addEventListener('emoji-click', handler);
		return {
			destroy() {
				node.removeEventListener('emoji-click', handler);
			}
		};
	}

	async function selectChannel(channelId: string) {
		channelStore.setActive(channelId);
		messageStore.clearUnread(channelId);
		sidebarOpen = false;
		localStorage.setItem('chatalot:activeChannel', channelId);

		// Preload members for @mention autocomplete
		getChannelMembers(channelId)
			.then((members) => {
				memberStore.setMembers(channelId, members);
				userStore.setUsers(members.map(m => ({
					id: m.user_id,
					username: m.username,
					display_name: m.display_name,
					avatar_url: m.avatar_url,
					status: 'online',
					custom_status: null
				})));
			})
			.catch(() => {});

		// Load message history if not already fetched from server
		if (!messageStore.hasLoadedHistory(channelId)) {
			messageStore.setLoading(channelId, true);
			try {
				const rawMessages = await getMessages(channelId, undefined, FETCH_LIMIT);
				const chatMessages: ChatMessage[] = rawMessages.reverse().map(m => ({
					id: m.id,
					channelId: m.channel_id,
					senderId: m.sender_id,
					// TODO: Decrypt with Double Ratchet. For dev, decode as UTF-8.
					content: new TextDecoder().decode(new Uint8Array(m.ciphertext)),
					messageType: m.message_type,
					replyToId: m.reply_to_id,
					editedAt: m.edited_at,
					createdAt: m.created_at
				}));
				messageStore.setMessages(channelId, chatMessages, FETCH_LIMIT);

				// Mark the latest message as read
				if (chatMessages.length > 0) {
					const lastMsg = chatMessages[chatMessages.length - 1];
					wsClient.send({ type: 'mark_read', channel_id: channelId, message_id: lastMsg.id });
				}
			} catch (err) {
				console.error('Failed to load messages:', err);
			} finally {
				messageStore.setLoading(channelId, false);
			}
		}

		// Refresh member list if panel is open
		if (showMemberPanel) {
			getChannelMembers(channelId)
				.then((members) => {
					memberStore.setMembers(channelId, members);
					userStore.setUsers(members.map(m => ({
						id: m.user_id,
						username: m.username,
						display_name: m.display_name,
						avatar_url: m.avatar_url,
						status: 'online',
						custom_status: null
					})));
				})
				.catch(console.error);
		}

		await tick();
		scrollToBottom();
	}

	function scrollToBottom() {
		if (messageListEl) {
			messageListEl.scrollTop = messageListEl.scrollHeight;
		}
	}

	async function sendMessage(e: SubmitEvent) {
		e.preventDefault();
		const text = messageInput.trim();
		if (!text || !channelStore.activeChannelId) return;

		// TODO: Encrypt with Double Ratchet. For dev, send as raw UTF-8 bytes.
		const encoder = new TextEncoder();
		const ciphertext = Array.from(encoder.encode(text));
		const nonce = Array.from(crypto.getRandomValues(new Uint8Array(12)));

		// Optimistic add
		const tempId = `temp-${Date.now()}`;
		messageStore.addMessage(channelStore.activeChannelId, {
			id: tempId,
			channelId: channelStore.activeChannelId,
			senderId: authStore.user?.id ?? '',
			content: text,
			messageType: 'text',
			replyToId: replyingTo?.id ?? null,
			editedAt: null,
			createdAt: new Date().toISOString(),
			pending: true
		});

		messageInput = '';
		if (messageInputEl) messageInputEl.style.height = 'auto';

		// Send via WebSocket
		wsClient.send({
			type: 'send_message',
			channel_id: channelStore.activeChannelId,
			ciphertext,
			nonce,
			message_type: 'text',
			reply_to: replyingTo?.id ?? null,
			sender_key_id: null
		});

		// Clear reply state
		replyingTo = null;

		// Clear typing indicator
		if (typingTimeout) {
			clearTimeout(typingTimeout);
			typingTimeout = null;
		}

		await tick();
		scrollToBottom();
	}

	function handleInputKeydown(e: KeyboardEvent) {
		if (!channelStore.activeChannelId) return;

		// Enter to send, Shift+Enter for newline
		if (e.key === 'Enter' && !e.shiftKey && !showMentionPopup) {
			e.preventDefault();
			if (messageInput.trim()) {
				// Dispatch submit event on the form
				messageInputEl?.closest('form')?.requestSubmit();
			}
			return;
		}

		// Up arrow to edit last own message (when input is empty)
		if (e.key === 'ArrowUp' && !messageInput.trim()) {
			const myId = authStore.user?.id;
			if (myId) {
				const lastOwn = [...messages].reverse().find(m => m.senderId === myId && !m.pending);
				if (lastOwn) {
					e.preventDefault();
					startEditMessage(lastOwn);
					return;
				}
			}
		}

		// Formatting shortcuts (Ctrl/Cmd + key)
		if ((e.ctrlKey || e.metaKey) && messageInputEl) {
			const shortcuts: Record<string, [string, string]> = {
				'b': ['**', '**'],   // Bold
				'i': ['_', '_'],     // Italic
				'e': ['`', '`'],     // Inline code
				'k': ['[', '](url)'], // Link
			};
			const wrap = shortcuts[e.key];
			if (wrap) {
				e.preventDefault();
				wrapSelection(wrap[0], wrap[1]);
				return;
			}
		}

		// Typing indicator
		if (!typingTimeout) {
			wsClient.send({ type: 'typing', channel_id: channelStore.activeChannelId });
			typingTimeout = setTimeout(() => {
				typingTimeout = null;
			}, 3000);
		}
	}

	function wrapSelection(before: string, after: string) {
		if (!messageInputEl) return;
		const start = messageInputEl.selectionStart ?? 0;
		const end = messageInputEl.selectionEnd ?? 0;
		const val = messageInputEl.value;
		const selected = val.slice(start, end);
		const replacement = before + (selected || 'text') + after;
		messageInput = val.slice(0, start) + replacement + val.slice(end);
		tick().then(() => {
			if (!messageInputEl) return;
			if (selected) {
				// Select the wrapped text
				messageInputEl.selectionStart = start;
				messageInputEl.selectionEnd = start + replacement.length;
			} else {
				// Place cursor inside the wrapper on the placeholder text
				messageInputEl.selectionStart = start + before.length;
				messageInputEl.selectionEnd = start + before.length + 4; // "text"
			}
			messageInputEl.focus();
		});
	}

	function autoResizeTextarea() {
		if (!messageInputEl) return;
		messageInputEl.style.height = 'auto';
		messageInputEl.style.height = Math.min(messageInputEl.scrollHeight, 200) + 'px';
	}

	async function handleCreateChannel(e: SubmitEvent) {
		e.preventDefault();
		const name = newChannelName.trim();
		if (!name) return;

		try {
			const channel = await createChannel(name, 'text');
			channelStore.addChannel(channel);
			wsClient.send({ type: 'subscribe', channel_ids: [channel.id] });
			selectChannel(channel.id);
			newChannelName = '';
			showCreateChannel = false;
		} catch (err) {
			console.error('Failed to create channel:', err);
		}
	}

	// DM search with debounce
	function handleDmSearch() {
		if (dmSearchTimeout) clearTimeout(dmSearchTimeout);
		dmSearchTimeout = setTimeout(async () => {
			if (dmSearchQuery.length >= 2) {
				try {
					dmSearchResults = await searchUsers(dmSearchQuery);
					// Filter out self
					dmSearchResults = dmSearchResults.filter(u => u.id !== authStore.user?.id);
				} catch (err) {
					console.error('Search failed:', err);
				}
			} else {
				dmSearchResults = [];
			}
		}, 300);
	}

	async function startDm(user: UserPublic) {
		try {
			const dm = await createDm(user.id);
			// Add to local DM list if not already there
			if (!dmChannels.some(d => d.channel.id === dm.channel.id)) {
				dmChannels = [dm, ...dmChannels];
			}
			// Cache the other user's info
			userStore.setUser(dm.other_user);
			// Subscribe to the DM channel
			wsClient.send({ type: 'subscribe', channel_ids: [dm.channel.id] });
			// Make sure channel store has it
			channelStore.addChannel(dm.channel);
			selectChannel(dm.channel.id);
			showNewDm = false;
			dmSearchQuery = '';
			dmSearchResults = [];
		} catch (err) {
			console.error('Failed to create DM:', err);
		}
	}

	async function handleFileUpload(fileArg?: File) {
		const file = fileArg || fileInputEl?.files?.[0];
		if (!file || !channelStore.activeChannelId) return;

		uploading = true;
		try {
			const result = await uploadFile(file, channelStore.activeChannelId);
			// Send a file message with the file ID
			const encoder = new TextEncoder();
			const fileMsg = JSON.stringify({
				file_id: result.id,
				filename: file.name,
				size: result.size_bytes
			});
			const ciphertext = Array.from(encoder.encode(fileMsg));
			const nonce = Array.from(crypto.getRandomValues(new Uint8Array(12)));

			wsClient.send({
				type: 'send_message',
				channel_id: channelStore.activeChannelId,
				ciphertext,
				nonce,
				message_type: 'file',
				reply_to: null,
				sender_key_id: null
			});

			// Optimistic add
			messageStore.addMessage(channelStore.activeChannelId, {
				id: `temp-${Date.now()}`,
				channelId: channelStore.activeChannelId,
				senderId: authStore.user?.id ?? '',
				content: `[File: ${file.name}]`,
				messageType: 'file',
				replyToId: null,
				editedAt: null,
				createdAt: new Date().toISOString(),
				pending: true
			});
			await tick();
			scrollToBottom();
		} catch (err) {
			console.error('File upload failed:', err);
		} finally {
			uploading = false;
			if (fileInputEl) fileInputEl.value = '';
		}
	}

	// Drag & drop handlers
	function handleDragEnter(e: DragEvent) {
		e.preventDefault();
		dragCounter++;
		if (e.dataTransfer?.types.includes('Files')) {
			dragOver = true;
		}
	}

	function handleDragLeave(e: DragEvent) {
		e.preventDefault();
		dragCounter--;
		if (dragCounter === 0) {
			dragOver = false;
		}
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
	}

	async function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragOver = false;
		dragCounter = 0;
		const files = e.dataTransfer?.files;
		if (!files?.length) return;
		await handleFileUpload(files[0]);
	}

	// Message actions
	function handleDeleteMessage(messageId: string) {
		wsClient.send({ type: 'delete_message', message_id: messageId });
		messageStore.deleteMessage(messageId);
		contextMenuMessageId = null;
	}

	async function startEditMessage(msg: ChatMessage) {
		editingMessageId = msg.id;
		editInput = msg.content;
		contextMenuMessageId = null;
		await tick();
		const el = document.querySelector<HTMLInputElement>('input[data-edit-input]');
		el?.focus();
	}

	function cancelEdit() {
		editingMessageId = null;
		editInput = '';
	}

	function submitEdit(messageId: string) {
		const text = editInput.trim();
		if (!text) return;
		const encoder = new TextEncoder();
		const ciphertext = Array.from(encoder.encode(text));
		const nonce = Array.from(crypto.getRandomValues(new Uint8Array(12)));
		wsClient.send({ type: 'edit_message', message_id: messageId, ciphertext, nonce });
		messageStore.editMessage(messageId, text, new Date().toISOString());
		editingMessageId = null;
		editInput = '';
	}

	function handleEditKeydown(e: KeyboardEvent, messageId: string) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			submitEdit(messageId);
		} else if (e.key === 'Escape') {
			cancelEdit();
		}
	}

	function autoResizeEditTextarea(e: Event) {
		const el = e.target as HTMLTextAreaElement;
		el.style.height = 'auto';
		el.style.height = Math.min(el.scrollHeight, 200) + 'px';
	}

	function showContextMenu(e: MouseEvent, messageId: string) {
		e.preventDefault();
		e.stopPropagation();
		contextMenuMessageId = messageId;
		contextMenuPos = { x: e.clientX, y: e.clientY };
	}

	// Reactions
	function toggleReaction(messageId: string, emoji: string) {
		const msg = messages.find(m => m.id === messageId);
		const userId = authStore.user?.id ?? '';
		const hasReacted = msg?.reactions?.get(emoji)?.has(userId);

		if (hasReacted) {
			wsClient.send({ type: 'remove_reaction', message_id: messageId, emoji });
			messageStore.removeReaction(messageId, userId, emoji);
		} else {
			wsClient.send({ type: 'add_reaction', message_id: messageId, emoji });
			messageStore.addReaction(messageId, userId, emoji);
		}
		reactionPickerMessageId = null;
	}

	function formatTime(isoString: string): string {
		const d = new Date(isoString);
		return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
	}

	const IMAGE_EXTS = /\.(png|jpe?g|gif|webp|svg|bmp|ico)$/i;
	const VIDEO_EXTS = /\.(mp4|webm|mov|ogg)$/i;
	const IMAGE_URL_REGEX = /https?:\/\/[^\s<>"']+\.(png|jpe?g|gif|webp|svg|bmp|ico)(\?[^\s<>"']*)?/gi;
	const URL_REGEX = /https?:\/\/[^\s<>"'\)]+/gi;

	function extractImageUrls(text: string): string[] {
		const matches = text.match(IMAGE_URL_REGEX);
		if (!matches) return [];
		return [...new Set(matches)];
	}

	function extractNonImageUrls(text: string): string[] {
		const allUrls = text.match(URL_REGEX) || [];
		return [...new Set(allUrls)].filter(u => !IMAGE_URL_REGEX.test(u));
	}

	function parseFileMessage(content: string): { file_id: string; filename: string; size: number } | null {
		try { return JSON.parse(content); } catch { return null; }
	}

	function formatFileSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	// Configure marked for chat
	marked.setOptions({
		breaks: true,
		gfm: true,
	});

	const SPECIAL_MENTIONS = ['everyone', 'here', 'channel'];

	function renderMarkdown(text: string): string {
		// Replace @mentions before markdown parsing
		let processed = text.replace(/@(\w+)/g, (match, username) => {
			// Special group mentions
			if (SPECIAL_MENTIONS.includes(username)) {
				return `<span class="mention mention-group">@${username}</span>`;
			}
			const users = Array.from(userStore.getAllUsers?.() ?? []);
			const found = users.find(u => u.username === username);
			if (found) {
				const isSelf = found.id === authStore.user?.id;
				return `<span class="mention ${isSelf ? 'mention-self' : ''}">@${found.display_name}</span>`;
			}
			return match;
		});
		const html = marked.parse(processed) as string;
		return DOMPurify.sanitize(html, { ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'del', 'code', 'pre', 'a', 'ul', 'ol', 'li', 'blockquote', 'span'], ALLOWED_ATTR: ['href', 'target', 'rel', 'class'] });
	}

	function startReply(msg: ChatMessage) {
		replyingTo = msg;
		contextMenuMessageId = null;
		messageInputEl?.focus();
	}

	function cancelReply() {
		replyingTo = null;
	}

	// Special mention entries for autocomplete
	const SPECIAL_MENTION_ENTRIES = [
		{ user_id: '__everyone__', username: 'everyone', display_name: 'everyone', avatar_url: null, role: 'special', joined_at: '', description: 'Notify all members' },
		{ user_id: '__here__', username: 'here', display_name: 'here', avatar_url: null, role: 'special', joined_at: '', description: 'Notify online members' },
		{ user_id: '__channel__', username: 'channel', display_name: 'channel', avatar_url: null, role: 'special', joined_at: '', description: 'Notify channel members' },
	];

	// Mention autocomplete
	let mentionResults = $derived(() => {
		if (!showMentionPopup || !channelStore.activeChannelId) return [];
		const members = memberStore.getMembers(channelStore.activeChannelId);
		const q = mentionQuery.toLowerCase();

		// Filter special mentions
		const specials = q
			? SPECIAL_MENTION_ENTRIES.filter(s => s.username.startsWith(q))
			: SPECIAL_MENTION_ENTRIES;

		// Filter real members
		const people = q
			? members.filter(m =>
				m.username.toLowerCase().startsWith(q) ||
				m.display_name.toLowerCase().startsWith(q)
			)
			: members;

		return [...specials, ...people].slice(0, 10);
	});

	function handleMentionInput() {
		if (!messageInputEl) return;
		const val = messageInputEl.value;
		const pos = messageInputEl.selectionStart ?? val.length;
		// Find @ before cursor
		const before = val.slice(0, pos);
		const match = before.match(/@(\w*)$/);
		if (match) {
			mentionQuery = match[1];
			showMentionPopup = true;
			mentionIndex = 0;
		} else {
			showMentionPopup = false;
		}
	}

	function selectMention(username: string) {
		if (!messageInputEl) return;
		const val = messageInputEl.value;
		const pos = messageInputEl.selectionStart ?? val.length;
		const before = val.slice(0, pos);
		const after = val.slice(pos);
		const replaced = before.replace(/@\w*$/, `@${username} `);
		messageInput = replaced + after;
		showMentionPopup = false;
		tick().then(() => messageInputEl?.focus());
	}

	function handleMentionKeydown(e: KeyboardEvent) {
		if (!showMentionPopup) return;
		const results = mentionResults();
		if (results.length === 0) return;
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			mentionIndex = (mentionIndex + 1) % results.length;
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			mentionIndex = (mentionIndex - 1 + results.length) % results.length;
		} else if (e.key === 'Tab' || e.key === 'Enter') {
			if (results[mentionIndex]) {
				e.preventDefault();
				selectMention(results[mentionIndex].username);
			}
		} else if (e.key === 'Escape') {
			e.preventDefault();
			showMentionPopup = false;
		}
	}

	// Invite helpers
	async function handleCreateInvite(groupId: string) {
		try {
			const invite = await createInvite(groupId);
			await navigator.clipboard.writeText(invite.code);
			toastStore.success(`Invite code copied: ${invite.code}`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to create invite');
		}
	}

	async function handleAcceptInvite() {
		if (!joinInviteCode.trim()) return;
		try {
			const result = await acceptInvite(joinInviteCode.trim());
			// Reload groups
			const groups = await listGroups();
			groupStore.setGroups(groups);
			// Load channels for the new group
			const chs = await listGroupChannels(result.group_id);
			const newMap = new Map(groupChannelsMap);
			newMap.set(result.group_id, chs);
			groupChannelsMap = newMap;
			for (const ch of chs) {
				channelStore.addChannel(ch);
			}
			wsClient.send({ type: 'subscribe', channel_ids: chs.map(c => c.id) });
			expandedGroupIds = new Set([...expandedGroupIds, result.group_id]);
			if (chs.length > 0) selectChannel(chs[0].id);
			showJoinInvite = false;
			joinInviteCode = '';
			invitePreview = null;
			toastStore.success(`Joined "${result.group_name}"`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to accept invite');
		}
	}

	async function handlePreviewInvite() {
		if (!joinInviteCode.trim()) return;
		try {
			invitePreview = await getInviteInfo(joinInviteCode.trim());
		} catch (err: any) {
			toastStore.error(err?.message || 'Invalid invite code');
			invitePreview = null;
		}
	}

	// Group functions
	async function toggleGroupExpand(groupId: string) {
		const next = new Set(expandedGroupIds);
		if (next.has(groupId)) {
			next.delete(groupId);
		} else {
			next.add(groupId);
			// Load channels if not already loaded
			if (!groupChannelsMap.has(groupId)) {
				try {
					const chs = await listGroupChannels(groupId);
					const newMap = new Map(groupChannelsMap);
					newMap.set(groupId, chs);
					groupChannelsMap = newMap;
					for (const ch of chs) {
						channelStore.addChannel(ch);
					}
					wsClient.send({ type: 'subscribe', channel_ids: chs.map(c => c.id) });
				} catch (err) {
					console.error('Failed to load group channels:', err);
				}
			}
		}
		expandedGroupIds = next;
	}

	async function handleCreateGroup(e: SubmitEvent) {
		e.preventDefault();
		const name = newGroupName.trim();
		if (!name) return;
		try {
			const group = await apiCreateGroup(name, newGroupDescription.trim() || undefined);
			groupStore.addGroup(group);
			// Load the auto-created #general channel
			const chs = await listGroupChannels(group.id);
			const newMap = new Map(groupChannelsMap);
			newMap.set(group.id, chs);
			groupChannelsMap = newMap;
			for (const ch of chs) {
				channelStore.addChannel(ch);
			}
			wsClient.send({ type: 'subscribe', channel_ids: chs.map(c => c.id) });
			expandedGroupIds = new Set([...expandedGroupIds, group.id]);
			if (chs.length > 0) selectChannel(chs[0].id);
			newGroupName = '';
			newGroupDescription = '';
			showCreateGroup = false;
			toastStore.success(`Group "${group.name}" created`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to create group');
		}
	}

	async function handleJoinGroup(group: Group) {
		try {
			await joinGroup(group.id);
			groupStore.addGroup(group);
			const chs = await listGroupChannels(group.id);
			const newMap = new Map(groupChannelsMap);
			newMap.set(group.id, chs);
			groupChannelsMap = newMap;
			for (const ch of chs) {
				channelStore.addChannel(ch);
			}
			wsClient.send({ type: 'subscribe', channel_ids: chs.map(c => c.id) });
			expandedGroupIds = new Set([...expandedGroupIds, group.id]);
			showDiscoverGroups = false;
			toastStore.success(`Joined "${group.name}"`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to join group');
		}
	}

	async function handleLeaveGroup(group: Group) {
		if (!confirm(`Leave "${group.name}"?`)) return;
		try {
			await leaveGroup(group.id);
			groupStore.removeGroup(group.id);
			const chs = groupChannelsMap.get(group.id) ?? [];
			const newMap = new Map(groupChannelsMap);
			newMap.delete(group.id);
			groupChannelsMap = newMap;
			// Remove group channels from channel store
			for (const ch of chs) {
				channelStore.removeChannel(ch.id);
			}
			// If viewing a channel in this group, deselect
			if (chs.some(c => c.id === channelStore.activeChannelId)) {
				channelStore.setActive(null);
			}
			toastStore.success(`Left "${group.name}"`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to leave group');
		}
	}

	async function handleDeleteGroup(group: Group) {
		if (!confirm(`Delete "${group.name}"? This will delete all channels in the group.`)) return;
		try {
			await deleteGroup(group.id);
			groupStore.removeGroup(group.id);
			const chs = groupChannelsMap.get(group.id) ?? [];
			const newMap = new Map(groupChannelsMap);
			newMap.delete(group.id);
			groupChannelsMap = newMap;
			for (const ch of chs) {
				channelStore.removeChannel(ch.id);
			}
			if (chs.some(c => c.id === channelStore.activeChannelId)) {
				channelStore.setActive(null);
			}
			toastStore.success(`Deleted "${group.name}"`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to delete group');
		}
	}

	async function handleCreateGroupChannel(e: SubmitEvent, groupId: string) {
		e.preventDefault();
		const name = newGroupChannelName.trim();
		if (!name) return;
		try {
			const ch = await createGroupChannel(groupId, name, newGroupChannelType);
			channelStore.addChannel(ch);
			const newMap = new Map(groupChannelsMap);
			const existing = newMap.get(groupId) ?? [];
			newMap.set(groupId, [...existing, ch]);
			groupChannelsMap = newMap;
			wsClient.send({ type: 'subscribe', channel_ids: [ch.id] });
			selectChannel(ch.id);
			newGroupChannelName = '';
			newGroupChannelType = 'text';
			showGroupChannelCreate = null;
			toastStore.success(`Channel "${ch.name}" created`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to create channel');
		}
	}

	async function handleDeleteGroupChannel(groupId: string, channelId: string) {
		if (!confirm('Delete this channel?')) return;
		try {
			await apiDeleteChannel(groupId, channelId);
			channelStore.removeChannel(channelId);
			const newMap = new Map(groupChannelsMap);
			const existing = newMap.get(groupId) ?? [];
			newMap.set(groupId, existing.filter(c => c.id !== channelId));
			groupChannelsMap = newMap;
			if (channelStore.activeChannelId === channelId) {
				channelStore.setActive(null);
			}
			toastStore.success('Channel deleted');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to delete channel');
		}
	}

	async function handleDiscoverGroups() {
		showDiscoverGroups = true;
		try {
			const all = await discoverGroups();
			// Filter out groups user is already in
			const myIds = new Set(groupStore.groups.map(g => g.id));
			discoverGroupsList = all.filter(g => !myIds.has(g.id));
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to load groups');
		}
	}

	function getDmDisplayName(dmChannel: DmChannel): string {
		return dmChannel.other_user.display_name;
	}

	function getChannelDisplayName(): string {
		if (!activeChannel) return '';
		if (activeChannel.channel_type === 'dm') {
			const dm = dmChannels.find(d => d.channel.id === activeChannel!.id);
			return dm ? getDmDisplayName(dm) : 'Direct Message';
		}
		return activeChannel.name ?? 'Unnamed';
	}

	function getReactionEntries(msg: ChatMessage): [string, Set<string>][] {
		if (!msg.reactions) return [];
		return Array.from(msg.reactions.entries());
	}

	// ── Search ──
	function toggleSearch() {
		showSearch = !showSearch;
		if (!showSearch) {
			searchQuery = '';
			searchResults = [];
		}
	}

	function handleSearchInput() {
		if (searchTimeout) clearTimeout(searchTimeout);
		if (!searchQuery.trim()) {
			searchResults = [];
			return;
		}
		searchTimeout = setTimeout(async () => {
			if (!channelStore.activeChannelId || !searchQuery.trim()) return;
			searching = true;
			try {
				const raw = await searchMessages(channelStore.activeChannelId, searchQuery.trim());
				searchResults = raw.reverse().map(m => ({
					id: m.id,
					channelId: m.channel_id,
					senderId: m.sender_id,
					content: new TextDecoder().decode(new Uint8Array(m.ciphertext)),
					messageType: m.message_type,
					replyToId: m.reply_to_id,
					editedAt: m.edited_at,
					createdAt: m.created_at
				}));
			} catch (err) {
				console.error('Search failed:', err);
				searchResults = [];
			} finally {
				searching = false;
			}
		}, 300);
	}

	function jumpToSearchResult(msgId: string) {
		showSearch = false;
		searchQuery = '';
		searchResults = [];
		const el = document.getElementById('msg-' + msgId);
		if (el) {
			el.scrollIntoView({ behavior: 'smooth', block: 'center' });
			el.classList.add('bg-[var(--accent)]/10');
			setTimeout(() => el.classList.remove('bg-[var(--accent)]/10'), 2000);
		}
	}

	// ── Infinite scroll ──
	async function handleMessageScroll(e: Event) {
		const el = e.target as HTMLDivElement;
		if (el.scrollTop > 200 || loadingOlder || !channelStore.activeChannelId) return;
		if (!messageStore.hasMore(channelStore.activeChannelId)) return;

		const currentMessages = messageStore.getMessages(channelStore.activeChannelId);
		if (currentMessages.length === 0) return;

		const oldestMsg = currentMessages[0];
		loadingOlder = true;
		const prevHeight = el.scrollHeight;
		try {
			const raw = await getMessages(channelStore.activeChannelId, oldestMsg.id, FETCH_LIMIT);
			const olderMessages: ChatMessage[] = raw.reverse().map(m => ({
				id: m.id,
				channelId: m.channel_id,
				senderId: m.sender_id,
				content: new TextDecoder().decode(new Uint8Array(m.ciphertext)),
				messageType: m.message_type,
				replyToId: m.reply_to_id,
				editedAt: m.edited_at,
				createdAt: m.created_at
			}));
			messageStore.prependMessages(channelStore.activeChannelId, olderMessages, FETCH_LIMIT);
			// Preserve scroll position
			await tick();
			el.scrollTop = el.scrollHeight - prevHeight;
		} catch (err) {
			console.error('Failed to load older messages:', err);
		} finally {
			loadingOlder = false;
		}
	}

	// ── Date separators ──
	function formatDateSeparator(dateStr: string): string {
		const d = new Date(dateStr);
		const now = new Date();
		const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
		const yesterday = new Date(today.getTime() - 86400000);
		const msgDate = new Date(d.getFullYear(), d.getMonth(), d.getDate());

		if (msgDate.getTime() === today.getTime()) return 'Today';
		if (msgDate.getTime() === yesterday.getTime()) return 'Yesterday';
		return d.toLocaleDateString(undefined, { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' });
	}

	function shouldShowDateSeparator(msgs: ChatMessage[], index: number): boolean {
		if (index === 0) return true;
		const prev = new Date(msgs[index - 1].createdAt);
		const curr = new Date(msgs[index].createdAt);
		return prev.toDateString() !== curr.toDateString();
	}

	// ── Tab notifications ──
	$effect(() => {
		let total = 0;
		for (const ch of channelStore.channels) {
			total += messageStore.getUnreadCount(ch.id);
		}
		for (const dm of dmChannels) {
			total += messageStore.getUnreadCount(dm.channel.id);
		}
		document.title = total > 0 ? `(${total}) Chatalot` : 'Chatalot';
	});

	// Persist sidebar tab and expanded groups
	$effect(() => {
		localStorage.setItem('chatalot:sidebarTab', sidebarTab);
	});
	$effect(() => {
		localStorage.setItem('chatalot:expandedGroups', JSON.stringify([...expandedGroupIds]));
	});

	// Auto-scroll when new messages arrive
	$effect(() => {
		if (messages.length > 0) {
			tick().then(scrollToBottom);
		}
	});
</script>

{#if authStore.isAuthenticated}
	<div class="flex h-screen">
		<!-- Mobile sidebar overlay -->
		{#if sidebarOpen}
			<button
				class="fixed inset-0 z-30 bg-black/50 md:hidden"
				transition:fade={{ duration: 150 }}
				onclick={() => (sidebarOpen = false)}
				aria-label="Close sidebar"
			></button>
		{/if}

		<!-- Sidebar -->
		<aside class="fixed inset-y-0 left-0 z-40 flex w-64 flex-col border-r border-white/10 bg-[var(--bg-secondary)] transition-transform md:static md:translate-x-0 {sidebarOpen ? 'translate-x-0' : '-translate-x-full'}">
			<div class="flex h-14 items-center justify-between border-b border-white/10 px-4">
				<h1 class="text-lg font-bold text-[var(--text-primary)]">Chatalot</h1>
				<div class="flex items-center gap-1">
					{#if authStore.user?.is_admin}
						<button
							onclick={() => goto('/admin')}
							class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
							title="Admin Panel"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
							</svg>
						</button>
					{/if}
					<button
						onclick={() => goto('/settings')}
						class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						title="Settings"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
						</svg>
					</button>
					<button
						onclick={() => {
							if (sidebarTab === 'groups') {
								showCreateGroup = !showCreateGroup;
							} else if (sidebarTab === 'channels') {
								showCreateChannel = !showCreateChannel;
							} else {
								showNewDm = !showNewDm;
							}
						}}
						class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						title={sidebarTab === 'groups' ? 'Create group' : sidebarTab === 'channels' ? 'Create channel' : 'New DM'}
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
						</svg>
					</button>
				</div>
			</div>

			<!-- Tab switcher -->
			<div class="flex border-b border-white/10">
				<button
					onclick={() => (sidebarTab = 'groups')}
					class="flex-1 px-3 py-2 text-sm font-medium transition {sidebarTab === 'groups' ? 'border-b-2 border-[var(--accent)] text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
				>
					Groups
				</button>
				<button
					onclick={() => (sidebarTab = 'channels')}
					class="flex-1 px-3 py-2 text-sm font-medium transition {sidebarTab === 'channels' ? 'border-b-2 border-[var(--accent)] text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
				>
					Channels
				</button>
				<button
					onclick={() => (sidebarTab = 'dms')}
					class="flex-1 px-3 py-2 text-sm font-medium transition {sidebarTab === 'dms' ? 'border-b-2 border-[var(--accent)] text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
				>
					DMs
				</button>
			</div>

			{#if showCreateChannel && sidebarTab === 'channels'}
				<form onsubmit={handleCreateChannel} class="border-b border-white/10 p-3">
					<input
						type="text"
						bind:value={newChannelName}
						placeholder="Channel name..."
						maxlength="64"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
					<button
						type="submit"
						class="mt-2 w-full rounded-lg bg-[var(--accent)] px-3 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
					>
						Create
					</button>
				</form>
			{/if}

			{#if showCreateGroup && sidebarTab === 'groups'}
				<form onsubmit={handleCreateGroup} class="border-b border-white/10 p-3 space-y-2">
					<input
						type="text"
						bind:value={newGroupName}
						placeholder="Group name..."
						maxlength="64"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
					<input
						type="text"
						bind:value={newGroupDescription}
						placeholder="Description (optional)..."
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
					<button
						type="submit"
						class="w-full rounded-lg bg-[var(--accent)] px-3 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
					>
						Create Group
					</button>
				</form>
			{/if}

			{#if showNewDm && sidebarTab === 'dms'}
				<div class="border-b border-white/10 p-3">
					<input
						type="text"
						bind:value={dmSearchQuery}
						oninput={handleDmSearch}
						placeholder="Search users..."
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
					{#if dmSearchResults.length > 0}
						<div class="mt-2 space-y-1">
							{#each dmSearchResults as user (user.id)}
								<button
									onclick={() => startDm(user)}
									class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
								>
									<Avatar userId={user.id} size="xs" showStatus />
									<span>{user.display_name}</span>
									<span class="text-xs text-[var(--text-secondary)]">@{user.username}</span>
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/if}

			<div class="flex-1 overflow-y-auto p-2">
				{#if sidebarTab === 'groups'}
					<!-- Discover groups button -->
					<button
						onclick={handleDiscoverGroups}
						class="mb-2 flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
						</svg>
						Discover Groups
					</button>
					<button
						onclick={() => { showJoinInvite = !showJoinInvite; invitePreview = null; joinInviteCode = ''; }}
						class="mb-2 flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" /></svg>
						Join via Invite
					</button>

					{#if showJoinInvite}
						<div class="mb-2 rounded-lg border border-white/10 bg-[var(--bg-primary)] p-3 space-y-2">
							<input
								type="text"
								bind:value={joinInviteCode}
								placeholder="Enter invite code..."
								class="w-full rounded border border-white/10 bg-[var(--bg-secondary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
							/>
							{#if invitePreview}
								<div class="rounded bg-white/5 p-2 text-sm">
									<div class="font-medium text-[var(--text-primary)]">{invitePreview.group_name}</div>
									{#if invitePreview.group_description}
										<div class="text-xs text-[var(--text-secondary)]">{invitePreview.group_description}</div>
									{/if}
									<div class="text-xs text-[var(--text-secondary)]">{invitePreview.member_count} members</div>
								</div>
								<button onclick={handleAcceptInvite} class="w-full rounded bg-[var(--accent)] px-3 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]">Join Group</button>
							{:else}
								<button onclick={handlePreviewInvite} class="w-full rounded bg-white/10 px-3 py-1.5 text-sm font-medium text-[var(--text-primary)] transition hover:bg-white/15">Look Up</button>
							{/if}
						</div>
					{/if}

					{#each groupStore.groups as group (group.id)}
						<!-- Group header -->
						<div class="group/grp">
							<button
								onclick={() => toggleGroupExpand(group.id)}
								class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm font-medium transition hover:bg-white/5 {expandedGroupIds.has(group.id) ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 transition-transform {expandedGroupIds.has(group.id) ? 'rotate-90' : ''}" viewBox="0 0 24 24" fill="currentColor">
									<path d="M8 5l8 7-8 7z" />
								</svg>
								<span class="flex-1 truncate">{group.name}</span>
								<span class="text-xs text-[var(--text-secondary)]">{group.member_count}</span>
							</button>
							<!-- Group context actions (show on hover) -->
							<div class="hidden group-hover/grp:flex absolute right-2 top-1/2 -translate-y-1/2 items-center gap-0.5">
							</div>
						</div>

						{#if expandedGroupIds.has(group.id)}
							<!-- Group channels -->
							<div class="ml-2 border-l border-white/5 pl-2">
								{#each (groupChannelsMap.get(group.id) ?? []) as channel (channel.id)}
									{@const unreadCount = messageStore.getUnreadCount(channel.id)}
									<div class="group/ch flex items-center">
										<button
											onclick={() => selectChannel(channel.id)}
											class="flex flex-1 items-center gap-2 rounded-lg px-3 py-1.5 text-left text-sm transition {channelStore.activeChannelId === channel.id ? 'bg-white/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
										>
											{#if channel.channel_type === 'voice'}
												<span class="text-[var(--text-secondary)]" title="Voice channel">🔊</span>
											{:else}
												<span class="text-[var(--text-secondary)]">#</span>
											{/if}
											<span class="flex-1 truncate {unreadCount > 0 ? 'font-semibold text-[var(--text-primary)]' : ''}">{channel.name}</span>
											{#if unreadCount > 0 && channelStore.activeChannelId !== channel.id}
												<span class="flex h-5 min-w-5 items-center justify-center rounded-full bg-[var(--accent)] px-1.5 text-xs font-bold text-white">
													{unreadCount > 99 ? '99+' : unreadCount}
												</span>
											{/if}
										</button>
										{#if group.owner_id === authStore.user?.id}
											<button
												onclick={() => handleDeleteGroupChannel(group.id, channel.id)}
												class="hidden group-hover/ch:block rounded p-0.5 text-[var(--text-secondary)] transition hover:text-[var(--danger)]"
												title="Delete channel"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
													<polyline points="3 6 5 6 21 6" /><path d="M19 6l-2 14H7L5 6" /><path d="M10 11v6" /><path d="M14 11v6" />
												</svg>
											</button>
										{/if}
									</div>
								{/each}

								<!-- Add channel button (owner/admin) -->
								{#if group.owner_id === authStore.user?.id}
									{#if showGroupChannelCreate === group.id}
										<form onsubmit={(e) => handleCreateGroupChannel(e, group.id)} class="space-y-1 px-3 py-1.5">
											<input
												type="text"
												bind:value={newGroupChannelName}
												placeholder="Channel name..."
												maxlength="64"
												class="w-full rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
											/>
											<div class="flex gap-1">
												<button
													type="button"
													onclick={() => { newGroupChannelType = 'text'; }}
													class="flex-1 rounded px-2 py-1 text-xs transition {newGroupChannelType === 'text' ? 'bg-[var(--accent)] text-white' : 'bg-white/5 text-[var(--text-secondary)]'}"
												>
													# Text
												</button>
												<button
													type="button"
													onclick={() => { newGroupChannelType = 'voice'; }}
													class="flex-1 rounded px-2 py-1 text-xs transition {newGroupChannelType === 'voice' ? 'bg-[var(--accent)] text-white' : 'bg-white/5 text-[var(--text-secondary)]'}"
												>
													🔊 Voice
												</button>
											</div>
											<button
												type="submit"
												class="w-full rounded bg-[var(--accent)] px-2 py-1 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)]"
											>
												Create
											</button>
										</form>
									{:else}
										<button
											onclick={() => { showGroupChannelCreate = group.id; }}
											class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-left text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
											</svg>
											Add Channel
										</button>
									{/if}
								{/if}

								<!-- Group actions -->
								<div class="mt-1 flex items-center gap-1 px-3">
									<button
										onclick={() => handleCreateInvite(group.id)}
										class="rounded px-2 py-0.5 text-xs text-[var(--accent)] transition hover:bg-[var(--accent)]/10"
										title="Create invite link"
									>
										Invite
									</button>
									{#if group.owner_id === authStore.user?.id}
										<button
											onclick={() => handleDeleteGroup(group)}
											class="rounded px-2 py-0.5 text-xs text-[var(--danger)] transition hover:bg-[var(--danger)]/10"
											title="Delete group"
										>
											Delete
										</button>
									{:else}
										<button
											onclick={() => handleLeaveGroup(group)}
											class="rounded px-2 py-0.5 text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--danger)]"
											title="Leave group"
										>
											Leave
										</button>
									{/if}
								</div>
							</div>
						{/if}
					{/each}

					{#if groupStore.groups.length === 0}
						<p class="px-2 text-sm text-[var(--text-secondary)]">No groups yet. Create one or discover existing groups.</p>
					{/if}

					<!-- Discover groups modal -->
					{#if showDiscoverGroups}
						<div class="mt-2 rounded-lg border border-white/10 bg-[var(--bg-primary)] p-3">
							<div class="mb-2 flex items-center justify-between">
								<h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Discover</h3>
								<button
									onclick={() => { showDiscoverGroups = false; }}
									class="text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
								>&times;</button>
							</div>
							{#each discoverGroupsList as group (group.id)}
								<div class="flex items-center justify-between rounded-lg px-2 py-1.5 text-sm hover:bg-white/5">
									<div>
										<div class="text-[var(--text-primary)]">{group.name}</div>
										<div class="text-xs text-[var(--text-secondary)]">{group.member_count} members</div>
									</div>
									<button
										onclick={() => handleJoinGroup(group)}
										class="rounded bg-[var(--accent)] px-2 py-1 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)]"
									>
										Join
									</button>
								</div>
							{/each}
							{#if discoverGroupsList.length === 0}
								<p class="text-xs text-[var(--text-secondary)]">No groups to discover.</p>
							{/if}
						</div>
					{/if}
				{:else if sidebarTab === 'channels'}
					<h2 class="mb-2 px-2 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
						Channels
					</h2>
					{#each channelStore.channels.filter(c => c.channel_type !== 'dm' && !c.group_id) as channel (channel.id)}
						{@const unreadCount = messageStore.getUnreadCount(channel.id)}
						<button
							onclick={() => selectChannel(channel.id)}
							class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm transition {channelStore.activeChannelId === channel.id ? 'bg-white/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
						>
							{#if channel.channel_type === 'voice'}
								<span class="text-[var(--text-secondary)]" title="Voice channel">🔊</span>
							{:else}
								<span class="text-[var(--text-secondary)]">#</span>
							{/if}
							<span class="flex-1 truncate {unreadCount > 0 ? 'font-semibold text-[var(--text-primary)]' : ''}">{channel.name}</span>
							{#if unreadCount > 0 && channelStore.activeChannelId !== channel.id}
								<span class="flex h-5 min-w-5 items-center justify-center rounded-full bg-[var(--accent)] px-1.5 text-xs font-bold text-white">
									{unreadCount > 99 ? '99+' : unreadCount}
								</span>
							{/if}
						</button>
						<!-- Voice participants -->
						{#if channel.channel_type === 'voice'}
							{@const voiceUsers = voiceStore.getChannelParticipants(channel.id)}
							{#if voiceUsers.length > 0}
								<div class="ml-6 space-y-0.5 pb-1">
									{#each voiceUsers as uid (uid)}
										<div class="flex items-center gap-1.5 px-2 py-0.5 text-xs text-[var(--text-secondary)]">
											<div class="h-1.5 w-1.5 rounded-full bg-[var(--success)]"></div>
											<span class="truncate">{userStore.getDisplayName(uid)}</span>
										</div>
									{/each}
								</div>
							{/if}
						{/if}
					{/each}
					{#if channelStore.channels.filter(c => c.channel_type !== 'dm' && !c.group_id).length === 0}
						<p class="px-2 text-sm text-[var(--text-secondary)]">No channels yet.</p>
					{/if}
				{:else}
					<h2 class="mb-2 px-2 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
						Direct Messages
					</h2>
					{#each dmChannels as dm (dm.channel.id)}
						{@const unreadCount = messageStore.getUnreadCount(dm.channel.id)}
						<button
							onclick={() => { channelStore.addChannel(dm.channel); selectChannel(dm.channel.id); }}
							class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm transition {channelStore.activeChannelId === dm.channel.id ? 'bg-white/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
						>
							<Avatar userId={dm.other_user.id} size="xs" showStatus />
							<span class="flex-1 truncate {unreadCount > 0 ? 'font-semibold text-[var(--text-primary)]' : ''}">{getDmDisplayName(dm)}</span>
							{#if unreadCount > 0 && channelStore.activeChannelId !== dm.channel.id}
								<span class="flex h-5 min-w-5 items-center justify-center rounded-full bg-[var(--accent)] px-1.5 text-xs font-bold text-white">
									{unreadCount > 99 ? '99+' : unreadCount}
								</span>
							{/if}
						</button>
					{/each}
					{#if dmChannels.length === 0}
						<p class="px-2 text-sm text-[var(--text-secondary)]">No direct messages yet.</p>
					{/if}
				{/if}
			</div>

			<!-- User info -->
			<div class="relative flex items-center gap-3 border-t border-white/10 p-3">
				{#if authStore.user}
					<button onclick={(e) => { e.stopPropagation(); showStatusPicker = !showStatusPicker; }} class="rounded-full transition hover:ring-2 hover:ring-[var(--accent)]/50" title="Set status">
						<Avatar userId={authStore.user.id} size="sm" showStatus />
					</button>
				{/if}
				<div class="flex-1 overflow-hidden">
					<div class="truncate text-sm font-medium text-[var(--text-primary)]">
						{authStore.user?.display_name}
					</div>
					<div class="truncate text-xs text-[var(--text-secondary)]">
						{statusOptions.find(s => s.value === (authStore.user ? presenceStore.getStatus(authStore.user.id) : 'offline'))?.label ?? 'Online'}
					</div>
				</div>

				<!-- Status picker dropdown -->
				{#if showStatusPicker}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div onclick={(e) => e.stopPropagation()} class="absolute bottom-full left-2 mb-2 w-56 rounded-lg border border-white/10 bg-[var(--bg-secondary)] py-1 shadow-xl" transition:scale={{ start: 0.95, duration: 150 }}>
						<div class="px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Set Status</div>
						{#each statusOptions as opt (opt.value)}
							<button
								onclick={() => setUserStatus(opt.value)}
								class="flex w-full items-center gap-3 px-3 py-2 text-left text-sm transition hover:bg-white/5"
							>
								<span class="h-2.5 w-2.5 rounded-full {opt.color}"></span>
								<div>
									<div class="font-medium text-[var(--text-primary)]">{opt.label}</div>
									<div class="text-xs text-[var(--text-secondary)]">{opt.desc}</div>
								</div>
							</button>
						{/each}
					</div>
				{/if}
				<button
					onclick={() => (showFeedback = true)}
					class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--accent)]"
					title="Send feedback"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
					</svg>
				</button>
				<button
					onclick={() => { authStore.logout(); wsClient.disconnect(); goto('/login'); }}
					class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--danger)]"
					title="Sign out"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
						<polyline points="16 17 21 12 16 7" /><line x1="21" y1="12" x2="9" y2="12" />
					</svg>
				</button>
			</div>
		</aside>

		<!-- Main chat area -->
		<main
			class="relative flex flex-1 flex-col bg-[var(--bg-primary)]"
			ondragenter={handleDragEnter}
			ondragleave={handleDragLeave}
			ondragover={handleDragOver}
			ondrop={handleDrop}
		>
			{#if dragOver && activeChannel}
				<div class="absolute inset-0 z-50 flex items-center justify-center rounded-lg border-2 border-dashed border-[var(--accent)] bg-[var(--bg-primary)]/80 backdrop-blur-sm m-2">
					<div class="flex flex-col items-center gap-2">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
							<polyline points="17 8 12 3 7 8" />
							<line x1="12" y1="3" x2="12" y2="15" />
						</svg>
						<span class="text-lg font-semibold text-[var(--text-primary)]">Drop file to upload</span>
					</div>
				</div>
			{/if}
			{#if activeChannel}
				<!-- Channel header -->
				<header class="flex h-14 items-center justify-between border-b border-white/10 px-4 md:px-6">
					<div class="flex items-center">
						<!-- Mobile menu button -->
						<button
							onclick={() => (sidebarOpen = true)}
							class="mr-3 rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] md:hidden"
							aria-label="Open sidebar"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="18" x2="21" y2="18" />
							</svg>
						</button>
						{#if activeChannel.channel_type === 'dm'}
							<span class="mr-2 text-[var(--text-secondary)]">@</span>
						{:else if activeChannel.channel_type === 'voice'}
							<span class="mr-2">🔊</span>
						{:else}
							<span class="mr-2 text-[var(--text-secondary)]">#</span>
						{/if}
						<h2 class="font-semibold text-[var(--text-primary)]">{getChannelDisplayName()}</h2>
						{#if activeChannel.topic}
							<span class="ml-4 truncate text-sm text-[var(--text-secondary)]">{activeChannel.topic}</span>
						{/if}
					</div>
					<div class="flex items-center gap-1">
						<button
							onclick={toggleSearch}
							class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] {showSearch ? 'text-[var(--accent)]' : ''}"
							title="Search messages"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
							</svg>
						</button>
						{#if activeChannel.channel_type !== 'dm'}
							<div class="relative">
								<button
									onclick={(e) => { e.stopPropagation(); showNotifDropdown = !showNotifDropdown; }}
									class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
									title="Notification settings"
								>
									{#if notificationStore.getChannelLevel(activeChannel.id) === 'nothing'}
										<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<path d="M13.73 21a2 2 0 0 1-3.46 0" /><path d="M18.63 13A17.89 17.89 0 0 1 18 8" /><path d="M6.26 6.26A5.86 5.86 0 0 0 6 8c0 7-3 9-3 9h14" /><path d="M18 8a6 6 0 0 0-9.33-5" /><line x1="1" y1="1" x2="23" y2="23" />
										</svg>
									{:else}
										<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" /><path d="M13.73 21a2 2 0 0 1-3.46 0" />
										</svg>
									{/if}
								</button>
								{#if showNotifDropdown}
									<div class="absolute right-0 top-full z-50 mt-1 w-48 rounded-lg border border-white/10 bg-[var(--bg-secondary)] py-1 shadow-xl" onclick={(e) => e.stopPropagation()}>
										<p class="px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Notifications</p>
										{#each [['all', 'All Messages'], ['mentions', 'Only @mentions'], ['nothing', 'Nothing']] as [value, label]}
											<button
												onclick={() => { notificationStore.setChannelLevel(activeChannel.id, value as NotificationLevel); showNotifDropdown = false; }}
												class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--text-primary)] transition hover:bg-white/5"
											>
												<span class="w-4">{notificationStore.getChannelLevel(activeChannel.id) === value ? '✓' : ''}</span>
												{label}
											</button>
										{/each}
									</div>
								{/if}
							</div>
						{/if}
						<CallControls channelId={activeChannel.id} channelType={activeChannel.channel_type} />
						{#if activeChannel.channel_type !== 'dm'}
							<button
								onclick={toggleMemberPanel}
								class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] {showMemberPanel ? 'text-[var(--accent)]' : ''}"
								title="Members"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" /><path d="M23 21v-2a4 4 0 0 0-3-3.87" /><path d="M16 3.13a4 4 0 0 1 0 7.75" />
								</svg>
							</button>
						{/if}
					</div>
				</header>

				<!-- Search panel -->
				{#if showSearch}
					<div class="border-b border-white/10 bg-[var(--bg-secondary)] px-4 py-3" transition:slide={{ duration: 150 }}>
						<input
							type="text"
							bind:value={searchQuery}
							oninput={handleSearchInput}
							placeholder="Search messages..."
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						/>
						{#if searching}
							<p class="mt-2 text-xs text-[var(--text-secondary)]">Searching...</p>
						{:else if searchResults.length > 0}
							<div class="mt-2 max-h-60 space-y-1 overflow-y-auto">
								{#each searchResults as result (result.id)}
									<button
										onclick={() => jumpToSearchResult(result.id)}
										class="flex w-full flex-col rounded-lg px-3 py-2 text-left transition hover:bg-white/5"
									>
										<div class="flex items-baseline gap-2">
											<span class="text-xs font-semibold text-[var(--text-primary)]">{userStore.getDisplayName(result.senderId)}</span>
											<span class="text-xs text-[var(--text-secondary)]">{formatTime(result.createdAt)}</span>
										</div>
										<span class="truncate text-sm text-[var(--text-secondary)]">{result.content}</span>
									</button>
								{/each}
							</div>
						{:else if searchQuery.trim()}
							<p class="mt-2 text-xs text-[var(--text-secondary)]">No results found.</p>
						{/if}
					</div>
				{/if}

				<!-- Video grid (visible when in a call) -->
				<VideoGrid />

				<!-- Messages -->
				<div bind:this={messageListEl} class="flex-1 overflow-y-auto px-6 py-4" onscroll={handleMessageScroll}>
					{#if loadingOlder}
						<Skeleton variant="message" count={3} />
					{/if}
					{#each messages as msg, idx (msg.id)}
						<!-- Date separator -->
						{#if shouldShowDateSeparator(messages, idx)}
							<div class="my-4 flex items-center gap-4">
								<div class="flex-1 border-t border-white/10"></div>
								<span class="text-xs font-medium text-[var(--text-secondary)]">{formatDateSeparator(msg.createdAt)}</span>
								<div class="flex-1 border-t border-white/10"></div>
							</div>
						{/if}
						<div
							id="msg-{msg.id}" class="group relative mb-4 flex gap-3 rounded-lg px-2 py-1 transition hover:bg-white/[0.02] {msg.pending ? 'opacity-50' : ''}"
							oncontextmenu={(e) => { if (msg.senderId === authStore.user?.id) showContextMenu(e, msg.id); }}
							role="article"
							aria-label="Message from {userStore.getDisplayName(msg.senderId)}"
						>
							<Avatar userId={msg.senderId} size="md" />
							<div class="min-w-0 flex-1">
								{#if msg.replyToId}
									{@const repliedMsg = messages.find(m => m.id === msg.replyToId)}
									<button
										onclick={() => { const el = document.getElementById('msg-' + msg.replyToId); el?.scrollIntoView({ behavior: 'smooth', block: 'center' }); }}
										class="mb-1 flex items-center gap-1.5 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 14 4 9 9 4" /><path d="M20 20v-7a4 4 0 0 0-4-4H4" /></svg>
										{#if repliedMsg}
											<span class="font-medium">{userStore.getDisplayName(repliedMsg.senderId)}</span>
											<span class="truncate max-w-[200px] opacity-70">{repliedMsg.content.slice(0, 60)}{repliedMsg.content.length > 60 ? '...' : ''}</span>
										{:else}
											<span class="italic opacity-50">Original message not loaded</span>
										{/if}
									</button>
								{/if}
								<div class="flex items-baseline gap-2">
									<span class="text-sm font-semibold text-[var(--text-primary)]">
										{userStore.getDisplayName(msg.senderId)}
									</span>
									<span class="text-xs text-[var(--text-secondary)]">
										{formatTime(msg.createdAt)}
									</span>
									{#if msg.editedAt}
										<span class="text-xs text-[var(--text-secondary)]">(edited)</span>
									{/if}
								</div>

								{#if editingMessageId === msg.id}
									<!-- Edit mode -->
									<div class="mt-1">
										<input
											data-edit-input
											type="text"
											bind:value={editInput}
											onkeydown={(e) => handleEditKeydown(e, msg.id)}
											class="w-full rounded border border-[var(--accent)] bg-[var(--bg-secondary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none"
										/>
										<div class="mt-1 flex gap-2 text-xs">
											<button onclick={() => submitEdit(msg.id)} class="text-[var(--accent)] hover:underline">Save</button>
											<button onclick={cancelEdit} class="text-[var(--text-secondary)] hover:underline">Cancel</button>
											<span class="text-[var(--text-secondary)]">esc to cancel, enter to save</span>
										</div>
									</div>
								{:else if msg.messageType === 'file'}
									{@const fileInfo = parseFileMessage(msg.content)}
									{#if fileInfo && IMAGE_EXTS.test(fileInfo.filename)}
										<div class="mt-1">
											{#await getAuthenticatedBlobUrl(fileInfo.file_id)}
											<div class="flex h-40 w-64 items-center justify-center rounded-lg border border-white/10 bg-[var(--bg-secondary)]">
												<span class="text-xs text-[var(--text-secondary)]">Loading image...</span>
											</div>
											{:then blobUrl}
											<a href={blobUrl} target="_blank" rel="noopener noreferrer">
												<img src={blobUrl} alt={fileInfo.filename} class="max-h-80 max-w-sm rounded-lg border border-white/10" />
											</a>
											{:catch}
											<div class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2">
												<span class="text-sm text-[var(--text-secondary)]">Failed to load image</span>
											</div>
											{/await}
											<div class="mt-1 flex items-center gap-2 text-xs text-[var(--text-secondary)]">
												<span>{fileInfo.filename}</span>
												<span>({formatFileSize(fileInfo.size)})</span>
												{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
													<a href={blobUrl} download={fileInfo.filename} class="text-[var(--accent)] hover:underline">Download</a>
												{/await}
											</div>
										</div>
									{:else if fileInfo && VIDEO_EXTS.test(fileInfo.filename)}
										<div class="mt-1">
											{#await getAuthenticatedBlobUrl(fileInfo.file_id)}
											<div class="flex h-48 w-80 items-center justify-center rounded-lg border border-white/10 bg-[var(--bg-secondary)]">
												<span class="text-xs text-[var(--text-secondary)]">Loading video...</span>
											</div>
											{:then blobUrl}
											<!-- svelte-ignore a11y_media_has_caption -->
											<video src={blobUrl} controls class="max-h-96 max-w-lg rounded-lg border border-white/10"></video>
											{:catch}
											<div class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2">
												<span class="text-sm text-[var(--text-secondary)]">Failed to load video</span>
											</div>
											{/await}
											<div class="mt-1 flex items-center gap-2 text-xs text-[var(--text-secondary)]">
												<span>{fileInfo.filename}</span>
												<span>({formatFileSize(fileInfo.size)})</span>
												{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
													<a href={blobUrl} download={fileInfo.filename} class="text-[var(--accent)] hover:underline">Download</a>
												{/await}
											</div>
										</div>
									{:else if fileInfo}
										<div class="mt-1 inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2">
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
												<polyline points="14 2 14 8 20 8" />
											</svg>
											<span class="text-sm text-[var(--text-primary)]">{fileInfo.filename}</span>
											<span class="text-xs text-[var(--text-secondary)]">({formatFileSize(fileInfo.size)})</span>
											{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
												<a href={blobUrl} download={fileInfo.filename} class="text-xs text-[var(--accent)] hover:underline">Download</a>
											{/await}
										</div>
									{:else}
										<div class="mt-1 inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2">
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
												<polyline points="14 2 14 8 20 8" />
											</svg>
											<span class="text-sm text-[var(--text-primary)]">{msg.content}</span>
										</div>
									{/if}
								{:else}
									{@const imageUrls = extractImageUrls(msg.content)}
									{@const linkUrls = extractNonImageUrls(msg.content)}
									<div class="markdown-content mt-0.5 text-sm text-[var(--text-primary)] leading-relaxed">{@html renderMarkdown(msg.content)}</div>
									{#if imageUrls.length > 0}
										<div class="mt-2 flex flex-col gap-2">
											{#each imageUrls as imgUrl}
												<a href={imgUrl} target="_blank" rel="noopener noreferrer">
													<img
														src={imgUrl}
														alt="Linked image"
														class="max-h-80 max-w-sm rounded-lg border border-white/10"
														loading="lazy"
														onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
													/>
												</a>
											{/each}
										</div>
									{/if}
									{#if linkUrls.length > 0}
										{#each linkUrls.slice(0, 3) as linkUrl}
											{#await fetchLinkPreview(linkUrl) then preview}
												{#if preview && (preview.title || preview.description)}
													<a href={linkUrl} target="_blank" rel="noopener noreferrer" class="link-embed mt-2 block max-w-md rounded-lg border-l-4 border-[var(--accent)] bg-[var(--bg-secondary)] p-3 transition hover:bg-white/5">
														{#if preview.site_name}
															<div class="text-xs text-[var(--text-secondary)]">{preview.site_name}</div>
														{/if}
														{#if preview.title}
															<div class="text-sm font-semibold text-[var(--accent)]">{preview.title}</div>
														{/if}
														{#if preview.description}
															<div class="mt-1 text-xs text-[var(--text-secondary)] line-clamp-3">{preview.description}</div>
														{/if}
														{#if preview.image}
															<img src={preview.image} alt="" class="mt-2 max-h-40 rounded border border-white/10" loading="lazy" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
														{/if}
													</a>
												{/if}
											{/await}
										{/each}
									{/if}
								{/if}

								<!-- Reactions -->
								{#if msg.reactions && msg.reactions.size > 0}
									<div class="mt-1.5 flex flex-wrap gap-1">
										{#each Array.from(msg.reactions.entries()) as [emoji, users]}
											{@const hasReacted = users.has(authStore.user?.id ?? '')}
											<button
												onclick={() => toggleReaction(msg.id, emoji)}
												class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs transition {hasReacted ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 text-[var(--text-secondary)] hover:border-white/20 hover:bg-white/5'}"
											>
												<span>{emoji}</span>
												<span class="font-medium">{users.size}</span>
											</button>
										{/each}
									</div>
								{/if}
							</div>

							<!-- Action buttons (visible on hover) -->
							{#if !msg.pending}
								<div class="absolute right-2 top-0 hidden gap-0.5 rounded border border-white/10 bg-[var(--bg-secondary)] shadow-lg group-hover:flex">
									<button
										onclick={(e) => { e.stopPropagation(); reactionPickerMessageId = reactionPickerMessageId === msg.id ? null : msg.id; }}
										class="p-1.5 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
										title="Add reaction"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<circle cx="12" cy="12" r="10" /><path d="M8 14s1.5 2 4 2 4-2 4-2" /><line x1="9" y1="9" x2="9.01" y2="9" /><line x1="15" y1="9" x2="15.01" y2="9" />
										</svg>
									</button>
									<button
										onclick={() => startReply(msg)}
										class="p-1.5 text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
										title="Reply"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 14 4 9 9 4" /><path d="M20 20v-7a4 4 0 0 0-4-4H4" /></svg>
									</button>
									{#if msg.senderId === authStore.user?.id}
										<button
											onclick={() => startEditMessage(msg)}
											class="p-1.5 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
											title="Edit"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" /><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
											</svg>
										</button>
										<button
											onclick={() => handleDeleteMessage(msg.id)}
											class="p-1.5 text-[var(--text-secondary)] transition hover:text-[var(--danger)]"
											title="Delete"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
											</svg>
										</button>
									{:else if myRole === 'owner' || myRole === 'admin'}
										<button
											onclick={() => handleDeleteMessage(msg.id)}
											class="p-1.5 text-[var(--text-secondary)] transition hover:text-[var(--danger)]"
											title="Delete (mod)"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
											</svg>
										</button>
									{/if}
								</div>
							{/if}

							<!-- Reaction picker popup -->
							{#if reactionPickerMessageId === msg.id}
								<div
									class="absolute right-2 top-8 z-10 flex items-center gap-1 rounded-lg border border-white/10 bg-[var(--bg-secondary)] p-2 shadow-xl"
									transition:scale={{ start: 0.9, duration: 150 }}
									role="toolbar"
									aria-label="Reaction picker"
								>
									{#each QUICK_REACTIONS as emoji}
										<button
											onclick={() => toggleReaction(msg.id, emoji)}
											class="rounded p-1 text-lg transition hover:bg-white/10"
										>
											{emoji}
										</button>
									{/each}
									<button
										onclick={(e) => { e.stopPropagation(); openFullEmojiPicker(msg.id); }}
										class="rounded p-1 text-lg text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]"
										title="More emojis"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="16" /><line x1="8" y1="12" x2="16" y2="12" />
										</svg>
									</button>
								</div>
							{/if}

							<!-- Full emoji picker -->
							{#if fullEmojiPickerMessageId === msg.id}
								<!-- svelte-ignore a11y_click_events_have_key_events -->
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div
									class="absolute right-2 top-8 z-20"
									transition:scale={{ start: 0.9, duration: 150 }}
									onclick={(e) => e.stopPropagation()}
								>
									<emoji-picker
										use:bindEmojiPicker={msg.id}
										class="dark"
									></emoji-picker>
								</div>
							{/if}
						</div>
					{/each}

					{#if messages.length === 0}
						<div class="flex h-full items-center justify-center">
							<p class="text-[var(--text-secondary)]">No messages yet. Say something!</p>
						</div>
					{/if}
				</div>

				<!-- Typing indicator -->
				{#if typingUsers.length > 0}
					<div class="px-6 py-1 text-xs text-[var(--text-secondary)]">
						{typingUsers.length === 1
							? `${userStore.getDisplayName(typingUsers[0])} is typing...`
							: `${typingUsers.length} people are typing...`}
					</div>
				{/if}

				<!-- Reply banner -->
				{#if replyingTo}
					<div class="flex items-center gap-2 border-t border-white/10 bg-[var(--bg-secondary)] px-4 py-2">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 14 4 9 9 4" /><path d="M20 20v-7a4 4 0 0 0-4-4H4" /></svg>
						<span class="text-xs text-[var(--text-secondary)]">Replying to</span>
						<span class="text-xs font-medium text-[var(--text-primary)]">{userStore.getDisplayName(replyingTo.senderId)}</span>
						<span class="flex-1 truncate text-xs text-[var(--text-secondary)]">{replyingTo.content.slice(0, 60)}</span>
						<button onclick={cancelReply} class="shrink-0 rounded p-0.5 text-[var(--text-secondary)] hover:text-[var(--text-primary)]" title="Cancel reply">
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
						</button>
					</div>
				{/if}

				<!-- Message input -->
				<form onsubmit={sendMessage} class="{replyingTo ? '' : 'border-t border-white/10'} relative p-4">
					<!-- Mention autocomplete popup -->
					{#if showMentionPopup && mentionResults().length > 0}
						<div class="absolute bottom-full left-4 right-4 mb-1 rounded-lg border border-white/10 bg-[var(--bg-secondary)] shadow-lg overflow-hidden">
							{#each mentionResults() as member, i (member.user_id)}
								<button
									onclick={() => selectMention(member.username)}
									class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition {i === mentionIndex ? 'bg-[var(--accent)]/20 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
								>
									{#if member.role === 'special'}
										<div class="flex h-6 w-6 items-center justify-center rounded-full bg-yellow-500/20">
											<span class="text-xs font-bold text-yellow-400">@</span>
										</div>
										<span class="font-semibold text-yellow-400">@{member.username}</span>
										<span class="text-xs opacity-50">{member.description}</span>
									{:else}
										<Avatar userId={member.user_id} size="xs" />
										<span class="font-medium">{member.display_name}</span>
										<span class="text-xs opacity-60">@{member.username}</span>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
					<div class="flex gap-2">
						<!-- File upload button -->
						<button
							type="button"
							onclick={() => fileInputEl?.click()}
							disabled={uploading}
							class="rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] disabled:opacity-30"
							title="Upload file"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48" />
							</svg>
						</button>
						<input
							bind:this={fileInputEl}
							type="file"
							onchange={handleFileUpload}
							class="hidden"
						/>
						<textarea
							bind:this={messageInputEl}
							bind:value={messageInput}
							oninput={(e) => { handleMentionInput(); autoResizeTextarea(); }}
							onkeydown={(e) => { handleMentionKeydown(e); if (!showMentionPopup) handleInputKeydown(e); }}
							placeholder="Message {activeChannel.channel_type === 'dm' ? '@' : '#'}{getChannelDisplayName()}..."
							rows="1"
							class="flex-1 resize-none rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition placeholder:text-[var(--text-secondary)]/50 focus:border-[var(--accent)]"
						></textarea>
						<button
							type="submit"
							disabled={!messageInput.trim()}
							class="rounded-lg bg-[var(--accent)] px-4 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-30"
						>
							Send
						</button>
					</div>
					<div class="mt-1 flex items-center gap-1">
						<div class="flex items-center gap-0.5">
							<button type="button" onclick={() => wrapSelection('**', '**')} class="rounded px-1.5 py-0.5 text-xs font-bold text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]" title="Bold (Ctrl+B)">B</button>
							<button type="button" onclick={() => wrapSelection('*', '*')} class="rounded px-1.5 py-0.5 text-xs italic text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]" title="Italic (Ctrl+I)">I</button>
							<button type="button" onclick={() => wrapSelection('~~', '~~')} class="rounded px-1.5 py-0.5 text-xs line-through text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]" title="Strikethrough">S</button>
							<button type="button" onclick={() => wrapSelection('`', '`')} class="rounded px-1.5 py-0.5 text-xs font-mono text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]" title="Code (Ctrl+E)">&lt;&gt;</button>
							<button type="button" onclick={() => wrapSelection('[', '](url)')} class="rounded px-1.5 py-0.5 text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]" title="Link (Ctrl+K)">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" /></svg>
							</button>
						</div>
						<span class="mx-1 h-3 w-px bg-white/10"></span>
						<div class="flex items-center gap-2 text-[10px] text-[var(--text-secondary)]/50">
							<span><kbd class="rounded bg-white/5 px-1">Enter</kbd> send</span>
							<span><kbd class="rounded bg-white/5 px-1">Shift+Enter</kbd> new line</span>
							<span class="hidden sm:inline"><kbd class="rounded bg-white/5 px-1">↑</kbd> edit last</span>
						</div>
					</div>
				</form>
			{:else}
				<div class="flex flex-1 flex-col items-center justify-center gap-4">
					<!-- Mobile menu button when no channel selected -->
					<button
						onclick={() => (sidebarOpen = true)}
						class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] md:hidden"
					>
						Open channels
					</button>
					<div class="text-center">
						<h2 class="mb-2 text-2xl font-bold text-[var(--text-primary)]">Welcome to Chatalot</h2>
						<p class="text-[var(--text-secondary)]">
							{channelStore.channels.length === 0
								? 'Create a channel to get started.'
								: 'Select a channel to start chatting.'}
						</p>
					</div>
				</div>
			{/if}
		</main>

		<!-- Member panel (right sidebar) -->
		{#if showMemberPanel && activeChannel && activeChannel.channel_type !== 'dm'}
			<aside class="hidden w-60 flex-shrink-0 border-l border-white/10 bg-[var(--bg-secondary)] overflow-y-auto md:block">
				<div class="p-4">
					<h3 class="mb-3 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
						Members ({channelMembers.length})
					</h3>
					{#if membersLoading}
						<Skeleton variant="member" count={4} />
					{:else}
						{#each channelMembers as member (member.user_id)}
							<div class="group flex items-center gap-2 rounded-lg px-2 py-1.5 hover:bg-white/5">
								<Avatar userId={member.user_id} size="sm" showStatus />
								<div class="min-w-0 flex-1">
									<div class="flex items-center gap-1.5">
										<span class="truncate text-sm text-[var(--text-primary)]">
											{member.display_name}
										</span>
										{#if member.role === 'owner'}
											<span class="text-xs text-yellow-400" title="Owner">&#9733;</span>
										{:else if member.role === 'admin'}
											<span class="text-xs text-blue-400" title="Admin">&#9830;</span>
										{/if}
									</div>
								</div>
								<!-- Mod actions (hover) -->
								{#if member.user_id !== authStore.user?.id && (myRole === 'owner' || (myRole === 'admin' && member.role === 'member'))}
									<div class="hidden gap-0.5 group-hover:flex">
										{#if myRole === 'owner'}
											<button
												onclick={() => handleRoleChange(member.user_id, member.role === 'admin' ? 'member' : 'admin')}
												class="rounded p-1 text-xs text-[var(--text-secondary)] hover:text-[var(--accent)]"
												title={member.role === 'admin' ? 'Remove admin' : 'Make admin'}
											>
												{#if member.role === 'admin'}
													<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9" /></svg>
												{:else}
													<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="18 15 12 9 6 15" /></svg>
												{/if}
											</button>
										{/if}
										<button
											onclick={() => handleKick(member.user_id, member.display_name)}
											class="rounded p-1 text-xs text-[var(--text-secondary)] hover:text-[var(--danger)]"
											title="Kick"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
										</button>
										<button
											onclick={() => handleBan(member.user_id, member.display_name)}
											class="rounded p-1 text-xs text-[var(--text-secondary)] hover:text-[var(--danger)]"
											title="Ban"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><line x1="4.93" y1="4.93" x2="19.07" y2="19.07" /></svg>
										</button>
									</div>
								{/if}
							</div>
						{/each}
					{/if}
				</div>
			</aside>
		{/if}
	</div>

	<!-- Feedback modal -->
	{#if showFeedback}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" transition:fade={{ duration: 150 }}>
			<div class="w-full max-w-md rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6 shadow-2xl" transition:scale={{ start: 0.95, duration: 200 }}>
				<h2 class="mb-1 text-lg font-semibold text-[var(--text-primary)]">Send Feedback</h2>
				<p class="mb-4 text-sm text-[var(--text-secondary)]">Help us improve Chatalot. Your feedback creates an issue for the developers.</p>

				<form onsubmit={handleFeedbackSubmit} class="space-y-4">
					<div>
						<label for="fb-category" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">Category</label>
						<select
							id="fb-category"
							bind:value={feedbackCategory}
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						>
							<option value="bug">Bug Report</option>
							<option value="feature">Feature Request</option>
							<option value="ui">UI/UX</option>
							<option value="other">Other</option>
						</select>
					</div>

					<div>
						<label for="fb-title" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">Title</label>
						<input
							id="fb-title"
							type="text"
							bind:value={feedbackTitle}
							maxlength="200"
							placeholder="Brief summary..."
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none placeholder:text-[var(--text-secondary)]/50 focus:border-[var(--accent)]"
						/>
					</div>

					<div>
						<label for="fb-desc" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">Description</label>
						<textarea
							id="fb-desc"
							bind:value={feedbackDescription}
							maxlength="5000"
							rows="4"
							placeholder="Describe the issue or suggestion in detail..."
							class="w-full resize-none rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none placeholder:text-[var(--text-secondary)]/50 focus:border-[var(--accent)]"
						></textarea>
					</div>

					<div class="flex justify-end gap-3">
						<button
							type="button"
							onclick={() => (showFeedback = false)}
							class="rounded-lg px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						>
							Cancel
						</button>
						<button
							type="submit"
							disabled={feedbackSubmitting || !feedbackTitle.trim() || !feedbackDescription.trim()}
							class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-30"
						>
							{feedbackSubmitting ? 'Submitting...' : 'Submit'}
						</button>
					</div>
				</form>
			</div>
		</div>
	{/if}
{/if}
