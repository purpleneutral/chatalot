<script lang="ts" module>
	declare const __APP_VERSION__: string;
</script>

<script lang="ts">
	import { goto } from '$app/navigation';
	import { listChannels, createChannel, getMessages, getThreadMessages, searchMessages, searchMessagesGlobal, getChannelMembers, updateMemberRole, kickMember, banMember, getEditHistory, getReadCursors, type Channel, type ChannelMember, type Message, type MessageEdit, type ReactionInfo, type SearchOptions } from '$lib/api/channels';
	import { readReceiptStore } from '$lib/stores/readReceipts.svelte';
	import { listDms, createDm, type DmChannel } from '$lib/api/dms';
	import { searchUsers, listBlockedUsers, createReport, type UserPublic } from '$lib/api/users';
	import { uploadFile, getAuthenticatedBlobUrl, type FileUploadResponse } from '$lib/api/files';
	import { fetchLinkPreview } from '$lib/api/link-preview';
	import { getServerConfig, getPublicUrl } from '$lib/api/auth';
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
	import { listGroups, createGroup as apiCreateGroup, joinGroup, leaveGroup, deleteGroup, discoverGroups, listGroupChannels, createGroupChannel, updateGroup as apiUpdateGroup, updateChannel as apiUpdateChannel, deleteChannel as apiDeleteChannel, listGroupMembers, createInvite, acceptInvite, getInviteInfo, type Group, type GroupMember, type InviteInfo } from '$lib/api/groups';
	import { listCommunities, listCommunityGroups, listMembers as listCommunityMembers, createCommunity, getInviteInfo as getCommunityInviteInfo, acceptInvite as acceptCommunityInvite, type Community } from '$lib/api/communities';
	import { getPinnedMessages, pinMessage as apiPinMessage, unpinMessage as apiUnpinMessage, type PinnedMessage } from '$lib/api/channels';
	import { searchGifs, getTrendingGifs, type GifResult } from '$lib/api/gifs';
	import { addBookmark, removeBookmark as apiRemoveBookmark, listBookmarks } from '$lib/api/bookmarks';
	import { scheduleMessage as apiScheduleMessage, listScheduledMessages, cancelScheduledMessage, type ScheduledMessage } from '$lib/api/scheduled';
	import { createPoll as apiCreatePoll, listPolls, getPoll, votePoll, removeVote as apiRemoveVote, closePoll as apiClosePoll, type Poll } from '$lib/api/polls';
	import { listUndismissed as listUndismissedAnnouncements, dismissAnnouncement, type Announcement } from '$lib/api/announcements';
	import { listCommunityEmojis, type CustomEmoji } from '$lib/api/custom-emoji';
	import { bookmarkStore } from '$lib/stores/bookmarks.svelte';
	import { communityMemberStore } from '$lib/stores/communityMembers.svelte';
	import { preferencesStore } from '$lib/stores/preferences.svelte';
	import { searchEmoji } from '$lib/utils/emoji';
	import UserProfileCard from '$lib/components/UserProfileCard.svelte';
	import GroupSettingsCard from '$lib/components/GroupSettingsCard.svelte';
	import ChannelSettingsCard from '$lib/components/ChannelSettingsCard.svelte';
	import WhatsNew from '$lib/components/WhatsNew.svelte';
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import hljs from 'highlight.js/lib/core';
	import javascript from 'highlight.js/lib/languages/javascript';
	import typescript from 'highlight.js/lib/languages/typescript';
	import python from 'highlight.js/lib/languages/python';
	import rust from 'highlight.js/lib/languages/rust';
	import css from 'highlight.js/lib/languages/css';
	import xml from 'highlight.js/lib/languages/xml';
	import json from 'highlight.js/lib/languages/json';
	import bash from 'highlight.js/lib/languages/bash';
	import sql from 'highlight.js/lib/languages/sql';
	import yaml from 'highlight.js/lib/languages/yaml';
	import go from 'highlight.js/lib/languages/go';
	import java from 'highlight.js/lib/languages/java';
	import cpp from 'highlight.js/lib/languages/cpp';
	import markdown from 'highlight.js/lib/languages/markdown';

	// Register highlight.js languages
	hljs.registerLanguage('javascript', javascript);
	hljs.registerLanguage('js', javascript);
	hljs.registerLanguage('typescript', typescript);
	hljs.registerLanguage('ts', typescript);
	hljs.registerLanguage('python', python);
	hljs.registerLanguage('py', python);
	hljs.registerLanguage('rust', rust);
	hljs.registerLanguage('rs', rust);
	hljs.registerLanguage('css', css);
	hljs.registerLanguage('html', xml);
	hljs.registerLanguage('xml', xml);
	hljs.registerLanguage('json', json);
	hljs.registerLanguage('bash', bash);
	hljs.registerLanguage('sh', bash);
	hljs.registerLanguage('shell', bash);
	hljs.registerLanguage('sql', sql);
	hljs.registerLanguage('yaml', yaml);
	hljs.registerLanguage('yml', yaml);
	hljs.registerLanguage('go', go);
	hljs.registerLanguage('java', java);
	hljs.registerLanguage('cpp', cpp);
	hljs.registerLanguage('c', cpp);
	hljs.registerLanguage('markdown', markdown);
	hljs.registerLanguage('md', markdown);
	import { groupStore } from '$lib/stores/groups.svelte';
	import { communityStore } from '$lib/stores/communities.svelte';
	import { onMount, onDestroy, tick } from 'svelte';
	import { fade, slide, fly, scale } from 'svelte/transition';
	import { initCrypto, getSessionManager, getKeyManager, getCryptoStorage, getPersonalKey } from '$lib/crypto';
	import { getCrypto } from '$lib/crypto/wasm-loader';
	import { decryptMessage } from '$lib/crypto/decrypt';
	import { getSenderKeys } from '$lib/api/sender-keys';
	import { pushStore } from '$lib/stores/push.svelte';
	import { encryptionStore } from '$lib/stores/encryption.svelte';

	let messageInput = $state('');
	let newChannelName = $state('');
	let newChannelType = $state('text');
	let showCreateChannel = $state(false);
	let showSidebarCreateMenu = $state(false);
	let messageListEl: HTMLDivElement | undefined = $state();
	let whatsNewRef: WhatsNew | undefined = $state();
	let typingTimeout: ReturnType<typeof setTimeout> | null = null;
	let unsubWs: (() => void) | null = null;

	// App initialization flag — prevents empty-state flash before data loads
	let initialized = $state(false);
	let initError = $state<string | null>(null);

	async function loadInitialData() {
		initialized = false;
		initError = null;

		try {
			const [communities, channels, dms] = await Promise.all([
				listCommunities(),
				listChannels(),
				listDms()
			]);

			communityStore.setCommunities(communities);
			channelStore.setChannels(channels);
			dmChannels = dms;

			// Auto-select first community if none saved
			if (!communityStore.activeCommunityId && communities.length > 0) {
				communityStore.setActive(communities[0].id);
			}

			// Load groups + members for the active community
			let groups: Group[] = [];
			if (communityStore.activeCommunityId) {
				const [loadedGroups, communityMembers] = await Promise.all([
					loadCommunityGroups(communityStore.activeCommunityId),
					listCommunityMembers(communityStore.activeCommunityId)
				]);
				groups = loadedGroups;
				communityMemberStore.setMembers(communityStore.activeCommunityId, communityMembers);
			}

			// Load server-synced preferences + bookmarks
			preferencesStore.loadFromServer();
			listBookmarks().then(b => bookmarkStore.setBookmarks(b)).catch((err) => console.warn('Failed to load bookmarks:', err));
			loadScheduledMessages();

			// Populate user cache from DM contacts
			userStore.setUsers(dms.map(d => d.other_user));

			// Restore previous session state or fall back to defaults
			const savedChannel = localStorage.getItem('chatalot:activeChannel');
			const savedExpanded = localStorage.getItem('chatalot:expandedGroups');

			// Restore expanded groups
			if (savedExpanded) {
				try {
					const ids = JSON.parse(savedExpanded) as string[];
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
				...Array.from(groupChannelsMap.values()).flat().map(c => c.id)
			]);

			if (savedChannel && allChannelIds.has(savedChannel)) {
				selectChannel(savedChannel);
			} else if (groups.length > 0) {
				const firstGroupChannels = groupChannelsMap.get(groups[0].id);
				if (firstGroupChannels && firstGroupChannels.length > 0) {
					selectChannel(firstGroupChannels[0].id);
				}
			} else if (channels.length > 0) {
				selectChannel(channels[0].id);
			}

			// Subscribe to all channels + DMs via WebSocket
			subscribeToAllChannels();

			// Fetch unread counts
			try {
				const activeAtStart = channelStore.activeChannelId;
				const res = await fetch('/api/channels/unread', {
					headers: { 'Authorization': `Bearer ${authStore.accessToken}` }
				});
				if (res.ok) {
					const counts = await res.json();
					messageStore.setUnreadCounts(counts);
					// Re-clear the active channel since setUnreadCounts replaces the entire map
					const active = channelStore.activeChannelId;
					if (active && active === activeAtStart) {
						messageStore.clearUnread(active);
					}
				}
			} catch { /* ignore */ }
		} catch (err) {
			console.error('Failed to load channels:', err);
			initError = err instanceof Error ? err.message : 'Failed to load data';
			initialized = true;
			return;
		}

		initError = null;
		initialized = true;
	}

	// WebSocket connection status
	let connectionStatus = $state<'connected' | 'reconnecting' | null>(null);
	let connectionStatusTimer: ReturnType<typeof setTimeout> | null = null;

	// Blocked users
	let blockedUserIds = $state<string[]>([]);

	// DM state
	let dmChannels = $state<DmChannel[]>([]);
	let showNewDm = $state(false);
	let dmSearchQuery = $state('');
	let dmSearchResults = $state<UserPublic[]>([]);
	let dmSearchError = $state(false);
	let dmSearchLoading = $state(false);
	let dmSearchDone = $state(false);
	let dmSearchTimeout: ReturnType<typeof setTimeout> | null = null;

	/** Look up the other user's ID for a given DM channel. */
	function getPeerUserIdForDm(channelId: string | null): string | null {
		if (!channelId) return null;
		const dm = dmChannels.find(d => d.channel.id === channelId);
		return dm?.other_user.id ?? null;
	}

	/**
	 * Encode text for a channel.
	 * E2E encryption is disabled until key backup & multi-device are implemented.
	 * When re-enabled, this function should encrypt for DMs (Double Ratchet)
	 * and groups (Sender Keys) via the SessionManager.
	 */
	function encryptContent(_channelId: string, text: string): { ciphertext: number[]; nonce: number[] } {
		return {
			ciphertext: Array.from(new TextEncoder().encode(text)),
			nonce: Array.from(crypto.getRandomValues(new Uint8Array(12))),
		};
	}

	// Platform detection for keyboard shortcut display
	const modKey = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.userAgent) ? 'Cmd' : 'Ctrl';

	// File upload state
	let fileInputEl: HTMLInputElement | undefined = $state();
	let uploading = $state(false);

	// Drag & drop state
	let dragOver = $state(false);
	let dragCounter = $state(0);

	// Sidebar tab (restore from localStorage)
	let sidebarTab = $state<'groups' | 'dms'>(() => {
		if (typeof localStorage === 'undefined') return 'groups';
		const saved = localStorage.getItem('chatalot:sidebarTab');
		return saved === 'dms' ? 'dms' : 'groups';
	});

	// Sidebar search filter
	let sidebarFilter = $state('');

	// Group state
	let showCreateGroup = $state(false);
	let newGroupName = $state('');
	let newGroupDescription = $state('');
	let newGroupVisibility = $state('public');
	let newGroupAssignMemberId = $state('');
	let expandedGroupIds = $state<Set<string>>(new Set());
	let groupChannelsMap = $state<Map<string, Channel[]>>(new Map());
	let communitySwitchId = 0; // Guards against stale community-switch results
	let showDiscoverGroups = $state(false);
	let discoverGroupsList = $state<Group[]>([]);
	let showGroupChannelCreate = $state<string | null>(null);
	let newGroupChannelName = $state('');
	let newGroupChannelType = $state('text');
	let renamingChannelId = $state<string | null>(null);
	let renameChannelInput = $state('');
	let renamingGroupId = $state<string | null>(null);
	let renameGroupInput = $state('');

	// Edit state
	let editingMessageId = $state<string | null>(null);
	let editInput = $state('');

	// Reaction picker state
	let reactionPickerMessageId = $state<string | null>(null);
	let fullEmojiPickerMessageId = $state<string | null>(null);

	// Context menu state
	let contextMenuMessageId = $state<string | null>(null);
	let contextMenuPos = $state({ x: 0, y: 0 });

	// Voice context menu state (volume + kick)
	let voiceContextMenu = $state<{ userId: string; channelId: string; x: number; y: number } | null>(null);
	let isVoiceMenuSelf = $derived(voiceContextMenu?.userId === authStore.user?.id);

	// Mobile sidebar state
	let sidebarOpen = $state(false);

	// Top nav dropdown state
	let showCommunityPicker = $state(false);
	let showNavDropdown = $state(false);
	let showUserMenu = $state(false);
	let navCollapsed = $state(
		typeof localStorage !== 'undefined' && localStorage.getItem('chatalot:navCollapsed') === 'true'
	);

	let isExpandedSidebar = $derived(
		preferencesStore.preferences.sidebarLayout === 'expanded'
	);

	// Auto-close dropdown when switching to expanded mode
	$effect(() => { if (isExpandedSidebar) showNavDropdown = false; });

	function closeAllNavDropdowns() {
		showCommunityPicker = false;
		showNavDropdown = false;
		showUserMenu = false;
	}

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

	// Community invite state
	let showJoinCommunity = $state(false);
	let joinCommunityCode = $state('');
	let communityInvitePreview = $state<{ community_name: string; community_description: string | null; member_count: number; code: string } | null>(null);

	// Community creation state
	let showCreateCommunity = $state(false);
	let newCommunityName = $state('');
	let newCommunityDescription = $state('');
	let creatingCommunity = $state(false);
	let creatingGroup = $state(false);
	let creatingChannel = $state(false);
	let creatingGroupChannel = $state(false);

	// Welcome splash state
	let showWelcomeSplash = $state(false);
	let welcomeCommunity = $state<Community | null>(null);

	// Announcements
	let announcements = $state<Announcement[]>([]);

	// Custom emoji map for current community
	let customEmojiMap = $state<Map<string, CustomEmoji>>(new Map());
	let loadedCommunityEmojiId = '';

	// Report modal state
	let reportingMessageId = $state<string | null>(null);
	let reportReason = $state('');
	let submittingReport = $state(false);

	// Slow mode cooldown
	let slowModeCooldown = $state(0);
	let slowModeTimer: ReturnType<typeof setInterval> | null = null;

	// Encryption verification modal state
	let showEncryptionInfo = $state(false);
	let encryptionInfoLoading = $state(false);
	let safetyNumber = $state('');
	let ownFingerprint = $state('');
	let peerFingerprint = $state('');
	let safetyNumberCopied = $state(false);

	async function loadEncryptionInfo() {
		if (!activeChannel || activeChannel.channel_type !== 'dm') return;
		const peerUserId = getPeerUserIdForDm(activeChannel.id);
		if (!peerUserId) return;

		encryptionInfoLoading = true;
		try {
			await initCrypto();
			const crypto = await getCrypto();
			const storage = getCryptoStorage();

			const ownKey = await getKeyManager().getVerifyingKey();
			const peerKey = await storage.getPeerIdentity(peerUserId);

			ownFingerprint = crypto.compute_fingerprint(ownKey);
			if (peerKey) {
				peerFingerprint = crypto.compute_fingerprint(peerKey);
				safetyNumber = crypto.compute_safety_number(ownKey, peerKey);
			} else {
				peerFingerprint = '';
				safetyNumber = '';
			}
		} catch (err) {
			console.error('Failed to load encryption info:', err);
			safetyNumber = '';
			ownFingerprint = '';
			peerFingerprint = '';
		} finally {
			encryptionInfoLoading = false;
		}
		showEncryptionInfo = true;
	}

	// Guard against stale async loads on rapid channel switching
	let channelLoadId = 0;

	// Notification dropdown state
	let showNotifDropdown = $state(false);

	// Profile card state
	let profileCardUserId = $state<string | null>(null);
	let profileCardAnchor = $state({ x: 0, y: 0 });

	// Group settings card state
	let groupSettingsGroup = $state<import('$lib/api/groups').Group | null>(null);
	let groupSettingsAnchor = $state({ x: 0, y: 0 });

	// Channel settings card state
	let channelSettingsChannel = $state<import('$lib/api/channels').Channel | null>(null);
	let channelSettingsGroupId = $state<string | null>(null);
	let channelSettingsAnchor = $state({ x: 0, y: 0 });

	// Topic editing state
	let editingTopic = $state(false);
	let topicInput = $state('');

	// Pinned messages state
	let showPinnedPanel = $state(false);
	let pinnedMessages = $state<(PinnedMessage & { _decryptedContent?: string })[]>([]);
	let loadingPins = $state(false);
	let loadingPinsError = $state(false);

	// Polls state
	let showPollPanel = $state(false);
	let polls = $state<Poll[]>([]);
	let loadingPolls = $state(false);
	let closingPollId = $state<string | null>(null);
	let votingPollKey = $state<string | null>(null);
	let showCreatePoll = $state(false);
	let newPollQuestion = $state('');
	let newPollOptions = $state(['', '']);
	let newPollMultiSelect = $state(false);
	let newPollAnonymous = $state(false);
	let newPollExpiry = $state<number | null>(null);
	let creatingPoll = $state(false);

	// Anonymous poll vote tracking (localStorage-backed, since server strips voter_ids)
	let anonVotes = $state<Record<string, number[]>>({});

	function loadAnonVotes() {
		try {
			const stored = localStorage.getItem('chatalot_anon_votes');
			if (stored) anonVotes = JSON.parse(stored);
		} catch { /* ignore corrupt data */ }
	}

	function saveAnonVote(pollId: string, optionIndex: number, multiSelect: boolean) {
		const current = anonVotes[pollId] ?? [];
		anonVotes = {
			...anonVotes,
			[pollId]: multiSelect ? [...current, optionIndex] : [optionIndex]
		};
		localStorage.setItem('chatalot_anon_votes', JSON.stringify(anonVotes));
	}

	function removeAnonVote(pollId: string, optionIndex: number) {
		const current = anonVotes[pollId] ?? [];
		anonVotes = {
			...anonVotes,
			[pollId]: current.filter(i => i !== optionIndex)
		};
		localStorage.setItem('chatalot_anon_votes', JSON.stringify(anonVotes));
	}

	// Scroll-to-bottom button
	let showScrollBottom = $state(false);

	// Per-channel scroll position persistence
	let scrollPositions = new Map<string, number>();

	// Unread separator: track the first unread message ID per channel switch
	let unreadSeparatorMsgId = $state<string | null>(null);

	// Member list filter
	let memberFilter = $state('');

	// Keyboard shortcuts modal
	let showShortcutsModal = $state(false);

	// GIF picker state
	let showGifPicker = $state(false);
	let gifSearchQuery = $state('');
	let gifResults = $state<GifResult[]>([]);
	let gifLoading = $state(false);
	let gifError = $state(false);
	let gifSearchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

	// Emoji autocomplete state
	let showEmojiPopup = $state(false);
	let emojiQuery = $state('');
	let emojiResults = $state<{ name: string; emoji: string; custom?: boolean; url?: string }[]>([]);
	let emojiIndex = $state(0);

	// Image lightbox state
	let lightboxImage = $state<{ src: string; alt: string } | null>(null);

	// Quick switcher (Ctrl+K)
	let showQuickSwitcher = $state(false);
	let quickSwitcherQuery = $state('');
	let quickSwitcherIndex = $state(0);
	let quickSwitcherInputEl: HTMLInputElement | undefined = $state();

	type QuickSwitchItem = { id: string; name: string; type: 'channel' | 'dm' | 'group-channel'; groupName?: string; icon: string };

	let quickSwitcherResults = $derived.by(() => {
		const items: QuickSwitchItem[] = [];
		// Group channels
		for (const group of groupStore.groups) {
			for (const ch of (groupChannelsMap.get(group.id) ?? [])) {
				items.push({ id: ch.id, name: ch.name ?? '', type: 'group-channel', groupName: group.name, icon: ch.channel_type === 'voice' ? '🔊' : '#' });
			}
		}
		// Standalone channels
		for (const ch of channelStore.channels.filter(c => c.channel_type !== 'dm' && !c.group_id)) {
			items.push({ id: ch.id, name: ch.name ?? '', type: 'channel', icon: ch.channel_type === 'voice' ? '🔊' : '#' });
		}
		// DMs
		for (const dm of dmChannels) {
			items.push({ id: dm.channel.id, name: dm.other_user.display_name, type: 'dm', icon: '@' });
		}
		if (!quickSwitcherQuery.trim()) return items.slice(0, 10);
		const q = quickSwitcherQuery.toLowerCase();
		return items.filter(item => {
			const name = item.name.toLowerCase();
			const group = item.groupName?.toLowerCase() ?? '';
			return name.includes(q) || group.includes(q);
		}).slice(0, 10);
	});

	function openQuickSwitcher() {
		showQuickSwitcher = true;
		quickSwitcherQuery = '';
		quickSwitcherIndex = 0;
		tick().then(() => quickSwitcherInputEl?.focus());
	}

	function quickSwitcherSelect(item: QuickSwitchItem) {
		showQuickSwitcher = false;
		channelStore.addChannel(channelStore.channels.find(c => c.id === item.id) ?? { id: item.id, name: item.name, channel_type: 'text', topic: null, created_by: null, created_at: new Date().toISOString(), group_id: null, read_only: false, slow_mode_seconds: 0, discoverable: false, archived: false, voice_background: null });
		selectChannel(item.id);
	}

	function handleQuickSwitcherKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			quickSwitcherIndex = Math.min(quickSwitcherIndex + 1, quickSwitcherResults.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			quickSwitcherIndex = Math.max(quickSwitcherIndex - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			const item = quickSwitcherResults[quickSwitcherIndex];
			if (item) quickSwitcherSelect(item);
		}
	}

	// Notification permission prompt
	let showNotifPrompt = $state(false);
	let notifPromptDismissed = $state(
		typeof localStorage !== 'undefined' && localStorage.getItem('chatalot:notifPromptDismissed') === 'true'
	);

	// Confirmation dialog state
	let confirmDialog = $state<{
		title: string;
		message: string;
		confirmLabel?: string;
		danger?: boolean;
		inputPlaceholder?: string;
		onConfirm: (inputValue?: string) => void;
	} | null>(null);
	let confirmInput = $state('');

	function showConfirmDialog(opts: typeof confirmDialog & {}) {
		confirmInput = '';
		confirmDialog = opts;
	}

	// Collect all viewable images from current messages for lightbox navigation
	const imgRegex = /!\[[^\]]*\]\(([^)]+)\)/g;
	const srcRegex = /src="([^"]+\.(png|jpg|jpeg|gif|webp|svg)[^"]*)"/gi;
	let channelImages = $derived.by(() => {
		const seen = new Set<string>();
		const imgs: { src: string; alt: string }[] = [];
		for (const msg of messages) {
			if (!msg.content) continue;
			// Quick check before expensive regex
			if (!msg.content.includes('![') && !msg.content.includes('src=')) continue;
			// Match image URLs from file messages and inline images
			imgRegex.lastIndex = 0;
			let m: RegExpExecArray | null;
			while ((m = imgRegex.exec(msg.content)) !== null) {
				if (!seen.has(m[1])) {
					seen.add(m[1]);
					imgs.push({ src: m[1], alt: 'Image' });
				}
			}
			// Also check for direct image blob URLs in content
			srcRegex.lastIndex = 0;
			while ((m = srcRegex.exec(msg.content)) !== null) {
				const url = m[1];
				if (!seen.has(url)) {
					seen.add(url);
					imgs.push({ src: url, alt: 'Image' });
				}
			}
		}
		return imgs;
	});

	function openLightbox(src: string, alt: string = 'Image') {
		lightboxImage = { src, alt };
	}

	function closeLightbox() {
		lightboxImage = null;
	}

	function lightboxPrev() {
		if (!lightboxImage) return;
		const idx = channelImages.findIndex(i => i.src === lightboxImage!.src);
		if (idx > 0) lightboxImage = channelImages[idx - 1];
	}

	function lightboxNext() {
		if (!lightboxImage) return;
		const idx = channelImages.findIndex(i => i.src === lightboxImage!.src);
		if (idx >= 0 && idx < channelImages.length - 1) lightboxImage = channelImages[idx + 1];
	}

	function maybeShowNotifPrompt() {
		if (notifPromptDismissed) return;
		if (notificationStore.permissionState === 'granted') return;
		if (notificationStore.permissionState === 'unsupported') return;
		showNotifPrompt = true;
	}

	async function acceptNotifPrompt() {
		await notificationStore.requestPermission();
		showNotifPrompt = false;
		notifPromptDismissed = true;
		localStorage.setItem('chatalot:notifPromptDismissed', 'true');
	}

	function dismissNotifPrompt() {
		showNotifPrompt = false;
		notifPromptDismissed = true;
		localStorage.setItem('chatalot:notifPromptDismissed', 'true');
	}

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
	let searchError = $state(false);
	let searchScope = $state<'channel' | 'global'>('channel');
	let searchTimeout: ReturnType<typeof setTimeout> | null = null;
	let showSearchFilters = $state(false);
	let searchFilterSender = $state('');
	let searchFilterAfter = $state('');
	let searchFilterBefore = $state('');
	let searchFilterHasFile = $state(false);
	let searchInputEl = $state<HTMLInputElement | null>(null);

	// Infinite scroll state
	let loadingOlder = $state(false);
	let loadingOlderError = $state(false);
	const FETCH_LIMIT = 50;

	function parseReactions(reactions?: ReactionInfo[]): Map<string, Set<string>> | undefined {
		if (!reactions || reactions.length === 0) return undefined;
		const map = new Map<string, Set<string>>();
		for (const r of reactions) {
			map.set(r.emoji, new Set(r.user_ids));
		}
		return map;
	}

	// Member panel state
	let showMemberPanel = $state(false);
	let membersLoading = $state(false);

	// Bookmarks panel state
	let showBookmarksPanel = $state(false);

	// Edit history modal state
	let showEditHistory = $state(false);
	let editHistoryEntries = $state<{ content: string; editedAt: string }[]>([]);
	let editHistoryLoading = $state(false);

	// Scheduled messages state
	let showSchedulePicker = $state(false);
	let scheduleDate = $state('');
	let scheduleTime = $state('');
	let scheduledMessages = $state<ScheduledMessage[]>([]);
	let showScheduledPanel = $state(false);

	// Thread panel state
	let showThreadPanel = $state(false);
	let activeThreadRootId = $state<string | null>(null);
	let activeThreadRoot = $state<ChatMessage | null>(null);
	let threadMessages = $state<ChatMessage[]>([]);
	let threadLoading = $state(false);
	let threadMessageInput = $state('');
	let threadReactionPickerMsgId = $state<string | null>(null);
	let threadFullEmojiPickerMsgId = $state<string | null>(null);
	let threadTextareaEl: HTMLTextAreaElement | undefined = $state();

	// Chat collapse state (during voice calls)
	let chatCollapsed = $state(false);

	// Deferred update reload (don't interrupt voice calls)
	let pendingUpdate = $state(false);

	// Feedback modal state
	let showFeedback = $state(false);
	let feedbackTitle = $state('');
	let feedbackDescription = $state('');
	let feedbackCategory = $state('bug');
	let feedbackSubmitting = $state(false);
	let feedbackScreenshot = $state<File | null>(null);
	let feedbackScreenshotPreview = $state<string | null>(null);
	let feedbackFileInput = $state<HTMLInputElement | null>(null);

	function setFeedbackScreenshot(file: File | null) {
		if (feedbackScreenshotPreview) {
			URL.revokeObjectURL(feedbackScreenshotPreview);
			feedbackScreenshotPreview = null;
		}
		feedbackScreenshot = file;
		if (file) {
			feedbackScreenshotPreview = URL.createObjectURL(file);
		}
	}

	function handleFeedbackPaste(e: ClipboardEvent) {
		const items = e.clipboardData?.items;
		if (!items) return;
		for (const item of items) {
			if (item.type.startsWith('image/')) {
				const file = item.getAsFile();
				if (file) {
					setFeedbackScreenshot(file);
					break;
				}
			}
		}
	}

	async function handleFeedbackSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!feedbackTitle.trim() || !feedbackDescription.trim()) return;
		feedbackSubmitting = true;
		try {
			const res = await submitFeedback({
				title: feedbackTitle.trim(),
				description: feedbackDescription.trim(),
				category: feedbackCategory,
				screenshot: feedbackScreenshot
			});
			toastStore.success(res.message);
			showFeedback = false;
			feedbackTitle = '';
			feedbackDescription = '';
			feedbackCategory = 'bug';
			setFeedbackScreenshot(null);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to submit feedback');
		} finally {
			feedbackSubmitting = false;
		}
	}

	// Member panel functions
	async function toggleMemberPanel() {
		showMemberPanel = !showMemberPanel;
		if (showMemberPanel) { sidebarOpen = false; showNavDropdown = false; showBookmarksPanel = false; showScheduledPanel = false; showThreadPanel = false; } // Ensure mutual exclusivity
		if (showMemberPanel && channelStore.activeChannelId) {
			membersLoading = true;
			try {
				const members = await getChannelMembers(channelStore.activeChannelId);
				memberStore.setMembers(channelStore.activeChannelId, members);
				// Populate user cache from member info
				userStore.setUsers(members.map(m => {
					const existing = userStore.getUser(m.user_id);
					return {
						...existing,
						id: m.user_id,
						username: m.username,
						display_name: m.display_name,
						avatar_url: m.avatar_url,
						banner_url: existing?.banner_url ?? null,
						status: existing?.status ?? 'online',
						custom_status: existing?.custom_status ?? null
					};
				}));
			} catch (err) {
				toastStore.error('Failed to load members');
			} finally {
				membersLoading = false;
			}
		}
	}

	function toggleBookmarksPanel() {
		showBookmarksPanel = !showBookmarksPanel;
		if (showBookmarksPanel) {
			showMemberPanel = false;
			showScheduledPanel = false;
			showThreadPanel = false;
			sidebarOpen = false;
			showNavDropdown = false;
		}
	}

	async function removeBookmarkFromPanel(bookmarkId: string) {
		try {
			await apiRemoveBookmark(bookmarkId);
			bookmarkStore.removeBookmark(bookmarkId);
			toastStore.success('Bookmark removed');
		} catch { toastStore.error('Failed to remove bookmark'); }
	}

	async function loadEditHistory(channelId: string, messageId: string) {
		editHistoryLoading = true;
		showEditHistory = true;
		try {
			const edits = await getEditHistory(channelId, messageId);
			// Look up the sender so we can decrypt properly
			const msg = messageStore.getMessages(channelId).find(m => m.id === messageId);
			const senderId = msg?.senderId ?? '';
			editHistoryEntries = await Promise.all(edits.map(async (e) => ({
				content: (await decryptMessage(
					channelId,
					senderId,
					e.old_ciphertext,
					undefined,
					senderId === authStore.user?.id ? getPeerUserIdForDm(channelId) : undefined,
				)).content,
				editedAt: e.edited_at
			})));
		} catch {
			toastStore.error('Failed to load edit history');
			showEditHistory = false;
		} finally {
			editHistoryLoading = false;
		}
	}

	async function encryptPreview(text: string): Promise<string | undefined> {
		try {
			const key = await getPersonalKey();
			if (!key) return undefined;
			const { personalEncrypt } = await import('$lib/crypto/personal-key');
			return await personalEncrypt(key, text);
		} catch { return undefined; }
	}

	async function decryptPreview(encrypted: string): Promise<string | null> {
		try {
			const key = await getPersonalKey();
			if (!key) return null;
			const { personalDecrypt } = await import('$lib/crypto/personal-key');
			return await personalDecrypt(key, encrypted);
		} catch { return null; }
	}

	async function handleScheduleMessage() {
		const text = messageInput.trim();
		if (!text || !channelStore.activeChannelId || !scheduleDate || !scheduleTime) return;
		const scheduledFor = new Date(`${scheduleDate}T${scheduleTime}`);
		if (scheduledFor.getTime() <= Date.now()) {
			toastStore.error('Scheduled time must be in the future');
			return;
		}
		try {
			const { ciphertext, nonce } = await encryptContent(channelStore.activeChannelId, text);
			const preview = await encryptPreview(text);
			const msg = await apiScheduleMessage(channelStore.activeChannelId, JSON.stringify(ciphertext), JSON.stringify(nonce), scheduledFor.toISOString(), preview);
			scheduledMessages = [...scheduledMessages, { ...msg, content: text }];
			messageInput = '';
			showSchedulePicker = false;
			scheduleDate = '';
			scheduleTime = '';
			toastStore.success('Message scheduled');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to schedule message');
		}
	}

	async function loadScheduledMessages() {
		try {
			const msgs = await listScheduledMessages();
			// Decrypt previews in parallel
			const decrypted = await Promise.all(msgs.map(async (m) => {
				if (m.content_preview) {
					const content = await decryptPreview(m.content_preview);
					return { ...m, content: content ?? undefined };
				}
				return m;
			}));
			scheduledMessages = decrypted;
		} catch { /* ignore */ }
	}

	function toggleScheduledPanel() {
		showScheduledPanel = !showScheduledPanel;
		if (showScheduledPanel) {
			showMemberPanel = false;
			showBookmarksPanel = false;
			showThreadPanel = false;
			sidebarOpen = false;
			showNavDropdown = false;
			loadScheduledMessages();
		}
	}

	async function handleCancelScheduled(id: string) {
		try {
			await cancelScheduledMessage(id);
			scheduledMessages = scheduledMessages.filter(m => m.id !== id);
			toastStore.success('Scheduled message cancelled');
		} catch { toastStore.error('Failed to cancel'); }
	}

	async function openThread(rootId: string) {
		const channelId = channelStore.activeChannelId;
		if (!channelId) return;

		// Find the root message
		const root = messageStore.getMessages(channelId).find(m => m.id === rootId);
		if (!root) return;

		activeThreadRootId = rootId;
		activeThreadRoot = root;
		threadMessages = [];
		showThreadPanel = true;
		threadLoading = true;
		// Close other panels
		showMemberPanel = false;
		showBookmarksPanel = false;
		showScheduledPanel = false;
		sidebarOpen = false;
		showNavDropdown = false;

		try {
			const msgs = await getThreadMessages(channelId, rootId);
			// Guard: user may have opened a different thread or closed the panel during fetch
			if (activeThreadRootId !== rootId) return;
			threadMessages = await Promise.all(msgs.map(async (m) => {
				const { content, encrypted } = await decryptMessage(
					m.channel_id,
					m.sender_id ?? '',
					m.ciphertext,
					m.id,
					m.sender_id === authStore.user?.id ? getPeerUserIdForDm(channelId) : undefined,
				);
				return {
					id: m.id,
					channelId: m.channel_id,
					senderId: m.sender_id ?? '',
					content,
					encryptionStatus: encrypted ? 'encrypted' as const : 'plaintext' as const,
					messageType: m.message_type,
					replyToId: m.reply_to_id ?? null,
					editedAt: m.edited_at ?? null,
					createdAt: m.created_at,
					threadId: m.thread_id ?? null,
					reactions: m.reactions ? new Map(m.reactions.map(r => [r.emoji, new Set(r.user_ids)])) : undefined,
				};
			}));
		} catch {
			if (activeThreadRootId === rootId) toastStore.error('Failed to load thread');
		} finally {
			if (activeThreadRootId === rootId) threadLoading = false;
		}
	}

	function closeThread() {
		showThreadPanel = false;
		activeThreadRootId = null;
		activeThreadRoot = null;
		threadMessages = [];
		threadMessageInput = '';
		threadReactionPickerMsgId = null;
		threadFullEmojiPickerMsgId = null;
	}

	async function sendThreadMessage() {
		const text = threadMessageInput.trim();
		if (!text || !channelStore.activeChannelId || !activeThreadRootId) return;

		if (slowModeCooldown > 0) {
			toastStore.error(`Slow mode active — wait ${slowModeCooldown}s`);
			return;
		}

		const channelId = channelStore.activeChannelId;
		const threadId = activeThreadRootId;
		const { ciphertext, nonce } = await encryptContent(channelId, text);

		// Optimistic add to thread panel
		const tempId = crypto.randomUUID();
		const optimisticMsg: ChatMessage = {
			id: tempId,
			channelId,
			senderId: authStore.user?.id ?? '',
			content: text,
			messageType: 'text',
			replyToId: null,
			editedAt: null,
			createdAt: new Date().toISOString(),
			threadId,
			pending: true,
		};
		threadMessages = [...threadMessages, optimisticMsg];
		threadMessageInput = '';

		const sent = wsClient.send({
			type: 'send_message',
			channel_id: channelId,
			ciphertext,
			nonce,
			message_type: 'text',
			reply_to: null,
			sender_key_id: null,
			thread_id: threadId,
		});
		if (!sent) {
			threadMessages = threadMessages.filter(m => m.id !== tempId);
			toastStore.error('Thread reply not sent — connection lost');
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

	function handleKick(userId: string, displayName: string) {
		if (!channelStore.activeChannelId) return;
		showConfirmDialog({
			title: 'Kick Member',
			message: `Are you sure you want to kick ${displayName} from this channel?`,
			confirmLabel: 'Kick',
			danger: true,
			onConfirm: async () => {
				try {
					await kickMember(channelStore.activeChannelId!, userId);
					memberStore.removeMember(channelStore.activeChannelId!, userId);
					toastStore.success(`${displayName} was kicked`);
				} catch (err: any) {
					toastStore.error(err?.message || 'Failed to kick member');
				}
			}
		});
	}

	function handleVoiceKick(userId: string, channelId: string) {
		const displayName = userStore.getDisplayName(userId);
		voiceContextMenu = null;
		showConfirmDialog({
			title: 'Kick from Voice',
			message: `Kick ${displayName} from the voice channel?`,
			confirmLabel: 'Kick',
			danger: true,
			onConfirm: () => {
				wsClient.send({ type: 'kick_from_voice', channel_id: channelId, user_id: userId });
				toastStore.success(`${displayName} was kicked from voice`);
			},
		});
	}

	function handleBan(userId: string, displayName: string) {
		if (!channelStore.activeChannelId) return;
		showConfirmDialog({
			title: 'Ban Member',
			message: `Ban ${displayName} from this channel? You can optionally provide a reason.`,
			confirmLabel: 'Ban',
			danger: true,
			inputPlaceholder: 'Reason (optional)',
			onConfirm: async (reason) => {
				try {
					await banMember(channelStore.activeChannelId!, userId, reason || undefined);
					memberStore.removeMember(channelStore.activeChannelId!, userId);
					toastStore.success(`${displayName} was banned`);
				} catch (err: any) {
					toastStore.error(err?.message || 'Failed to ban member');
				}
			}
		});
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
			? presenceStore.getTypingUsers(channelStore.activeChannelId).filter(uid => !blockedUserIds.includes(uid))
			: []
	);
	let myRole = $derived(
		channelStore.activeChannelId && authStore.user?.id
			? memberStore.getMyRole(channelStore.activeChannelId, authStore.user.id)
			: 'member'
	);
	// Is the active channel read-only for the current user?
	let isReadOnlyForMe = $derived.by(() => {
		const ch = channelStore.activeChannel;
		if (!ch?.read_only) return false;
		// Admins/owners are exempt from read-only
		if (ch.group_id) {
			const role = getMyGroupRole(ch.group_id);
			if (role === 'owner' || role === 'admin') return false;
		}
		return true;
	});

	// Can the current user kick others from voice in a given channel?
	function canKickInChannel(channelId: string): boolean {
		if (!authStore.user?.id) return false;
		for (const [groupId, channels] of groupChannelsMap) {
			if (channels.some(c => c.id === channelId)) {
				const group = groupStore.groups.find(g => g.id === groupId);
				return group?.owner_id === authStore.user.id;
			}
		}
		return false;
	}
	// Derived for VideoGrid (checks active call channel)
	let canKickFromVoice = $derived(
		voiceStore.activeCall?.channelId ? canKickInChannel(voiceStore.activeCall.channelId) : false
	);
	let voiceChannelBackground = $derived.by(() => {
		const callChId = voiceStore.activeCall?.channelId;
		if (!callChId) return null;
		for (const channels of groupChannelsMap.values()) {
			const ch = channels.find(c => c.id === callChId);
			if (ch?.voice_background) return ch.voice_background;
		}
		return null;
	});

	let channelMembers = $derived(
		channelStore.activeChannelId
			? memberStore.getMembers(channelStore.activeChannelId)
			: []
	);
	let filteredMembers = $derived(
		memberFilter.trim()
			? channelMembers.filter(m => {
				const q = memberFilter.toLowerCase();
				return m.username.toLowerCase().includes(q)
					|| m.display_name.toLowerCase().includes(q);
			})
			: channelMembers
	);

	const ONLINE_STATUSES = new Set(['online', 'idle', 'dnd']);
	let onlineMembers = $derived(
		filteredMembers.filter(m => ONLINE_STATUSES.has(presenceStore.getStatus(m.user_id)))
	);
	let offlineMembers = $derived(
		filteredMembers.filter(m => !ONLINE_STATUSES.has(presenceStore.getStatus(m.user_id)))
	);

	function subscribeToAllChannels() {
		// Rejoin voice call FIRST — send join_voice before subscribe so the
		// server has us in the voice session before the subscribe handler
		// sends VoiceStateUpdate to us. This prevents a race where we get
		// a premature participant list that triggers dead peer connections.
		webrtcManager.rejoinAfterReconnect();

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
		loadAnonVotes();

		if (!authStore.isAuthenticated) {
			goto('/login');
			return;
		}

		// Fetch server config (caches public URL for invite links)
		getServerConfig().catch((err) => console.warn('Failed to load server config:', err));

		// Refresh user data from server (keeps is_admin, avatar_url etc. current)
		try {
			const me = await getMe();
			authStore.updateUser(me);
		} catch (err) {
			console.warn('Failed to refresh user data:', err);
		}

		// Populate user cache with current user
		if (authStore.user) {
			userStore.setUser(authStore.user as UserPublic);
		}

		// Initialize E2E crypto, ensure keys exist, and replenish prekeys
		try {
			await initCrypto();
			await getKeyManager().ensureKeysRegistered();
			getKeyManager().replenishPrekeys().catch((err) =>
				console.error('Failed to replenish prekeys:', err)
			);
		} catch (err) {
			console.error('Failed to initialize crypto:', err);
		}

		// Initialize push notifications (silently checks VAPID key availability)
		pushStore.init().catch((err) => console.warn('Push init failed:', err));

		// Listen for SW postMessage from notification clicks
		navigator.serviceWorker?.addEventListener('message', handleSwMessage);

		// Listen for version update events BEFORE connecting WS to avoid race
		window.addEventListener('chatalot:update-available', handleUpdateAvailable);
		window.addEventListener('chatalot:connection', handleConnectionChange as EventListener);

		// Connect WebSocket (or re-register handler if already connected)
		unsubWs = wsClient.onMessage(handleServerMessage);
		wsClient.onAuthenticated(subscribeToAllChannels);
		if (wsClient.isConnected) {
			// Already connected (navigated back from admin/settings) — re-subscribe
			subscribeToAllChannels();
		} else {
			wsClient.connect();
		}

		// Load blocked user IDs
		listBlockedUsers().then(blocks => {
			blockedUserIds = blocks.map(b => b.blocked_id);
		}).catch((err) => console.warn('Failed to load blocked users:', err));

		// Listen for block/unblock events to refresh the list
		window.addEventListener('chatalot:blocks-changed', handleBlocksChanged);

		// Listen for identity key changes (TOFU violations)
		window.addEventListener('chatalot:identity-key-changed', handleIdentityKeyChanged as EventListener);

		// Listen for slow mode cooldown events
		window.addEventListener('chatalot:slow-mode', handleSlowModeEvent as EventListener);

		// Load communities + channels + DMs
		await loadInitialData();

		if (!initError) {
			// Show welcome splash for active community on first visit
			if (communityStore.activeCommunityId) {
				const activeCom = communityStore.communities.find(c => c.id === communityStore.activeCommunityId);
				if (activeCom?.welcome_message) {
					const dismissKey = `chatalot:welcomeDismissed:${activeCom.id}`;
					if (!localStorage.getItem(dismissKey)) {
						welcomeCommunity = activeCom;
						showWelcomeSplash = true;
					}
				}
			}

			// Load announcements
			try {
				announcements = await listUndismissedAnnouncements();
			} catch (err) {
				console.warn('Failed to load announcements:', err);
			}

			// Load custom emojis for active community
			if (communityStore.activeCommunityId) {
				loadCustomEmojis(communityStore.activeCommunityId);
			}
		}

		// Close context menu on click outside
		document.addEventListener('click', closeContextMenu);

		// Notification click → navigate to channel
		window.addEventListener('chatalot:navigate-channel', handleNotifNavigate as EventListener);

		// New DM channel → add to sidebar
		window.addEventListener('chatalot:new-dm-channel', handleNewDmChannel as EventListener);

		// Poll events
		window.addEventListener('chatalot:poll-created', handlePollCreated as EventListener);
		window.addEventListener('chatalot:poll-voted', handlePollVoted as EventListener);
		window.addEventListener('chatalot:poll-vote-removed', handlePollVoteRemoved as EventListener);
		window.addEventListener('chatalot:poll-closed', handlePollClosed as EventListener);

		// Announcement events
		window.addEventListener('chatalot:announcement', handleAnnouncementEvent as EventListener);

		// Thread reply events
		window.addEventListener('chatalot:thread-reply', handleThreadReply as EventListener);
		window.addEventListener('chatalot:thread-message-confirmed', handleThreadMessageConfirmed as EventListener);
		window.addEventListener('chatalot:thread-message-edited', handleThreadMessageEdited as EventListener);
		window.addEventListener('chatalot:thread-message-deleted', handleThreadMessageDeleted as EventListener);
		window.addEventListener('chatalot:thread-reaction-updated', handleThreadReactionUpdated as EventListener);
		window.addEventListener('chatalot:message-edit-cancelled', handleExternalEditCancel as EventListener);
		window.addEventListener('chatalot:message-reply-cancelled', handleExternalReplyCancel as EventListener);

		// Idle detection
		setupIdleDetection();

		// Leave voice call on page unload (refresh/close) so the server cleans up
		// immediately instead of waiting for the 15s grace period
		window.addEventListener('beforeunload', handleBeforeUnload);
	});

	onDestroy(() => {
		unsubWs?.();
		// Don't disconnect WS or leave voice call on navigation —
		// only on explicit logout (handled in the logout button handler).
		// This keeps voice calls alive when visiting admin/settings.
		document.removeEventListener('click', closeContextMenu);
		window.removeEventListener('chatalot:navigate-channel', handleNotifNavigate as EventListener);
		window.removeEventListener('chatalot:new-dm-channel', handleNewDmChannel as EventListener);
		window.removeEventListener('chatalot:update-available', handleUpdateAvailable);
		window.removeEventListener('chatalot:connection', handleConnectionChange as EventListener);
		window.removeEventListener('chatalot:blocks-changed', handleBlocksChanged);
		window.removeEventListener('chatalot:identity-key-changed', handleIdentityKeyChanged as EventListener);
		window.removeEventListener('chatalot:slow-mode', handleSlowModeEvent as EventListener);
		window.removeEventListener('chatalot:poll-created', handlePollCreated as EventListener);
		window.removeEventListener('chatalot:poll-voted', handlePollVoted as EventListener);
		window.removeEventListener('chatalot:poll-vote-removed', handlePollVoteRemoved as EventListener);
		window.removeEventListener('chatalot:poll-closed', handlePollClosed as EventListener);
		window.removeEventListener('chatalot:announcement', handleAnnouncementEvent as EventListener);
		window.removeEventListener('chatalot:thread-reply', handleThreadReply as EventListener);
		window.removeEventListener('chatalot:thread-message-confirmed', handleThreadMessageConfirmed as EventListener);
		window.removeEventListener('chatalot:thread-message-edited', handleThreadMessageEdited as EventListener);
		window.removeEventListener('chatalot:thread-message-deleted', handleThreadMessageDeleted as EventListener);
		window.removeEventListener('chatalot:thread-reaction-updated', handleThreadReactionUpdated as EventListener);
		window.removeEventListener('chatalot:message-edit-cancelled', handleExternalEditCancel as EventListener);
		window.removeEventListener('chatalot:message-reply-cancelled', handleExternalReplyCancel as EventListener);
		window.removeEventListener('beforeunload', handleBeforeUnload);
		navigator.serviceWorker?.removeEventListener('message', handleSwMessage);

		// Clean up timers
		if (typingTimeout) clearTimeout(typingTimeout);
		if (dmSearchTimeout) clearTimeout(dmSearchTimeout);
		if (slowModeTimer) clearInterval(slowModeTimer);
		if (gifSearchDebounceTimer) clearTimeout(gifSearchDebounceTimer);
		if (searchTimeout) clearTimeout(searchTimeout);
		if (connectionStatusTimer) clearTimeout(connectionStatusTimer);
		clearIdleDetection();

		// Clean up blob URLs
		if (feedbackScreenshotPreview) URL.revokeObjectURL(feedbackScreenshotPreview);
	});

	// ── Idle Detection ──
	let idleTimer: ReturnType<typeof setTimeout> | undefined;
	let isAutoIdle = false;
	const IDLE_TIMEOUT = 5 * 60 * 1000; // 5 minutes

	function resetIdleTimer() {
		if (idleTimer) clearTimeout(idleTimer);
		// If we auto-set idle, restore to online on activity
		if (isAutoIdle) {
			isAutoIdle = false;
			const savedStatus = localStorage.getItem('chatalot:status') ?? 'online';
			if (savedStatus === 'online' || savedStatus === 'idle') {
				wsClient.send({ type: 'update_presence', status: 'online' });
				if (authStore.user) presenceStore.setStatus(authStore.user.id, 'online');
			}
		}
		idleTimer = setTimeout(() => {
			// Only auto-idle if user's chosen status is "online"
			const savedStatus = localStorage.getItem('chatalot:status') ?? 'online';
			if (savedStatus === 'online' && wsClient.isConnected) {
				isAutoIdle = true;
				wsClient.send({ type: 'update_presence', status: 'idle' });
				if (authStore.user) presenceStore.setStatus(authStore.user.id, 'idle');
			}
		}, IDLE_TIMEOUT);
	}

	function setupIdleDetection() {
		const events = ['mousemove', 'mousedown', 'keydown', 'touchstart', 'scroll'];
		events.forEach(e => document.addEventListener(e, resetIdleTimer, { passive: true }));
		resetIdleTimer();
	}

	function clearIdleDetection() {
		if (idleTimer) clearTimeout(idleTimer);
		const events = ['mousemove', 'mousedown', 'keydown', 'touchstart', 'scroll'];
		events.forEach(e => document.removeEventListener(e, resetIdleTimer));
	}

	function handleSwMessage(event: MessageEvent) {
		if (event.data?.type === 'navigate-channel' && event.data.channelId) {
			window.dispatchEvent(
				new CustomEvent('chatalot:navigate-channel', { detail: event.data.channelId })
			);
		}
	}

	function handleUpdateAvailable() {
		pendingUpdate = true;
	}

	async function handleBlocksChanged() {
		try {
			const blocks = await listBlockedUsers();
			blockedUserIds = blocks.map(b => b.blocked_id);
		} catch (err) {
			console.warn('Failed to load blocked users:', err);
		}
	}

	function handleIdentityKeyChanged(e: CustomEvent<{ userId: string }>) {
		encryptionStore.addKeyChange(e.detail.userId);
	}

	function handlePollCreated(e: CustomEvent<{ pollId: string; channelId: string; createdBy: string; question: string }>) {
		if (e.detail.channelId === channelStore.activeChannelId && showPollPanel) {
			// Reload polls to get full data
			loadPolls();
		}
	}

	function handlePollVoted(e: CustomEvent<{ pollId: string; channelId: string; optionIndex: number; voterId: string | null }>) {
		if (e.detail.channelId !== channelStore.activeChannelId) return;
		const { pollId, optionIndex, voterId } = e.detail;
		// Skip if this is our own vote (already optimistically updated)
		if (voterId === authStore.user?.id) return;
		polls = polls.map(p => {
			if (p.id !== pollId) return p;
			return { ...p, votes: p.votes.map((v, i) => {
				if (i === optionIndex) {
					const newVoterIds = voterId && !p.anonymous ? [...v.voter_ids, voterId] : v.voter_ids;
					return { ...v, count: v.count + 1, voter_ids: newVoterIds };
				}
				return v;
			}) };
		});
	}

	function handlePollVoteRemoved(e: CustomEvent<{ pollId: string; channelId: string; optionIndex: number; voterId: string | null }>) {
		if (e.detail.channelId !== channelStore.activeChannelId) return;
		const { pollId, optionIndex, voterId } = e.detail;
		// Skip if this is our own unvote (already optimistically updated)
		if (voterId === authStore.user?.id) return;
		polls = polls.map(p => {
			if (p.id !== pollId) return p;
			return { ...p, votes: p.votes.map((v, i) => {
				if (i === optionIndex) {
					const newVoterIds = voterId ? v.voter_ids.filter(id => id !== voterId) : v.voter_ids;
					return { ...v, count: Math.max(0, v.count - 1), voter_ids: newVoterIds };
				}
				return v;
			}) };
		});
	}

	function handlePollClosed(e: CustomEvent<{ pollId: string; channelId: string }>) {
		if (e.detail.channelId !== channelStore.activeChannelId) return;
		polls = polls.map(p => p.id === e.detail.pollId ? { ...p, closed: true } : p);
	}

	function handleAnnouncementEvent(e: CustomEvent<{ id: string; title: string; body: string; created_by: string; created_at: string }>) {
		const ann = e.detail;
		// Add to list if not already present
		if (!announcements.find(a => a.id === ann.id)) {
			announcements = [ann, ...announcements];
		}
	}

	function handleThreadReply(e: CustomEvent<{ threadId: string; message: ChatMessage }>) {
		if (showThreadPanel && activeThreadRootId === e.detail.threadId) {
			// Add to thread panel if not a duplicate and not from us (our optimistic message is already there)
			const msg = e.detail.message;
			if (!threadMessages.some(m => m.id === msg.id) && msg.senderId !== authStore.user?.id) {
				threadMessages = [...threadMessages, msg];
			}
		}
	}

	function handleThreadMessageConfirmed(e: CustomEvent<{ channelId: string; newId: string; createdAt: string; threadId: string }>) {
		if (!showThreadPanel || activeThreadRootId !== e.detail.threadId) return;
		const idx = threadMessages.findIndex(m => m.pending && m.threadId === e.detail.threadId);
		if (idx !== -1) {
			threadMessages[idx] = { ...threadMessages[idx], id: e.detail.newId, createdAt: e.detail.createdAt, pending: false };
			threadMessages = [...threadMessages];
		}
	}

	function handleThreadMessageEdited(e: CustomEvent<{ messageId: string; content: string; editedAt: string }>) {
		if (!showThreadPanel) return;
		const { messageId, content, editedAt } = e.detail;
		if (activeThreadRoot?.id === messageId) {
			activeThreadRoot = { ...activeThreadRoot, content, editedAt };
		}
		const idx = threadMessages.findIndex(m => m.id === messageId);
		if (idx !== -1) {
			threadMessages[idx] = { ...threadMessages[idx], content, editedAt };
			threadMessages = [...threadMessages];
		}
	}

	function handleThreadMessageDeleted(e: CustomEvent<{ messageId: string }>) {
		if (!showThreadPanel) return;
		const { messageId } = e.detail;
		if (activeThreadRoot?.id === messageId) {
			closeThread();
			toastStore.error('Thread was deleted');
			return;
		}
		threadMessages = threadMessages.filter(m => m.id !== messageId);
	}

	function handleThreadReactionUpdated(e: CustomEvent<{ messageId: string; userId: string; emoji: string; action: 'add' | 'remove' }>) {
		if (!showThreadPanel) return;
		const { messageId, userId, emoji, action } = e.detail;

		function updateReactions(msg: ChatMessage): ChatMessage {
			const reactions = new Map(msg.reactions ?? []);
			const users = new Set(reactions.get(emoji) ?? []);
			if (action === 'add') {
				users.add(userId);
				reactions.set(emoji, users);
			} else {
				users.delete(userId);
				if (users.size === 0) reactions.delete(emoji);
				else reactions.set(emoji, users);
			}
			return { ...msg, reactions };
		}

		if (activeThreadRoot?.id === messageId) {
			activeThreadRoot = updateReactions(activeThreadRoot);
		}
		const idx = threadMessages.findIndex(m => m.id === messageId);
		if (idx !== -1) {
			threadMessages[idx] = updateReactions(threadMessages[idx]);
			threadMessages = [...threadMessages];
		}
	}

	async function handleDismissAnnouncement(id: string) {
		announcements = announcements.filter(a => a.id !== id);
		try {
			await dismissAnnouncement(id);
		} catch (err) {
			console.warn('Failed to dismiss announcement:', err);
		}
	}

	function handleSlowModeEvent(e: CustomEvent<{ seconds: number }>) {
		slowModeCooldown = e.detail.seconds;
		if (slowModeTimer) clearInterval(slowModeTimer);
		slowModeTimer = setInterval(() => {
			slowModeCooldown--;
			if (slowModeCooldown <= 0) {
				slowModeCooldown = 0;
				if (slowModeTimer) { clearInterval(slowModeTimer); slowModeTimer = null; }
			}
		}, 1000);
	}

	function handleConnectionChange(e: CustomEvent<string>) {
		if (e.detail === 'reconnecting') {
			connectionStatus = 'reconnecting';
		} else if (e.detail === 'connected') {
			connectionStatus = 'connected';
			// Auto-hide "connected" after 3 seconds
			if (connectionStatusTimer) clearTimeout(connectionStatusTimer);
			connectionStatusTimer = setTimeout(() => {
				if (connectionStatus === 'connected') connectionStatus = null;
				connectionStatusTimer = null;
			}, 3000);

			// Re-sync unread counts after reconnect
			const activeBeforeSync = channelStore.activeChannelId;
			fetch('/api/channels/unread', {
				headers: { 'Authorization': `Bearer ${authStore.accessToken}` }
			}).then(async res => {
				if (res.ok) {
					const counts = await res.json();
					messageStore.setUnreadCounts(counts);
					const active = channelStore.activeChannelId;
					if (active && active === activeBeforeSync) messageStore.clearUnread(active);
				}
			}).catch((err) => console.warn('Failed to sync unread counts:', err));

			// Clear stale pending messages across all channels (confirmations lost during disconnect)
			messageStore.clearAllPending();

			// Reload messages for the active channel to catch anything missed
			const activeId = channelStore.activeChannelId;
			if (activeId) {
				getMessages(activeId, undefined, FETCH_LIMIT).then(async rawMessages => {
					// Guard: channel may have changed during the fetch
					if (channelStore.activeChannelId !== activeId) return;
					const reversed = rawMessages.reverse();
					const chatMsgs: ChatMessage[] = await Promise.all(reversed.map(async m => {
						const { content, encrypted } = await decryptMessage(
							m.channel_id,
							m.sender_id,
							m.ciphertext,
							m.id,
							m.sender_id === authStore.user?.id ? getPeerUserIdForDm(activeId) : undefined,
						);
						return {
							id: m.id,
							channelId: m.channel_id,
							senderId: m.sender_id,
							content,
							encryptionStatus: encrypted ? 'encrypted' as const : 'plaintext' as const,
							messageType: m.message_type,
							replyToId: m.reply_to_id,
							editedAt: m.edited_at,
							createdAt: m.created_at,
							threadId: m.thread_id ?? null,
							threadReplyCount: m.thread_reply_count ?? undefined,
							threadLastReplyAt: m.thread_last_reply_at ?? undefined,
							reactions: m.reactions ? new Map(m.reactions.map((r: ReactionInfo) => [r.emoji, new Set(r.user_ids)])) : undefined,
						};
					}));
					messageStore.setMessages(activeId, chatMsgs, FETCH_LIMIT);
				}).catch((err) => console.warn('Failed to reload messages after reconnect:', err));
			}
		}
	}

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
				group_id: null,
				read_only: false,
				slow_mode_seconds: 0,
				discoverable: false,
				archived: false,
				voice_background: null
			});
		}
	}

	function closeContextMenu() {
		contextMenuMessageId = null;
		reactionPickerMessageId = null;
		fullEmojiPickerMessageId = null;
		threadReactionPickerMsgId = null;
		threadFullEmojiPickerMsgId = null;
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
				threadFullEmojiPickerMsgId = null;
			}
		};
		node.addEventListener('emoji-click', handler);
		return {
			destroy() {
				node.removeEventListener('emoji-click', handler);
			}
		};
	}

	function openThreadFullEmojiPicker(messageId: string) {
		threadReactionPickerMsgId = null;
		threadFullEmojiPickerMsgId = messageId;
	}

	async function selectChannel(channelId: string) {
		// Increment load ID to invalidate any in-flight async loads from previous channel
		const thisLoadId = ++channelLoadId;

		// Send stop_typing for the channel we're leaving
		if (channelStore.activeChannelId && typingTimeout) {
			wsClient.send({ type: 'stop_typing', channel_id: channelStore.activeChannelId });
			clearTimeout(typingTimeout);
			typingTimeout = null;
		}

		// Reset slow mode cooldown on channel switch
		slowModeCooldown = 0;
		if (slowModeTimer) { clearInterval(slowModeTimer); slowModeTimer = null; }

		// Close search panel on channel switch to avoid showing stale results
		if (showSearch) {
			showSearch = false;
			searchQuery = '';
			searchResults = [];
			searchScope = 'channel';
		}

		// Reset message count tracker to prevent scroll-to-bottom on channel switch
		prevMessageCount = 0;

		// Save draft of current channel before switching
		if (channelStore.activeChannelId) {
			const draft = messageInput.trim();
			if (draft) {
				localStorage.setItem(`chatalot:draft:${channelStore.activeChannelId}`, draft);
			} else {
				localStorage.removeItem(`chatalot:draft:${channelStore.activeChannelId}`);
			}
		}

		// Restore draft for new channel
		messageInput = localStorage.getItem(`chatalot:draft:${channelId}`) ?? '';

		// Save scroll position of current channel before switching
		if (channelStore.activeChannelId && messageListEl) {
			scrollPositions.set(channelStore.activeChannelId, messageListEl.scrollTop);
		}

		// Compute unread separator before clearing
		const unreadCount = messageStore.getUnreadCount(channelId);
		const existingMsgs = messageStore.getMessages(channelId);
		if (unreadCount > 0 && existingMsgs.length >= unreadCount) {
			// The separator goes before the first unread message
			unreadSeparatorMsgId = existingMsgs[existingMsgs.length - unreadCount].id;
		} else {
			unreadSeparatorMsgId = null;
		}
		channelStore.setActive(channelId);
		messageStore.clearUnread(channelId);
		sidebarOpen = false;
		showNavDropdown = false;
		memberFilter = '';
		editingMessageId = null;
		editInput = '';
		showEditHistory = false;
		showGifPicker = false;
		replyingTo = null;
		showThreadPanel = false;
		activeThreadRootId = null;
		threadMessages = [];
		polls = [];
		if (showPollPanel) loadPolls();
		localStorage.setItem('chatalot:activeChannel', channelId);
		tick().then(() => messageInputEl?.focus());

		// Preload members for @mention autocomplete
		getChannelMembers(channelId)
			.then((members) => {
				if (thisLoadId !== channelLoadId) return; // stale
				memberStore.setMembers(channelId, members);
				userStore.setUsers(members.map(m => {
					const existing = userStore.getUser(m.user_id);
					return {
						...existing,
						id: m.user_id,
						username: m.username,
						display_name: m.display_name,
						avatar_url: m.avatar_url,
						banner_url: existing?.banner_url ?? null,
						status: existing?.status ?? 'online',
						custom_status: existing?.custom_status ?? null
					};
				}));
			})
			.catch((err) => console.warn('Failed to load channel members:', err));

		// Load message history if not already fetched from server
		if (!messageStore.hasLoadedHistory(channelId)) {
			messageStore.setLoading(channelId, true);
			try {
				const rawMessages = await getMessages(channelId, undefined, FETCH_LIMIT);
				// Discard if user switched channels during fetch
				if (thisLoadId !== channelLoadId) return;
				const reversed = rawMessages.reverse();
				const ch = channelStore.channels.find(c => c.id === channelId);
				const isDmChannel = ch?.channel_type === 'dm';

				let chatMessages: ChatMessage[];
				if (isDmChannel) {
					await initCrypto();
					if (thisLoadId !== channelLoadId) return;
					const sm = getSessionManager();
					chatMessages = await Promise.all(reversed.map(async (m) => {
						const bytes = new Uint8Array(m.ciphertext);
						const content = await sm.decryptOrFallback(
							m.sender_id === authStore.user?.id ? getPeerUserIdForDm(channelId) : m.sender_id,
							bytes,
							m.id,
							m.channel_id,
						);
						return {
							id: m.id,
							channelId: m.channel_id,
							senderId: m.sender_id,
							content,
							encryptionStatus: 'encrypted' as const,
							messageType: m.message_type,
							replyToId: m.reply_to_id,
							editedAt: m.edited_at,
							createdAt: m.created_at,
							threadId: m.thread_id ?? null,
							threadReplyCount: m.thread_reply_count ?? undefined,
							threadLastReplyAt: m.thread_last_reply_at ?? undefined,
							reactions: parseReactions(m.reactions),
						};
					}));
				} else {
					chatMessages = await Promise.all(reversed.map(async (m) => {
						const { content, encrypted } = await decryptMessage(
							m.channel_id,
							m.sender_id,
							m.ciphertext,
							m.id,
						);
						return {
							id: m.id,
							channelId: m.channel_id,
							senderId: m.sender_id,
							content,
							encryptionStatus: encrypted ? 'encrypted' as const : 'plaintext' as const,
							messageType: m.message_type,
							replyToId: m.reply_to_id,
							editedAt: m.edited_at,
							createdAt: m.created_at,
							threadId: m.thread_id ?? null,
							threadReplyCount: m.thread_reply_count ?? undefined,
							threadLastReplyAt: m.thread_last_reply_at ?? undefined,
							reactions: parseReactions(m.reactions),
						};
					}));
				}
				if (thisLoadId !== channelLoadId) return;
				messageStore.setMessages(channelId, chatMessages, FETCH_LIMIT);

				// Mark the latest message as read
				if (chatMessages.length > 0) {
					const lastMsg = chatMessages[chatMessages.length - 1];
					wsClient.send({ type: 'mark_read', channel_id: channelId, message_id: lastMsg.id });
				}

				// Load read cursors for read receipt display
				getReadCursors(channelId)
					.then(cursors => readReceiptStore.setChannelCursors(channelId, cursors))
					.catch(err => console.warn('Failed to load read cursors:', err));

				// Load sender keys for group channels
				if (!isDmChannel) {
					try {
						await initCrypto();
						const distributions = await getSenderKeys(channelId);
						const sm = getSessionManager();
						for (const dist of distributions) {
							if (dist.user_id !== authStore.user?.id) {
								await sm.processSenderKeyDistribution(
									channelId,
									dist.user_id,
									JSON.stringify(dist.distribution),
								);
							}
						}
					} catch (err) {
						console.error('Failed to load sender keys:', err);
					}
				}
			} catch (err) {
				console.error('Failed to load messages:', err);
			} finally {
				messageStore.setLoading(channelId, false);
			}
		} else {
			// History already loaded — still send mark_read to update server cursor
			const msgs = messageStore.getMessages(channelId);
			if (msgs.length > 0) {
				const lastMsg = msgs[msgs.length - 1];
				wsClient.send({ type: 'mark_read', channel_id: channelId, message_id: lastMsg.id });
			}
		}

		// Refresh member list if panel is open
		if (showMemberPanel) {
			getChannelMembers(channelId)
				.then((members) => {
					memberStore.setMembers(channelId, members);
					userStore.setUsers(members.map(m => {
						const existing = userStore.getUser(m.user_id);
						return {
							...existing,
							id: m.user_id,
							username: m.username,
							display_name: m.display_name,
							avatar_url: m.avatar_url,
							banner_url: existing?.banner_url ?? null,
							status: existing?.status ?? 'online',
							custom_status: existing?.custom_status ?? null
						};
					}));
				})
				.catch(console.error);
		}

		await tick();
		const savedScroll = scrollPositions.get(channelId);
		if (messageListEl && savedScroll !== undefined && savedScroll < messageListEl.scrollHeight - messageListEl.clientHeight - 150) {
			messageListEl.scrollTop = savedScroll;
		} else {
			scrollToBottom();
		}
	}

	function scrollToBottom(smooth = false) {
		if (messageListEl) {
			if (smooth) {
				messageListEl.scrollTo({ top: messageListEl.scrollHeight, behavior: 'smooth' });
			} else {
				messageListEl.scrollTop = messageListEl.scrollHeight;
			}
		}
	}

	function isNearBottom(): boolean {
		if (!messageListEl) return true;
		return messageListEl.scrollHeight - messageListEl.scrollTop - messageListEl.clientHeight < 150;
	}

	function processSlashCommands(text: string): string {
		if (text.startsWith('/me ')) {
			return `*${text.slice(4)}*`;
		}
		if (text === '/shrug' || text.startsWith('/shrug ')) {
			const rest = text.slice(6).trim();
			return rest ? `${rest} ¯\\_(ツ)_/¯` : '¯\\_(ツ)_/¯';
		}
		if (text === '/tableflip' || text.startsWith('/tableflip ')) {
			const rest = text.slice(10).trim();
			return rest ? `${rest} (╯°□°)╯︵ ┻━┻` : '(╯°□°)╯︵ ┻━┻';
		}
		if (text === '/unflip' || text.startsWith('/unflip ')) {
			const rest = text.slice(7).trim();
			return rest ? `${rest} ┬─┬ノ( º _ ºノ)` : '┬─┬ノ( º _ ºノ)';
		}
		if (text === '/lenny' || text.startsWith('/lenny ')) {
			const rest = text.slice(6).trim();
			return rest ? `${rest} ( ͡° ͜ʖ ͡°)` : '( ͡° ͜ʖ ͡°)';
		}
		return text;
	}

	async function sendMessage(e: SubmitEvent) {
		e.preventDefault();
		let text = messageInput.trim();
		if (!text || !channelStore.activeChannelId) return;

		if (slowModeCooldown > 0) {
			toastStore.error(`Slow mode active — wait ${slowModeCooldown}s`);
			return;
		}

		// Process slash commands
		text = processSlashCommands(text);

		// Encrypt for DM (Double Ratchet) or group (Sender Keys)
		const { ciphertext, nonce } = await encryptContent(channelStore.activeChannelId, text);

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
		if (channelStore.activeChannelId) {
			localStorage.removeItem(`chatalot:draft:${channelStore.activeChannelId}`);
		}
		if (messageInputEl) messageInputEl.style.height = 'auto';

		// Send via WebSocket
		const sent = wsClient.send({
			type: 'send_message',
			channel_id: channelStore.activeChannelId,
			ciphertext,
			nonce,
			message_type: 'text',
			reply_to: replyingTo?.id ?? null,
			sender_key_id: null
		});

		if (!sent) {
			// Remove the optimistic pending message
			messageStore.removeMessage(channelStore.activeChannelId, tempId);
			messageInput = text;
			toastStore.error('Message not sent — connection lost');
			return;
		}

		// Clear reply state
		replyingTo = null;

		// Prompt for notification permission on first message
		maybeShowNotifPrompt();

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

		// Send behavior: Enter or Ctrl+Enter (configurable)
		if (e.key === 'Enter' && !showMentionPopup) {
			const sendOnEnter = preferencesStore.preferences.sendBehavior === 'enter';
			const shouldSend = sendOnEnter ? !e.shiftKey : (e.ctrlKey || e.metaKey);
			if (shouldSend) {
				e.preventDefault();
				if (messageInput.trim()) {
					messageInputEl?.closest('form')?.requestSubmit();
				}
				return;
			}
		}

		// Up arrow to edit last own message (when input is empty)
		if (e.key === 'ArrowUp' && !messageInput.trim()) {
			const myId = authStore.user?.id;
			if (myId) {
				const lastOwn = [...messages].reverse().find(m => m.senderId === myId && !m.pending && m.messageType !== 'file');
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
		if (!name || creatingChannel) return;
		creatingChannel = true;
		try {
			const channel = await createChannel(name, newChannelType);
			channelStore.addChannel(channel);
			wsClient.send({ type: 'subscribe', channel_ids: [channel.id] });
			selectChannel(channel.id);
			newChannelName = '';
			newChannelType = 'text';
			showCreateChannel = false;
		} catch (err) {
			console.error('Failed to create channel:', err);
			toastStore.error(err instanceof Error ? err.message : 'Failed to create channel');
		} finally {
			creatingChannel = false;
		}
	}

	// DM search with debounce
	function handleDmSearch() {
		if (dmSearchTimeout) clearTimeout(dmSearchTimeout);
		dmSearchDone = false;
		if (dmSearchQuery.length < 2) {
			dmSearchResults = [];
			dmSearchLoading = false;
			return;
		}
		dmSearchLoading = true;
		dmSearchTimeout = setTimeout(async () => {
			try {
				dmSearchResults = await searchUsers(dmSearchQuery);
				dmSearchResults = dmSearchResults.filter(u => u.id !== authStore.user?.id);
				dmSearchError = false;
			} catch (err) {
				toastStore.error('User search failed');
				dmSearchResults = [];
				dmSearchError = true;
			} finally {
				dmSearchLoading = false;
				dmSearchDone = true;
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
			dmSearchDone = false;
		} catch (err) {
			console.error('Failed to create DM:', err);
			toastStore.error(err instanceof Error ? err.message : 'Failed to start conversation');
		}
	}

	async function handleFileUpload(fileArg?: File) {
		const file = fileArg || fileInputEl?.files?.[0];
		if (!file || !channelStore.activeChannelId) return;

		const MAX_FILE_SIZE = 100 * 1024 * 1024; // 100 MB (matches server default)
		if (file.size > MAX_FILE_SIZE) {
			toastStore.error(`File too large (${(file.size / 1024 / 1024).toFixed(1)} MB). Maximum is 100 MB.`);
			return;
		}

		const channelId = channelStore.activeChannelId;
		uploading = true;
		try {
			const result = await uploadFile(file, channelId);

			// Guard: user may have switched channels during upload
			if (channelStore.activeChannelId !== channelId) {
				toastStore.error('File uploaded but channel changed — message not sent');
				return;
			}

			// Send a file message with the file ID
			const fileMsg = JSON.stringify({
				file_id: result.id,
				filename: file.name,
				size: result.size_bytes
			});

			const { ciphertext, nonce } = await encryptContent(channelId, fileMsg);

			// Optimistic add BEFORE WS send to prevent race with server echo
			const tempId = `temp-${Date.now()}`;
			messageStore.addMessage(channelId, {
				id: tempId,
				channelId,
				senderId: authStore.user?.id ?? '',
				content: fileMsg,
				messageType: 'file',
				replyToId: null,
				editedAt: null,
				createdAt: new Date().toISOString(),
				pending: true
			});

			const fileSent = wsClient.send({
				type: 'send_message',
				channel_id: channelId,
				ciphertext,
				nonce,
				message_type: 'file',
				reply_to: null,
				sender_key_id: null
			});

			if (!fileSent) {
				messageStore.removeMessage(channelId, tempId);
				toastStore.error('File not sent — connection lost');
				return;
			}
			await tick();
			scrollToBottom();
		} catch (err) {
			console.error('File upload failed:', err);
			toastStore.error(err instanceof Error ? err.message : 'Failed to upload file');
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
		showConfirmDialog({
			title: 'Delete Message',
			message: 'Are you sure you want to delete this message? This cannot be undone.',
			confirmLabel: 'Delete',
			danger: true,
			onConfirm: () => {
				const sent = wsClient.send({ type: 'delete_message', message_id: messageId });
				if (sent) {
					messageStore.deleteMessage(messageId);
					if (replyingTo?.id === messageId) replyingTo = null;
				} else {
					toastStore.error('Failed to delete — connection lost');
				}
			}
		});
		contextMenuMessageId = null;
	}

	async function startEditMessage(msg: ChatMessage) {
		if (msg.messageType === 'file') return;
		editingMessageId = msg.id;
		editInput = msg.content;
		contextMenuMessageId = null;
		await tick();
		const el = document.querySelector<HTMLTextAreaElement>('textarea[data-edit-input]');
		el?.focus();
	}

	function cancelEdit() {
		editingMessageId = null;
		editInput = '';
	}

	function handleExternalEditCancel(e: Event) {
		const { messageId } = (e as CustomEvent).detail;
		if (editingMessageId === messageId) {
			cancelEdit();
		}
	}

	function handleExternalReplyCancel(e: Event) {
		const { messageId } = (e as CustomEvent).detail;
		if (replyingTo?.id === messageId) {
			replyingTo = null;
		}
	}

	async function submitEdit(messageId: string) {
		const text = editInput.trim();
		if (!text || !channelStore.activeChannelId) return;

		const { ciphertext, nonce } = await encryptContent(channelStore.activeChannelId, text);

		const sent = wsClient.send({ type: 'edit_message', message_id: messageId, ciphertext, nonce });
		if (!sent) {
			toastStore.error('Cannot edit message while offline');
			editingMessageId = null;
			editInput = '';
			return;
		}
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
		// Clamp position to keep menu within viewport
		const menuW = 200, menuH = 280;
		const x = Math.min(e.clientX, window.innerWidth - menuW);
		const y = Math.min(e.clientY, window.innerHeight - menuH);
		contextMenuPos = { x: Math.max(0, x), y: Math.max(0, y) };
	}

	// Reactions
	function toggleReaction(messageId: string, emoji: string) {
		const msg = messages.find(m => m.id === messageId)
			?? threadMessages.find(m => m.id === messageId)
			?? (activeThreadRoot?.id === messageId ? activeThreadRoot : null);
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
		threadReactionPickerMsgId = null;
	}

	function formatRelativeTime(isoString: string): string {
		const d = new Date(isoString);
		const now = Date.now();
		const diff = now - d.getTime();
		const secs = Math.floor(diff / 1000);
		if (secs < 60) return 'just now';
		const mins = Math.floor(secs / 60);
		if (mins < 60) return `${mins}m ago`;
		const hrs = Math.floor(mins / 60);
		if (hrs < 24) return `${hrs}h ago`;
		const days = Math.floor(hrs / 24);
		if (days < 7) return `${days}d ago`;
		return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
	}

	function formatTime(isoString: string): string {
		if (preferencesStore.preferences.relativeTimestamps) {
			return formatRelativeTime(isoString);
		}
		const d = new Date(isoString);
		const use24h = preferencesStore.preferences.timeFormat === '24h';
		return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: !use24h });
	}

	function formatFullTimestamp(isoString: string): string {
		const d = new Date(isoString);
		const use24h = preferencesStore.preferences.timeFormat === '24h';
		return d.toLocaleDateString([], { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })
			+ ' at '
			+ d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: !use24h });
	}

	/** Get display name respecting community nicknames. */
	function getDisplayNameForContext(userId: string | null): string {
		if (!userId) return 'Deleted User';
		if (communityStore.activeCommunityId) {
			const nickname = communityMemberStore.getNickname(communityStore.activeCommunityId, userId);
			if (nickname) return nickname;
		}
		return userStore.getDisplayName(userId);
	}

	async function copyToClipboard(text: string, successMsg: string) {
		try {
			await navigator.clipboard.writeText(text);
			toastStore.success(successMsg);
		} catch {
			toastStore.error('Failed to copy to clipboard');
		}
	}

	function copyMessageText(msgId: string) {
		const msg = messages.find(m => m.id === msgId);
		if (msg) {
			copyToClipboard(msg.content, 'Message text copied');
		}
		contextMenuMessageId = null;
	}

	async function toggleBookmark(messageId: string) {
		contextMenuMessageId = null;
		if (bookmarkStore.isBookmarked(messageId)) {
			const bookmark = bookmarkStore.getByMessageId(messageId);
			if (bookmark) {
				try {
					await apiRemoveBookmark(bookmark.id);
					bookmarkStore.removeBookmark(bookmark.id);
					toastStore.success('Bookmark removed');
				} catch { toastStore.error('Failed to remove bookmark'); }
			}
		} else {
			try {
				const bookmark = await addBookmark(messageId);
				bookmarkStore.addBookmark(bookmark);
				toastStore.success('Message bookmarked');
			} catch { toastStore.error('Failed to bookmark message'); }
		}
	}

	async function saveTopic() {
		if (!activeChannel?.group_id || !channelStore.activeChannelId) return;
		try {
			const updated = await apiUpdateChannel(activeChannel.group_id, channelStore.activeChannelId, { topic: topicInput.trim() || undefined });
			channelStore.updateChannel(updated);
			// Also update in groupChannelsMap
			const groupChannels = groupChannelsMap.get(activeChannel.group_id);
			if (groupChannels) {
				const idx = groupChannels.findIndex(c => c.id === channelStore.activeChannelId);
				if (idx >= 0) groupChannels[idx] = updated;
			}
			toastStore.success('Topic updated');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to update topic');
		}
		editingTopic = false;
	}

	function openProfileCard(userId: string, event: MouseEvent) {
		profileCardUserId = userId;
		profileCardAnchor = { x: event.clientX, y: event.clientY };
	}

	function closeProfileCard() {
		profileCardUserId = null;
	}

	function openGroupSettings(group: import('$lib/api/groups').Group, event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		groupSettingsGroup = group;
		groupSettingsAnchor = { x: event.clientX, y: event.clientY };
	}

	function closeGroupSettings() {
		groupSettingsGroup = null;
	}

	function openChannelSettings(channel: import('$lib/api/channels').Channel, groupId: string, event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		channelSettingsChannel = channel;
		channelSettingsGroupId = groupId;
		channelSettingsAnchor = { x: event.clientX, y: event.clientY };
	}

	function closeChannelSettings() {
		channelSettingsChannel = null;
		channelSettingsGroupId = null;
	}

	function getMyGroupRole(groupId: string): string {
		const group = groupStore.groups.find(g => g.id === groupId);
		if (!group) return 'member';
		if (group.owner_id === authStore.user?.id) return 'owner';
		// Personal groups: assigned member gets 'owner' role
		if (group.assigned_member_id === authStore.user?.id) return 'owner';
		// Community moderator+ gets 'admin' for personal groups
		if (group.assigned_member_id && communityStore.activeCommunityId && authStore.user?.id) {
			const cm = communityMemberStore.getMember(communityStore.activeCommunityId, authStore.user.id);
			if (cm && (cm.role === 'owner' || cm.role === 'admin' || cm.role === 'moderator')) return 'admin';
		}
		// Check community-level role
		if (communityStore.activeCommunityId && authStore.user?.id) {
			const cm = communityMemberStore.getMember(communityStore.activeCommunityId, authStore.user.id);
			if (cm) return cm.role;
		}
		return 'member';
	}

	function isCommunityModeratorOrAbove(): boolean {
		if (!communityStore.activeCommunityId || !authStore.user?.id) return false;
		const cm = communityMemberStore.getMember(communityStore.activeCommunityId, authStore.user.id);
		if (!cm) return false;
		return cm.role === 'owner' || cm.role === 'admin' || cm.role === 'moderator';
	}

	async function startDmFromProfileCard(targetUserId: string) {
		let user = userStore.getUser(targetUserId);
		if (!user) {
			try {
				user = await getUser(targetUserId);
				userStore.setUser(user);
			} catch {
				toastStore.error('Failed to open DM');
				return;
			}
		}
		await startDm(user);
	}

	async function loadPinnedMessages() {
		if (!channelStore.activeChannelId) return;
		loadingPins = true;
		loadingPinsError = false;
		try {
			const pins = await getPinnedMessages(channelStore.activeChannelId);
			pinnedMessages = await Promise.all(pins.map(async (pin) => ({
				...pin,
				_decryptedContent: (await decryptMessage(
					pin.channel_id,
					pin.sender_id ?? '',
					pin.ciphertext,
					pin.id,
					pin.sender_id === authStore.user?.id
						? getPeerUserIdForDm(channelStore.activeChannelId)
						: undefined,
				)).content,
			})));
			messageStore.setPinnedIds(channelStore.activeChannelId, pins.map(p => p.id));
		} catch (err) {
			console.warn('Failed to load pins:', err);
			loadingPinsError = true;
		} finally {
			loadingPins = false;
		}
	}

	async function handlePinMessage(messageId: string) {
		if (!channelStore.activeChannelId) return;
		const pinCount = messageStore.getPinnedCount(channelStore.activeChannelId);
		if (pinCount >= 50) {
			toastStore.error('Pin limit reached (50 per channel). Unpin a message first.');
			return;
		}
		try {
			await apiPinMessage(channelStore.activeChannelId, messageId);
			if (pinCount >= 45) {
				toastStore.success(`Message pinned (${pinCount + 1}/50 pins used)`);
			} else {
				toastStore.success('Message pinned');
			}
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to pin message');
		}
	}

	async function handleUnpinMessage(messageId: string) {
		if (!channelStore.activeChannelId) return;
		try {
			await apiUnpinMessage(channelStore.activeChannelId, messageId);
			pinnedMessages = pinnedMessages.filter(p => p.id !== messageId);
			toastStore.success('Message unpinned');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to unpin message');
		}
	}

	async function togglePinnedPanel() {
		showPinnedPanel = !showPinnedPanel;
		if (showPinnedPanel) {
			await loadPinnedMessages();
		}
	}

	// ── Polls ──

	async function loadPolls() {
		if (!channelStore.activeChannelId) return;
		loadingPolls = true;
		try {
			polls = await listPolls(channelStore.activeChannelId);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to load polls');
		} finally {
			loadingPolls = false;
		}
	}

	async function togglePollPanel() {
		showPollPanel = !showPollPanel;
		if (showPollPanel) {
			await loadPolls();
		}
	}

	function openCreatePoll() {
		newPollQuestion = '';
		newPollOptions = ['', ''];
		newPollMultiSelect = false;
		newPollAnonymous = false;
		newPollExpiry = null;
		showCreatePoll = true;
	}

	async function handleCreatePoll() {
		if (!channelStore.activeChannelId || creatingPoll) return;
		const question = newPollQuestion.trim();
		const options = newPollOptions.map(o => o.trim()).filter(o => o);
		if (!question || options.length < 2) {
			toastStore.error('Need a question and at least 2 options');
			return;
		}
		creatingPoll = true;
		try {
			const poll = await apiCreatePoll(channelStore.activeChannelId, {
				question,
				options,
				multi_select: newPollMultiSelect,
				anonymous: newPollAnonymous,
				expires_in_minutes: newPollExpiry ?? undefined,
			});
			polls = [poll, ...polls];
			showCreatePoll = false;
			showPollPanel = true;
			toastStore.success('Poll created');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to create poll');
		} finally {
			creatingPoll = false;
		}
	}

	async function handleVotePoll(pollId: string, optionIndex: number) {
		const voteKey = `${pollId}:${optionIndex}`;
		if (votingPollKey) return;
		const poll = polls.find(p => p.id === pollId);
		if (!poll || poll.closed) return;
		// Also check client-side expiry
		if (poll.expires_at && new Date(poll.expires_at) < new Date()) return;
		const userId = authStore.user?.id;
		if (!userId) return;
		votingPollKey = voteKey;

		// Check if already voted for this option
		const optVotes = poll.votes[optionIndex];
		const alreadyVoted = poll.anonymous
			? (anonVotes[pollId] ?? []).includes(optionIndex)
			: optVotes?.voter_ids.includes(userId);

		try {
			if (alreadyVoted) {
				await apiRemoveVote(pollId, optionIndex);
				if (poll.anonymous) removeAnonVote(pollId, optionIndex);
				// Optimistic update
				polls = polls.map(p => {
					if (p.id !== pollId) return p;
					return { ...p, votes: p.votes.map((v, i) => i === optionIndex ? { ...v, count: Math.max(0, v.count - 1), voter_ids: v.voter_ids.filter(id => id !== userId) } : v) };
				});
			} else {
				const prevAnonVotes = poll.anonymous ? (anonVotes[pollId] ?? []) : [];
				await votePoll(pollId, optionIndex);
				if (poll.anonymous) saveAnonVote(pollId, optionIndex, poll.multi_select);
				// For single-select, remove previous votes first (optimistic)
				polls = polls.map(p => {
					if (p.id !== pollId) return p;
					const newVotes = p.votes.map((v, i) => {
						if (i === optionIndex) {
							return { ...v, count: v.count + 1, voter_ids: p.anonymous ? v.voter_ids : [...v.voter_ids, userId] };
						}
						if (!p.multi_select) {
							const hadVote = p.anonymous
								? prevAnonVotes.includes(i)
								: v.voter_ids.includes(userId);
							if (hadVote) {
								return { ...v, count: Math.max(0, v.count - 1), voter_ids: p.anonymous ? v.voter_ids : v.voter_ids.filter(id => id !== userId) };
							}
						}
						return v;
					});
					return { ...p, votes: newVotes };
				});
			}
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to vote');
			// Reload to get correct state
			await loadPolls();
		} finally {
			votingPollKey = null;
		}
	}

	async function handleClosePoll(pollId: string) {
		if (closingPollId) return;
		closingPollId = pollId;
		try {
			await apiClosePoll(pollId);
			polls = polls.map(p => p.id === pollId ? { ...p, closed: true } : p);
			toastStore.success('Poll closed');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to close poll');
		} finally {
			closingPollId = null;
		}
	}

	function isReadReceiptPoint(msgs: typeof messages, idx: number): boolean {
		const msg = msgs[idx];
		if (activeChannel?.channel_type === 'dm') {
			// In DMs: only show on own messages, at the end of a consecutive own-message group
			if (msg.senderId !== authStore.user?.id) return false;
			const next = msgs[idx + 1];
			return !next || next.senderId !== msg.senderId;
		}
		// In group channels: show at the end of each sender group
		const next = msgs[idx + 1];
		return !next || next.senderId !== msg.senderId;
	}

	function isGroupedMessage(msgs: typeof messages, idx: number): boolean {
		if (idx === 0) return false;
		const prev = msgs[idx - 1];
		const curr = msgs[idx];
		if (prev.senderId !== curr.senderId) return false;
		if (curr.replyToId) return false;
		if (curr.messageType === 'file' || prev.messageType === 'file') return false;
		const prevTime = new Date(prev.createdAt).getTime();
		const currTime = new Date(curr.createdAt).getTime();
		return (currTime - prevTime) < 5 * 60 * 1000;
	}

	const IMAGE_EXTS = /\.(png|jpe?g|gif|webp|svg|bmp|ico)$/i;
	const VIDEO_EXTS = /\.(mp4|webm|mov)$/i;
	const AUDIO_EXTS = /\.(mp3|wav|ogg|flac|m4a|aac|opus|wma)$/i;
	const IMAGE_URL_REGEX = /https?:\/\/[^\s<>"']+\.(png|jpe?g|gif|webp|svg|bmp|ico)(\?[^\s<>"']*)?/gi;
	const URL_REGEX = /https?:\/\/[^\s<>"'\)]+/gi;

	function extractImageUrls(text: string): string[] {
		const matches = text.match(IMAGE_URL_REGEX);
		if (!matches) return [];
		return [...new Set(matches)];
	}

	const IMAGE_URL_TEST = /\.(png|jpe?g|gif|webp|svg|bmp|ico)(\?[^\s<>"']*)?$/i;
	function extractNonImageUrls(text: string): string[] {
		const allUrls = text.match(URL_REGEX) || [];
		return [...new Set(allUrls)].filter(u => !IMAGE_URL_TEST.test(u));
	}

	function parseFileMessage(content: string): { file_id: string; filename: string; size: number } | null {
		try {
			const parsed = JSON.parse(content);
			if (parsed && typeof parsed.file_id === 'string' && typeof parsed.filename === 'string') {
				return parsed;
			}
			return null;
		} catch { return null; }
	}

	function formatFileSize(bytes: number): string {
		if (!Number.isFinite(bytes) || bytes < 0) return 'Unknown size';
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
	}

	// Configure marked for chat
	marked.setOptions({
		breaks: true,
		gfm: true,
	});

	const SPECIAL_MENTIONS = ['everyone', 'here', 'channel'];

	function getUserColor(userId: string | null): string {
		if (!userId) return 'hsl(0, 0%, 65%)';
		let hash = 0;
		for (let i = 0; i < userId.length; i++) {
			hash = userId.charCodeAt(i) + ((hash << 5) - hash);
		}
		const hue = ((hash % 360) + 360) % 360;
		return `hsl(${hue}, 70%, 65%)`;
	}

	function isEncryptedMessage(text: string): boolean {
		if (!text.startsWith('{"v":')) return false;
		try {
			const parsed = JSON.parse(text);
			return parsed.v === 1 && (parsed.message?.ciphertext || parsed.ciphertext);
		} catch {
			return false;
		}
	}

	// HTML-escape user-controlled values before interpolation into HTML strings
	function escapeHtml(s: string): string {
		return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
	}

	// Reusable renderer for marked — avoids allocating a new one per call
	const markdownRenderer = new marked.Renderer();
	markdownRenderer.code = ({ text: code, lang }: { text: string; lang?: string }) => {
		let highlighted: string;
		if (lang && hljs.getLanguage(lang)) {
			highlighted = hljs.highlight(code, { language: lang }).value;
		} else {
			try {
				highlighted = hljs.highlightAuto(code).value;
			} catch {
				highlighted = code;
			}
		}
		const safeLang = lang ? escapeHtml(lang) : '';
		const langLabel = safeLang ? `<span class="code-lang-label">${safeLang}</span>` : '';
		return `<div class="code-block-wrapper">${langLabel}<pre><code class="hljs${safeLang ? ` language-${safeLang}` : ''}">${highlighted}</code></pre><button class="code-copy-btn" type="button">Copy</button></div>`;
	};

	// Markdown render cache — keyed by raw text, avoids re-parsing unchanged messages
	const markdownCache = new Map<string, string>();
	const MAX_MARKDOWN_CACHE = 600;
	const DOMPURIFY_CONFIG = { ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'del', 'code', 'pre', 'a', 'ul', 'ol', 'li', 'blockquote', 'span', 'div', 'button', 'img'], ALLOWED_ATTR: ['href', 'target', 'rel', 'class', 'type', 'src', 'alt', 'title'] };

	function renderMarkdown(text: string): string {
		const cached = markdownCache.get(text);
		if (cached !== undefined) return cached;

		let html: string;

		// Detect E2E encrypted messages and show placeholder
		if (isEncryptedMessage(text)) {
			html = '<span class="italic opacity-50">Encrypted message (E2E decryption not available)</span>';
		} else {
			// Replace :custom_emoji: shortcodes with inline images
			let processed = text.replace(/:(\w{2,32}):/g, (match, shortcode) => {
				const emoji = customEmojiMap.get(shortcode);
				if (emoji && /^https?:\/\//.test(emoji.url)) {
					return `<img src="${escapeHtml(emoji.url)}" alt=":${shortcode}:" title=":${shortcode}:" class="custom-emoji" />`;
				}
				return match;
			});

			// Replace @mentions before markdown parsing
			processed = processed.replace(/@(\w+)/g, (match, username) => {
				// Special group mentions
				if (SPECIAL_MENTIONS.includes(username)) {
					return `<span class="mention mention-group">@${username}</span>`;
				}
				const users = Array.from(userStore.getAllUsers?.() ?? []);
				const found = users.find(u => u.username === username);
				if (found) {
					const isSelf = found.id === authStore.user?.id;
					return `<span class="mention ${isSelf ? 'mention-self' : ''}">@${escapeHtml(found.display_name)}</span>`;
				}
				return match;
			});

			const rawHtml = marked.parse(processed, { renderer: markdownRenderer }) as string;
			html = DOMPurify.sanitize(rawHtml, DOMPURIFY_CONFIG);
		}

		// Evict oldest entries when cache is full
		if (markdownCache.size >= MAX_MARKDOWN_CACHE) {
			const first = markdownCache.keys().next().value;
			if (first !== undefined) markdownCache.delete(first);
		}
		markdownCache.set(text, html);
		return html;
	}

	function startReply(msg: ChatMessage) {
		replyingTo = msg;
		contextMenuMessageId = null;
		messageInputEl?.focus();
	}

	function cancelReply() {
		replyingTo = null;
	}

	function forwardMessage(msg: ChatMessage) {
		const senderName = getDisplayNameForContext(msg.senderId);
		const preview = msg.content.length > 200 ? msg.content.slice(0, 200) + '...' : msg.content;
		messageInput = `> **${senderName}**: ${preview}\n\n`;
		contextMenuMessageId = null;
		messageInputEl?.focus();
		// Move cursor to end
		tick().then(() => {
			if (messageInputEl) {
				messageInputEl.selectionStart = messageInput.length;
				messageInputEl.selectionEnd = messageInput.length;
			}
		});
	}

	function handlePaste(e: ClipboardEvent) {
		const items = e.clipboardData?.items;
		if (!items) return;
		for (const item of items) {
			if (item.type.startsWith('image/')) {
				e.preventDefault();
				const file = item.getAsFile();
				if (file) handleFileUpload(file);
				return;
			}
		}
	}

	// GIF picker functions
	async function toggleGifPicker() {
		showGifPicker = !showGifPicker;
		if (showGifPicker && gifResults.length === 0) {
			await loadTrendingGifs();
		}
	}

	async function loadTrendingGifs() {
		gifLoading = true;
		try {
			const resp = await getTrendingGifs(20);
			gifResults = resp.results;
			gifError = false;
		} catch {
			gifResults = [];
			gifError = true;
		}
		gifLoading = false;
	}

	function handleGifSearch(query: string) {
		gifSearchQuery = query;
		if (gifSearchDebounceTimer) clearTimeout(gifSearchDebounceTimer);
		if (!query.trim()) {
			loadTrendingGifs().catch(() => {});
			return;
		}
		gifSearchDebounceTimer = setTimeout(async () => {
			gifLoading = true;
			try {
				const resp = await searchGifs(query.trim(), 20);
				gifResults = resp.results;
				gifError = false;
			} catch {
				gifResults = [];
				gifError = true;
			}
			gifLoading = false;
		}, 300);
	}

	async function selectGif(gif: GifResult) {
		showGifPicker = false;
		gifSearchQuery = '';
		const channelId = channelStore.activeChannelId;
		if (!channelId) return;

		const { ciphertext, nonce } = await encryptContent(channelId, gif.url);

		// Optimistic add
		const tempId = `temp-${Date.now()}`;
		messageStore.addMessage(channelId, {
			id: tempId,
			channelId,
			senderId: authStore.user?.id ?? '',
			content: gif.url,
			messageType: 'text',
			replyToId: null,
			editedAt: null,
			createdAt: new Date().toISOString(),
			pending: true
		});

		const sent = wsClient.send({
			type: 'send_message',
			channel_id: channelId,
			ciphertext,
			nonce,
			message_type: 'text',
			reply_to: null,
			sender_key_id: null
		});

		if (!sent) {
			messageStore.removeMessage(channelId, tempId);
			toastStore.error('GIF not sent — connection lost');
		}
	}

	function markAllRead() {
		wsClient.send({ type: 'mark_all_read' });
		messageStore.clearAllUnread();
	}

	// ── GIF freeze on window blur ──
	// GIPHY serves still frames at the same URL with _s suffix (e.g. giphy_s.gif)
	function gifToStill(src: string): string | null {
		if (src.includes('giphy.com') && src.includes('/giphy.gif')) {
			return src.replace('/giphy.gif', '/giphy_s.gif');
		}
		// fixed_width_small animated URLs like /200w.gif → /200w_s.gif
		const m = src.match(/\/(\d+w?)\.gif/);
		if (m && src.includes('giphy.com')) {
			return src.replace(`/${m[1]}.gif`, `/${m[1]}_s.gif`);
		}
		return null;
	}

	function freezeGifs() {
		if (!messageListEl) return;
		const imgs = messageListEl.querySelectorAll<HTMLImageElement>('img[src*=".gif"]');
		for (const img of imgs) {
			if (img.dataset.gifSrc) continue;
			const still = gifToStill(img.src);
			if (still) {
				const originalSrc = img.src;
				img.dataset.gifSrc = originalSrc;
				// If the still URL fails (404), restore the animated version
				const prevOnerror = img.onerror;
				img.onerror = () => {
					img.onerror = prevOnerror;
					img.src = originalSrc;
					delete img.dataset.gifSrc;
				};
				img.src = still;
			}
		}
	}

	function unfreezeGifs() {
		if (!messageListEl) return;
		const imgs = messageListEl.querySelectorAll<HTMLImageElement>('img[data-gif-src]');
		for (const img of imgs) {
			img.src = img.dataset.gifSrc!;
			delete img.dataset.gifSrc;
		}
	}

	function handleWindowFocus() {
		unfreezeGifs();
	}

	function handleWindowBlurGifs() {
		freezeGifs();
	}

	// ── Push-to-Talk / Toggle-Mute ──
	let pttActive = $state(false);

	function isTextInput(e: KeyboardEvent): boolean {
		const tag = (e.target as HTMLElement)?.tagName;
		return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT'
			|| (e.target as HTMLElement)?.isContentEditable === true;
	}

	// Auto-mute when joining a call in PTT mode
	let wasInCall = false;
	$effect(() => {
		const inCall = voiceStore.isInCall;
		if (inCall && !wasInCall && preferencesStore.preferences.voiceActivationMode === 'push-to-talk') {
			voiceStore.setAudioEnabled(false);
		}
		if (!inCall && pttActive) {
			pttActive = false;
		}
		wasInCall = inCall;
	});

	function handleGlobalKeyup(e: KeyboardEvent) {
		if (pttActive && preferencesStore.preferences.voiceActivationMode === 'push-to-talk'
			&& e.key === preferencesStore.preferences.pttKey) {
			pttActive = false;
			voiceStore.setAudioEnabled(false);
		}
	}

	function handleBeforeUnload() {
		if (voiceStore.isInCall && voiceStore.activeCall?.channelId) {
			wsClient.send({ type: 'leave_voice', channel_id: voiceStore.activeCall.channelId });
		}
	}

	function handleWindowBlur() {
		if (pttActive) {
			pttActive = false;
			if (voiceStore.isInCall && preferencesStore.preferences.voiceActivationMode === 'push-to-talk') {
				voiceStore.setAudioEnabled(false);
			}
		}
	}

	function handleGlobalKeydown(e: KeyboardEvent) {
		// Push-to-Talk / Toggle-Mute (before other shortcuts, only when in a call and not typing)
		if (voiceStore.isInCall && !isTextInput(e) && !e.ctrlKey && !e.metaKey && !e.altKey) {
			const mode = preferencesStore.preferences.voiceActivationMode;

			if (mode === 'push-to-talk' && e.key === preferencesStore.preferences.pttKey) {
				e.preventDefault();
				if (!pttActive && !e.repeat) {
					pttActive = true;
					voiceStore.setAudioEnabled(true);
				}
				return;
			}

			if (mode === 'toggle-mute' && e.key === preferencesStore.preferences.toggleMuteKey && !e.repeat) {
				e.preventDefault();
				voiceStore.setAudioEnabled(!voiceStore.activeCall!.audioEnabled);
				return;
			}
		}

		// Shift+Escape to mark all channels as read
		if (e.key === 'Escape' && e.shiftKey && !isTextInput(e)) {
			e.preventDefault();
			markAllRead();
			return;
		}
		// Ctrl+K / Cmd+K for quick channel switcher
		if (e.key === 'k' && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			if (showQuickSwitcher) {
				showQuickSwitcher = false;
			} else {
				openQuickSwitcher();
			}
			return;
		}
		// Ctrl+F / Cmd+F for channel search
		if (e.key === 'f' && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			showSearch = !showSearch;
			if (showSearch) {
				tick().then(() => searchInputEl?.focus());
			} else {
				searchQuery = '';
				searchResults = [];
			}
			return;
		}
		// End key to scroll to latest message
		if (e.key === 'End') {
			const tag = (e.target as HTMLElement)?.tagName;
			if (tag === 'INPUT' || tag === 'TEXTAREA') return;
			e.preventDefault();
			scrollToBottom();
			return;
		}
		// Home key to scroll to top of messages
		if (e.key === 'Home') {
			const tag = (e.target as HTMLElement)?.tagName;
			if (tag === 'INPUT' || tag === 'TEXTAREA') return;
			e.preventDefault();
			if (messageListEl) messageListEl.scrollTop = 0;
			return;
		}
		// Ctrl+T / Cmd+T to focus message input
		if (e.key === 't' && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			messageInputEl?.focus();
			return;
		}
		// ? or Ctrl+/ to show shortcuts
		if ((e.key === '?' && !e.ctrlKey && !e.metaKey) || (e.key === '/' && (e.ctrlKey || e.metaKey))) {
			const tag = (e.target as HTMLElement)?.tagName;
			if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;
			e.preventDefault();
			showShortcutsModal = !showShortcutsModal;
		}
		// Lightbox arrow key navigation
		if (lightboxImage) {
			if (e.key === 'ArrowLeft') { e.preventDefault(); lightboxPrev(); return; }
			if (e.key === 'ArrowRight') { e.preventDefault(); lightboxNext(); return; }
		}
		// Escape to close modals and panels
		if (e.key === 'Escape') {
			if (confirmDialog) { confirmDialog = null; confirmInput = ''; e.preventDefault(); return; }
			if (reportingMessageId) { reportingMessageId = null; reportReason = ''; e.preventDefault(); return; }
			if (showQuickSwitcher) { showQuickSwitcher = false; e.preventDefault(); return; }
			if (showEditHistory) { showEditHistory = false; e.preventDefault(); return; }
			if (lightboxImage) { closeLightbox(); e.preventDefault(); return; }
			if (showGifPicker) { showGifPicker = false; e.preventDefault(); return; }
			if (showShortcutsModal) { showShortcutsModal = false; e.preventDefault(); return; }
			if (showSearch) { showSearch = false; e.preventDefault(); return; }
			if (showPinnedPanel) { showPinnedPanel = false; e.preventDefault(); return; }
			if (showPollPanel) { showPollPanel = false; e.preventDefault(); return; }
			if (showCreatePoll) { showCreatePoll = false; e.preventDefault(); return; }
			if (showMemberPanel) { showMemberPanel = false; e.preventDefault(); return; }
			if (showThreadPanel) { closeThread(); e.preventDefault(); return; }
			if (showBookmarksPanel) { showBookmarksPanel = false; e.preventDefault(); return; }
			if (showScheduledPanel) { showScheduledPanel = false; e.preventDefault(); return; }
			if (showSchedulePicker) { showSchedulePicker = false; e.preventDefault(); return; }
			if (showFeedback) { showFeedback = false; setFeedbackScreenshot(null); e.preventDefault(); return; }
			if (showWelcomeSplash) { showWelcomeSplash = false; e.preventDefault(); return; }
			if (showJoinCommunity) { showJoinCommunity = false; e.preventDefault(); return; }
			if (showCreateCommunity) { showCreateCommunity = false; e.preventDefault(); return; }
		}
	}

	// Special mention entries for autocomplete
	type MentionEntry = ChannelMember & { description?: string };
	const SPECIAL_MENTION_ENTRIES: MentionEntry[] = [
		{ user_id: '__everyone__', username: 'everyone', display_name: 'everyone', avatar_url: null, role: 'special', joined_at: '', description: 'Notify all members' },
		{ user_id: '__here__', username: 'here', display_name: 'here', avatar_url: null, role: 'special', joined_at: '', description: 'Notify online members' },
		{ user_id: '__channel__', username: 'channel', display_name: 'channel', avatar_url: null, role: 'special', joined_at: '', description: 'Notify channel members' },
	];

	// Mention autocomplete
	let mentionResults = $derived.by((): MentionEntry[] => {
		if (!showMentionPopup || !channelStore.activeChannelId) return [];
		const members = memberStore.getMembers(channelStore.activeChannelId);
		const q = mentionQuery.toLowerCase();

		// Filter special mentions
		const specials = q
			? SPECIAL_MENTION_ENTRIES.filter(s => s.username.startsWith(q))
			: SPECIAL_MENTION_ENTRIES;

		// Filter real members
		const people: MentionEntry[] = q
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
		const before = val.slice(0, pos);
		// Check for :emoji pattern first
		const emojiMatch = before.match(/:(\w{2,})$/);
		if (emojiMatch) {
			emojiQuery = emojiMatch[1];
			const standardResults = searchEmoji(emojiQuery, 6);
			// Also search custom community emojis
			const customResults: typeof emojiResults = [];
			const q = emojiQuery.toLowerCase();
			for (const [shortcode, emoji] of customEmojiMap) {
				if (shortcode.toLowerCase().includes(q)) {
					customResults.push({ name: shortcode, emoji: '', custom: true, url: emoji.url });
					if (customResults.length >= 4) break;
				}
			}
			emojiResults = [...customResults, ...standardResults].slice(0, 8);
			showEmojiPopup = emojiResults.length > 0;
			emojiIndex = 0;
			if (showEmojiPopup) {
				showMentionPopup = false;
				return;
			}
		} else {
			showEmojiPopup = false;
		}
		// Find @ before cursor
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
		tick().then(() => {
			messageInputEl?.focus();
			const newPos = replaced.length;
			messageInputEl!.selectionStart = newPos;
			messageInputEl!.selectionEnd = newPos;
		});
	}

	function selectEmoji(emoji: string, isCustom = false) {
		if (!messageInputEl) return;
		const val = messageInputEl.value;
		const pos = messageInputEl.selectionStart ?? val.length;
		const before = val.slice(0, pos);
		const after = val.slice(pos);
		const insert = isCustom ? `:${emoji}: ` : emoji;
		const replaced = before.replace(/:\w{2,}$/, insert);
		messageInput = replaced + after;
		showEmojiPopup = false;
		tick().then(() => {
			messageInputEl?.focus();
			const newPos = replaced.length;
			messageInputEl!.selectionStart = newPos;
			messageInputEl!.selectionEnd = newPos;
		});
	}

	function handleMentionKeydown(e: KeyboardEvent) {
		// Handle emoji autocomplete first
		if (showEmojiPopup && emojiResults.length > 0) {
			if (e.key === 'ArrowDown') {
				e.preventDefault();
				emojiIndex = (emojiIndex + 1) % emojiResults.length;
				return;
			} else if (e.key === 'ArrowUp') {
				e.preventDefault();
				emojiIndex = (emojiIndex - 1 + emojiResults.length) % emojiResults.length;
				return;
			} else if (e.key === 'Tab' || e.key === 'Enter') {
				if (emojiResults[emojiIndex]) {
					e.preventDefault();
					const entry = emojiResults[emojiIndex];
					selectEmoji(entry.custom ? entry.name : entry.emoji, entry.custom);
					return;
				}
			} else if (e.key === 'Escape') {
				e.preventDefault();
				showEmojiPopup = false;
				return;
			}
		}
		if (!showMentionPopup) return;
		const results = mentionResults;
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
			const link = `${getPublicUrl()}/invite/${invite.code}`;
			await navigator.clipboard.writeText(link);
			toastStore.success('Invite link copied!');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to create invite');
		}
	}

	/** Extract invite code from a full URL or plain code. */
	function extractInviteCode(input: string): string {
		const trimmed = input.trim();
		const match = trimmed.match(/\/invite\/([A-Za-z0-9]+)\/?$/);
		return match ? match[1] : trimmed;
	}

	async function handleAcceptInvite() {
		if (!joinInviteCode.trim()) return;
		try {
			const result = await acceptInvite(extractInviteCode(joinInviteCode));
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
			invitePreview = await getInviteInfo(extractInviteCode(joinInviteCode));
		} catch (err: any) {
			toastStore.error(err?.message || 'Invalid invite code');
			invitePreview = null;
		}
	}

	// Community functions
	async function loadCommunityGroups(communityId: string): Promise<Group[]> {
		const groups = await listCommunityGroups(communityId);
		groupStore.setGroups(groups);

		// Load channels for each group (handle per-group errors gracefully)
		const groupChannelPromises = groups.map(async (g) => {
			try {
				const chs = await listGroupChannels(g.id);
				return [g.id, chs] as [string, Channel[]];
			} catch {
				return [g.id, []] as [string, Channel[]];
			}
		});
		const results = await Promise.all(groupChannelPromises);
		const newMap = new Map<string, Channel[]>();
		for (const [gid, chs] of results) {
			newMap.set(gid, chs);
			for (const ch of chs) {
				channelStore.addChannel(ch);
			}
		}
		groupChannelsMap = newMap;

		// Subscribe to new channels
		const allGroupChannelIds = Array.from(newMap.values()).flat().map(c => c.id);
		if (allGroupChannelIds.length > 0) {
			wsClient.send({ type: 'subscribe', channel_ids: allGroupChannelIds });
		}

		return groups;
	}

	async function loadCustomEmojis(communityId: string) {
		if (loadedCommunityEmojiId === communityId) return;
		try {
			const emojis = await listCommunityEmojis(communityId);
			const map = new Map<string, CustomEmoji>();
			for (const e of emojis) map.set(e.shortcode, e);
			customEmojiMap = map;
			loadedCommunityEmojiId = communityId;
			markdownCache.clear();
		} catch (err) {
			console.warn('Failed to load custom emojis:', err);
		}
	}

	async function handleSubmitReport() {
		if (!reportingMessageId || !reportReason.trim()) return;
		submittingReport = true;
		try {
			await createReport('message', reportingMessageId, reportReason.trim());
			toastStore.success('Report submitted — admins will review it.');
			reportingMessageId = null;
			reportReason = '';
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to submit report');
		} finally {
			submittingReport = false;
		}
	}

	async function switchCommunity(communityId: string) {
		if (communityId === communityStore.activeCommunityId) return;
		communityStore.setActive(communityId);
		sidebarTab = 'groups';
		loadCustomEmojis(communityId);

		const switchId = ++communitySwitchId;
		try {
			const groups = await loadCommunityGroups(communityId);
			// Guard: user may have switched communities again during the async load
			if (switchId !== communitySwitchId) return;
			// Expand the first group by default
			if (groups.length > 0) {
				expandedGroupIds = new Set([groups[0].id]);
				const firstChs = groupChannelsMap.get(groups[0].id);
				if (firstChs && firstChs.length > 0) {
					selectChannel(firstChs[0].id);
				}
			}

			// Show welcome splash if community has a welcome message and user hasn't seen it
			const c = communityStore.communities.find((c) => c.id === communityId);
			if (c?.welcome_message) {
				const dismissKey = `chatalot:welcomeDismissed:${communityId}`;
				if (!localStorage.getItem(dismissKey)) {
					welcomeCommunity = c;
					showWelcomeSplash = true;
				}
			}
		} catch (err: any) {
			if (switchId !== communitySwitchId) return;
			toastStore.error(err?.message || 'Failed to load community');
		}
	}

	async function handlePreviewCommunityInvite() {
		if (!joinCommunityCode.trim()) return;
		try {
			communityInvitePreview = await getCommunityInviteInfo(extractInviteCode(joinCommunityCode));
		} catch (err: any) {
			toastStore.error(err?.message || 'Invalid invite code');
			communityInvitePreview = null;
		}
	}

	async function handleAcceptCommunityInvite() {
		if (!joinCommunityCode.trim()) return;
		try {
			const result = await acceptCommunityInvite(extractInviteCode(joinCommunityCode));
			// Reload communities
			const communities = await listCommunities();
			communityStore.setCommunities(communities);
			// Switch to the new community
			await switchCommunity(result.community_id);
			showJoinCommunity = false;
			joinCommunityCode = '';
			communityInvitePreview = null;
			toastStore.success(`Joined "${result.community_name}"`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to join community');
		}
	}

	async function handleCreateCommunity() {
		const name = newCommunityName.trim();
		if (!name) return;
		creatingCommunity = true;
		try {
			const community = await createCommunity(name, newCommunityDescription.trim() || undefined);
			communityStore.addCommunity(community);
			communityStore.setActive(community.id);
			showCreateCommunity = false;
			newCommunityName = '';
			newCommunityDescription = '';
			// Load groups for the new (empty) community
			await loadCommunityGroups(community.id);
			groupStore.setGroups([]);
			groupChannelsMap = new Map();
			expandedGroupIds = new Set();
			toastStore.success(`Community "${community.name}" created`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to create community');
		} finally {
			creatingCommunity = false;
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
		if (!name || !communityStore.activeCommunityId || creatingGroup) return;
		creatingGroup = true;
		try {
			const assignId = newGroupAssignMemberId || undefined;
			const vis = assignId ? undefined : newGroupVisibility;
			const group = await apiCreateGroup(communityStore.activeCommunityId, name, newGroupDescription.trim() || undefined, vis, assignId);
			groupStore.addGroup(group);
			// Load the auto-created #general channel (only visible if we're the assigned member or a group member)
			try {
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
			} catch {
				// Moderator created a personal group for someone else — they may not be a group member
				expandedGroupIds = new Set([...expandedGroupIds, group.id]);
			}
			newGroupName = '';
			newGroupDescription = '';
			newGroupVisibility = 'public';
			newGroupAssignMemberId = '';
			showCreateGroup = false;
			toastStore.success(`Group "${group.name}" created`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to create group');
		} finally {
			creatingGroup = false;
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

	function handleLeaveGroup(group: Group) {
		showConfirmDialog({
			title: 'Leave Group',
			message: `Are you sure you want to leave "${group.name}"?`,
			confirmLabel: 'Leave',
			danger: true,
			onConfirm: async () => {
				try {
					await leaveGroup(group.id);
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
					toastStore.success(`Left "${group.name}"`);
				} catch (err: any) {
					toastStore.error(err?.message || 'Failed to leave group');
				}
			}
		});
	}

	function handleDeleteGroup(group: Group) {
		showConfirmDialog({
			title: 'Delete Group',
			message: `Delete "${group.name}"? This will permanently delete all channels in the group. This cannot be undone.`,
			confirmLabel: 'Delete Group',
			danger: true,
			onConfirm: async () => {
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
		});
	}

	async function handleCreateGroupChannel(e: SubmitEvent, groupId: string) {
		e.preventDefault();
		const name = newGroupChannelName.trim();
		if (!name || creatingGroupChannel) return;
		creatingGroupChannel = true;
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
		} finally {
			creatingGroupChannel = false;
		}
	}

	async function handleRenameGroup(groupId: string) {
		const newName = renameGroupInput.trim();
		if (!newName) { renamingGroupId = null; return; }
		try {
			const updated = await apiUpdateGroup(groupId, { name: newName });
			groupStore.updateGroup(groupId, updated);
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to rename group');
		}
		renamingGroupId = null;
	}

	async function handleRenameGroupChannel(groupId: string, channelId: string) {
		const newName = renameChannelInput.trim();
		if (!newName) { renamingChannelId = null; return; }
		try {
			const updated = await apiUpdateChannel(groupId, channelId, { name: newName });
			channelStore.updateChannel(updated);
			const newMap = new Map(groupChannelsMap);
			const existing = newMap.get(groupId) ?? [];
			newMap.set(groupId, existing.map(c => c.id === channelId ? { ...c, name: updated.name } : c));
			groupChannelsMap = newMap;
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to rename channel');
		}
		renamingChannelId = null;
	}

	function handleDeleteGroupChannel(groupId: string, channelId: string) {
		const ch = (groupChannelsMap.get(groupId) ?? []).find(c => c.id === channelId);
		showConfirmDialog({
			title: 'Delete Channel',
			message: `Delete "${ch?.name ?? 'this channel'}"? All messages will be permanently lost.`,
			confirmLabel: 'Delete',
			danger: true,
			onConfirm: async () => {
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
		});
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
		if (showSearch) {
			tick().then(() => searchInputEl?.focus());
		} else {
			searchQuery = '';
			searchResults = [];
			searchScope = 'channel';
		}
	}

	function handleSearchInput() {
		if (searchTimeout) clearTimeout(searchTimeout);
		if (!searchQuery.trim()) {
			searchResults = [];
			return;
		}
		searchTimeout = setTimeout(async () => {
			const q = searchQuery.trim();
			if (!q) return;
			if (searchScope === 'channel' && !channelStore.activeChannelId) return;
			searching = true;
			searchError = false;
			const opts: SearchOptions = {};
			if (searchFilterSender.trim()) opts.sender = searchFilterSender.trim();
			if (searchFilterAfter) opts.after = new Date(searchFilterAfter).toISOString();
			if (searchFilterBefore) opts.before = new Date(searchFilterBefore + 'T23:59:59').toISOString();
			if (searchFilterHasFile) opts.has_file = true;
			try {
				const raw = searchScope === 'global'
					? await searchMessagesGlobal(q, opts)
					: await searchMessages(channelStore.activeChannelId!, q, opts);
				searchResults = await Promise.all(raw.reverse().map(async (m) => {
					const { content, encrypted } = await decryptMessage(
						m.channel_id,
						m.sender_id,
						m.ciphertext,
						m.id,
						m.sender_id === authStore.user?.id ? getPeerUserIdForDm(m.channel_id) : undefined,
					);
					return {
						id: m.id,
						channelId: m.channel_id,
						senderId: m.sender_id,
						content,
						encryptionStatus: encrypted ? 'encrypted' as const : 'plaintext' as const,
						messageType: m.message_type,
						replyToId: m.reply_to_id,
						editedAt: m.edited_at,
						createdAt: m.created_at,
						threadId: m.thread_id ?? null,
					};
				}));
			} catch (err) {
				console.warn('Search failed:', err);
				searchResults = [];
				searchError = true;
			} finally {
				searching = false;
			}
		}, 300);
	}

	function highlightSearchMatch(text: string, query: string): string {
		if (!query.trim()) return DOMPurify.sanitize(text, { ALLOWED_TAGS: [] });
		const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
		const regex = new RegExp(`(${escaped})`, 'gi');
		const sanitized = DOMPurify.sanitize(text, { ALLOWED_TAGS: [] });
		return sanitized.replace(regex, '<mark class="rounded bg-yellow-500/30 text-[var(--text-primary)] px-0.5">$1</mark>');
	}

	function highlightMessage(msgId: string) {
		const el = document.getElementById('msg-' + msgId);
		if (el) {
			el.scrollIntoView({ behavior: 'smooth', block: 'center' });
			// Restart animation by removing and re-adding class
			el.classList.remove('msg-highlight');
			void el.offsetWidth; // force reflow
			el.classList.add('msg-highlight');
		}
	}

	async function jumpToSearchResult(msgId: string, channelId?: string) {
		showSearch = false;
		searchQuery = '';
		searchResults = [];
		// If the result is from a different channel, navigate there first
		if (channelId && channelId !== channelStore.activeChannelId) {
			selectChannel(channelId);
			await tick();
			// Wait for messages to load before highlighting
			await new Promise(r => setTimeout(r, 300));
		}
		highlightMessage(msgId);
	}

	// ── Infinite scroll ──
	function handleCodeCopyClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (target.classList.contains('code-copy-btn')) {
			e.stopPropagation();
			const wrapper = target.closest('.code-block-wrapper');
			const pre = wrapper?.querySelector('pre');
			if (pre) {
				navigator.clipboard.writeText(pre.textContent || '').then(() => {
					target.textContent = 'Copied!';
					setTimeout(() => { target.textContent = 'Copy'; }, 2000);
				}).catch(() => {
					target.textContent = 'Failed';
					setTimeout(() => { target.textContent = 'Copy'; }, 2000);
				});
			}
		}
	}

	async function handleMessageScroll(e: Event) {
		const el = e.target as HTMLDivElement;
		// Show/hide scroll-to-bottom button
		showScrollBottom = el.scrollHeight - el.scrollTop - el.clientHeight > 300;
		if (el.scrollTop > 200 || loadingOlder || !channelStore.activeChannelId) return;
		if (!messageStore.hasMore(channelStore.activeChannelId)) return;

		const activeId = channelStore.activeChannelId;
		const currentMessages = messageStore.getMessages(activeId);
		if (currentMessages.length === 0) return;

		const oldestMsg = currentMessages[0];
		loadingOlder = true;
		loadingOlderError = false;
		const prevHeight = el.scrollHeight;
		try {
			const raw = await getMessages(activeId, oldestMsg.id, FETCH_LIMIT);
			if (channelStore.activeChannelId !== activeId) return; // channel switched during fetch
			const olderMessages: ChatMessage[] = await Promise.all(raw.reverse().map(async (m) => {
				const { content, encrypted } = await decryptMessage(
					m.channel_id,
					m.sender_id,
					m.ciphertext,
					m.id,
					m.sender_id === authStore.user?.id ? getPeerUserIdForDm(activeId) : undefined,
				);
				return {
					id: m.id,
					channelId: m.channel_id,
					senderId: m.sender_id,
					content,
					encryptionStatus: encrypted ? 'encrypted' as const : 'plaintext' as const,
					messageType: m.message_type,
					replyToId: m.reply_to_id,
					editedAt: m.edited_at,
					createdAt: m.created_at,
					threadId: m.thread_id ?? null,
					threadReplyCount: m.thread_reply_count ?? undefined,
					threadLastReplyAt: m.thread_last_reply_at ?? undefined,
					reactions: parseReactions(m.reactions),
				};
			}));
			messageStore.prependMessages(activeId, olderMessages, FETCH_LIMIT);
			// Preserve scroll position
			await tick();
			el.scrollTop = el.scrollHeight - prevHeight;
		} catch (err) {
			console.warn('Failed to load older messages:', err);
			loadingOlderError = true;
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
	let dmUnreadTotal = $derived(
		dmChannels.reduce((sum, dm) => sum + messageStore.getUnreadCount(dm.channel.id), 0)
	);
	let channelUnreadTotal = $derived(
		Array.from(groupChannelsMap.values()).flat().reduce((sum, c) => sum + messageStore.getUnreadCount(c.id), 0)
	);
	$effect(() => {
		const total = channelUnreadTotal + dmUnreadTotal;
		document.title = total > 0 ? `(${total}) Chatalot` : 'Chatalot';
	});

	// Persist sidebar tab, expanded groups, and nav collapsed
	$effect(() => {
		localStorage.setItem('chatalot:sidebarTab', sidebarTab);
	});
	$effect(() => {
		localStorage.setItem('chatalot:navCollapsed', String(navCollapsed));
	});
	$effect(() => {
		localStorage.setItem('chatalot:expandedGroups', JSON.stringify([...expandedGroupIds]));
	});

	// Auto-expand groups with unread messages
	$effect(() => {
		let changed = false;
		const next = new Set(expandedGroupIds);
		for (const group of groupStore.groups) {
			if (next.has(group.id)) continue;
			const channels = groupChannelsMap.get(group.id) ?? [];
			const hasUnread = channels.some(c => messageStore.getUnreadCount(c.id) > 0);
			if (hasUnread) {
				next.add(group.id);
				changed = true;
			}
		}
		if (changed) expandedGroupIds = next;
	});

	// Auto-scroll when new messages arrive (only if near bottom)
	let prevMessageCount = $state(0);
	$effect(() => {
		const count = messages.length;
		if (count > prevMessageCount && prevMessageCount > 0) {
			// New message arrived — only scroll if user is near the bottom
			if (isNearBottom()) {
				tick().then(() => scrollToBottom());
			}
		} else if (count > 0 && prevMessageCount === 0) {
			// Initial load — scroll to bottom
			tick().then(() => scrollToBottom());
		}
		prevMessageCount = count;
	});
</script>

<svelte:window onkeydown={handleGlobalKeydown} onkeyup={handleGlobalKeyup} onfocus={handleWindowFocus} onblur={(e) => { handleWindowBlur(); handleWindowBlurGifs(); }} onmousedown={() => { showSidebarCreateMenu = false; }} />

{#if authStore.isAuthenticated}
	<div class="flex flex-col h-screen overflow-hidden">
		<!-- ═══ TOP NAVIGATION BAR ═══ -->
		{#if !navCollapsed}
		<nav class="relative flex items-center gap-2 h-12 md:h-14 shrink-0 border-b border-white/10 bg-[var(--bg-secondary)] px-2 md:px-4">
			<!-- LEFT SECTION -->
			<div class="flex items-center gap-1">
				<!-- Mobile menu button -->
				<button
					onclick={() => { showNavDropdown = !showNavDropdown; showCommunityPicker = false; showUserMenu = false; }}
					class="rounded-lg p-1.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] md:hidden"
					aria-label="Open navigation"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="18" x2="21" y2="18" />
					</svg>
				</button>

				<!-- Community Dropdown Trigger -->
				<button
					onclick={() => { showCommunityPicker = !showCommunityPicker; showNavDropdown = false; showUserMenu = false; }}
					class="flex items-center gap-1.5 rounded-xl px-2 py-1.5 text-sm font-semibold text-[var(--text-primary)] transition hover:bg-white/5 {showCommunityPicker ? 'bg-white/5' : ''}"
					title="Switch community"
				>
					{#if communityStore.activeCommunity?.icon_url}
						<img src={communityStore.activeCommunity.icon_url} alt={communityStore.activeCommunity.name} class="h-5 w-5 rounded-full object-cover" onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }} />
					{:else}
						<span class="flex h-5 w-5 items-center justify-center rounded-full bg-[var(--accent)] text-[10px] font-bold text-white">
							{(communityStore.activeCommunity?.name ?? 'CH').slice(0, 2).toUpperCase()}
						</span>
					{/if}
					<span class="hidden sm:inline max-w-[120px] truncate">{communityStore.activeCommunity?.name ?? 'Chatalot'}</span>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg>
				</button>

				<!-- Separator -->
				<span class="h-4 w-px bg-white/10"></span>

				<!-- Channel/Nav Dropdown Trigger -->
				{#if isExpandedSidebar}
					<!-- Expanded mode: static channel name on desktop -->
					<span class="hidden md:flex items-center gap-1.5 px-2 py-1.5 text-sm text-[var(--text-primary)]">
						{#if activeChannel}
							{#if activeChannel.channel_type === 'dm'}
								<span class="text-[var(--text-secondary)]">@</span>
							{:else if activeChannel.channel_type === 'voice'}
								<span>🔊</span>
							{:else}
								<span class="text-[var(--text-secondary)]">#</span>
							{/if}
							<span class="max-w-[160px] truncate">{getChannelDisplayName()}</span>
						{:else}
							<span class="text-[var(--text-secondary)]">Select channel</span>
						{/if}
					</span>
				{:else}
					<!-- Compact mode: clickable dropdown trigger -->
					<button
						onclick={() => { showNavDropdown = !showNavDropdown; showCommunityPicker = false; showUserMenu = false; }}
						class="hidden md:flex items-center gap-1.5 rounded-xl px-2 py-1.5 text-sm text-[var(--text-primary)] transition hover:bg-white/5 {showNavDropdown ? 'bg-white/5' : ''}"
					>
						{#if activeChannel}
							{#if activeChannel.channel_type === 'dm'}
								<span class="text-[var(--text-secondary)]">@</span>
							{:else if activeChannel.channel_type === 'voice'}
								<span>🔊</span>
							{:else}
								<span class="text-[var(--text-secondary)]">#</span>
							{/if}
							<span class="max-w-[160px] truncate">{getChannelDisplayName()}</span>
						{:else}
							<span class="text-[var(--text-secondary)]">Select channel</span>
						{/if}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg>
						{#if channelUnreadTotal + dmUnreadTotal > 0}
							<span class="flex h-4 min-w-4 items-center justify-center rounded-full bg-[var(--accent)] px-1 text-[10px] font-bold text-white">
								{channelUnreadTotal + dmUnreadTotal > 99 ? '99+' : channelUnreadTotal + dmUnreadTotal}
							</span>
						{/if}
					</button>
				{/if}

				<!-- E2E Encryption Badge -->
				{#if activeChannel?.channel_type === 'dm'}
					<button
						onclick={loadEncryptionInfo}
						class="hidden md:flex items-center gap-1 rounded-full bg-green-500/15 px-2 py-0.5 text-[10px] font-bold text-green-400 transition hover:bg-green-500/25"
						title="End-to-end encrypted — click to verify"
						aria-label="Verify encryption"
					>
						<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
						E2E
					</button>
				{:else if activeChannel}
					<span class="hidden md:flex items-center gap-1 rounded-full bg-green-500/10 px-2 py-0.5 text-[10px] font-medium text-green-400/60" title="Messages are encrypted">
						<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
						E2E
					</span>
				{/if}
			</div>

			<!-- CENTER SECTION -->
			<div class="hidden md:flex flex-1 justify-center max-w-md mx-auto">
				<button
					onclick={openQuickSwitcher}
					class="flex w-full items-center gap-2 rounded-xl border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:border-white/20"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
					</svg>
					<span>Search...</span>
					<kbd class="ml-auto rounded bg-white/5 px-1.5 py-0.5 text-[10px] text-[var(--text-tertiary)]">{modKey}+K</kbd>
				</button>
			</div>

			<!-- RIGHT SECTION -->
			<div class="flex items-center gap-0.5 ml-auto">
				<!-- Voice connected indicator -->
				{#if voiceStore.activeCall}
					<button
						onclick={() => voiceStore.activeCall && selectChannel(voiceStore.activeCall.channelId)}
						class="flex items-center gap-1.5 rounded-xl px-2 py-1 text-xs font-medium text-[var(--success)] transition hover:bg-white/5"
					>
						<span class="h-1.5 w-1.5 animate-pulse rounded-full bg-[var(--success)]"></span>
						<span class="hidden sm:inline">Voice</span>
					</button>
				{/if}

				<!-- Call Controls -->
				{#if activeChannel}
					<CallControls channelId={activeChannel.id} channelType={activeChannel.channel_type} />
				{/if}

				<!-- Bookmarks -->
				<button
					onclick={toggleBookmarksPanel}
					class="rounded-xl p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] {showBookmarksPanel ? 'text-[var(--accent)]' : ''}"
					title="Saved Items"
					aria-label="Saved Items"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
					</svg>
				</button>

				<!-- Scheduled -->
				<button
					onclick={toggleScheduledPanel}
					class="relative rounded-xl p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] {showScheduledPanel ? 'text-[var(--accent)]' : ''}"
					title="Scheduled Messages"
					aria-label="Scheduled Messages"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
					</svg>
					{#if scheduledMessages.length > 0}
						<span class="absolute -right-0.5 -top-0.5 flex h-3.5 w-3.5 items-center justify-center rounded-full bg-[var(--accent)] text-[8px] font-bold text-white">
							{scheduledMessages.length}
						</span>
					{/if}
				</button>

				<!-- Mobile search -->
				<button
					onclick={toggleSearch}
					class="rounded-xl p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] md:hidden"
					title="Search"
					aria-label="Search"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
					</svg>
				</button>

				<!-- Collapse nav -->
				<button
					onclick={() => navCollapsed = true}
					class="hidden md:block rounded-xl p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					title="Collapse navigation"
					aria-label="Collapse navigation"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="18 15 12 9 6 15" /></svg>
				</button>

				<!-- Admin -->
				{#if authStore.user?.is_admin || authStore.user?.is_owner}
					<button
						onclick={() => goto('/admin')}
						class="hidden md:block rounded-xl p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						title="Admin Panel"
						aria-label="Admin Panel"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
						</svg>
					</button>
				{/if}

				<!-- Settings -->
				<button
					onclick={() => goto('/settings')}
					class="hidden md:block rounded-xl p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					title="Settings"
					aria-label="Settings"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
					</svg>
				</button>

				<!-- User Avatar + Menu Trigger -->
				<button
					onclick={(e) => { e.stopPropagation(); showUserMenu = !showUserMenu; showCommunityPicker = false; showNavDropdown = false; }}
					class="flex items-center gap-1.5 rounded-xl p-1 transition hover:bg-white/5"
					title="User menu"
				>
					{#if authStore.user}
						<Avatar userId={authStore.user.id} size="xs" showStatus />
					{/if}
				</button>
			</div>
		</nav>
		{:else}
		<!-- Collapsed nav: minimal breadcrumb -->
		<nav class="flex items-center h-8 shrink-0 border-b border-white/10 bg-[var(--bg-secondary)] px-2">
			<button
				onclick={() => navCollapsed = false}
				class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
				title="Expand navigation"
				aria-label="Expand navigation"
			>
				<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg>
			</button>
			<span class="mx-2 truncate text-xs text-[var(--text-secondary)]">
				{communityStore.activeCommunity?.name ?? 'Chatalot'} / {activeChannel ? getChannelDisplayName() : 'No channel'}
			</span>
			<div class="ml-auto flex items-center gap-1">
				{#if voiceStore.activeCall}
					<span class="h-1.5 w-1.5 animate-pulse rounded-full bg-[var(--success)]"></span>
				{/if}
				{#if channelUnreadTotal + dmUnreadTotal > 0}
					<span class="flex h-4 min-w-4 items-center justify-center rounded-full bg-[var(--accent)] px-1 text-[10px] font-bold text-white">
						{channelUnreadTotal + dmUnreadTotal > 99 ? '99+' : channelUnreadTotal + dmUnreadTotal}
					</span>
				{/if}
				{#if authStore.user}
					<Avatar userId={authStore.user.id} size="xs" showStatus />
				{/if}
			</div>
		</nav>
		{/if}

		<!-- ═══ COMMUNITY PICKER DROPDOWN ═══ -->
		{#if showCommunityPicker}
			<button class="fixed inset-0 z-30" onclick={() => showCommunityPicker = false} aria-label="Close community picker"></button>
			<div class="absolute left-2 top-12 md:top-14 z-40 w-72 max-h-[70vh] overflow-y-auto rounded-2xl border border-white/10 bg-[var(--bg-secondary)] p-3 shadow-xl" transition:fly={{ y: -10, duration: 150 }}>
				<h3 class="mb-2 px-1 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Communities</h3>
				{#each communityStore.communities as community (community.id)}
					<button
						onclick={() => { switchCommunity(community.id); showCommunityPicker = false; }}
						class="flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left transition hover:bg-white/5 {communityStore.activeCommunityId === community.id ? 'bg-white/10' : ''}"
					>
						{#if community.icon_url}
							<img src={community.icon_url} alt={community.name} class="h-8 w-8 rounded-full object-cover" onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }} />
						{:else}
							<span class="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--accent)] text-xs font-bold text-white">{community.name.slice(0, 2).toUpperCase()}</span>
						{/if}
						<span class="flex-1 truncate text-sm font-medium text-[var(--text-primary)]">{community.name}</span>
						{#if communityStore.activeCommunityId === community.id}
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12" /></svg>
						{/if}
					</button>
				{/each}
				<div class="my-2 h-px bg-white/10"></div>
				<button
					onclick={() => { showJoinCommunity = true; showCommunityPicker = false; }}
					class="flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left text-sm text-[var(--success)] transition hover:bg-white/5"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg>
					Join a Community
				</button>
				{#if authStore.user?.is_admin || authStore.user?.is_owner}
					<button
						onclick={() => { showCreateCommunity = true; showCommunityPicker = false; }}
						class="flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left text-sm text-[var(--accent)] transition hover:bg-white/5"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5z" /><path d="M2 17l10 5 10-5" /><path d="M2 12l10 5 10-5" /></svg>
						Create a Community
					</button>
				{/if}
				{#if communityStore.activeCommunityId}
					<button
						onclick={() => { goto('/community'); showCommunityPicker = false; }}
						class="flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left text-sm text-[var(--text-secondary)] transition hover:bg-white/5"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" /><path d="M23 21v-2a4 4 0 0 0-3-3.87" /><path d="M16 3.13a4 4 0 0 1 0 7.75" /></svg>
						Community Settings
					</button>
				{/if}
			</div>
		{/if}

		<!-- ═══ SIDEBAR NAV CONTENT SNIPPET ═══ -->
		{#snippet sidebarNavContent()}
				<!-- Nav header with create button -->
				<div class="flex items-center justify-between border-b border-white/10 px-3 py-2">
					<span class="text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Navigate</span>
					{#if sidebarTab === 'groups'}
						<div class="relative">
							<button
								onclick={() => { showSidebarCreateMenu = !showSidebarCreateMenu; }}
								class="rounded-lg p-1.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
								title="Create..."
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg>
							</button>
							{#if showSidebarCreateMenu}
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div class="absolute right-0 top-full z-50 mt-1 w-40 rounded-lg border border-white/10 bg-[var(--bg-secondary)] py-1 shadow-xl"
									onmousedown={(e) => e.stopPropagation()}>
									<button
										onclick={() => { showSidebarCreateMenu = false; showCreateGroup = true; showCreateChannel = false; }}
										class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-[var(--text-primary)] transition hover:bg-white/5"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
										Group
									</button>
									<button
										onclick={() => { showSidebarCreateMenu = false; showCreateChannel = true; showCreateGroup = false; }}
										class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm text-[var(--text-primary)] transition hover:bg-white/5"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 11a9 9 0 0 1 9 9"/><path d="M4 4a16 16 0 0 1 16 16"/><circle cx="5" cy="19" r="1"/></svg>
										Channel
									</button>
								</div>
							{/if}
						</div>
					{:else}
						<button
							onclick={() => { showNewDm = !showNewDm; }}
							class="rounded-lg p-1.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
							title="New DM"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg>
						</button>
					{/if}
				</div>

			<!-- Tab switcher -->
			<div class="flex border-b border-white/10" role="tablist">
				<button
					onclick={() => (sidebarTab = 'groups')}
					role="tab"
					aria-selected={sidebarTab === 'groups'}
					class="flex-1 px-3 py-2 text-sm font-medium transition {sidebarTab === 'groups' ? 'border-b-2 border-[var(--accent)] text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
				>
					Community
				</button>
				<button
					onclick={() => (sidebarTab = 'dms')}
					role="tab"
					aria-selected={sidebarTab === 'dms'}
					class="flex-1 items-center justify-center gap-1.5 px-3 py-2 text-sm font-medium transition inline-flex {sidebarTab === 'dms' ? 'border-b-2 border-[var(--accent)] text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
				>
					DMs
					{#if dmUnreadTotal > 0 && sidebarTab !== 'dms'}
						<span class="inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-[var(--accent)] px-1 text-[10px] font-bold text-white">
							{dmUnreadTotal > 99 ? '99+' : dmUnreadTotal}
						</span>
					{/if}
				</button>
			</div>

			{#if showCreateGroup && sidebarTab === 'groups'}
				<form onsubmit={handleCreateGroup} class="border-b border-white/10 p-3 space-y-2">
					<p class="text-xs font-semibold text-[var(--text-secondary)]">New Group</p>
					<input
						type="text"
						bind:value={newGroupName}
						placeholder="Group name..."
						maxlength="64"
						onkeydown={(e) => { if (e.key === 'Escape') { showCreateGroup = false; newGroupName = ''; newGroupDescription = ''; newGroupVisibility = 'public'; } }}
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
					<input
						type="text"
						bind:value={newGroupDescription}
						placeholder="Description (optional)..."
						maxlength="2048"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
					{#if !newGroupAssignMemberId}
						<div class="flex items-center gap-3">
							<span class="text-xs text-[var(--text-secondary)]">Visibility</span>
							<div class="flex rounded-lg border border-white/10 overflow-hidden">
								<button type="button" onclick={() => newGroupVisibility = 'public'}
									class="px-3 py-1 text-xs font-medium transition {newGroupVisibility === 'public' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}">
									Public
								</button>
								<button type="button" onclick={() => newGroupVisibility = 'private'}
									class="px-3 py-1 text-xs font-medium transition {newGroupVisibility === 'private' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}">
									Private
								</button>
							</div>
						</div>
					{/if}
					{#if isCommunityModeratorOrAbove() && communityStore.activeCommunityId}
						<select
							bind:value={newGroupAssignMemberId}
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						>
							<option value="">Regular group (no assignment)</option>
							{#each communityMemberStore.getMembers(communityStore.activeCommunityId).filter(m => m.user_id !== authStore.user?.id) as member}
								<option value={member.user_id}>{member.display_name || member.username} (@{member.username})</option>
							{/each}
						</select>
					{/if}
					<div class="flex gap-2">
						<button
							type="button"
							onclick={() => { showCreateGroup = false; newGroupName = ''; newGroupDescription = ''; newGroupVisibility = 'public'; newGroupAssignMemberId = ''; }}
							class="flex-1 rounded-lg border border-white/10 px-3 py-1.5 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						>
							Cancel
						</button>
						<button
							type="submit"
							disabled={creatingGroup || !newGroupName.trim()}
							class="flex-1 rounded-lg bg-[var(--accent)] px-3 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
						>
							{creatingGroup ? 'Creating...' : newGroupAssignMemberId ? 'Create Personal Group' : 'Create Group'}
						</button>
					</div>
				</form>
			{/if}

			{#if showCreateChannel && sidebarTab === 'groups'}
				<form onsubmit={handleCreateChannel} class="border-b border-white/10 p-3 space-y-2">
					<p class="text-xs font-semibold text-[var(--text-secondary)]">New Channel</p>
					<input
						type="text"
						bind:value={newChannelName}
						placeholder="Channel name..."
						maxlength="64"
						onkeydown={(e) => { if (e.key === 'Escape') { showCreateChannel = false; newChannelName = ''; newChannelType = 'text'; } }}
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
					<div class="flex items-center gap-3">
						<span class="text-xs text-[var(--text-secondary)]">Type</span>
						<div class="flex rounded-lg border border-white/10 overflow-hidden">
							<button type="button" onclick={() => newChannelType = 'text'}
								class="px-3 py-1 text-xs font-medium transition {newChannelType === 'text' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}">
								Text
							</button>
							<button type="button" onclick={() => newChannelType = 'voice'}
								class="px-3 py-1 text-xs font-medium transition {newChannelType === 'voice' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}">
								Voice
							</button>
						</div>
					</div>
					<div class="flex gap-2">
						<button
							type="button"
							onclick={() => { showCreateChannel = false; newChannelName = ''; newChannelType = 'text'; }}
							class="flex-1 rounded-lg border border-white/10 px-3 py-1.5 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						>
							Cancel
						</button>
						<button
							type="submit"
							disabled={creatingChannel || !newChannelName.trim()}
							class="flex-1 rounded-lg bg-[var(--accent)] px-3 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
						>
							{creatingChannel ? 'Creating...' : 'Create Channel'}
						</button>
					</div>
				</form>
			{/if}

			{#if showNewDm && sidebarTab === 'dms'}
				<div class="border-b border-white/10 p-3">
					<div class="flex items-center gap-2">
						<input
							type="text"
							bind:value={dmSearchQuery}
							oninput={handleDmSearch}
							placeholder="Search users..."
							class="flex-1 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						/>
						<button
							onclick={() => { showNewDm = false; dmSearchQuery = ''; dmSearchResults = []; dmSearchDone = false; }}
							class="rounded-lg p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
							title="Cancel"
							aria-label="Cancel"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
						</button>
					</div>
					{#if dmSearchError}
						<p class="mt-2 px-3 text-xs text-[var(--danger)]">Search failed. Try again.</p>
					{:else if dmSearchLoading}
						<div class="mt-3 flex items-center justify-center gap-2 text-xs text-[var(--text-secondary)]">
							<div class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
							Searching...
						</div>
					{:else if dmSearchResults.length > 0}
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
					{:else if dmSearchDone && dmSearchQuery.length >= 2}
						<p class="mt-2 px-3 text-xs text-[var(--text-secondary)]">No users found</p>
					{:else if dmSearchQuery.length > 0 && dmSearchQuery.length < 2}
						<p class="mt-2 px-3 text-xs text-[var(--text-secondary)]">Type at least 2 characters</p>
					{/if}
				</div>
			{/if}

			<!-- Sidebar search filter -->
			<div class="flex items-center gap-1.5 px-2 pt-2">
				<div class="relative flex-1">
					<svg xmlns="http://www.w3.org/2000/svg" class="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
					</svg>
					<input
						type="text"
						bind:value={sidebarFilter}
						placeholder="Filter..."
						class="w-full rounded-md border border-white/10 bg-[var(--bg-primary)] py-1.5 pl-8 pr-8 text-xs text-[var(--text-primary)] outline-none transition placeholder:text-[var(--text-secondary)]/50 focus:border-[var(--accent)]"
					/>
					{#if sidebarFilter}
						<button
							onclick={() => { sidebarFilter = ''; }}
							title="Clear filter"
							aria-label="Clear filter"
							class="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
						</button>
					{/if}
				</div>
				{#if messageStore.hasAnyUnread}
					<button
						onclick={markAllRead}
						title="Mark all as read (Shift+Esc)"
						aria-label="Mark all as read"
						class="shrink-0 rounded-md border border-white/10 bg-[var(--bg-primary)] p-1.5 text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" />
						</svg>
					</button>
				{/if}
			</div>

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
								placeholder="Paste invite link or code..."
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

					{#each groupStore.groups.filter(g => {
						if (!sidebarFilter) return true;
						const q = sidebarFilter.toLowerCase();
						if (g.name.toLowerCase().includes(q)) return true;
						const chs = groupChannelsMap.get(g.id) ?? [];
						return chs.some(c => c.name?.toLowerCase().includes(q));
					}) as group (group.id)}
						<!-- Group header -->
						<div class="group/grp relative">
							{#if renamingGroupId === group.id}
								<form
									onsubmit={(e) => { e.preventDefault(); handleRenameGroup(group.id); }}
									class="flex items-center gap-2 px-3 py-2"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="currentColor">
										<path d="M8 5l8 7-8 7z" />
									</svg>
									<!-- svelte-ignore a11y_autofocus -->
									<input
										type="text"
										bind:value={renameGroupInput}
										onkeydown={(e) => { if (e.key === 'Escape') renamingGroupId = null; }}
										onblur={() => handleRenameGroup(group.id)}
										maxlength="64"
										class="flex-1 rounded border border-[var(--accent)] bg-[var(--bg-primary)] px-1.5 py-0.5 text-sm font-medium text-[var(--text-primary)] outline-none"
										autofocus
									/>
								</form>
							{:else}
								<button
									onclick={() => toggleGroupExpand(group.id)}
									oncontextmenu={(e) => openGroupSettings(group, e)}
									ondblclick={() => { if (group.owner_id === authStore.user?.id) { renamingGroupId = group.id; renameGroupInput = group.name; } }}
									class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm font-medium transition hover:bg-white/5 {expandedGroupIds.has(group.id) ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 transition-transform {expandedGroupIds.has(group.id) ? 'rotate-90' : ''}" viewBox="0 0 24 24" fill="currentColor">
										<path d="M8 5l8 7-8 7z" />
									</svg>
									{#if group.icon_url}
										<img src={group.icon_url} alt={group.name} class="h-4 w-4 shrink-0 rounded-full object-cover" onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }} />
									{:else if group.assigned_member_id}
										<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
									{:else if group.visibility === 'private'}
										<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-yellow-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
									{/if}
									<span class="flex-1 truncate" title={group.name} style={group.accent_color ? `color: ${group.accent_color}` : ''}>{group.name}</span>
									<span class="text-xs text-[var(--text-secondary)]">{group.member_count}</span>
								</button>
								<!-- Settings gear (hover) -->
								<button
									onclick={(e) => openGroupSettings(group, e)}
									class="hidden group-hover/grp:flex absolute right-2 top-1/2 -translate-y-1/2 rounded p-0.5 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
									title="Group settings"
									aria-label="Group settings"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
									</svg>
								</button>
							{/if}
						</div>

						{#if expandedGroupIds.has(group.id)}
							<!-- Group channels -->
							<div class="ml-2 border-l border-white/5 pl-2">
								{#each (groupChannelsMap.get(group.id) ?? []).filter(c => !sidebarFilter || (c.name ?? '').toLowerCase().includes(sidebarFilter.toLowerCase()) || group.name.toLowerCase().includes(sidebarFilter.toLowerCase())) as channel (channel.id)}
									{@const unreadCount = messageStore.getUnreadCount(channel.id)}
									<div class="group/ch flex items-center">
										{#if renamingChannelId === channel.id}
											<form
												onsubmit={(e) => { e.preventDefault(); handleRenameGroupChannel(group.id, channel.id); }}
												class="flex flex-1 items-center gap-1 px-2 py-0.5"
											>
												{#if channel.channel_type === 'voice'}
													<span class="text-[var(--text-secondary)]">🔊</span>
												{:else}
													<span class="text-[var(--text-secondary)]">#</span>
												{/if}
											<!-- svelte-ignore a11y_autofocus -->
												<input
													type="text"
													bind:value={renameChannelInput}
													onkeydown={(e) => { if (e.key === 'Escape') renamingChannelId = null; }}
													onblur={() => handleRenameGroupChannel(group.id, channel.id)}
													maxlength="64"
													class="flex-1 rounded border border-[var(--accent)] bg-[var(--bg-primary)] px-1.5 py-0.5 text-sm text-[var(--text-primary)] outline-none"
													autofocus
												/>
											</form>
										{:else}
											<button
												onclick={() => selectChannel(channel.id)}
												oncontextmenu={(e) => openChannelSettings(channel, group.id, e)}
												ondblclick={() => { if (group.owner_id === authStore.user?.id) { renamingChannelId = channel.id; renameChannelInput = channel.name ?? ''; } }}
												class="flex flex-1 items-center gap-2 rounded-lg px-3 py-1.5 text-left text-sm transition {channelStore.activeChannelId === channel.id ? 'bg-white/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
											>
												{#if channel.channel_type === 'voice'}
													<span class="text-[var(--text-secondary)]" title="Voice channel">🔊</span>
												{:else}
													<span class="text-[var(--text-secondary)]">#</span>
												{/if}
												<span class="flex-1 truncate {unreadCount > 0 ? 'font-semibold text-[var(--text-primary)]' : ''} {channel.archived ? 'opacity-50 italic' : ''}">{channel.name}</span>
												{#if channel.archived}
													<span title="Archived"><svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-orange-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="21 8 21 21 3 21 3 8"/><rect x="1" y="3" width="22" height="5"/><line x1="10" y1="12" x2="14" y2="12"/></svg></span>
												{:else if channel.read_only}
													<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-yellow-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
												{/if}
												{#if unreadCount > 0 && channelStore.activeChannelId !== channel.id}
													<span class="flex h-5 min-w-5 items-center justify-center rounded-full bg-[var(--accent)] px-1.5 text-xs font-bold text-white">
														{unreadCount > 99 ? '99+' : unreadCount}
													</span>
												{/if}
											</button>
											<button
												onclick={(e) => openChannelSettings(channel, group.id, e)}
												class="hidden group-hover/ch:block rounded p-0.5 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
												title="Channel settings"
												aria-label="Channel settings"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
													<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
												</svg>
											</button>
										{/if}
									</div>
									<!-- Voice participants (grouped channels) -->
									{#if channel.channel_type === 'voice'}
										{#if voiceStore.getChannelParticipants(channel.id).length > 0}
											<div class="ml-8 space-y-0.5 pb-1">
												{#each voiceStore.getChannelParticipants(channel.id) as uid (uid)}
													<button
														class="flex w-full items-center gap-1.5 rounded px-2 py-0.5 text-xs text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]"
														onclick={(e) => { e.stopPropagation(); openProfileCard(uid, e); }}
														oncontextmenu={(e) => {
															e.preventDefault();
															e.stopPropagation();
															voiceContextMenu = { userId: uid, channelId: channel.id, x: e.clientX, y: e.clientY };
														}}
													>
														<div class="h-1.5 w-1.5 shrink-0 rounded-full bg-[var(--success)]"></div>
														<span class="truncate" title={userStore.getDisplayName(uid)}>{userStore.getDisplayName(uid)}</span>
													</button>
												{/each}
											</div>
										{/if}
									{/if}
								{/each}

								<!-- Add channel button (owner/admin) -->
								{#if getMyGroupRole(group.id) === 'owner' || getMyGroupRole(group.id) === 'admin'}
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
											<div class="flex gap-1">
												<button
													type="button"
													onclick={() => { showGroupChannelCreate = null; newGroupChannelName = ''; }}
													class="flex-1 rounded border border-white/10 px-2 py-1 text-xs font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
												>
													Cancel
												</button>
												<button
													type="submit"
													disabled={creatingGroupChannel || !newGroupChannelName.trim()}
													class="flex-1 rounded bg-[var(--accent)] px-2 py-1 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
												>
													{creatingGroupChannel ? 'Creating...' : 'Create'}
												</button>
											</div>
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

					{#if !initialized}
						<!-- Loading skeleton -->
						<div class="space-y-2">
							{#each [1, 2, 3] as _}
								<div class="h-9 animate-pulse rounded-lg bg-[var(--bg-tertiary)]"></div>
							{/each}
						</div>
					{:else if groupStore.groups.length === 0}
						<div class="rounded-lg border border-dashed border-white/10 p-4 text-center">
							<p class="text-sm text-[var(--text-primary)]">No groups yet</p>
							<p class="mt-1 text-xs text-[var(--text-secondary)]">Create a group, discover existing ones, or join via an invite link.</p>
						</div>
					{/if}

					<!-- Discover groups modal -->
					{#if showDiscoverGroups}
						<div class="mt-2 rounded-lg border border-white/10 bg-[var(--bg-primary)] p-3">
							<div class="mb-2 flex items-center justify-between">
								<h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Discover</h3>
								<button
									onclick={() => { showDiscoverGroups = false; }}
									class="rounded p-0.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
									aria-label="Close discover groups"
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
				{:else}
					<h2 class="mb-2 px-2 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
						Direct Messages
					</h2>
					{#each dmChannels.filter(dm => !sidebarFilter || dm.other_user.display_name.toLowerCase().includes(sidebarFilter.toLowerCase()) || dm.other_user.username.toLowerCase().includes(sidebarFilter.toLowerCase())) as dm (dm.channel.id)}
						{@const unreadCount = messageStore.getUnreadCount(dm.channel.id)}
						<button
							onclick={() => { channelStore.addChannel(dm.channel); selectChannel(dm.channel.id); }}
							class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm transition {channelStore.activeChannelId === dm.channel.id ? 'bg-white/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
						>
							<Avatar userId={dm.other_user.id} size="xs" showStatus />
							<span class="flex-1 truncate {unreadCount > 0 ? 'font-semibold text-[var(--text-primary)]' : ''}" title={getDmDisplayName(dm)}>{getDmDisplayName(dm)}</span>
							{#if unreadCount > 0 && channelStore.activeChannelId !== dm.channel.id}
								<span class="flex h-5 min-w-5 items-center justify-center rounded-full bg-[var(--accent)] px-1.5 text-xs font-bold text-white">
									{unreadCount > 99 ? '99+' : unreadCount}
								</span>
							{/if}
						</button>
					{/each}
					{#if !initialized}
						<div class="space-y-2">
							{#each [1, 2, 3] as _}
								<div class="h-9 animate-pulse rounded-lg bg-[var(--bg-tertiary)]"></div>
							{/each}
						</div>
					{:else if dmChannels.length === 0}
						<div class="rounded-lg border border-dashed border-white/10 p-6 text-center">
							<svg xmlns="http://www.w3.org/2000/svg" class="mx-auto mb-3 h-10 w-10 text-[var(--text-secondary)]/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
								<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
							</svg>
							<p class="text-sm font-medium text-[var(--text-primary)]">No conversations yet</p>
							<p class="mt-1 text-xs text-[var(--text-secondary)]">Start chatting with someone!</p>
							<button
								onclick={() => { showNewDm = true; }}
								class="mt-3 inline-flex items-center gap-1.5 rounded-lg bg-[var(--accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)]"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" /><circle cx="8.5" cy="7" r="4" /><line x1="20" y1="8" x2="20" y2="14" /><line x1="23" y1="11" x2="17" y2="11" />
								</svg>
								Find someone to chat with
							</button>
						</div>
					{/if}
				{/if}
			</div>
		{/snippet}

		<!-- ═══ NAVIGATION DROPDOWN (compact mode + mobile) ═══ -->
		{#if showNavDropdown}
			<button class="fixed inset-0 z-30 {isExpandedSidebar ? 'md:hidden' : ''}" onclick={() => showNavDropdown = false} aria-label="Close navigation"></button>
			<div class="fixed inset-x-0 top-12 bottom-0 z-40 flex flex-col bg-[var(--bg-secondary)] {isExpandedSidebar ? 'md:hidden' : 'md:absolute md:inset-auto md:left-20 lg:left-40 md:top-12 lg:top-14 md:w-80 md:max-h-[75vh] md:rounded-2xl md:border md:border-white/10 md:shadow-xl'}" transition:fly={{ y: -10, duration: 150 }}>
				{@render sidebarNavContent()}
			</div>
		{/if}

		<!-- ═══ USER MENU DROPDOWN ═══ -->
		{#if showUserMenu}
			<button class="fixed inset-0 z-30" onclick={() => showUserMenu = false} aria-label="Close user menu"></button>
			<div class="absolute right-2 top-12 md:top-14 z-40 w-64 rounded-2xl border border-white/10 bg-[var(--bg-secondary)] shadow-xl" transition:fly={{ y: -10, duration: 150 }}>
				<!-- User info -->
				{#if authStore.user}
					<div class="flex items-center gap-3 border-b border-white/10 p-4">
						<Avatar userId={authStore.user.id} size="sm" showStatus />
						<div class="flex-1 overflow-hidden">
							<div class="truncate text-sm font-medium text-[var(--text-primary)]">{authStore.user.display_name}</div>
							<div class="truncate text-xs text-[var(--text-secondary)]">
								{statusOptions.find(s => s.value === (authStore.user ? presenceStore.getStatus(authStore.user.id) : 'offline'))?.label ?? 'Online'}
							</div>
						</div>
					</div>
				{/if}
				<!-- Status picker -->
				<div class="border-b border-white/10 p-2">
					<p class="px-2 pb-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Status</p>
					{#each statusOptions as opt (opt.value)}
						<button
							onclick={() => { setUserStatus(opt.value); }}
							class="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left text-sm transition hover:bg-white/5"
						>
							<span class="h-2 w-2 rounded-full {opt.color}"></span>
							<span class="text-[var(--text-primary)]">{opt.label}</span>
						</button>
					{/each}
				</div>
				<!-- Voice controls (when in call) -->
				{#if voiceStore.activeCall}
					{@const voiceChannel = channelStore.channels.find(c => c.id === voiceStore.activeCall?.channelId)}
					<div class="border-b border-white/10 p-3">
						<button
							onclick={() => { voiceStore.activeCall && selectChannel(voiceStore.activeCall.channelId); showUserMenu = false; }}
							class="mb-2 flex items-center gap-2 text-xs font-medium text-[var(--success)] transition hover:brightness-110"
						>
							<span class="h-2 w-2 animate-pulse rounded-full bg-[var(--success)]"></span>
							Voice Connected — {voiceChannel?.name ?? 'Voice'}
						</button>
						<div class="flex items-center justify-center gap-1">
							<button onclick={() => webrtcManager.toggleAudio()} class="rounded-md p-1.5 transition {voiceStore.activeCall?.audioEnabled ? 'text-[var(--text-secondary)] hover:bg-white/10' : 'bg-red-500/20 text-red-400'}" title={voiceStore.activeCall?.audioEnabled ? 'Mute' : 'Unmute'} aria-label={voiceStore.activeCall?.audioEnabled ? 'Mute' : 'Unmute'}>
								{#if voiceStore.activeCall?.audioEnabled}
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" /><path d="M19 10v2a7 7 0 0 1-14 0v-2" /><line x1="12" y1="19" x2="12" y2="23" /><line x1="8" y1="23" x2="16" y2="23" /></svg>
								{:else}
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="1" y1="1" x2="23" y2="23" /><path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6" /><path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .4-.03.8-.1 1.17" /><line x1="12" y1="19" x2="12" y2="23" /><line x1="8" y1="23" x2="16" y2="23" /></svg>
								{/if}
							</button>
							<button onclick={() => webrtcManager.toggleVideo()} class="rounded-md p-1.5 transition {voiceStore.activeCall?.videoEnabled ? 'text-[var(--text-primary)] hover:bg-white/10' : 'text-[var(--text-secondary)] hover:bg-white/10'}" title={voiceStore.activeCall?.videoEnabled ? 'Turn off camera' : 'Turn on camera'} aria-label={voiceStore.activeCall?.videoEnabled ? 'Turn off camera' : 'Turn on camera'}>
								{#if voiceStore.activeCall?.videoEnabled}
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="23 7 16 12 23 17 23 7" /><rect x="1" y="5" width="15" height="14" rx="2" ry="2" /></svg>
								{:else}
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 16v1a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h2m5.66 0H14a2 2 0 0 1 2 2v3.34l1 1L23 7v10" /><line x1="1" y1="1" x2="23" y2="23" /></svg>
								{/if}
							</button>
							<button onclick={() => webrtcManager.toggleScreenShare()} class="rounded-md p-1.5 transition {voiceStore.activeCall?.screenSharing ? 'bg-[var(--accent)]/20 text-[var(--accent)]' : 'text-[var(--text-secondary)] hover:bg-white/10'}" title={voiceStore.activeCall?.screenSharing ? 'Stop sharing' : 'Share screen'} aria-label={voiceStore.activeCall?.screenSharing ? 'Stop sharing' : 'Share screen'}>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2" /><line x1="8" y1="21" x2="16" y2="21" /><line x1="12" y1="17" x2="12" y2="21" /></svg>
							</button>
							<button onclick={() => { webrtcManager.leaveCall(); chatCollapsed = false; showUserMenu = false; }} class="rounded-md bg-red-500/20 p-1.5 text-red-400 transition hover:bg-red-500/30" title="Disconnect" aria-label="Disconnect from call">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91" /><line x1="23" y1="1" x2="1" y2="23" /></svg>
							</button>
						</div>
					</div>
				{/if}
				<!-- Actions -->
				<div class="p-2">
					<button
						onclick={() => { whatsNewRef?.open(); showUserMenu = false; }}
						class="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" /></svg>
						Changelog <span class="ml-auto text-[10px] text-[var(--text-secondary)]">v{__APP_VERSION__}</span>
					</button>
					<button
						onclick={() => { showFeedback = true; showUserMenu = false; }}
						class="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
						Send Feedback
					</button>
					{#if authStore.user?.is_admin || authStore.user?.is_owner}
					<a
						href="/admin"
						onclick={() => { showUserMenu = false; }}
						class="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left text-sm text-[var(--accent)] transition hover:bg-white/5"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" /></svg>
						Admin Panel
					</a>
					{/if}
					<div class="my-1 h-px bg-white/10"></div>
					<button
						onclick={() => { webrtcManager.leaveCall(); authStore.logout(); goto('/login'); }}
						class="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left text-sm text-[var(--danger)] transition hover:bg-white/5"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" /><polyline points="16 17 21 12 16 7" /><line x1="21" y1="12" x2="9" y2="12" /></svg>
						Sign Out
					</button>
				</div>
			</div>
		{/if}


		<!-- Join Community modal (overlays the sidebar area) -->
		{#if showJoinCommunity}
			<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" transition:fade={{ duration: 150 }}>
				<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
				<div role="dialog" aria-modal="true" aria-label="Join a community" tabindex="-1" class="w-full max-w-sm rounded-2xl bg-[var(--bg-secondary)] p-6 shadow-xl" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
					<div class="mb-4 flex items-center justify-between">
						<h3 class="text-lg font-bold text-[var(--text-primary)]">Join a Community</h3>
						<button onclick={() => { showJoinCommunity = false; }} class="text-[var(--text-secondary)] hover:text-[var(--text-primary)]" aria-label="Close">&times;</button>
					</div>
					<input
						type="text"
						bind:value={joinCommunityCode}
						placeholder="Paste invite link or code..."
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/50"
					/>
					{#if communityInvitePreview}
						<div class="mt-3 rounded-lg bg-white/5 p-3">
							<div class="font-medium text-[var(--text-primary)]">{communityInvitePreview.community_name}</div>
							{#if communityInvitePreview.community_description}
								<div class="mt-1 text-sm text-[var(--text-secondary)]">{communityInvitePreview.community_description}</div>
							{/if}
							<div class="mt-1 text-xs text-[var(--text-secondary)]">{communityInvitePreview.member_count} members</div>
						</div>
						<button onclick={handleAcceptCommunityInvite} class="mt-3 w-full rounded-lg bg-[var(--accent)] px-4 py-2.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]">Join Community</button>
					{:else}
						<button onclick={handlePreviewCommunityInvite} class="mt-3 w-full rounded-lg bg-white/10 px-4 py-2.5 text-sm font-medium text-[var(--text-primary)] transition hover:bg-white/15">Look Up</button>
					{/if}
				</div>
			</div>
		{/if}

		<!-- Create Community modal -->
		{#if showCreateCommunity}
			<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" transition:fade={{ duration: 150 }}>
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions a11y_click_events_have_key_events -->
				<form
					onsubmit={(e) => { e.preventDefault(); handleCreateCommunity(); }}
					role="dialog"
					aria-modal="true"
					aria-label="Create a community"
					class="w-full max-w-sm rounded-2xl bg-[var(--bg-secondary)] p-6 shadow-xl"
					onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}
				>
					<div class="mb-4 flex items-center justify-between">
						<h3 class="text-lg font-bold text-[var(--text-primary)]">Create a Community</h3>
						<button type="button" onclick={() => { showCreateCommunity = false; }} class="text-[var(--text-secondary)] hover:text-[var(--text-primary)]" aria-label="Close">&times;</button>
					</div>
					<div class="space-y-3">
						<div>
							<label for="create-community-name" class="mb-1 block text-xs text-[var(--text-secondary)]">Name</label>
							<input id="create-community-name"
								type="text"
								bind:value={newCommunityName}
								placeholder="My Community"
								maxlength="64"
								required
								class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/50"
							/>
						</div>
						<div>
							<label for="create-community-desc" class="mb-1 block text-xs text-[var(--text-secondary)]">Description (optional)</label>
							<input id="create-community-desc"
								type="text"
								bind:value={newCommunityDescription}
								placeholder="What's this community about?"
								class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/50"
							/>
						</div>
					</div>
					<button
						type="submit"
						disabled={creatingCommunity || !newCommunityName.trim()}
						class="mt-4 w-full rounded-lg bg-[var(--accent)] px-4 py-2.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
					>
						{creatingCommunity ? 'Creating...' : 'Create Community'}
					</button>
				</form>
			</div>
		{/if}

		<!-- Welcome Splash modal -->
		{#if showWelcomeSplash && welcomeCommunity}
			<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" transition:fade={{ duration: 200 }}>
				<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
				<div role="dialog" aria-modal="true" aria-label="Welcome to community" tabindex="-1" class="w-full max-w-md overflow-hidden rounded-2xl bg-[var(--bg-secondary)] shadow-xl" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
					{#if welcomeCommunity.banner_url}
						<img src={welcomeCommunity.banner_url} alt="Community banner" class="h-32 w-full object-cover" />
					{:else}
						<div class="h-20 bg-gradient-to-r from-[var(--accent)] to-[var(--accent-hover)]"></div>
					{/if}
					<div class="p-6">
						<div class="flex items-center gap-3">
							{#if welcomeCommunity.icon_url}
								<img src={welcomeCommunity.icon_url} alt="Community icon" class="h-12 w-12 rounded-full border-2 border-[var(--bg-secondary)] object-cover" onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }} />
							{/if}
							<h3 class="text-xl font-bold text-[var(--text-primary)]">{welcomeCommunity.name}</h3>
						</div>
						<p class="mt-4 whitespace-pre-wrap text-sm text-[var(--text-secondary)]">{welcomeCommunity.welcome_message}</p>
						<button
							onclick={() => {
								localStorage.setItem(`chatalot:welcomeDismissed:${welcomeCommunity!.id}`, '1');
								showWelcomeSplash = false;
								welcomeCommunity = null;
							}}
							class="mt-6 w-full rounded-lg bg-[var(--accent)] px-4 py-2.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
						>
							Got it
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- Main content row -->
		<div class="flex flex-1 overflow-hidden">

		<!-- Expanded sidebar panel (desktop only) -->
		{#if isExpandedSidebar}
			<aside class="hidden md:flex w-64 lg:w-72 shrink-0 flex-col border-r border-white/10 bg-[var(--bg-secondary)]">
				{@render sidebarNavContent()}
			</aside>
		{/if}

		<!-- Main chat area -->
		<main
			class="relative flex flex-1 flex-col overflow-hidden bg-[var(--bg-primary)]"
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
			{#if pendingUpdate}
				<button
					onclick={async () => {
						if ('caches' in window) {
							const keys = await caches.keys();
							await Promise.all(keys.map(k => caches.delete(k)));
						}
						const regs = await navigator.serviceWorker?.getRegistrations();
						if (regs) await Promise.all(regs.map(r => r.unregister()));
						location.reload();
					}}
					class="flex w-full items-center justify-center gap-2 bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:brightness-110 cursor-pointer"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" /><path d="M21 3v5h-5" /></svg>
					A new version is available — click to update
				</button>
			{/if}
			{#if activeChannel}
				<!-- Channel header -->
				<header class="flex h-10 items-center justify-between border-b border-white/10 px-3 md:px-6">
					<div class="flex items-center">
						{#if activeChannel.channel_type === 'dm'}
							<span class="mr-2 text-[var(--text-secondary)]">@</span>
						{:else if activeChannel.channel_type === 'voice'}
							<span class="mr-2">🔊</span>
						{:else}
							<span class="mr-2 text-[var(--text-secondary)]">#</span>
						{/if}
						<h2 class="font-semibold text-[var(--text-primary)]">{getChannelDisplayName()}</h2>
						<span class="ml-1.5 text-green-400" title="End-to-end encrypted">
							<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
						</span>
						{#if activeChannel.read_only}
							<span class="ml-2 inline-flex items-center gap-1 rounded bg-yellow-500/20 px-1.5 py-0.5 text-xs text-yellow-400">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
								Read-only
							</span>
						{/if}
						{#if activeChannel.slow_mode_seconds > 0}
							<span class="ml-1 inline-flex items-center gap-1 rounded bg-blue-500/20 px-1.5 py-0.5 text-xs text-blue-400">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/></svg>
								{activeChannel.slow_mode_seconds >= 60 ? `${Math.floor(activeChannel.slow_mode_seconds / 60)}m` : `${activeChannel.slow_mode_seconds}s`}
							</span>
						{/if}
						{#if editingTopic}
							<div class="ml-4 hidden md:flex items-center gap-1">
								<input
									type="text"
									bind:value={topicInput}
									onkeydown={(e) => { if (e.key === 'Enter') saveTopic(); if (e.key === 'Escape') editingTopic = false; }}
									class="w-48 rounded border border-[var(--accent)] bg-[var(--bg-primary)] px-2 py-0.5 text-sm text-[var(--text-primary)] outline-none"
									placeholder="Set a topic..."
								/>
								<button onclick={saveTopic} class="rounded px-1.5 py-0.5 text-xs text-[var(--accent)] hover:bg-white/5">Save</button>
								<button onclick={() => editingTopic = false} class="rounded px-1.5 py-0.5 text-xs text-[var(--text-secondary)] hover:bg-white/5">Cancel</button>
							</div>
						{:else if activeChannel.topic}
							<button
								onclick={() => { if ((myRole === 'owner' || myRole === 'admin') && activeChannel?.group_id) { topicInput = activeChannel.topic ?? ''; editingTopic = true; } }}
								class="ml-4 hidden md:block truncate text-sm text-[var(--text-secondary)] {(myRole === 'owner' || myRole === 'admin') && activeChannel.group_id ? 'cursor-pointer hover:text-[var(--text-primary)]' : 'cursor-default'}"
								title={(myRole === 'owner' || myRole === 'admin') && activeChannel.group_id ? 'Click to edit topic' : activeChannel.topic}
							>
								{activeChannel.topic}
							</button>
						{:else if (myRole === 'owner' || myRole === 'admin') && activeChannel.group_id}
							<button
								onclick={() => { topicInput = ''; editingTopic = true; }}
								class="ml-4 hidden md:block truncate text-sm text-[var(--text-secondary)]/50 hover:text-[var(--text-secondary)] cursor-pointer"
							>
								Set a topic...
							</button>
						{/if}
					</div>
					<div class="flex items-center gap-0.5 md:gap-1">
						<button
							onclick={toggleSearch}
							class="rounded-lg p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] {showSearch ? 'text-[var(--accent)]' : ''}"
							title="Search messages"
							aria-label="Search messages"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
							</svg>
						</button>
						<button
							onclick={togglePinnedPanel}
							class="relative hidden md:block rounded-lg p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] {showPinnedPanel ? 'text-[var(--accent)]' : ''}"
							title="Pinned messages"
							aria-label="Pinned messages"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2z"/>
							</svg>
							{#if channelStore.activeChannelId && messageStore.getPinnedCount(channelStore.activeChannelId) > 0}
								<span class="absolute -right-0.5 -top-0.5 flex h-3.5 w-3.5 items-center justify-center rounded-full bg-[var(--accent)] text-[8px] font-bold text-white">
									{messageStore.getPinnedCount(channelStore.activeChannelId)}
								</span>
							{/if}
						</button>
						{#if activeChannel.channel_type !== 'dm'}
						<button
							onclick={togglePollPanel}
							class="relative hidden md:block rounded-lg p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] {showPollPanel ? 'text-[var(--accent)]' : ''}"
							title="Polls"
							aria-label="Polls"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M7 16V12"/><path d="M12 16V8"/><path d="M17 16v-5"/>
							</svg>
							{#if polls.filter(p => !p.closed).length > 0}
								<span class="absolute -right-0.5 -top-0.5 flex h-3.5 w-3.5 items-center justify-center rounded-full bg-[var(--accent)] text-[8px] font-bold text-white">
									{polls.filter(p => !p.closed).length}
								</span>
							{/if}
						</button>
						{/if}
						{#if activeChannel.channel_type !== 'dm'}
							<div class="relative hidden md:block">
								<button
									onclick={(e) => { e.stopPropagation(); showNotifDropdown = !showNotifDropdown; }}
									class="rounded-lg p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
									title="Notification settings"
								aria-label="Notification settings"
								>
									{#if notificationStore.getChannelLevel(activeChannel.id) === 'nothing'}
										<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<path d="M13.73 21a2 2 0 0 1-3.46 0" /><path d="M18.63 13A17.89 17.89 0 0 1 18 8" /><path d="M6.26 6.26A5.86 5.86 0 0 0 6 8c0 7-3 9-3 9h14" /><path d="M18 8a6 6 0 0 0-9.33-5" /><line x1="1" y1="1" x2="23" y2="23" />
										</svg>
									{:else}
										<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" /><path d="M13.73 21a2 2 0 0 1-3.46 0" />
										</svg>
									{/if}
								</button>
								{#if showNotifDropdown}
									<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
									<div role="menu" tabindex="-1" class="absolute right-0 top-full z-50 mt-1 w-48 rounded-xl bg-[var(--bg-secondary)] py-1 shadow-lg" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
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
						{#if activeChannel.channel_type !== 'dm'}
							<button
								onclick={toggleMemberPanel}
								class="rounded-lg p-2 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] {showMemberPanel ? 'text-[var(--accent)]' : ''}"
								title="Members"
								aria-label="Members"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-[18px] w-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" /><path d="M23 21v-2a4 4 0 0 0-3-3.87" /><path d="M16 3.13a4 4 0 0 1 0 7.75" />
								</svg>
							</button>
						{/if}
					</div>
				</header>

				<!-- Search panel -->
				{#if showSearch}
					<div class="border-b border-white/10 bg-[var(--bg-secondary)] px-4 py-3" transition:slide={{ duration: 150 }}>
						<div class="flex gap-2">
							<input
								type="text"
								bind:this={searchInputEl}
								bind:value={searchQuery}
								oninput={handleSearchInput}
								placeholder={searchScope === 'global' ? 'Search all channels...' : 'Search this channel...'}
								class="flex-1 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
							/>
							<button
								onclick={() => { searchScope = searchScope === 'channel' ? 'global' : 'channel'; searchResults = []; if (searchQuery.trim()) handleSearchInput(); }}
								class="shrink-0 rounded-lg border px-2.5 py-1 text-xs font-medium transition {searchScope === 'global' ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 text-[var(--text-secondary)] hover:bg-white/5'}"
								title="Toggle between channel and global search"
							>
								{searchScope === 'global' ? 'All' : 'Channel'}
							</button>
						</div>
						<!-- Filter toggle + filter controls -->
						<div class="mt-2 flex items-center gap-2">
							<button
								onclick={() => showSearchFilters = !showSearchFilters}
								class="text-xs transition {showSearchFilters ? 'text-[var(--accent)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
							>
								{showSearchFilters ? 'Hide filters' : 'Filters'}
							</button>
							{#if searchFilterSender || searchFilterAfter || searchFilterBefore || searchFilterHasFile}
								<button
									onclick={() => { searchFilterSender = ''; searchFilterAfter = ''; searchFilterBefore = ''; searchFilterHasFile = false; if (searchQuery.trim()) handleSearchInput(); }}
									class="text-xs text-[var(--danger)] hover:underline"
								>Clear</button>
							{/if}
						</div>
						{#if showSearchFilters}
							<div class="mt-2 grid grid-cols-2 gap-2" transition:slide={{ duration: 150 }}>
								<div>
									<label class="mb-0.5 block text-[10px] text-[var(--text-secondary)]">From user</label>
									<input
										type="text"
										bind:value={searchFilterSender}
										onchange={handleSearchInput}
										placeholder="username"
										class="w-full rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
									/>
								</div>
								<div class="flex items-end">
									<label class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)]">
										<input type="checkbox" bind:checked={searchFilterHasFile} onchange={handleSearchInput} class="accent-[var(--accent)]" />
										Has file
									</label>
								</div>
								<div>
									<label class="mb-0.5 block text-[10px] text-[var(--text-secondary)]">After</label>
									<input
										type="date"
										bind:value={searchFilterAfter}
										onchange={handleSearchInput}
										class="w-full rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
									/>
								</div>
								<div>
									<label class="mb-0.5 block text-[10px] text-[var(--text-secondary)]">Before</label>
									<input
										type="date"
										bind:value={searchFilterBefore}
										onchange={handleSearchInput}
										class="w-full rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
									/>
								</div>
							</div>
						{/if}
						{#if searching}
							<p class="mt-2 text-xs text-[var(--text-secondary)]">Searching...</p>
						{:else if searchError}
							<p class="mt-2 text-xs text-[var(--danger)]">Search failed. Try again.</p>
						{:else if searchResults.length > 0}
							<div class="mt-2 max-h-60 space-y-1 overflow-y-auto">
								{#each searchResults as result (result.id)}
									<button
										onclick={() => jumpToSearchResult(result.id, result.channelId)}
										class="flex w-full flex-col rounded-lg px-3 py-2 text-left transition hover:bg-white/5"
									>
										<div class="flex items-baseline gap-2">
											<span class="text-xs font-semibold text-[var(--text-primary)]">{userStore.getDisplayName(result.senderId)}</span>
											{#if searchScope === 'global'}
												<span class="text-xs text-[var(--accent)]">#{channelStore.channels.find(c => c.id === result.channelId)?.name ?? 'unknown'}</span>
											{/if}
											<span class="text-xs text-[var(--text-secondary)]">{formatTime(result.createdAt)}</span>
										</div>
										<span class="truncate text-sm text-[var(--text-secondary)]">{@html highlightSearchMatch(result.content, searchQuery)}</span>
									</button>
								{/each}
							</div>
						{:else if searchQuery.trim()}
							<p class="mt-2 text-xs text-[var(--text-secondary)]">No results found.</p>
						{/if}
					</div>
				{/if}

				<!-- Pinned messages panel -->
				{#if showPinnedPanel}
					<div class="border-b border-white/10 bg-[var(--bg-secondary)] px-4 py-3" transition:slide={{ duration: 150 }}>
						<div class="flex items-center justify-between mb-2">
							<h3 class="text-sm font-semibold text-[var(--text-primary)]">
								Pinned Messages
								{#if channelStore.activeChannelId}
									<span class="ml-1 text-xs font-normal text-[var(--text-secondary)]">({messageStore.getPinnedCount(channelStore.activeChannelId)}/50)</span>
								{/if}
							</h3>
							<button aria-label="Close pinned messages" onclick={() => showPinnedPanel = false} class="text-[var(--text-secondary)] hover:text-[var(--text-primary)]">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
							</button>
						</div>
						{#if loadingPins}
							<p class="text-xs text-[var(--text-secondary)]">Loading...</p>
						{:else if loadingPinsError}
							<p class="text-xs text-[var(--danger)]">Failed to load pins. <button onclick={loadPinnedMessages} class="text-[var(--accent)] hover:underline">Retry</button></p>
						{:else if pinnedMessages.length > 0}
							<div class="max-h-60 space-y-2 overflow-y-auto">
								{#each pinnedMessages as pin (pin.id)}
									<div class="flex items-start gap-2 rounded-lg bg-white/5 px-3 py-2">
										<div class="min-w-0 flex-1">
											<div class="flex items-baseline gap-2">
												<span class="text-xs font-semibold text-[var(--text-primary)]">{getDisplayNameForContext(pin.sender_id ?? '')}</span>
												<span class="text-xs text-[var(--text-secondary)]">{formatTime(pin.created_at)}</span>
											</div>
											<p class="mt-0.5 text-sm text-[var(--text-secondary)] truncate" title={pin._decryptedContent ?? ''}>{pin._decryptedContent ?? '(encrypted message)'}</p>
										</div>
										{#if myRole === 'owner' || myRole === 'admin'}
											<button
												onclick={() => handleUnpinMessage(pin.id)}
												class="shrink-0 rounded p-1 text-[var(--text-secondary)] hover:text-[var(--danger)] transition"
												title="Unpin"
												aria-label="Unpin message"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
											</button>
										{/if}
									</div>
								{/each}
							</div>
						{:else}
							<p class="text-xs text-[var(--text-secondary)]">No pinned messages yet.</p>
						{/if}
					</div>
				{/if}

				<!-- Polls panel -->
				{#if showPollPanel}
					<div class="border-b border-white/10 bg-[var(--bg-secondary)] px-4 py-3" transition:slide={{ duration: 150 }}>
						<div class="flex items-center justify-between mb-2">
							<h3 class="text-sm font-semibold text-[var(--text-primary)]">
								Polls
								{#if polls.length > 0}
									<span class="ml-1 text-xs font-normal text-[var(--text-secondary)]">({polls.filter(p => !p.closed).length} active)</span>
								{/if}
							</h3>
							<div class="flex items-center gap-2">
								<button onclick={openCreatePoll} class="rounded bg-[var(--accent)] px-2 py-0.5 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)]">
									New Poll
								</button>
								<button aria-label="Close polls" onclick={() => showPollPanel = false} class="text-[var(--text-secondary)] hover:text-[var(--text-primary)]">
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
								</button>
							</div>
						</div>
						{#if loadingPolls}
							<p class="text-xs text-[var(--text-secondary)]">Loading...</p>
						{:else if polls.length > 0}
							<div class="max-h-80 space-y-3 overflow-y-auto">
								{#each polls as poll (poll.id)}
									{@const totalVotes = poll.votes.reduce((s, v) => s + v.count, 0)}
									{@const myUserId = authStore.user?.id ?? ''}
									{@const isExpired = poll.expires_at ? new Date(poll.expires_at) < new Date() : false}
									{@const isClosed = poll.closed || isExpired}
									<div class="rounded-lg bg-white/5 px-3 py-2.5">
										<div class="flex items-start justify-between gap-2">
											<div class="min-w-0 flex-1">
												<p class="text-sm font-medium text-[var(--text-primary)]">{poll.question}</p>
												<div class="mt-0.5 flex items-center gap-2 text-[10px] text-[var(--text-secondary)]">
													<span>by {getDisplayNameForContext(poll.created_by)}</span>
													{#if poll.multi_select}<span class="rounded bg-white/10 px-1">Multi-select</span>{/if}
													{#if poll.anonymous}<span class="rounded bg-white/10 px-1">Anonymous</span>{/if}
													{#if isClosed}<span class="rounded bg-red-500/20 px-1 text-red-400">{isExpired ? 'Expired' : 'Closed'}</span>{/if}
													{#if poll.expires_at && !isClosed}
														<span>Expires {new Date(poll.expires_at).toLocaleDateString()}</span>
													{/if}
												</div>
											</div>
											{#if !isClosed && (poll.created_by === myUserId || myRole === 'owner' || myRole === 'admin')}
												<button
													onclick={() => handleClosePoll(poll.id)}
													disabled={closingPollId === poll.id}
													class="shrink-0 rounded px-1.5 py-0.5 text-[10px] text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)] disabled:opacity-50"
													title="Close poll"
												>
													{closingPollId === poll.id ? 'Closing...' : 'Close'}
												</button>
											{/if}
										</div>
										<div class="mt-2 space-y-1.5">
											{#each poll.options as option, idx}
												{@const optVote = poll.votes[idx]}
												{@const count = optVote?.count ?? 0}
												{@const pct = totalVotes > 0 ? Math.round((count / totalVotes) * 100) : 0}
												{@const myVote = poll.anonymous ? (anonVotes[poll.id] ?? []).includes(idx) : optVote?.voter_ids.includes(myUserId)}
												<button
													onclick={() => handleVotePoll(poll.id, idx)}
													disabled={isClosed || votingPollKey !== null}
													class="group/opt relative flex w-full items-center gap-2 rounded-md border px-2.5 py-1.5 text-left text-sm transition {myVote ? 'border-[var(--accent)] bg-[var(--accent)]/10' : 'border-white/10 hover:border-white/20 hover:bg-white/5'} disabled:cursor-default disabled:opacity-70"
												>
													<div class="absolute inset-0 rounded-md bg-[var(--accent)]/10 transition-all" style="width: {pct}%"></div>
													<span class="relative flex-1 truncate text-[var(--text-primary)]" title={option}>{option}</span>
													<span class="relative text-xs font-medium text-[var(--text-secondary)]">
														{count} ({pct}%)
													</span>
													{#if myVote}
														<svg xmlns="http://www.w3.org/2000/svg" class="relative h-3.5 w-3.5 shrink-0 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
													{/if}
												</button>
											{/each}
										</div>
										<p class="mt-1.5 text-[10px] text-[var(--text-secondary)]">{totalVotes} vote{totalVotes !== 1 ? 's' : ''}</p>
									</div>
								{/each}
							</div>
						{:else}
							<div class="text-center py-4">
								<p class="text-xs text-[var(--text-secondary)]">No polls yet.</p>
								<button onclick={openCreatePoll} class="mt-2 rounded bg-[var(--accent)] px-3 py-1 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)]">Create one</button>
							</div>
						{/if}
					</div>
				{/if}

				<!-- Video grid (visible when in a call) -->
				<VideoGrid
					expanded={chatCollapsed}
					canKick={canKickFromVoice}
					channelVoiceBackground={voiceChannelBackground}
					onKickFromVoice={(userId) => {
						const channelId = voiceStore.activeCall?.channelId;
						if (channelId) handleVoiceKick(userId, channelId);
					}}
				/>

				{#if !chatCollapsed}
				<!-- Announcement banners -->
				{#each announcements as ann (ann.id)}
					<div class="flex items-start gap-3 border-b border-blue-500/20 bg-blue-500/10 px-4 py-2.5" transition:slide={{ duration: 150 }}>
						<svg xmlns="http://www.w3.org/2000/svg" class="mt-0.5 h-4 w-4 shrink-0 text-blue-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 17H2a3 3 0 0 0 3-3V9a7 7 0 0 1 14 0v5a3 3 0 0 0 3 3zm-8.27 4a2 2 0 0 1-3.46 0"/></svg>
						<div class="min-w-0 flex-1">
							<p class="text-sm font-semibold text-[var(--text-primary)]">{ann.title}</p>
							<p class="mt-0.5 text-xs text-[var(--text-secondary)]">{ann.body}</p>
						</div>
						<button
							onclick={() => handleDismissAnnouncement(ann.id)}
							class="shrink-0 rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]"
							title="Dismiss"
							aria-label="Dismiss"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
						</button>
					</div>
				{/each}
				<!-- Connection status banner -->
				{#if connectionStatus === 'reconnecting'}
					<div class="flex items-center justify-center gap-2 bg-amber-600/90 px-3 py-1.5 text-xs font-medium text-white">
						<svg class="h-3.5 w-3.5 animate-spin" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" opacity="0.3"/><path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round"/></svg>
						Reconnecting...
					</div>
				{:else if connectionStatus === 'connected'}
					<div class="flex items-center justify-center gap-2 bg-emerald-600/90 px-3 py-1.5 text-xs font-medium text-white">
						<svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
						Connected
					</div>
				{/if}

				<!-- Messages -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<div bind:this={messageListEl} role="log" aria-live="polite" aria-label="Messages" class="min-h-0 flex-1 overflow-y-auto" onscroll={handleMessageScroll} onclick={handleCodeCopyClick}>
				<div class="mx-auto max-w-5xl px-3 py-2 md:px-6 md:py-4">
					{#if loadingOlderError}
						<div class="mb-4 rounded-lg border border-[var(--danger)]/20 bg-[var(--danger)]/5 px-4 py-3 text-center">
							<p class="text-sm text-[var(--danger)]">Failed to load older messages</p>
							<button onclick={() => { loadingOlderError = false; handleMessageScroll(); }} class="mt-1 text-xs text-[var(--accent)] hover:underline">Retry</button>
						</div>
					{:else if loadingOlder}
						<Skeleton variant="message" count={3} />
					{/if}
					{#if initialized && messages.length === 0 && !loadingOlder}
						<div class="flex h-full items-center justify-center">
							<div class="text-center">
								<svg xmlns="http://www.w3.org/2000/svg" class="mx-auto mb-3 h-12 w-12 text-[var(--text-secondary)] opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
								<p class="text-sm text-[var(--text-primary)]">No messages yet</p>
								<p class="mt-1 text-xs text-[var(--text-secondary)] opacity-70">Be the first to say something!</p>
							</div>
						</div>
					{/if}
					{#each messages as msg, idx (msg.id)}
						{@const grouped = isGroupedMessage(messages, idx) && !shouldShowDateSeparator(messages, idx)}
						<!-- Unread separator -->
						{#if unreadSeparatorMsgId && msg.id === unreadSeparatorMsgId}
							<div class="my-3 flex items-center gap-4">
								<div class="flex-1 border-t border-[var(--danger)]"></div>
								<span class="text-xs font-semibold text-[var(--danger)]">NEW MESSAGES</span>
								<div class="flex-1 border-t border-[var(--danger)]"></div>
							</div>
						{/if}
						<!-- Date separator -->
						{#if shouldShowDateSeparator(messages, idx)}
							<div class="my-4 flex items-center gap-4">
								<div class="flex-1 border-t border-white/10"></div>
								<span class="text-xs font-medium text-[var(--text-secondary)]">{formatDateSeparator(msg.createdAt)}</span>
								<div class="flex-1 border-t border-white/10"></div>
							</div>
						{/if}
						<div
							id="msg-{msg.id}" class="chat-message chat-message-row group relative flex rounded-lg px-2 pl-3 transition hover:bg-white/[0.04] {msg.pending ? 'opacity-50' : ''} {msg.senderId === authStore.user?.id ? 'chat-message-own' : ''} {preferencesStore.preferences.messageDensity === 'compact' ? 'mb-0.5 gap-2 py-0.5' : grouped ? 'mb-0 gap-3 py-0.5' : 'mb-4 gap-3 py-1'}"
							style="border-left: 2px solid {getUserColor(msg.senderId)}; background: {getUserColor(msg.senderId).replace('65%)', '65% / 0.06)')};"
							oncontextmenu={(e) => showContextMenu(e, msg.id)}
							role="article"
							aria-label="Message from {getDisplayNameForContext(msg.senderId)}"
						>
							{#if preferencesStore.preferences.messageDensity !== 'compact'}
								{#if grouped}
									<!-- Hover timestamp in gutter for grouped messages -->
									<div class="flex w-10 shrink-0 items-start justify-center pt-0.5">
										<span class="hidden text-[10px] text-[var(--text-secondary)] opacity-0 group-hover:inline group-hover:opacity-100 transition-opacity" title={formatFullTimestamp(msg.createdAt)}>
											{formatTime(msg.createdAt)}
										</span>
									</div>
								{:else}
									<Avatar userId={msg.senderId} size="md" />
								{/if}
							{/if}
							<div class="min-w-0 flex-1">
								{#if !grouped}
									{#if msg.replyToId}
										{@const repliedMsg = messages.find(m => m.id === msg.replyToId)}
										<button
											onclick={() => msg.replyToId && highlightMessage(msg.replyToId)}
											class="mb-1 flex items-center gap-1.5 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 14 4 9 9 4" /><path d="M20 20v-7a4 4 0 0 0-4-4H4" /></svg>
											{#if repliedMsg}
												<span class="font-medium">{getDisplayNameForContext(repliedMsg.senderId)}</span>
												<span class="truncate max-w-[120px] sm:max-w-[200px] opacity-70">{isEncryptedMessage(repliedMsg.content) ? 'Encrypted message' : repliedMsg.content.slice(0, 60)}{!isEncryptedMessage(repliedMsg.content) && repliedMsg.content.length > 60 ? '...' : ''}</span>
											{:else}
												<span class="italic opacity-50">Original message not loaded</span>
											{/if}
										</button>
									{/if}
									<div class="flex items-baseline gap-2">
										<button
											class="text-sm font-semibold hover:underline cursor-pointer bg-transparent border-none p-0"
											style="color: {getUserColor(msg.senderId)}"
											onclick={(e) => { e.stopPropagation(); openProfileCard(msg.senderId, e); }}
										>
											{getDisplayNameForContext(msg.senderId)}
										</button>
										<span class="text-xs text-[var(--text-secondary)]" title={formatFullTimestamp(msg.createdAt)}>
											{formatTime(msg.createdAt)}
										</span>
										{#if msg.encryptionStatus === 'encrypted'}
											<svg class="inline-block h-3 w-3 text-green-500/60" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-label="End-to-end encrypted"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
										{:else if msg.encryptionStatus === 'decryption_failed'}
											<svg class="inline-block h-3 w-3 text-red-400/80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-label="Decryption failed"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><line x1="2" y1="2" x2="22" y2="22"/></svg>
										{/if}
										{#if msg.pending}
											<span class="text-xs text-[var(--text-secondary)] italic">sending...</span>
										{:else if msg.editedAt}
											<button
												class="text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:underline cursor-pointer bg-transparent border-none p-0"
												title="Edited {formatFullTimestamp(msg.editedAt)} — click to view history"
												onclick={(e) => { e.stopPropagation(); loadEditHistory(msg.channelId, msg.id); }}
											>(edited)</button>
										{/if}
										{#if channelStore.activeChannelId && messageStore.isPinned(channelStore.activeChannelId, msg.id)}
											<span title="Pinned"><svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-yellow-400" viewBox="0 0 24 24" fill="currentColor"><path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2z"/></svg></span>
										{/if}
									</div>
								{/if}

								{#if editingMessageId === msg.id}
									<!-- Edit mode -->
									<div class="mt-1">
										<textarea
											data-edit-input
											rows="2"
											bind:value={editInput}
											onkeydown={(e) => handleEditKeydown(e, msg.id)}
											class="w-full resize-y rounded border border-[var(--accent)] bg-[var(--bg-secondary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none"
										></textarea>
										<div class="mt-1 flex gap-2 text-xs">
											<button onclick={() => submitEdit(msg.id)} class="text-[var(--accent)] hover:underline">Save</button>
											<button onclick={cancelEdit} class="text-[var(--text-secondary)] hover:underline">Cancel</button>
											<span class="text-[var(--text-secondary)]">esc to cancel, enter to save, shift+enter for newline</span>
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
											<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
											<img
												src={blobUrl}
												alt={fileInfo.filename}
												class="max-h-80 max-w-[75vw] md:max-w-sm cursor-pointer rounded-lg border border-white/10 transition hover:brightness-90"
												onclick={() => openLightbox(blobUrl, fileInfo.filename)}
												onkeydown={(e) => { if (e.key === 'Enter') openLightbox(blobUrl, fileInfo.filename); }}
											/>
											{:catch}
											<div class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2">
												<span class="text-sm text-[var(--text-secondary)]">Failed to load image</span>
											</div>
											{/await}
											<div class="mt-1 flex items-center gap-2 text-xs text-[var(--text-secondary)]">
												<span class="truncate max-w-[200px]" title={fileInfo.filename}>{fileInfo.filename}</span>
												<span class="shrink-0">({formatFileSize(fileInfo.size)})</span>
												{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
													<a href={blobUrl} download={fileInfo.filename} class="text-[var(--accent)] hover:underline">Download</a>
												{:catch}
													<span class="text-xs text-[var(--text-secondary)]">Download unavailable</span>
												{/await}
											</div>
										</div>
									{:else if fileInfo && VIDEO_EXTS.test(fileInfo.filename)}
										<div class="mt-1">
											{#await getAuthenticatedBlobUrl(fileInfo.file_id)}
											<div class="flex h-48 w-full max-w-xs items-center justify-center rounded-lg border border-white/10 bg-[var(--bg-secondary)]">
												<span class="text-xs text-[var(--text-secondary)]">Loading video...</span>
											</div>
											{:then blobUrl}
											<!-- svelte-ignore a11y_media_has_caption -->
											<video src={blobUrl} controls class="max-h-96 max-w-[85vw] md:max-w-lg rounded-lg border border-white/10"></video>
											{:catch}
											<div class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2">
												<span class="text-sm text-[var(--text-secondary)]">Failed to load video</span>
											</div>
											{/await}
											<div class="mt-1 flex items-center gap-2 text-xs text-[var(--text-secondary)]">
												<span class="truncate max-w-[200px]" title={fileInfo.filename}>{fileInfo.filename}</span>
												<span class="shrink-0">({formatFileSize(fileInfo.size)})</span>
												{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
													<a href={blobUrl} download={fileInfo.filename} class="text-[var(--accent)] hover:underline">Download</a>
												{:catch}
													<span class="text-xs text-[var(--text-secondary)]">Download unavailable</span>
												{/await}
											</div>
										</div>
									{:else if fileInfo && AUDIO_EXTS.test(fileInfo.filename)}
										<div class="mt-1 max-w-[85vw] md:max-w-sm">
											<div class="flex items-center gap-3 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2.5">
												<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 shrink-0 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
													<path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" />
												</svg>
												<div class="min-w-0 flex-1">
													<p class="truncate text-sm font-medium text-[var(--text-primary)]" title={fileInfo.filename}>{fileInfo.filename}</p>
													<p class="text-xs text-[var(--text-secondary)]">{formatFileSize(fileInfo.size)}</p>
												</div>
												{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
													<a href={blobUrl} download={fileInfo.filename} class="shrink-0 text-xs text-[var(--accent)] hover:underline">Download</a>
												{:catch}
													<span class="shrink-0 text-xs text-[var(--text-secondary)]">Download unavailable</span>
												{/await}
											</div>
											{#await getAuthenticatedBlobUrl(fileInfo.file_id)}
												<div class="mt-1 h-8 rounded bg-[var(--bg-secondary)]"></div>
											{:then blobUrl}
												<audio src={blobUrl} controls class="mt-1 w-full rounded" style="height: 32px;"></audio>
											{:catch}
												<p class="mt-1 text-xs text-[var(--text-secondary)]">Could not load audio</p>
											{/await}
										</div>
									{:else if fileInfo}
										<div class="mt-1 inline-flex max-w-full items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2">
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
												<polyline points="14 2 14 8 20 8" />
											</svg>
											<span class="truncate text-sm text-[var(--text-primary)]" title={fileInfo.filename}>{fileInfo.filename}</span>
											<span class="shrink-0 text-xs text-[var(--text-secondary)]">({formatFileSize(fileInfo.size)})</span>
											{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
												<a href={blobUrl} download={fileInfo.filename} class="shrink-0 text-xs text-[var(--accent)] hover:underline">Download</a>
											{:catch}
												<span class="shrink-0 text-xs text-[var(--text-secondary)]">Download unavailable</span>
											{/await}
										</div>
									{:else}
										{#if isEncryptedMessage(msg.content)}
										<div class="mt-1 inline-flex items-center gap-2 rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-2">
											<svg class="h-4 w-4 shrink-0 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
											<span class="text-sm italic text-amber-300/80">Encrypted file (E2E decryption not available)</span>
										</div>
										{:else}
										<div class="mt-1 inline-flex max-w-full items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2">
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
												<polyline points="14 2 14 8 20 8" />
											</svg>
											<span class="truncate text-sm text-[var(--text-primary)]" title={msg.content}>{msg.content}</span>
										</div>
										{/if}
									{/if}
								{:else}
									{#if msg.encryptionStatus === 'decryption_failed'}
										<div class="mt-1 inline-flex items-center gap-2 rounded-lg border border-red-500/20 bg-red-500/10 px-3 py-2">
											<svg class="h-4 w-4 shrink-0 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><line x1="2" y1="2" x2="22" y2="22"/></svg>
											<span class="text-sm text-red-300">Unable to decrypt this message</span>
										</div>
									{:else}
									{@const imageUrls = extractImageUrls(msg.content)}
									{@const linkUrls = extractNonImageUrls(msg.content)}
									<div class="markdown-content mt-0.5 text-sm text-[var(--text-primary)] leading-relaxed">{@html renderMarkdown(msg.content)}</div>
									{#if imageUrls.length > 0}
										<div class="mt-2 flex flex-col gap-2">
											{#each imageUrls as imgUrl}
												<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
												<img
													src={imgUrl}
													alt="Linked content"
													crossorigin="anonymous"
													class="max-h-80 max-w-[75vw] md:max-w-sm cursor-pointer rounded-lg border border-white/10 transition hover:brightness-90"
													loading="lazy"
													onclick={() => openLightbox(imgUrl, 'Image')}
													onkeydown={(e) => { if (e.key === 'Enter') openLightbox(imgUrl, 'Image'); }}
													onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
												/>
											{/each}
										</div>
									{/if}
									{#if linkUrls.length > 0 && preferencesStore.preferences.showLinkPreviews}
										{#each linkUrls.slice(0, 3) as linkUrl}
											{#await fetchLinkPreview(linkUrl) then preview}
												{#if preview && (preview.title || preview.description)}
													<a href={linkUrl} target="_blank" rel="noopener noreferrer" class="link-embed mt-2 block max-w-[85vw] md:max-w-md rounded-lg border-l-4 border-[var(--accent)] bg-[var(--bg-secondary)] p-3 transition hover:bg-white/5">
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
															<img src={preview.image} alt="" class="mt-2 max-h-40 max-w-full rounded border border-white/10" loading="lazy" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
														{/if}
													</a>
												{/if}
											{/await}
										{/each}
									{/if}
									{/if}
								{/if}

								<!-- Reactions -->
								{#if msg.reactions && msg.reactions.size > 0}
									<div class="mt-1.5 flex flex-wrap gap-1">
										{#each Array.from(msg.reactions.entries()) as [emoji, users]}
											{@const hasReacted = users.has(authStore.user?.id ?? '')}
											{@const reacterNames = Array.from(users).map(uid => uid === authStore.user?.id ? 'You' : getDisplayNameForContext(uid)).join(', ')}
											<button
												onclick={() => toggleReaction(msg.id, emoji)}
												class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs transition {hasReacted ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 text-[var(--text-secondary)] hover:border-white/20 hover:bg-white/5'}"
												title="{reacterNames} reacted with {emoji}"
											>
												<span>{emoji}</span>
												<span class="font-medium">{users.size}</span>
											</button>
										{/each}
									</div>
								{/if}

								<!-- Thread reply badge -->
								{#if (msg.threadReplyCount ?? 0) > 0}
									<button
										onclick={() => openThread(msg.id)}
										class="mt-1.5 inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium text-[var(--accent)] hover:bg-[var(--accent)]/10 transition cursor-pointer bg-transparent border-none"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
										<span>{msg.threadReplyCount} {msg.threadReplyCount === 1 ? 'reply' : 'replies'}</span>
										{#if msg.threadLastReplyAt}
											<span class="text-[var(--text-secondary)]">— Last reply {formatRelativeTime(msg.threadLastReplyAt)}</span>
										{/if}
									</button>
								{/if}

								<!-- Read receipts -->
								{#if channelStore.activeChannelId && isReadReceiptPoint(messages, idx)}
									{#if activeChannel?.channel_type === 'dm' && msg.senderId === authStore.user?.id}
										{@const otherUserId = getPeerUserIdForDm(channelStore.activeChannelId)}
										{@const otherLastRead = otherUserId ? readReceiptStore.getLastReadMessageId(channelStore.activeChannelId, otherUserId) : null}
										{@const otherReadTimestamp = otherUserId ? readReceiptStore.getLastReadTimestamp(channelStore.activeChannelId, otherUserId) : null}
										{@const msgIdx = messages.findIndex(m => m.id === msg.id)}
										{@const readIdx = otherLastRead ? messages.findIndex(m => m.id === otherLastRead) : -1}
										{#if readIdx >= msgIdx && otherReadTimestamp}
											<div class="mt-1 flex items-center gap-1 text-[10px] text-[var(--text-secondary)]">
												<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
													<polyline points="20 6 9 17 4 12"/>
												</svg>
												<span>Read {formatRelativeTime(otherReadTimestamp)}</span>
											</div>
										{/if}
									{:else if activeChannel?.channel_type !== 'dm'}
										{@const readers = readReceiptStore.getReadersAtMessage(channelStore.activeChannelId, msg.id, messages, authStore.user?.id ?? '')}
										{#if readers.length > 0}
											<div class="mt-1 flex items-center gap-0.5" title="{readers.map(uid => getDisplayNameForContext(uid)).join(', ')}">
												<div class="flex -space-x-1.5">
													{#each readers.slice(0, 5) as readerId (readerId)}
														<Avatar userId={readerId} size="xs" />
													{/each}
												</div>
												{#if readers.length > 5}
													<span class="ml-1 text-[10px] text-[var(--text-secondary)]">+{readers.length - 5}</span>
												{/if}
											</div>
										{/if}
									{/if}
								{/if}
							</div>

							<!-- Action buttons (visible on hover) -->
							{#if !msg.pending}
								<div class="absolute -top-3 right-2 z-10 hidden items-center gap-0.5 rounded-lg border border-white/10 bg-[var(--bg-secondary)] shadow-lg group-hover:flex">
									<button
										onclick={(e) => { e.stopPropagation(); reactionPickerMessageId = reactionPickerMessageId === msg.id ? null : msg.id; }}
										class="p-1.5 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
										title="Add reaction"
											aria-label="Add reaction"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<circle cx="12" cy="12" r="10" /><path d="M8 14s1.5 2 4 2 4-2 4-2" /><line x1="9" y1="9" x2="9.01" y2="9" /><line x1="15" y1="9" x2="15.01" y2="9" />
										</svg>
									</button>
									<button
										onclick={() => startReply(msg)}
										class="p-1.5 text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
										title="Reply"
											aria-label="Reply"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 14 4 9 9 4" /><path d="M20 20v-7a4 4 0 0 0-4-4H4" /></svg>
									</button>
									<button
										onclick={() => openThread(msg.threadId ?? msg.id)}
										class="p-1.5 text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
										title="Reply in Thread"
										aria-label="Reply in Thread"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
									</button>
									<button
										onclick={() => forwardMessage(msg)}
										class="p-1.5 text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
										title="Forward"
											aria-label="Forward"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 14 20 9 15 4" /><path d="M4 20v-7a4 4 0 0 1 4-4h12" /></svg>
									</button>
									{#if (myRole === 'owner' || myRole === 'admin') && channelStore.activeChannelId}
										{#if messageStore.isPinned(channelStore.activeChannelId, msg.id)}
											<button
												onclick={() => handleUnpinMessage(msg.id)}
												class="p-1.5 text-yellow-400 transition hover:text-yellow-300"
												title="Unpin message"
												aria-label="Unpin message"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="1">
													<path d="M16 2L10 8l-4-2-4 4 7 7 4-4-2-4 6-6z" />
												</svg>
											</button>
										{:else}
											<button
												onclick={() => handlePinMessage(msg.id)}
												class="p-1.5 text-[var(--text-secondary)] transition hover:text-yellow-400"
												title="Pin message"
												aria-label="Pin message"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
													<path d="M16 2L10 8l-4-2-4 4 7 7 4-4-2-4 6-6z" />
												</svg>
											</button>
										{/if}
									{/if}
									{#if msg.senderId === authStore.user?.id}
										<button
											onclick={() => startEditMessage(msg)}
											class="p-1.5 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
											title="Edit"
											aria-label="Edit message"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" /><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
											</svg>
										</button>
										<button
											onclick={() => handleDeleteMessage(msg.id)}
											class="p-1.5 text-[var(--text-secondary)] transition hover:text-[var(--danger)]"
											title="Delete"
											aria-label="Delete message"
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
											aria-label="Delete message (moderator)"
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
									class="absolute right-2 top-8 z-10 flex items-center gap-1 rounded-xl bg-[var(--bg-secondary)] p-2 shadow-lg"
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
											aria-label="More emojis"
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
						<div class="flex h-full flex-col items-center justify-center gap-3">
							<div class="rounded-full bg-white/5 p-4">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
									<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
								</svg>
							</div>
							<h3 class="text-lg font-semibold text-[var(--text-primary)]">
								{#if activeChannel?.channel_type === 'dm'}
									Start a conversation
								{:else}
									Welcome to #{getChannelDisplayName()}
								{/if}
							</h3>
							<p class="max-w-sm text-center text-sm text-[var(--text-secondary)]">
								{#if activeChannel?.channel_type === 'dm'}
									This is the beginning of your direct message history.
								{:else}
									This is the start of the channel. Send a message, share a file, or use /shrug for fun.
								{/if}
							</p>
						</div>
					{/if}
				</div>
				</div>

				<!-- Context menu -->
				{#if contextMenuMessageId}
					{@const ctxMsg = messages.find(m => m.id === contextMenuMessageId)}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="fixed inset-0 z-40"
						onclick={() => contextMenuMessageId = null}
						oncontextmenu={(e) => { e.preventDefault(); contextMenuMessageId = null; }}
						onkeydown={(e) => { if (e.key === 'Escape') contextMenuMessageId = null; }}
					></div>
					<div
						role="menu"
						aria-label="Message actions"
						class="fixed z-50 min-w-[180px] max-w-[calc(100vw-16px)] rounded-xl bg-[var(--bg-secondary)] py-1 shadow-lg"
						style="left: {contextMenuPos.x}px; top: {contextMenuPos.y}px;"
						transition:scale={{ start: 0.9, duration: 100 }}
						onclick={(e) => e.stopPropagation()}
						oncontextmenu={(e) => e.stopPropagation()}
					>
						{#if ctxMsg}
							<button
								onclick={() => { if (ctxMsg) startReply(ctxMsg); contextMenuMessageId = null; }}
								role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--text-primary)] hover:bg-white/5"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 14 4 9 9 4" /><path d="M20 20v-7a4 4 0 0 0-4-4H4" /></svg>
								Reply
							</button>
							<button
								onclick={() => { if (ctxMsg) { openThread(ctxMsg.threadId ?? ctxMsg.id); contextMenuMessageId = null; } }}
								role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--text-primary)] hover:bg-white/5"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /><line x1="9" y1="10" x2="15" y2="10" /></svg>
								Reply in Thread
							</button>
							<button
								onclick={() => { if (ctxMsg) forwardMessage(ctxMsg); }}
								role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--text-primary)] hover:bg-white/5"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 14 20 9 15 4" /><path d="M4 20v-7a4 4 0 0 1 4-4h12" /></svg>
								Forward
							</button>
							<button
								onclick={() => copyMessageText(contextMenuMessageId!)}
								role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--text-primary)] hover:bg-white/5"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" /></svg>
								Copy Text
							</button>
							<button
								onclick={() => { copyToClipboard(`${window.location.origin}/channels#msg-${ctxMsg.id}`, 'Message link copied'); contextMenuMessageId = null; }}
								role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--text-primary)] hover:bg-white/5"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" /></svg>
								Copy Link
							</button>
							<button
								onclick={() => toggleBookmark(ctxMsg.id)}
								role="menuitem"
								class="flex w-full items-center gap-2 px-3 py-1.5 text-sm {bookmarkStore.isBookmarked(ctxMsg.id) ? 'text-yellow-400' : 'text-[var(--text-primary)]'} hover:bg-white/5"
							>
								{#if bookmarkStore.isBookmarked(ctxMsg.id)}
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor"><path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/></svg>
									Remove Bookmark
								{:else}
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/></svg>
									Bookmark
								{/if}
							</button>
							{#if (myRole === 'owner' || myRole === 'admin') && channelStore.activeChannelId}
								<div class="my-1 border-t border-white/10"></div>
								{#if messageStore.isPinned(channelStore.activeChannelId, ctxMsg.id)}
									<button
										onclick={() => { handleUnpinMessage(ctxMsg.id); contextMenuMessageId = null; }}
										role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-yellow-400 hover:bg-white/5"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor"><path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2z"/></svg>
										Unpin
									</button>
								{:else}
									<button
										onclick={() => { handlePinMessage(ctxMsg.id); contextMenuMessageId = null; }}
										role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--text-primary)] hover:bg-white/5"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2z"/></svg>
										Pin
									</button>
								{/if}
							{/if}
							{#if ctxMsg.senderId === authStore.user?.id}
								<div class="my-1 border-t border-white/10"></div>
								{#if ctxMsg.messageType !== 'file'}
									<button
										onclick={() => { startEditMessage(ctxMsg); contextMenuMessageId = null; }}
										role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--text-primary)] hover:bg-white/5"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" /><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" /></svg>
										Edit
									</button>
								{/if}
								<button
									onclick={() => { handleDeleteMessage(ctxMsg.id); contextMenuMessageId = null; }}
									role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--danger)] hover:bg-white/5"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /></svg>
									Delete
								</button>
							{:else if myRole === 'owner' || myRole === 'admin'}
								<div class="my-1 border-t border-white/10"></div>
								<button
									onclick={() => { handleDeleteMessage(ctxMsg.id); contextMenuMessageId = null; }}
									role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-[var(--danger)] hover:bg-white/5"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /></svg>
									Delete (mod)
								</button>
							{/if}
							{#if ctxMsg.senderId !== authStore.user?.id}
								<div class="my-1 border-t border-white/10"></div>
								<button
									onclick={() => { reportingMessageId = ctxMsg.id; contextMenuMessageId = null; }}
									role="menuitem" class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-orange-400 hover:bg-white/5"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z" /><line x1="4" y1="22" x2="4" y2="15" /></svg>
									Report
								</button>
							{/if}
						{/if}
					</div>
				{/if}

				<!-- Scroll to bottom button -->
				{#if showScrollBottom}
					<div class="relative">
						<button
							onclick={() => scrollToBottom(true)}
							class="absolute bottom-2 left-1/2 z-10 -translate-x-1/2 rounded-full border border-white/10 bg-[var(--bg-secondary)] p-2 shadow-lg transition hover:bg-[var(--bg-tertiary)]"
							title="Scroll to bottom"
							aria-label="Scroll to bottom"
							transition:scale={{ start: 0.8, duration: 150 }}
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<polyline points="6 9 12 15 18 9" />
							</svg>
						</button>
					</div>
				{/if}

				<!-- Typing indicator -->
				{#if typingUsers.length > 0}
					<div class="flex items-center gap-1.5 px-6 py-1 text-xs text-[var(--text-secondary)]">
						<span class="flex gap-0.5">
							<span class="inline-block h-1.5 w-1.5 animate-bounce rounded-full bg-[var(--text-secondary)]" style="animation-delay: 0ms"></span>
							<span class="inline-block h-1.5 w-1.5 animate-bounce rounded-full bg-[var(--text-secondary)]" style="animation-delay: 150ms"></span>
							<span class="inline-block h-1.5 w-1.5 animate-bounce rounded-full bg-[var(--text-secondary)]" style="animation-delay: 300ms"></span>
						</span>
						<span>
							{#if typingUsers.length === 1}
								<strong class="font-semibold text-[var(--text-primary)]">{getDisplayNameForContext(typingUsers[0])}</strong> is typing...
							{:else if typingUsers.length === 2}
								<strong class="font-semibold text-[var(--text-primary)]">{getDisplayNameForContext(typingUsers[0])}</strong> and <strong class="font-semibold text-[var(--text-primary)]">{getDisplayNameForContext(typingUsers[1])}</strong> are typing...
							{:else if typingUsers.length === 3}
								<strong class="font-semibold text-[var(--text-primary)]">{getDisplayNameForContext(typingUsers[0])}</strong>, <strong class="font-semibold text-[var(--text-primary)]">{getDisplayNameForContext(typingUsers[1])}</strong>, and <strong class="font-semibold text-[var(--text-primary)]">{getDisplayNameForContext(typingUsers[2])}</strong> are typing...
							{:else}
								<strong class="font-semibold text-[var(--text-primary)]">{getDisplayNameForContext(typingUsers[0])}</strong>, <strong class="font-semibold text-[var(--text-primary)]">{getDisplayNameForContext(typingUsers[1])}</strong>, and {typingUsers.length - 2} others are typing...
							{/if}
						</span>
					</div>
				{/if}

				<!-- Reply banner -->
				{#if replyingTo}
					<div class="flex items-center gap-2 border-t border-white/10 bg-[var(--bg-secondary)] px-3 py-2 md:px-4">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 14 4 9 9 4" /><path d="M20 20v-7a4 4 0 0 0-4-4H4" /></svg>
						<span class="hidden sm:inline text-xs text-[var(--text-secondary)]">Replying to</span>
						<span class="text-xs font-medium text-[var(--text-primary)]">{userStore.getDisplayName(replyingTo.senderId)}</span>
						<span class="flex-1 truncate text-xs text-[var(--text-secondary)]" title={replyingTo.content}>{replyingTo.content.slice(0, 60)}</span>
						<button onclick={cancelReply} class="shrink-0 rounded p-0.5 text-[var(--text-secondary)] hover:text-[var(--text-primary)]" title="Cancel reply" aria-label="Cancel reply">
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
						</button>
					</div>
				{/if}

				<!-- Read-only notice -->
				{#if isReadOnlyForMe}
					<div class="border-t border-white/10 bg-[var(--bg-primary)] px-4 py-3 text-center">
						<span class="inline-flex items-center gap-1.5 text-sm text-[var(--text-secondary)]">
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-yellow-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
							This channel is read-only
						</span>
					</div>
				{/if}
				<!-- Identity key change warning -->
				{#if activeChannel?.channel_type === 'dm' && getPeerUserIdForDm(activeChannel.id) && encryptionStore.hasKeyChanged(getPeerUserIdForDm(activeChannel.id)!)}
					{@const peerIdForBanner = getPeerUserIdForDm(activeChannel.id)!}
					<div class="border-t border-yellow-500/30 bg-yellow-500/10 px-4 py-2.5" transition:slide={{ duration: 200 }}>
						<div class="flex items-center justify-between gap-2">
							<div class="flex items-center gap-2 text-sm text-yellow-300">
								<svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
								<span>Safety number has changed for this contact. They may have re-registered or switched devices.</span>
							</div>
							<div class="flex shrink-0 items-center gap-2">
								<button
									onclick={() => { encryptionStore.acknowledgeKeyChange(peerIdForBanner); }}
									class="rounded-lg bg-yellow-500/20 px-3 py-1 text-xs font-medium text-yellow-300 transition hover:bg-yellow-500/30"
								>
									Acknowledge
								</button>
								<button
									onclick={() => { encryptionStore.acknowledgeKeyChange(peerIdForBanner); loadEncryptionInfo(); }}
									class="rounded-lg bg-[var(--accent)] px-3 py-1 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)]"
								>
									Verify
								</button>
							</div>
						</div>
					</div>
				{/if}

				<!-- Slow mode cooldown -->
				{#if slowModeCooldown > 0 && !isReadOnlyForMe}
					<div class="border-t border-white/10 bg-blue-500/10 px-4 py-1.5 text-center">
						<span class="inline-flex items-center gap-1.5 text-xs text-blue-400">
							<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
							Slow mode — wait {slowModeCooldown}s
						</span>
					</div>
				{/if}
				<!-- Message input -->
				<form onsubmit={sendMessage} class="{replyingTo ? '' : 'border-t border-white/10'} relative mx-auto max-w-5xl bg-[var(--bg-primary)] {isReadOnlyForMe ? 'hidden' : ''} px-2 py-2 md:p-4">
					<!-- Emoji autocomplete popup -->
					{#if showEmojiPopup && emojiResults.length > 0}
						<div class="absolute bottom-full left-4 right-4 mb-1 rounded-lg border border-white/10 bg-[var(--bg-secondary)] shadow-lg overflow-hidden z-10">
							{#each emojiResults as entry, i (entry.name)}
								<button
									onclick={() => selectEmoji(entry.custom ? entry.name : entry.emoji, entry.custom)}
									class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-sm transition {i === emojiIndex ? 'bg-[var(--accent)]/20 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
								>
									{#if entry.custom && entry.url}
										<img src={entry.url} alt={entry.name} class="h-5 w-5 object-contain" />
									{:else}
										<span class="text-lg">{entry.emoji}</span>
									{/if}
									<span class="font-medium">:{entry.name}:</span>
								</button>
							{/each}
						</div>
					{/if}
					<!-- Mention autocomplete popup -->
					{#if showMentionPopup && mentionResults.length > 0}
						<div class="absolute bottom-full left-4 right-4 mb-1 rounded-lg border border-white/10 bg-[var(--bg-secondary)] shadow-lg overflow-hidden">
							{#each mentionResults as member, i (member.user_id)}
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
					<!-- GIF picker panel -->
					{#if showGifPicker}
						<div class="absolute bottom-full left-1 md:left-4 z-20 mb-1 w-80 max-w-[calc(100vw-0.5rem)] max-h-[360px] rounded-2xl bg-[var(--bg-secondary)] shadow-xl overflow-hidden flex flex-col" transition:scale={{ start: 0.95, duration: 150 }}>
							<div class="flex items-center gap-2 border-b border-white/10 px-3 py-2">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg>
								<input
									type="text"
									value={gifSearchQuery}
									oninput={(e) => handleGifSearch((e.target as HTMLInputElement).value)}
									placeholder="Search for GIFs..."
									class="flex-1 bg-transparent text-sm text-[var(--text-primary)] outline-none placeholder:text-[var(--text-secondary)]/50"
								/>
								<button type="button" aria-label="Close GIF picker" onclick={() => { showGifPicker = false; }} class="p-1 text-[var(--text-secondary)] hover:text-[var(--text-primary)]">
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
								</button>
							</div>
							<div class="flex-1 overflow-y-auto p-2">
								{#if gifLoading}
									<div class="flex items-center justify-center py-8">
										<div class="h-6 w-6 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
									</div>
								{:else if gifError}
									<div class="flex flex-col items-center justify-center py-8 text-[var(--text-secondary)]">
										<p class="text-sm text-[var(--danger)]">Failed to load GIFs</p>
										<button onclick={() => { gifError = false; gifSearchQuery ? handleGifSearch(gifSearchQuery) : loadTrendingGifs(); }} class="mt-1 text-xs text-[var(--accent)] hover:underline">Retry</button>
									</div>
								{:else if gifResults.length === 0}
									<div class="flex flex-col items-center justify-center py-8 text-[var(--text-secondary)]">
										<svg xmlns="http://www.w3.org/2000/svg" class="mb-2 h-8 w-8 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="20" height="20" rx="5" /><text x="12" y="16" text-anchor="middle" font-size="8" fill="currentColor" stroke="none" font-weight="bold">GIF</text></svg>
										<p class="text-sm">{gifSearchQuery ? 'No GIFs found' : 'Search for a GIF'}</p>
									</div>
								{:else}
									<div style="columns: 2; column-gap: 0.375rem;">
										{#each gifResults as gif (gif.id)}
											<button
												type="button"
												onclick={() => selectGif(gif)}
												class="group/gif mb-1.5 block w-full overflow-hidden rounded-lg bg-black/20 transition hover:ring-2 hover:ring-[var(--accent)]"
												style="break-inside: avoid;"
												title={gif.title}
											>
												<img
													src={gif.preview_url}
													alt={gif.title}
													class="w-full rounded-lg"
													style="aspect-ratio: {gif.width}/{gif.height};"
													loading="lazy"
												/>
											</button>
										{/each}
									</div>
								{/if}
							</div>
							<div class="border-t border-white/10 px-3 py-1.5 text-right">
								<span class="text-[10px] text-[var(--text-secondary)]/50">Powered by GIPHY</span>
							</div>
						</div>
					{/if}
					<div class="flex gap-1.5 md:gap-2">
						<!-- File upload button -->
						<button
							type="button"
							onclick={() => fileInputEl?.click()}
							disabled={uploading}
							class="shrink-0 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-2 py-2 md:px-3 md:py-2.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] disabled:cursor-not-allowed disabled:opacity-30"
							title={uploading ? 'Uploading...' : 'Upload file'}
							aria-label={uploading ? 'Uploading file' : 'Upload file'}
						>
							{#if uploading}
								<svg class="h-5 w-5 animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path></svg>
							{:else}
								<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48" />
								</svg>
							{/if}
						</button>
						<!-- GIF button -->
						<button
							type="button"
							onclick={toggleGifPicker}
							class="hidden sm:block shrink-0 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] {showGifPicker ? 'border-[var(--accent)] text-[var(--accent)]' : ''}"
							title="GIF"
							aria-label="Insert GIF"
						>
							<span class="text-xs font-bold">GIF</span>
						</button>
						<!-- Poll button -->
						{#if activeChannel?.channel_type !== 'dm'}
							<button
								type="button"
								onclick={openCreatePoll}
								class="hidden sm:block shrink-0 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-2 py-2.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
								title="Create poll"
								aria-label="Create poll"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M7 16V12"/><path d="M12 16V8"/><path d="M17 16v-5"/>
								</svg>
							</button>
						{/if}
						<input
							bind:this={fileInputEl}
							type="file"
							onchange={() => handleFileUpload()}
							class="hidden"
						/>
						<textarea
							bind:this={messageInputEl}
							bind:value={messageInput}
							oninput={(e) => { handleMentionInput(); autoResizeTextarea(); }}
							onkeydown={(e) => { handleMentionKeydown(e); if (!showMentionPopup) handleInputKeydown(e); }}
							onpaste={handlePaste}
							maxlength={4000}
							placeholder="Message {activeChannel.channel_type === 'dm' ? '@' : '#'}{getChannelDisplayName()}..."
							rows="1"
							class="flex-1 resize-none rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2 md:px-4 md:py-2.5 text-sm md:text-base text-[var(--text-primary)] outline-none transition placeholder:text-[var(--text-secondary)]/50 focus:border-[var(--accent)]"
						></textarea>
						<button
							type="submit"
							disabled={!messageInput.trim()}
							class="shrink-0 rounded-lg bg-[var(--accent)] px-3 py-2 md:px-4 md:py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-30"
						>
							<span class="hidden sm:inline">Send</span>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 sm:hidden" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="22" y1="2" x2="11" y2="13" /><polygon points="22 2 15 22 11 13 2 9 22 2" /></svg>
						</button>
						<!-- Schedule message button -->
						<div class="relative">
							<button
								type="button"
								onclick={() => { showSchedulePicker = !showSchedulePicker; if (showSchedulePicker && !scheduleDate) { const now = new Date(); now.setHours(now.getHours() + 1, 0, 0, 0); scheduleDate = now.toISOString().slice(0, 10); scheduleTime = now.toTimeString().slice(0, 5); } }}
								disabled={!messageInput.trim()}
								class="hidden sm:block shrink-0 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-2 py-2.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] disabled:cursor-not-allowed disabled:opacity-30 {showSchedulePicker ? 'border-[var(--accent)] text-[var(--accent)]' : ''}"
								title="Schedule message"
								aria-label="Schedule message"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
								</svg>
							</button>
							{#if showSchedulePicker}
								<div class="absolute bottom-full right-0 mb-2 w-64 rounded-2xl bg-[var(--bg-secondary)] p-4 shadow-xl" transition:scale={{ start: 0.95, duration: 150 }}>
									<h4 class="mb-3 text-sm font-semibold text-[var(--text-primary)]">Schedule Message</h4>
									<div class="space-y-2">
										<div>
											<label for="schedule-date" class="mb-1 block text-xs text-[var(--text-secondary)]">Date</label>
											<input id="schedule-date" type="date" bind:value={scheduleDate} min={new Date().toISOString().slice(0, 10)} class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-2.5 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
										</div>
										<div>
											<label for="schedule-time" class="mb-1 block text-xs text-[var(--text-secondary)]">Time</label>
											<input id="schedule-time" type="time" bind:value={scheduleTime} class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-2.5 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
										</div>
									</div>
									<div class="mt-3 flex gap-2">
										<button type="button" onclick={() => showSchedulePicker = false} class="flex-1 rounded-lg border border-white/10 px-3 py-1.5 text-xs text-[var(--text-secondary)] transition hover:bg-white/5">Cancel</button>
										<button type="button" onclick={handleScheduleMessage} disabled={!scheduleDate || !scheduleTime} class="flex-1 rounded-lg bg-[var(--accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50">Schedule</button>
									</div>
								</div>
							{/if}
						</div>
						{#if voiceStore.isInCall}
							<button
								type="button"
								onclick={() => (chatCollapsed = !chatCollapsed)}
								class="rounded-lg border border-white/10 bg-transparent px-2 py-2.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
								title="Hide chat"
								aria-label="Hide chat"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="18 15 12 9 6 15" /></svg>
							</button>
						{/if}
					</div>
					{#if preferencesStore.preferences.showFormattingToolbar}
					<div class="mt-1 hidden sm:flex items-center gap-1">
						<div class="flex items-center gap-0.5">
							<button type="button" onclick={() => wrapSelection('**', '**')} class="rounded px-1.5 py-0.5 text-xs font-bold text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)] focus:bg-white/10 focus:outline-none" title="Bold ({modKey}+B)">B</button>
							<button type="button" onclick={() => wrapSelection('*', '*')} class="rounded px-1.5 py-0.5 text-xs italic text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)] focus:bg-white/10 focus:outline-none" title="Italic ({modKey}+I)">I</button>
							<button type="button" onclick={() => wrapSelection('~~', '~~')} class="rounded px-1.5 py-0.5 text-xs line-through text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)] focus:bg-white/10 focus:outline-none" title="Strikethrough">S</button>
							<button type="button" onclick={() => wrapSelection('`', '`')} class="rounded px-1.5 py-0.5 text-xs font-mono text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)] focus:bg-white/10 focus:outline-none" title="Code ({modKey}+E)">&lt;&gt;</button>
							<button type="button" onclick={() => wrapSelection('[', '](url)')} class="rounded px-1.5 py-0.5 text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)] focus:bg-white/10 focus:outline-none" title="Link" aria-label="Insert link">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" /></svg>
							</button>
						</div>
						<span class="mx-1 h-3 w-px bg-white/10"></span>
						<div class="flex items-center gap-2 text-[10px] text-[var(--text-secondary)]/50">
							{#if preferencesStore.preferences.sendBehavior === 'enter'}
								<span><kbd class="rounded bg-white/5 px-1">Enter</kbd> send</span>
								<span><kbd class="rounded bg-white/5 px-1">Shift+Enter</kbd> new line</span>
							{:else}
								<span><kbd class="rounded bg-white/5 px-1">{modKey}+Enter</kbd> send</span>
								<span><kbd class="rounded bg-white/5 px-1">Enter</kbd> new line</span>
							{/if}
							<span class="hidden sm:inline"><kbd class="rounded bg-white/5 px-1">↑</kbd> edit last</span>
						</div>
					</div>
				{/if}
				</form>
				{/if}
				{#if chatCollapsed && voiceStore.isInCall}
					<div class="shrink-0 border-t border-white/10 bg-[var(--bg-primary)] p-3">
						<button
							onclick={() => (chatCollapsed = false)}
							class="flex w-full items-center justify-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-4 py-2.5 text-sm text-[var(--text-primary)] transition hover:bg-white/10"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9" /></svg>
							Show Chat
						</button>
					</div>
				{/if}
			{:else}
				<div class="flex flex-1 flex-col items-center justify-center gap-4">
					{#if initError}
						<div class="text-center">
							<div class="mx-auto mb-4 rounded-full bg-[var(--danger)]/10 p-5">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-[var(--danger)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
									<circle cx="12" cy="12" r="10" />
									<line x1="12" y1="8" x2="12" y2="12" />
									<line x1="12" y1="16" x2="12.01" y2="16" />
								</svg>
							</div>
							<h2 class="mb-2 text-2xl font-bold text-[var(--text-primary)]">Failed to load</h2>
							<p class="mb-1 max-w-sm text-[var(--text-secondary)]">
								Could not connect to the server. Check your connection and try again.
							</p>
							<p class="mb-4 text-xs text-[var(--danger)]">{initError}</p>
							<button
								onclick={loadInitialData}
								class="rounded-xl bg-[var(--accent)] px-6 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)]"
							>
								Retry
							</button>
						</div>
					{:else if initialized}
						<!-- Mobile menu button when no channel selected -->
						<button
							onclick={() => { showNavDropdown = true; showMemberPanel = false; }}
							class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)] md:hidden"
						>
							Open channels
						</button>
						<div class="text-center">
							<div class="mx-auto mb-4 rounded-full bg-white/5 p-5">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
									<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
								</svg>
							</div>
							<h2 class="mb-2 text-2xl font-bold text-[var(--text-primary)]">Welcome to Chatalot</h2>
							<p class="max-w-sm text-[var(--text-secondary)]">
								Select a channel from the sidebar to start chatting, or create a new group to get organized.
							</p>
							<p class="mt-2 text-xs text-[var(--text-secondary)]/60">
								Press <kbd class="rounded bg-white/10 px-1 py-0.5 text-[var(--text-primary)]">?</kbd> for keyboard shortcuts
							</p>
						</div>
					{/if}
				</div>
			{/if}
		</main>

		<!-- Member panel (right sidebar) -->
		{#if showMemberPanel && activeChannel && activeChannel.channel_type !== 'dm'}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="fixed inset-0 z-30 bg-black/50 md:hidden" onclick={toggleMemberPanel} onkeydown={(e) => { if (e.key === 'Escape') toggleMemberPanel(); }} role="button" tabindex="-1" aria-label="Close member panel" transition:fade={{ duration: 150 }}></div>
			{#snippet memberRow(member: typeof channelMembers[0])}
				<div class="group flex items-center gap-2 rounded-lg px-2 py-1.5 hover:bg-white/5">
					<Avatar userId={member.user_id} size="sm" showStatus />
					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-1.5">
							<button
								class="truncate text-sm text-[var(--text-primary)] hover:underline cursor-pointer bg-transparent border-none p-0 text-left"
								onclick={(e) => { e.stopPropagation(); openProfileCard(member.user_id, e); }}
							>
								{getDisplayNameForContext(member.user_id)}
							</button>
							{#if communityStore.activeCommunityId && communityMemberStore.getNickname(communityStore.activeCommunityId, member.user_id)}
								<span class="truncate text-xs text-[var(--text-secondary)] opacity-60">({member.display_name})</span>
							{/if}
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
									aria-label={member.role === 'admin' ? 'Remove admin' : 'Make admin'}
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
								aria-label="Kick member"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
							</button>
							<button
								onclick={() => handleBan(member.user_id, member.display_name)}
								class="rounded p-1 text-xs text-[var(--text-secondary)] hover:text-[var(--danger)]"
								title="Ban"
								aria-label="Ban member"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><line x1="4.93" y1="4.93" x2="19.07" y2="19.07" /></svg>
							</button>
						</div>
					{/if}
				</div>
			{/snippet}

			<aside class="fixed inset-y-0 right-0 z-40 w-full md:static md:z-auto md:w-60 md:max-w-none flex-shrink-0 border-l border-white/10 bg-[var(--bg-secondary)] overflow-y-auto shadow-xl md:shadow-none">
				<div class="flex items-center justify-between border-b border-white/10 px-4 py-2">
					<h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
						Members
						{#if !membersLoading}
							<span class="ml-1 normal-case tracking-normal font-normal">— {onlineMembers.length} online, {onlineMembers.length + offlineMembers.length} total</span>
						{/if}
					</h3>
					<button
						onclick={toggleMemberPanel}
						class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						title="Collapse"
						aria-label="Collapse member panel"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6" /></svg>
					</button>
				</div>
				<div class="p-4">
					<input
						type="text"
						bind:value={memberFilter}
						placeholder="Filter members..."
						class="mb-3 w-full rounded-md border border-white/10 bg-[var(--bg-primary)] px-2.5 py-1.5 text-xs text-[var(--text-primary)] outline-none transition placeholder:text-[var(--text-secondary)]/50 focus:border-[var(--accent)]"
					/>
					{#if membersLoading}
						<Skeleton variant="member" count={4} />
					{:else}
						{#if onlineMembers.length > 0}
							<h3 class="mb-1.5 mt-1 text-[11px] font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
								Online — {onlineMembers.length}
							</h3>
							{#each onlineMembers as member (member.user_id)}
								{@render memberRow(member)}
							{/each}
						{/if}
						{#if offlineMembers.length > 0}
							<h3 class="mb-1.5 mt-3 text-[11px] font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
								Offline — {offlineMembers.length}
							</h3>
							{#each offlineMembers as member (member.user_id)}
								{@render memberRow(member)}
							{/each}
						{/if}
					{/if}
				</div>
			</aside>
		{/if}

		<!-- Bookmarks panel (right sidebar) -->
		{#if showBookmarksPanel}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="fixed inset-0 z-30 bg-black/50 md:hidden" onclick={toggleBookmarksPanel} onkeydown={(e) => { if (e.key === 'Escape') toggleBookmarksPanel(); }} role="button" tabindex="-1" aria-label="Close bookmarks panel" transition:fade={{ duration: 150 }}></div>
			<aside class="fixed inset-y-0 right-0 z-40 w-full md:static md:z-auto md:w-60 md:max-w-none flex-shrink-0 border-l border-white/10 bg-[var(--bg-secondary)] overflow-y-auto shadow-xl md:shadow-none">
				<div class="flex items-center justify-between border-b border-white/10 px-4 py-2">
					<h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
						Saved Items
						<span class="ml-1 normal-case tracking-normal font-normal">— {bookmarkStore.bookmarks.length}</span>
					</h3>
					<button
						onclick={toggleBookmarksPanel}
						class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						title="Close"
						aria-label="Close bookmarks panel"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6" /></svg>
					</button>
				</div>
				<div class="p-2">
					{#if bookmarkStore.bookmarks.length === 0}
						<div class="flex flex-col items-center justify-center py-8 text-center">
							<svg xmlns="http://www.w3.org/2000/svg" class="mb-2 h-8 w-8 text-[var(--text-secondary)]/30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
								<path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
							</svg>
							<p class="text-sm text-[var(--text-secondary)]">No saved items yet</p>
							<p class="mt-1 text-xs text-[var(--text-secondary)]/60">Right-click a message to bookmark it</p>
						</div>
					{:else}
						{#each bookmarkStore.bookmarks.toReversed() as bookmark (bookmark.id)}
							<div class="group rounded-lg p-2.5 transition hover:bg-white/5">
								<div class="flex items-start justify-between gap-1">
									<div class="min-w-0 flex-1">
										<div class="flex items-center gap-1.5 mb-1">
											<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-yellow-400" viewBox="0 0 24 24" fill="currentColor" stroke="none">
												<path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
											</svg>
											<p class="text-xs text-[var(--text-secondary)]">
												{new Date(bookmark.created_at).toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' })}
											</p>
										</div>
										{#if bookmark.note}
											<p class="text-xs text-[var(--text-primary)] leading-relaxed">{bookmark.note}</p>
										{:else}
											<p class="text-xs text-[var(--text-secondary)]/60 italic">Bookmarked message</p>
										{/if}
									</div>
									<button
										onclick={() => removeBookmarkFromPanel(bookmark.id)}
										class="shrink-0 rounded p-1 text-[var(--text-secondary)] opacity-0 transition hover:text-[var(--danger)] group-hover:opacity-100"
										title="Remove bookmark"
										aria-label="Remove bookmark"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
									</button>
								</div>
							</div>
						{/each}
					{/if}
				</div>
			</aside>
		{/if}

		<!-- Scheduled messages panel (right sidebar) -->
		{#if showScheduledPanel}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="fixed inset-0 z-30 bg-black/50 md:hidden" onclick={toggleScheduledPanel} onkeydown={(e) => { if (e.key === 'Escape') toggleScheduledPanel(); }} role="button" tabindex="-1" aria-label="Close scheduled panel" transition:fade={{ duration: 150 }}></div>
			<aside class="fixed inset-y-0 right-0 z-40 w-full md:static md:z-auto md:w-60 md:max-w-none flex-shrink-0 border-l border-white/10 bg-[var(--bg-secondary)] overflow-y-auto shadow-xl md:shadow-none">
				<div class="flex items-center justify-between border-b border-white/10 px-4 py-2">
					<h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">
						Scheduled
						<span class="ml-1 normal-case tracking-normal font-normal">— {scheduledMessages.length}</span>
					</h3>
					<button
						onclick={toggleScheduledPanel}
						class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						title="Close"
						aria-label="Close scheduled panel"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6" /></svg>
					</button>
				</div>
				<div class="p-2">
					{#if scheduledMessages.length === 0}
						<div class="flex flex-col items-center justify-center py-8 text-center">
							<svg xmlns="http://www.w3.org/2000/svg" class="mb-2 h-8 w-8 text-[var(--text-secondary)]/30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
								<circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
							</svg>
							<p class="text-sm text-[var(--text-secondary)]">No scheduled messages</p>
							<p class="mt-1 text-xs text-[var(--text-secondary)]/60">Use the clock icon next to Send</p>
						</div>
					{:else}
						{#each scheduledMessages as msg (msg.id)}
							{@const channel = channelStore.channels.find(c => c.id === msg.channel_id)}
							<div class="group rounded-xl border border-white/5 bg-[var(--bg-primary)]/50 p-3 transition hover:border-white/10">
								<!-- Message content -->
								<div class="mb-2">
									{#if msg.content}
										<p class="text-sm text-[var(--text-primary)] line-clamp-3 break-words">{msg.content}</p>
									{:else}
										<p class="text-sm italic text-[var(--text-secondary)]/60">Encrypted message</p>
									{/if}
								</div>
								<!-- Channel -->
								{#if channel}
									<div class="mb-1.5 flex items-center gap-1">
										<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-[var(--text-secondary)]/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 9l16 0"/><path d="M4 15l16 0"/><path d="M10 3l-2 18"/><path d="M16 3l-2 18"/></svg>
										<span class="text-[11px] text-[var(--text-secondary)]/60">{channel.name ?? 'DM'}</span>
									</div>
								{/if}
								<!-- Delivery time + cancel -->
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-1.5">
										<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
										</svg>
										<span class="text-[11px] text-[var(--text-secondary)]">
											{new Date(msg.scheduled_for).toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' })}
										</span>
									</div>
									<button
										type="button"
										onclick={() => handleCancelScheduled(msg.id)}
										class="shrink-0 rounded px-1.5 py-0.5 text-[10px] text-red-400/70 opacity-0 transition hover:bg-red-500/10 hover:text-red-400 group-hover:opacity-100"
										title="Cancel scheduled message"
										aria-label="Cancel scheduled message"
									>Cancel</button>
								</div>
							</div>
						{/each}
					{/if}
				</div>
			</aside>
		{/if}

		<!-- Thread panel (right sidebar) -->
		{#if showThreadPanel && activeThreadRoot}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="fixed inset-0 z-30 bg-black/50 md:hidden" onclick={closeThread} onkeydown={(e) => { if (e.key === 'Escape') closeThread(); }} role="button" tabindex="-1" aria-label="Close thread panel" transition:fade={{ duration: 150 }}></div>
			<aside class="fixed inset-y-0 right-0 z-40 w-full md:static md:z-auto md:w-[360px] md:max-w-none flex-shrink-0 border-l border-white/10 bg-[var(--bg-secondary)] flex flex-col shadow-xl md:shadow-none" transition:fly={{ x: 360, duration: 200 }}>
				<!-- Header -->
				<div class="flex items-center justify-between border-b border-white/10 px-4 py-2.5">
					<h3 class="text-sm font-semibold text-[var(--text-primary)]">Thread</h3>
					<button
						onclick={closeThread}
						class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						title="Close thread"
						aria-label="Close thread panel"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
					</button>
				</div>

				<!-- Root message -->
				<div class="border-b border-white/10 px-4 py-3">
					<div class="flex items-start gap-2.5">
						<Avatar userId={activeThreadRoot.senderId} size={32} />
						<div class="min-w-0 flex-1">
							<div class="flex items-baseline gap-2">
								<span class="text-sm font-medium text-[var(--text-primary)]">{getDisplayNameForContext(activeThreadRoot.senderId)}</span>
								<span class="text-[10px] text-[var(--text-secondary)]">{formatFullTimestamp(activeThreadRoot.createdAt)}</span>
								{#if activeThreadRoot.editedAt}
									<span class="text-[10px] text-[var(--text-secondary)]/60">(edited)</span>
								{/if}
							</div>
							{#if activeThreadRoot.messageType === 'file'}
								{@const fileInfo = parseFileMessage(activeThreadRoot.content)}
								{#if fileInfo && IMAGE_EXTS.test(fileInfo.filename)}
									<div class="mt-1">
										{#await getAuthenticatedBlobUrl(fileInfo.file_id)}
											<div class="flex h-40 w-full items-center justify-center rounded-lg border border-white/10 bg-[var(--bg-primary)]"><span class="text-xs text-[var(--text-secondary)]">Loading image...</span></div>
										{:then blobUrl}
											<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
											<img src={blobUrl} alt={fileInfo.filename} class="max-h-60 max-w-full cursor-pointer rounded-lg border border-white/10 transition hover:brightness-90" onclick={() => openLightbox(blobUrl, fileInfo.filename)} onkeydown={(e) => { if (e.key === 'Enter') openLightbox(blobUrl, fileInfo.filename); }} />
										{:catch}
											<div class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2"><span class="text-sm text-[var(--text-secondary)]">Failed to load image</span></div>
										{/await}
										<div class="mt-1 flex items-center gap-2 text-xs text-[var(--text-secondary)]"><span class="truncate max-w-[200px]" title={fileInfo.filename}>{fileInfo.filename}</span><span class="shrink-0">({formatFileSize(fileInfo.size)})</span></div>
									</div>
								{:else if fileInfo && VIDEO_EXTS.test(fileInfo.filename)}
									<div class="mt-1">
										{#await getAuthenticatedBlobUrl(fileInfo.file_id)}
											<div class="flex h-48 w-full items-center justify-center rounded-lg border border-white/10 bg-[var(--bg-primary)]"><span class="text-xs text-[var(--text-secondary)]">Loading video...</span></div>
										{:then blobUrl}
											<!-- svelte-ignore a11y_media_has_caption -->
											<video src={blobUrl} controls class="max-h-60 max-w-full rounded-lg border border-white/10"></video>
										{:catch}
											<div class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2"><span class="text-sm text-[var(--text-secondary)]">Failed to load video</span></div>
										{/await}
										<div class="mt-1 flex items-center gap-2 text-xs text-[var(--text-secondary)]"><span class="truncate max-w-[200px]" title={fileInfo.filename}>{fileInfo.filename}</span><span class="shrink-0">({formatFileSize(fileInfo.size)})</span></div>
									</div>
								{:else if fileInfo && AUDIO_EXTS.test(fileInfo.filename)}
									<div class="mt-1 max-w-full">
										<div class="flex items-center gap-3 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2.5">
											<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 shrink-0 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" /></svg>
											<div class="min-w-0 flex-1">
												<p class="truncate text-sm font-medium text-[var(--text-primary)]" title={fileInfo.filename}>{fileInfo.filename}</p>
												<p class="text-xs text-[var(--text-secondary)]">{formatFileSize(fileInfo.size)}</p>
											</div>
										</div>
										{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
											<audio src={blobUrl} controls class="mt-1 w-full rounded" style="height: 32px;"></audio>
										{:catch}
											<p class="mt-1 text-xs text-[var(--text-secondary)]">Could not load audio</p>
										{/await}
									</div>
								{:else if fileInfo}
									<div class="mt-1 inline-flex max-w-full items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2">
										<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" /></svg>
										<span class="truncate text-sm text-[var(--text-primary)]" title={fileInfo.filename}>{fileInfo.filename}</span>
										<span class="shrink-0 text-xs text-[var(--text-secondary)]">({formatFileSize(fileInfo.size)})</span>
										{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
											<a href={blobUrl} download={fileInfo.filename} class="shrink-0 text-xs text-[var(--accent)] hover:underline">Download</a>
										{:catch}
											<span class="shrink-0 text-xs text-[var(--text-secondary)]">Download unavailable</span>
										{/await}
									</div>
								{:else}
									<div class="mt-1 inline-flex items-center gap-2 rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-2">
										<svg class="h-4 w-4 shrink-0 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
										<span class="text-sm italic text-amber-300/80">Encrypted file (E2E decryption not available)</span>
									</div>
								{/if}
							{:else}
								{@const imageUrls = extractImageUrls(activeThreadRoot.content)}
								{@const linkUrls = extractNonImageUrls(activeThreadRoot.content)}
								<div class="markdown-content mt-0.5 text-sm text-[var(--text-primary)] leading-relaxed">{@html renderMarkdown(activeThreadRoot.content)}</div>
								{#if imageUrls.length > 0}
									<div class="mt-2 flex flex-col gap-2">
										{#each imageUrls as imgUrl}
											<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
											<img src={imgUrl} alt="Linked content" class="max-h-60 max-w-full cursor-pointer rounded-lg border border-white/10 transition hover:brightness-90" loading="lazy" onclick={() => openLightbox(imgUrl, 'Image')} onkeydown={(e) => { if (e.key === 'Enter') openLightbox(imgUrl, 'Image'); }} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
										{/each}
									</div>
								{/if}
								{#if linkUrls.length > 0 && preferencesStore.preferences.showLinkPreviews}
									{#each linkUrls.slice(0, 2) as linkUrl}
										{#await fetchLinkPreview(linkUrl) then preview}
											{#if preview && (preview.title || preview.description)}
												<a href={linkUrl} target="_blank" rel="noopener noreferrer" class="link-embed mt-2 block max-w-full rounded-lg border-l-4 border-[var(--accent)] bg-[var(--bg-primary)] p-3 transition hover:bg-white/5">
													{#if preview.site_name}<div class="text-xs text-[var(--text-secondary)]">{preview.site_name}</div>{/if}
													{#if preview.title}<div class="text-sm font-semibold text-[var(--accent)]">{preview.title}</div>{/if}
													{#if preview.description}<div class="mt-1 text-xs text-[var(--text-secondary)] line-clamp-3">{preview.description}</div>{/if}
													{#if preview.image}<img src={preview.image} alt="" class="mt-2 max-h-32 rounded border border-white/10" loading="lazy" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />{/if}
												</a>
											{/if}
										{/await}
									{/each}
								{/if}
							{/if}
							<!-- Root reactions -->
							{#if activeThreadRoot.reactions && activeThreadRoot.reactions.size > 0}
								<div class="mt-1.5 flex flex-wrap gap-1">
									{#each Array.from(activeThreadRoot.reactions.entries()) as [emoji, users]}
										{@const hasReacted = users.has(authStore.user?.id ?? '')}
										{@const reacterNames = Array.from(users).map(uid => uid === authStore.user?.id ? 'You' : getDisplayNameForContext(uid)).join(', ')}
										<button onclick={() => toggleReaction(activeThreadRoot.id, emoji)} class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs transition {hasReacted ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 text-[var(--text-secondary)] hover:border-white/20 hover:bg-white/5'}" title="{reacterNames} reacted with {emoji}">
											<span>{emoji}</span><span class="font-medium">{users.size}</span>
										</button>
									{/each}
								</div>
							{/if}
						</div>
					</div>
				</div>

				<!-- Replies -->
				<div class="flex-1 overflow-y-auto px-4 py-2 space-y-1">
					{#if threadLoading}
						<div class="flex items-center justify-center py-8">
							<div class="h-5 w-5 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
						</div>
					{:else if threadMessages.length === 0}
						<div class="flex flex-col items-center justify-center py-8 text-center">
							<svg xmlns="http://www.w3.org/2000/svg" class="mb-2 h-8 w-8 text-[var(--text-secondary)]/30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
							<p class="text-sm text-[var(--text-secondary)]">No replies yet</p>
							<p class="mt-1 text-xs text-[var(--text-secondary)]/60">Be the first to reply</p>
						</div>
					{:else}
						{#each threadMessages as reply (reply.id)}
							<div class="group relative rounded-md px-1 py-2 transition hover:bg-white/[0.02] {reply.pending ? 'opacity-50' : ''}">
								<div class="flex items-start gap-2.5">
									<Avatar userId={reply.senderId} size={28} />
									<div class="min-w-0 flex-1">
										<div class="flex items-baseline gap-2">
											<span class="text-xs font-medium text-[var(--text-primary)]">{getDisplayNameForContext(reply.senderId)}</span>
											<span class="text-[10px] text-[var(--text-secondary)]">{formatRelativeTime(reply.createdAt)}</span>
											{#if reply.pending}
												<span class="text-[10px] text-[var(--text-secondary)] italic">sending...</span>
											{:else if reply.editedAt}
												<span class="text-[10px] text-[var(--text-secondary)]/60">(edited)</span>
											{/if}
										</div>

										{#if editingMessageId === reply.id}
											<!-- Edit mode -->
											<div class="mt-1">
												<textarea
													data-edit-input
													rows="2"
													bind:value={editInput}
													onkeydown={(e) => handleEditKeydown(e, reply.id)}
													class="w-full resize-y rounded border border-[var(--accent)] bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none"
												></textarea>
												<div class="mt-1 flex gap-2 text-xs">
													<button onclick={() => submitEdit(reply.id)} class="text-[var(--accent)] hover:underline">Save</button>
													<button onclick={cancelEdit} class="text-[var(--text-secondary)] hover:underline">Cancel</button>
													<span class="text-[var(--text-secondary)]">esc to cancel, enter to save, shift+enter for newline</span>
												</div>
											</div>
										{:else if reply.messageType === 'file'}
											{@const fileInfo = parseFileMessage(reply.content)}
											{#if fileInfo && IMAGE_EXTS.test(fileInfo.filename)}
												<div class="mt-1">
													{#await getAuthenticatedBlobUrl(fileInfo.file_id)}
														<div class="flex h-32 w-full items-center justify-center rounded-lg border border-white/10 bg-[var(--bg-primary)]"><span class="text-xs text-[var(--text-secondary)]">Loading image...</span></div>
													{:then blobUrl}
														<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
														<img src={blobUrl} alt={fileInfo.filename} class="max-h-48 max-w-full cursor-pointer rounded-lg border border-white/10 transition hover:brightness-90" onclick={() => openLightbox(blobUrl, fileInfo.filename)} onkeydown={(e) => { if (e.key === 'Enter') openLightbox(blobUrl, fileInfo.filename); }} />
													{:catch}
														<div class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2"><span class="text-sm text-[var(--text-secondary)]">Failed to load image</span></div>
													{/await}
													<div class="mt-1 flex items-center gap-2 text-xs text-[var(--text-secondary)]"><span class="truncate max-w-[200px]" title={fileInfo.filename}>{fileInfo.filename}</span><span class="shrink-0">({formatFileSize(fileInfo.size)})</span></div>
												</div>
											{:else if fileInfo && VIDEO_EXTS.test(fileInfo.filename)}
												<div class="mt-1">
													{#await getAuthenticatedBlobUrl(fileInfo.file_id)}
														<div class="flex h-32 w-full items-center justify-center rounded-lg border border-white/10 bg-[var(--bg-primary)]"><span class="text-xs text-[var(--text-secondary)]">Loading video...</span></div>
													{:then blobUrl}
														<!-- svelte-ignore a11y_media_has_caption -->
														<video src={blobUrl} controls class="max-h-48 max-w-full rounded-lg border border-white/10"></video>
													{:catch}
														<div class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2"><span class="text-sm text-[var(--text-secondary)]">Failed to load video</span></div>
													{/await}
													<div class="mt-1 flex items-center gap-2 text-xs text-[var(--text-secondary)]"><span class="truncate max-w-[200px]" title={fileInfo.filename}>{fileInfo.filename}</span><span class="shrink-0">({formatFileSize(fileInfo.size)})</span></div>
												</div>
											{:else if fileInfo && AUDIO_EXTS.test(fileInfo.filename)}
												<div class="mt-1 max-w-full">
													<div class="flex items-center gap-3 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2.5">
														<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 shrink-0 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" /></svg>
														<div class="min-w-0 flex-1">
															<p class="truncate text-sm font-medium text-[var(--text-primary)]" title={fileInfo.filename}>{fileInfo.filename}</p>
															<p class="text-xs text-[var(--text-secondary)]">{formatFileSize(fileInfo.size)}</p>
														</div>
													</div>
													{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
														<audio src={blobUrl} controls class="mt-1 w-full rounded" style="height: 32px;"></audio>
													{:catch}
														<p class="mt-1 text-xs text-[var(--text-secondary)]">Could not load audio</p>
													{/await}
												</div>
											{:else if fileInfo}
												<div class="mt-1 inline-flex max-w-full items-center gap-2 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2">
													<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" /></svg>
													<span class="truncate text-sm text-[var(--text-primary)]" title={fileInfo.filename}>{fileInfo.filename}</span>
													<span class="shrink-0 text-xs text-[var(--text-secondary)]">({formatFileSize(fileInfo.size)})</span>
													{#await getAuthenticatedBlobUrl(fileInfo.file_id) then blobUrl}
														<a href={blobUrl} download={fileInfo.filename} class="shrink-0 text-xs text-[var(--accent)] hover:underline">Download</a>
													{:catch}
														<span class="shrink-0 text-xs text-[var(--text-secondary)]">Download unavailable</span>
													{/await}
												</div>
											{:else}
												<div class="mt-1 inline-flex items-center gap-2 rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-2">
													<svg class="h-4 w-4 shrink-0 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
													<span class="text-sm italic text-amber-300/80">Encrypted file (E2E decryption not available)</span>
												</div>
											{/if}
										{:else}
											{@const imageUrls = extractImageUrls(reply.content)}
											{@const linkUrls = extractNonImageUrls(reply.content)}
											<div class="markdown-content mt-0.5 text-sm text-[var(--text-primary)] leading-relaxed">{@html renderMarkdown(reply.content)}</div>
											{#if imageUrls.length > 0}
												<div class="mt-2 flex flex-col gap-2">
													{#each imageUrls as imgUrl}
														<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
														<img src={imgUrl} alt="Linked content" class="max-h-48 max-w-full cursor-pointer rounded-lg border border-white/10 transition hover:brightness-90" loading="lazy" onclick={() => openLightbox(imgUrl, 'Image')} onkeydown={(e) => { if (e.key === 'Enter') openLightbox(imgUrl, 'Image'); }} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
													{/each}
												</div>
											{/if}
											{#if linkUrls.length > 0 && preferencesStore.preferences.showLinkPreviews}
												{#each linkUrls.slice(0, 2) as linkUrl}
													{#await fetchLinkPreview(linkUrl) then preview}
														{#if preview && (preview.title || preview.description)}
															<a href={linkUrl} target="_blank" rel="noopener noreferrer" class="link-embed mt-2 block max-w-full rounded-lg border-l-4 border-[var(--accent)] bg-[var(--bg-primary)] p-2.5 transition hover:bg-white/5">
																{#if preview.title}<div class="text-sm font-semibold text-[var(--accent)]">{preview.title}</div>{/if}
																{#if preview.description}<div class="mt-1 text-xs text-[var(--text-secondary)] line-clamp-2">{preview.description}</div>{/if}
															</a>
														{/if}
													{/await}
												{/each}
											{/if}
										{/if}

										<!-- Reply reactions -->
										{#if reply.reactions && reply.reactions.size > 0}
											<div class="mt-1.5 flex flex-wrap gap-1">
												{#each Array.from(reply.reactions.entries()) as [emoji, users]}
													{@const hasReacted = users.has(authStore.user?.id ?? '')}
													{@const reacterNames = Array.from(users).map(uid => uid === authStore.user?.id ? 'You' : getDisplayNameForContext(uid)).join(', ')}
													<button onclick={() => toggleReaction(reply.id, emoji)} class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs transition {hasReacted ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 text-[var(--text-secondary)] hover:border-white/20 hover:bg-white/5'}" title="{reacterNames} reacted with {emoji}">
														<span>{emoji}</span><span class="font-medium">{users.size}</span>
													</button>
												{/each}
											</div>
										{/if}
									</div>
								</div>

								<!-- Hover actions -->
								{#if !reply.pending}
									<div class="absolute right-1 top-1 hidden gap-0.5 rounded border border-white/10 bg-[var(--bg-secondary)] shadow-lg group-hover:flex">
										<button
											onclick={(e) => { e.stopPropagation(); threadReactionPickerMsgId = threadReactionPickerMsgId === reply.id ? null : reply.id; }}
											class="p-1 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
											title="Add reaction"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10" /><path d="M8 14s1.5 2 4 2 4-2 4-2" /><line x1="9" y1="9" x2="9.01" y2="9" /><line x1="15" y1="9" x2="15.01" y2="9" /></svg>
										</button>
										{#if reply.senderId === authStore.user?.id}
											<button
												onclick={() => startEditMessage(reply)}
												class="p-1 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
												title="Edit"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" /><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" /></svg>
											</button>
											<button
												onclick={() => handleDeleteMessage(reply.id)}
												class="p-1 text-[var(--text-secondary)] transition hover:text-[var(--danger)]"
												title="Delete"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /></svg>
											</button>
										{:else if myRole === 'owner' || myRole === 'admin'}
											<button
												onclick={() => handleDeleteMessage(reply.id)}
												class="p-1 text-[var(--text-secondary)] transition hover:text-[var(--danger)]"
												title="Delete (mod)"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /></svg>
											</button>
										{/if}
									</div>
								{/if}

								<!-- Reaction picker popup -->
								{#if threadReactionPickerMsgId === reply.id}
									<div
										class="absolute right-1 top-8 z-10 flex items-center gap-1 rounded-xl bg-[var(--bg-secondary)] p-2 shadow-lg"
										transition:scale={{ start: 0.9, duration: 150 }}
										role="toolbar"
										aria-label="Reaction picker"
									>
										{#each QUICK_REACTIONS as emoji}
											<button onclick={() => toggleReaction(reply.id, emoji)} class="rounded p-1 text-lg transition hover:bg-white/10">{emoji}</button>
										{/each}
										<button
											onclick={(e) => { e.stopPropagation(); openThreadFullEmojiPicker(reply.id); }}
											class="rounded p-1 text-lg text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]"
											title="More emojis"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="16" /><line x1="8" y1="12" x2="16" y2="12" /></svg>
										</button>
									</div>
								{/if}
								{#if threadFullEmojiPickerMsgId === reply.id}
									<!-- svelte-ignore a11y_click_events_have_key_events -->
									<!-- svelte-ignore a11y_no_static_element_interactions -->
									<div class="absolute right-1 top-8 z-20" transition:scale={{ start: 0.9, duration: 150 }} onclick={(e) => e.stopPropagation()}>
										<emoji-picker use:bindEmojiPicker={reply.id} class="dark"></emoji-picker>
									</div>
								{/if}
							</div>
						{/each}
					{/if}
				</div>

				<!-- Thread composer -->
				<div class="border-t border-white/10 px-3 py-2.5">
					<div class="flex items-end gap-2">
						<textarea
							bind:this={threadTextareaEl}
							placeholder="Reply in thread..."
							bind:value={threadMessageInput}
							maxlength={4000}
							onkeydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendThreadMessage(); } }}
							oninput={() => { if (threadTextareaEl) { threadTextareaEl.style.height = 'auto'; threadTextareaEl.style.height = Math.min(threadTextareaEl.scrollHeight, 120) + 'px'; } }}
							rows={1}
							class="flex-1 resize-none rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none placeholder:text-[var(--text-secondary)]/50 focus:border-[var(--accent)]"
							style="max-height: 120px;"
						></textarea>
						<button
							onclick={sendThreadMessage}
							disabled={!threadMessageInput.trim()}
							class="rounded-lg bg-[var(--accent)] px-3 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50 disabled:cursor-not-allowed"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="22" y1="2" x2="11" y2="13" /><polygon points="22 2 15 22 11 13 2 9 22 2" /></svg>
						</button>
					</div>
				</div>
			</aside>
		{/if}
		</div><!-- end main content row -->
	</div>

	<!-- Feedback modal -->
	{#if showFeedback}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" transition:fade={{ duration: 150 }} onpaste={handleFeedbackPaste}>
			<div role="dialog" aria-modal="true" aria-label="Send feedback" tabindex="-1" class="w-full max-w-md max-h-[90vh] overflow-y-auto rounded-2xl bg-[var(--bg-secondary)] p-6 shadow-xl" transition:scale={{ start: 0.95, duration: 200 }}>
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

					<!-- Screenshot attachment -->
					<div>
						<label for="feedback-screenshot" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">Screenshot (optional)</label>
						<input id="feedback-screenshot"
							bind:this={feedbackFileInput}
							type="file"
							accept="image/png,image/jpeg,image/webp"
							class="hidden"
							onchange={(e) => {
								const file = (e.target as HTMLInputElement).files?.[0] ?? null;
								setFeedbackScreenshot(file);
							}}
						/>
						{#if feedbackScreenshotPreview}
							<div class="relative inline-block">
								<img src={feedbackScreenshotPreview} alt="Screenshot preview" class="max-h-40 rounded-lg border border-white/10" />
								<button
									type="button"
									onclick={() => { setFeedbackScreenshot(null); if (feedbackFileInput) feedbackFileInput.value = ''; }}
									class="absolute -right-2 -top-2 flex h-5 w-5 items-center justify-center rounded-full bg-red-500 text-xs text-white shadow hover:bg-red-600"
									title="Remove screenshot"
										aria-label="Remove screenshot"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
								</button>
							</div>
						{:else}
							<button
								type="button"
								onclick={() => feedbackFileInput?.click()}
								class="flex w-full items-center justify-center gap-2 rounded-lg border border-dashed border-white/20 px-3 py-3 text-sm text-[var(--text-secondary)] transition hover:border-[var(--accent)] hover:text-[var(--text-primary)]"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2" /><circle cx="8.5" cy="8.5" r="1.5" /><polyline points="21 15 16 10 5 21" /></svg>
								Attach screenshot or paste from clipboard
							</button>
						{/if}
					</div>

					<div class="flex justify-end gap-3">
						<button
							type="button"
							onclick={() => { showFeedback = false; setFeedbackScreenshot(null); }}
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

	<!-- Confirmation Dialog -->
	{#if confirmDialog}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
			role="dialog"
			tabindex="-1"
			aria-modal="true"
			aria-label={confirmDialog.title}
			transition:fade={{ duration: 150 }}
			onclick={() => { confirmDialog = null; confirmInput = ''; }}
			onkeydown={(e) => { if (e.key === 'Escape') { confirmDialog = null; confirmInput = ''; } }}
		>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-full max-w-sm rounded-2xl bg-[var(--bg-secondary)] p-5 shadow-xl"
				transition:scale={{ start: 0.95, duration: 200 }}
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
			>
				<h3 class="mb-2 text-base font-bold text-[var(--text-primary)]">{confirmDialog.title}</h3>
				<p class="mb-4 text-sm text-[var(--text-secondary)]">{confirmDialog.message}</p>
				{#if confirmDialog.inputPlaceholder}
					<!-- svelte-ignore a11y_autofocus -->
					<input
						type="text"
						bind:value={confirmInput}
						placeholder={confirmDialog.inputPlaceholder}
						autofocus
						onkeydown={(e) => { if (e.key === 'Enter') { confirmDialog?.onConfirm(confirmInput); confirmDialog = null; confirmInput = ''; } }}
						class="mb-4 w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
				{/if}
				<div class="flex justify-end gap-2">
					<button
						onclick={() => { confirmDialog = null; confirmInput = ''; }}
						class="rounded-lg px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Cancel
					</button>
					<button
						onclick={() => { confirmDialog?.onConfirm(confirmInput); confirmDialog = null; confirmInput = ''; }}
						class="rounded-lg px-4 py-2 text-sm font-medium text-white transition {confirmDialog.danger ? 'bg-[var(--danger)] hover:bg-red-600' : 'bg-[var(--accent)] hover:bg-[var(--accent-hover)]'}"
					>
						{confirmDialog.confirmLabel ?? 'Confirm'}
					</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Report Message Modal -->
	{#if reportingMessageId}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
			role="dialog"
			tabindex="-1"
			aria-modal="true"
			aria-label="Report Message"
			transition:fade={{ duration: 150 }}
			onclick={() => { reportingMessageId = null; reportReason = ''; }}
			onkeydown={(e) => { if (e.key === 'Escape') { reportingMessageId = null; reportReason = ''; } }}
		>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-full max-w-sm rounded-2xl bg-[var(--bg-secondary)] p-5 shadow-xl"
				transition:scale={{ start: 0.95, duration: 200 }}
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
			>
				<h3 class="mb-2 text-base font-bold text-[var(--text-primary)]">Report Message</h3>
				<p class="mb-3 text-sm text-[var(--text-secondary)]">Why are you reporting this message? Admins will review your report.</p>
				<textarea
					bind:value={reportReason}
					placeholder="Describe the issue..."
					rows="3"
					maxlength="1000"
					class="mb-4 w-full resize-none rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
				></textarea>
				<div class="flex justify-end gap-2">
					<button
						onclick={() => { reportingMessageId = null; reportReason = ''; }}
						class="rounded-lg px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Cancel
					</button>
					<button
						onclick={handleSubmitReport}
						disabled={submittingReport || !reportReason.trim()}
						class="rounded-lg bg-orange-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-orange-600 disabled:cursor-not-allowed disabled:opacity-50"
					>
						{submittingReport ? 'Submitting...' : 'Submit Report'}
					</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Create Poll Modal -->
	{#if showCreatePoll}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
			role="dialog"
			tabindex="-1"
			aria-modal="true"
			aria-label="Create Poll"
			transition:fade={{ duration: 150 }}
			onclick={() => showCreatePoll = false}
			onkeydown={(e) => { if (e.key === 'Escape') showCreatePoll = false; }}
		>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-6 shadow-xl"
				transition:scale={{ start: 0.95, duration: 200 }}
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
			>
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-bold text-[var(--text-primary)]">Create Poll</h2>
					<button aria-label="Close" onclick={() => showCreatePoll = false} class="rounded p-1 text-[var(--text-secondary)] hover:text-[var(--text-primary)]">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
					</button>
				</div>
				<div class="space-y-4">
					<div>
						<label for="poll-question" class="mb-1 block text-xs font-medium text-[var(--text-secondary)]">Question</label>
						<input
							id="poll-question"
							type="text"
							bind:value={newPollQuestion}
							maxlength="500"
							placeholder="What do you want to ask?"
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)]"
						/>
					</div>
					<div>
						<span class="mb-1 block text-xs font-medium text-[var(--text-secondary)]">Options (2-10)</span>
						<div class="space-y-1.5">
							{#each newPollOptions as opt, idx}
								<div class="flex items-center gap-1.5">
									<input
										type="text"
										bind:value={newPollOptions[idx]}
										maxlength="200"
										placeholder="Option {idx + 1}"
										class="flex-1 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)]"
									/>
									{#if newPollOptions.length > 2}
										<button
											type="button"
											aria-label="Remove option {idx + 1}"
											onclick={() => { newPollOptions = newPollOptions.filter((_, i) => i !== idx); }}
											class="shrink-0 rounded p-1 text-[var(--text-secondary)] transition hover:text-[var(--danger)]"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
										</button>
									{/if}
								</div>
							{/each}
							{#if newPollOptions.length < 10}
								<button
									type="button"
									onclick={() => { newPollOptions = [...newPollOptions, '']; }}
									class="flex items-center gap-1 rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-white/5"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
									Add option
								</button>
							{/if}
						</div>
					</div>
					<div class="flex flex-wrap gap-4">
						<label class="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
							<input type="checkbox" bind:checked={newPollMultiSelect} class="rounded" />
							Allow multiple votes
						</label>
						<label class="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
							<input type="checkbox" bind:checked={newPollAnonymous} class="rounded" />
							Anonymous voting
						</label>
					</div>
					<div>
						<label for="poll-expiry" class="mb-1 block text-xs font-medium text-[var(--text-secondary)]">Expires after</label>
						<select
							id="poll-expiry"
							bind:value={newPollExpiry}
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)]"
						>
							<option value={null}>Never</option>
							<option value={15}>15 minutes</option>
							<option value={60}>1 hour</option>
							<option value={360}>6 hours</option>
							<option value={1440}>1 day</option>
							<option value={4320}>3 days</option>
							<option value={10080}>1 week</option>
						</select>
					</div>
					<div class="flex justify-end gap-2">
						<button
							onclick={() => showCreatePoll = false}
							class="rounded-lg border border-white/10 px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5"
						>
							Cancel
						</button>
						<button
							onclick={handleCreatePoll}
							disabled={creatingPoll || !newPollQuestion.trim() || newPollOptions.map(o => o.trim()).filter(o => o).length < 2}
							class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
						>
							{creatingPoll ? 'Creating...' : 'Create Poll'}
						</button>
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- Encryption Verification Modal -->
	{#if showEncryptionInfo}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
			role="dialog"
			tabindex="-1"
			aria-modal="true"
			aria-label="Encryption Verification"
			transition:fade={{ duration: 150 }}
			onclick={() => showEncryptionInfo = false}
			onkeydown={(e) => { if (e.key === 'Escape') showEncryptionInfo = false; }}
		>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-6 shadow-xl"
				transition:scale={{ start: 0.95, duration: 200 }}
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
			>
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<div class="flex h-8 w-8 items-center justify-center rounded-full bg-green-500/15">
							<svg class="h-4 w-4 text-green-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
						</div>
						<h2 class="text-lg font-bold text-[var(--text-primary)]">End-to-End Encrypted</h2>
					</div>
					<button aria-label="Close encryption info" onclick={() => showEncryptionInfo = false} class="rounded p-1 text-[var(--text-secondary)] hover:text-[var(--text-primary)]">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
					</button>
				</div>

				{#if encryptionInfoLoading}
					<div class="flex items-center justify-center py-8">
						<div class="h-6 w-6 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
					</div>
				{:else if safetyNumber}
					<p class="mb-3 text-sm text-[var(--text-secondary)]">
						Messages in this conversation are secured with X3DH + Double Ratchet encryption. Verify the safety number below matches on both devices.
					</p>

					<!-- Safety Number -->
					<div class="mb-4">
						<div class="mb-1.5 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Safety Number</div>
						<div class="relative rounded-lg bg-[var(--bg-primary)] p-3">
							<div class="select-all break-all font-mono text-sm leading-relaxed tracking-widest text-[var(--text-primary)]">
								{safetyNumber}
							</div>
							<button
								onclick={() => {
									navigator.clipboard.writeText(safetyNumber);
									safetyNumberCopied = true;
									setTimeout(() => safetyNumberCopied = false, 2000);
								}}
								class="absolute right-2 top-2 rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]"
								title="Copy safety number"
								aria-label="Copy safety number"
							>
								{#if safetyNumberCopied}
									<svg class="h-4 w-4 text-green-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
								{:else}
									<svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
								{/if}
							</button>
						</div>
					</div>

					<!-- Fingerprints -->
					<div class="space-y-3">
						<div>
							<div class="mb-1 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Your Fingerprint</div>
							<div class="select-all rounded-lg bg-[var(--bg-primary)] px-3 py-2 font-mono text-xs leading-relaxed text-[var(--text-primary)] break-all">
								{ownFingerprint}
							</div>
						</div>
						{#if peerFingerprint}
							<div>
								<div class="mb-1 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Their Fingerprint</div>
								<div class="select-all rounded-lg bg-[var(--bg-primary)] px-3 py-2 font-mono text-xs leading-relaxed text-[var(--text-primary)] break-all">
									{peerFingerprint}
								</div>
							</div>
						{/if}
					</div>

					<p class="mt-4 text-xs text-[var(--text-tertiary)]">
						Compare these numbers with your contact through a separate trusted channel (in person, phone call, etc.) to confirm the encryption is secure.
					</p>
				{:else}
					<p class="text-sm text-[var(--text-secondary)]">
						No encryption session established yet. Send or receive a message to start an encrypted session.
					</p>
					{#if ownFingerprint}
						<div class="mt-3">
							<div class="mb-1 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Your Fingerprint</div>
							<div class="select-all rounded-lg bg-[var(--bg-primary)] px-3 py-2 font-mono text-xs leading-relaxed text-[var(--text-primary)] break-all">
								{ownFingerprint}
							</div>
						</div>
					{/if}
				{/if}
			</div>
		</div>
	{/if}

	<!-- Keyboard Shortcuts Modal -->
	{#if showShortcutsModal}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
			role="dialog"
			tabindex="-1"
			aria-modal="true"
			aria-label="Keyboard Shortcuts"
			transition:fade={{ duration: 150 }}
			onclick={() => showShortcutsModal = false}
			onkeydown={(e) => { if (e.key === 'Escape') showShortcutsModal = false; }}
		>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-full max-w-lg rounded-2xl bg-[var(--bg-secondary)] p-6 shadow-xl"
				transition:scale={{ start: 0.95, duration: 200 }}
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
			>
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-bold text-[var(--text-primary)]">Keyboard Shortcuts</h2>
					<button aria-label="Close keyboard shortcuts" onclick={() => showShortcutsModal = false} class="rounded p-1 text-[var(--text-secondary)] hover:text-[var(--text-primary)]">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
					</button>
				</div>
				<div class="grid grid-cols-2 gap-x-8 gap-y-2 text-sm">
					<div class="col-span-2 mt-1 mb-1 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Messages</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Send message</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">{preferencesStore.preferences.sendBehavior === 'enter' ? 'Enter' : `${modKey}+Enter`}</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">New line</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">{preferencesStore.preferences.sendBehavior === 'enter' ? 'Shift+Enter' : 'Enter'}</kbd>
					</div>
					<div class="col-span-2 mt-3 mb-1 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Formatting</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Bold</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">{modKey}+B</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Italic</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">{modKey}+I</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Inline code</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">{modKey}+E</kbd>
					</div>
					<div class="col-span-2 mt-3 mb-1 text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Navigation</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Quick switcher</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">{modKey}+K</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Search messages</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">{modKey}+F</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Show shortcuts</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">?</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Jump to latest</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">End</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Scroll to top</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">Home</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Focus message input</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">{modKey}+T</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Mark all read</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">Shift+Esc</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Close modal</span>
						<kbd class="rounded bg-white/10 px-1.5 py-0.5 text-xs font-mono text-[var(--text-primary)]">Esc</kbd>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-[var(--text-secondary)]">Upload file</span>
						<span class="text-xs text-[var(--text-secondary)]">Paste image</span>
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- User Profile Card -->
	{#if profileCardUserId}
		<UserProfileCard
			userId={profileCardUserId}
			communityId={communityStore.activeCommunityId ?? undefined}
			channelId={channelStore.activeChannelId ?? undefined}
			anchorRect={profileCardAnchor}
			onclose={closeProfileCard}
			onstartdm={startDmFromProfileCard}
			blockedIds={blockedUserIds}
			canModerate={isCommunityModeratorOrAbove()}
		/>
	{/if}

	<!-- Group Settings Card -->
	{#if groupSettingsGroup}
		<GroupSettingsCard
			group={groupSettingsGroup}
			myRole={getMyGroupRole(groupSettingsGroup.id)}
			anchorRect={groupSettingsAnchor}
			onclose={closeGroupSettings}
			isCommunityModerator={isCommunityModeratorOrAbove()}
			assignedMemberName={groupSettingsGroup.assigned_member_id ? (userStore.getUser(groupSettingsGroup.assigned_member_id)?.display_name ?? userStore.getUser(groupSettingsGroup.assigned_member_id)?.username) : undefined}
			ondeleted={() => {
				const gid = groupSettingsGroup?.id;
				closeGroupSettings();
				if (gid) {
					groupStore.removeGroup(gid);
					channelStore.removeChannelsForGroup(gid);
				}
			}}
			onleft={() => {
				const gid = groupSettingsGroup?.id;
				closeGroupSettings();
				if (gid) {
					groupStore.removeGroup(gid);
					channelStore.removeChannelsForGroup(gid);
				}
			}}
			oninvitecreated={() => {}}
		/>
	{/if}

	<!-- Channel Settings Card -->
	{#if channelSettingsChannel && channelSettingsGroupId}
		<ChannelSettingsCard
			channel={channelSettingsChannel}
			groupId={channelSettingsGroupId}
			myRole={getMyGroupRole(channelSettingsGroupId)}
			anchorRect={channelSettingsAnchor}
			onclose={closeChannelSettings}
			ondeleted={() => {
				const cid = channelSettingsChannel?.id;
				closeChannelSettings();
				if (cid) channelStore.removeChannel(cid);
			}}
			onupdated={(ch) => {
				channelStore.updateChannel(ch);
			}}
		/>
	{/if}

	<!-- Image Lightbox -->
	<!-- Quick Switcher (Ctrl+K) -->
	{#if showQuickSwitcher}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-[100] flex items-start justify-center bg-black/50 pt-[15vh]"
			onclick={() => showQuickSwitcher = false}
			onkeydown={(e) => { if (e.key === 'Escape') showQuickSwitcher = false; }}
			transition:fade={{ duration: 100 }}
		>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				role="dialog"
				aria-modal="true"
				aria-label="Quick switcher"
				class="w-full max-w-lg rounded-2xl bg-[var(--bg-secondary)] shadow-xl"
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
				transition:fly={{ y: -20, duration: 150 }}
			>
				<div class="flex items-center gap-3 border-b border-white/10 px-4 py-3">
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 shrink-0 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg>
					<!-- svelte-ignore a11y_autofocus -->
					<input
						bind:this={quickSwitcherInputEl}
						bind:value={quickSwitcherQuery}
						oninput={() => { quickSwitcherIndex = 0; }}
						onkeydown={handleQuickSwitcherKeydown}
						aria-label="Search channels and DMs"
						placeholder="Jump to a channel or DM..."
						autofocus
						class="flex-1 bg-transparent text-[var(--text-primary)] outline-none placeholder:text-[var(--text-secondary)]/50"
					/>
					<kbd class="rounded bg-white/5 px-1.5 py-0.5 text-[10px] text-[var(--text-secondary)]">ESC</kbd>
				</div>
				<div class="max-h-72 overflow-y-auto py-1">
					{#if quickSwitcherResults.length === 0}
						<div class="px-4 py-6 text-center text-sm text-[var(--text-secondary)]">No results found</div>
					{:else}
						{#each quickSwitcherResults as item, i (item.id)}
							<button
								onclick={() => quickSwitcherSelect(item)}
								onmouseenter={() => quickSwitcherIndex = i}
								class="flex w-full items-center gap-3 px-4 py-2 text-left text-sm transition {i === quickSwitcherIndex ? 'bg-[var(--accent)]/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
							>
								<span class="w-5 text-center {item.icon === '@' ? 'text-[var(--accent)]' : 'text-[var(--text-secondary)]'}">{item.icon}</span>
								<span class="flex-1 truncate" title={item.name}>{item.name}</span>
								{#if item.groupName}
									<span class="truncate text-xs text-[var(--text-secondary)]/60" title={item.groupName}>{item.groupName}</span>
								{/if}
								<span class="text-xs text-[var(--text-secondary)]/40">{item.type === 'dm' ? 'DM' : item.type === 'group-channel' ? 'Channel' : 'Channel'}</span>
							</button>
						{/each}
					{/if}
				</div>
			</div>
		</div>
	{/if}

	{#if lightboxImage}
		{@const imgIdx = channelImages.findIndex(i => i.src === lightboxImage!.src)}
		{@const hasPrev = imgIdx > 0}
		{@const hasNext = imgIdx >= 0 && imgIdx < channelImages.length - 1}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-[100] flex items-center justify-center bg-black/80 backdrop-blur-sm"
			role="dialog"
			aria-label="Image lightbox"
			onclick={closeLightbox}
			onkeydown={(e) => { if (e.key === 'Escape') closeLightbox(); }}
			transition:fade={{ duration: 150 }}
		>
			<!-- Top toolbar -->
			<div class="absolute right-4 top-4 flex items-center gap-2">
				<a
					href={lightboxImage.src}
					download={lightboxImage.alt}
					onclick={(e) => e.stopPropagation()}
					class="rounded-full bg-black/50 p-2 text-white transition hover:bg-black/70"
					title="Download"
					aria-label="Download image"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
					</svg>
				</a>
				<button
					onclick={closeLightbox}
					class="rounded-full bg-black/50 p-2 text-white transition hover:bg-black/70"
					title="Close"
					aria-label="Close lightbox"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>

			<!-- Prev button -->
			{#if hasPrev}
				<button
					onclick={(e) => { e.stopPropagation(); lightboxPrev(); }}
					class="absolute left-4 top-1/2 -translate-y-1/2 rounded-full bg-black/50 p-3 text-white transition hover:bg-black/70"
					title="Previous image"
					aria-label="Previous image"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
				</button>
			{/if}

			<!-- Next button -->
			{#if hasNext}
				<button
					onclick={(e) => { e.stopPropagation(); lightboxNext(); }}
					class="absolute right-4 top-1/2 -translate-y-1/2 rounded-full bg-black/50 p-3 text-white transition hover:bg-black/70"
					title="Next image"
					aria-label="Next image"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
				</button>
			{/if}

			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<img
				src={lightboxImage.src}
				alt={lightboxImage.alt}
				class="max-h-[90vh] max-w-[90vw] rounded-lg shadow-2xl"
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
			/>

			<!-- Bottom bar: filename + counter -->
			<div class="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-3 rounded-full bg-black/50 px-4 py-1.5 text-sm text-white/80 max-w-[90vw]">
				<span class="truncate">{lightboxImage.alt}</span>
				{#if imgIdx >= 0 && channelImages.length > 1}
					<span class="text-white/40">{imgIdx + 1} / {channelImages.length}</span>
				{/if}
			</div>
		</div>
	{/if}

	<!-- Notification Permission Prompt -->
	{#if showNotifPrompt}
		<div class="fixed bottom-4 right-4 z-[90] w-[calc(100vw-2rem)] max-w-xs sm:w-80 rounded-2xl bg-[var(--bg-secondary)] p-4 shadow-xl" transition:fly={{ y: 20, duration: 200 }}>
			<div class="mb-2 flex items-start gap-3">
				<div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-[var(--accent)]/20">
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" /><path d="M13.73 21a2 2 0 0 1-3.46 0" />
					</svg>
				</div>
				<div>
					<h4 class="text-sm font-semibold text-[var(--text-primary)]">Enable notifications?</h4>
					<p class="mt-0.5 text-xs text-[var(--text-secondary)]">Get notified when you receive new messages, even when this tab is in the background.</p>
				</div>
			</div>
			<div class="flex justify-end gap-2">
				<button
					onclick={dismissNotifPrompt}
					class="rounded-lg px-3 py-1.5 text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
				>
					No thanks
				</button>
				<button
					onclick={acceptNotifPrompt}
					class="rounded-lg bg-[var(--accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)]"
				>
					Enable
				</button>
			</div>
		</div>
	{/if}

	<!-- Voice context menu (volume + kick) -->
	{#if voiceContextMenu}
		<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
		<div
			class="fixed inset-0 z-40"
			onclick={() => voiceContextMenu = null} onkeydown={(e) => { if (e.key === "Escape") voiceContextMenu = null; }} role="presentation"
			oncontextmenu={(e) => { e.preventDefault(); voiceContextMenu = null; }}
		></div>
		<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
		<div role="menu" tabindex="-1"
			class="fixed z-50 w-56 rounded-xl bg-[var(--bg-secondary)] p-3 shadow-lg"
			style="left: {voiceContextMenu.x}px; top: {voiceContextMenu.y}px;"
			onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}
		>
			{#if isVoiceMenuSelf}
				<!-- Self: mic gain control -->
				<div class="mb-2 flex items-center gap-2 text-xs font-medium text-[var(--text-primary)]">
					<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" /><path d="M19 10v2a7 7 0 0 1-14 0v-2" /><line x1="12" y1="19" x2="12" y2="23" /><line x1="8" y1="23" x2="16" y2="23" />
					</svg>
					Mic Volume
				</div>
				<div class="mb-1 text-[10px] text-[var(--text-secondary)]">What others hear from you</div>
				<div class="flex items-center gap-2">
					<input
						type="range"
						min="0"
						max="200"
						value={preferencesStore.preferences.inputGain}
						oninput={(e) => webrtcManager.setMicGain(parseInt(e.currentTarget.value))}
						class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-white/10 accent-[var(--accent)]"
					/>
					<span class="w-10 text-right text-xs font-medium text-[var(--text-secondary)]">
						{preferencesStore.preferences.inputGain}%
					</span>
				</div>
				{#if preferencesStore.preferences.inputGain !== 100}
					<button
						onclick={() => webrtcManager.setMicGain(100)}
						class="mt-2 w-full rounded px-2 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Reset to 100%
					</button>
				{/if}
			{:else}
				<!-- Remote: playback volume control -->
				<div class="mb-2 flex items-center gap-2 text-xs font-medium text-[var(--text-primary)]">
					<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" /><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" />
					</svg>
					{userStore.getDisplayName(voiceContextMenu.userId)}
				</div>
				<div class="flex items-center gap-2">
					<input
						type="range"
						min="0"
						max="500"
						value={voiceStore.getUserVolume(voiceContextMenu.userId)}
						oninput={(e) => { if (voiceContextMenu) voiceStore.setUserVolume(voiceContextMenu.userId, parseInt(e.currentTarget.value)); }}
						class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-white/10 accent-[var(--accent)]"
					/>
					<span class="w-10 text-right text-xs font-medium text-[var(--text-secondary)]">
						{voiceStore.getUserVolume(voiceContextMenu.userId)}%
					</span>
				</div>
				{#if voiceStore.getUserVolume(voiceContextMenu.userId) !== 100}
					<button
						onclick={() => { if (voiceContextMenu) voiceStore.setUserVolume(voiceContextMenu.userId, 100); }}
						class="mt-2 w-full rounded px-2 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Reset to 100%
					</button>
				{/if}
				{#if canKickInChannel(voiceContextMenu.channelId)}
					<div class="my-1.5 border-t border-white/10"></div>
					<button
						onclick={() => { if (voiceContextMenu) { handleVoiceKick(voiceContextMenu.userId, voiceContextMenu.channelId); } }}
						class="flex w-full items-center gap-2 rounded px-2 py-1 text-xs text-[var(--danger)] transition hover:bg-white/5"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" /><circle cx="8.5" cy="7" r="4" /><line x1="18" y1="8" x2="23" y2="13" /><line x1="23" y1="8" x2="18" y2="13" />
						</svg>
						Kick from voice
					</button>
				{/if}
			{/if}
		</div>
	{/if}

	<!-- Edit History Modal -->
	{#if showEditHistory}
		<div class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 p-4" transition:fade={{ duration: 150 }} onclick={() => showEditHistory = false} onkeydown={(e) => { if (e.key === 'Escape') showEditHistory = false; }} role="dialog" aria-modal="true" aria-label="Edit History" tabindex="-1">
			<div class="w-full max-w-lg max-h-[90vh] overflow-y-auto rounded-2xl bg-[var(--bg-secondary)] p-6 shadow-xl" onclick={(e) => e.stopPropagation()}>
				<div class="flex items-center justify-between mb-4">
					<h3 class="text-lg font-semibold text-[var(--text-primary)]">Edit History</h3>
					<button onclick={() => showEditHistory = false} class="rounded p-1 text-[var(--text-secondary)] hover:bg-white/10 hover:text-[var(--text-primary)] transition">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
					</button>
				</div>
				{#if editHistoryLoading}
					<div class="flex items-center justify-center py-8">
						<div class="h-6 w-6 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
					</div>
				{:else if editHistoryEntries.length === 0}
					<p class="text-center text-sm text-[var(--text-secondary)] py-8">No previous versions found.</p>
				{:else}
					<div class="space-y-3 max-h-96 overflow-y-auto">
						{#each editHistoryEntries as entry, i}
							<div class="rounded-lg bg-[var(--bg-primary)] p-3 border border-white/5">
								<div class="flex items-center justify-between mb-1.5">
									<span class="text-xs font-medium text-[var(--text-secondary)]">Version {editHistoryEntries.length - i}</span>
									<span class="text-xs text-[var(--text-secondary)]">{formatFullTimestamp(entry.editedAt)}</span>
								</div>
								<p class="text-sm text-[var(--text-primary)] whitespace-pre-wrap break-words">{entry.content}</p>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	{/if}

	<!-- What's New changelog modal -->
	<WhatsNew bind:this={whatsNewRef} />
{/if}
