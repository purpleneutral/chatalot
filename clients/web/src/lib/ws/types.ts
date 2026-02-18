// Mirror of the Rust WebSocket message types

export type ClientMessage =
	| { type: 'authenticate'; token: string }
	| { type: 'send_message'; channel_id: string; ciphertext: number[]; nonce: number[]; message_type: 'text' | 'file' | 'system'; reply_to: string | null; sender_key_id: string | null; thread_id?: string | null }
	| { type: 'edit_message'; message_id: string; ciphertext: number[]; nonce: number[] }
	| { type: 'delete_message'; message_id: string }
	| { type: 'update_presence'; status: 'online' | 'idle' | 'dnd' | 'invisible' }
	| { type: 'typing'; channel_id: string }
	| { type: 'stop_typing'; channel_id: string }
	| { type: 'subscribe'; channel_ids: string[] }
	| { type: 'unsubscribe'; channel_ids: string[] }
	| { type: 'rtc_offer'; target_user_id: string; session_id: string; sdp: string }
	| { type: 'rtc_answer'; target_user_id: string; session_id: string; sdp: string }
	| { type: 'rtc_ice_candidate'; target_user_id: string; session_id: string; candidate: string }
	| { type: 'join_voice'; channel_id: string }
	| { type: 'leave_voice'; channel_id: string }
	| { type: 'kick_from_voice'; channel_id: string; user_id: string }
	| { type: 'add_reaction'; message_id: string; emoji: string }
	| { type: 'remove_reaction'; message_id: string; emoji: string }
	| { type: 'mark_read'; channel_id: string; message_id: string }
	| { type: 'mark_all_read' }
	| { type: 'ping'; timestamp: number };

export type ServerMessage =
	| { type: 'authenticated'; user_id: string; server_version: string }
	| { type: 'new_message'; id: string; channel_id: string; sender_id: string | null; ciphertext: number[]; nonce: number[]; message_type: 'text' | 'file' | 'system'; reply_to: string | null; sender_key_id: string | null; created_at: string; thread_id?: string | null }
	| { type: 'message_sent'; id: string; channel_id: string; created_at: string; thread_id?: string | null }
	| { type: 'message_edited'; message_id: string; channel_id: string; sender_id: string | null; ciphertext: number[]; nonce: number[]; edited_at: string }
	| { type: 'message_deleted'; message_id: string }
	| { type: 'presence_update'; user_id: string; status: 'online' | 'idle' | 'dnd' | 'invisible' | 'offline' }
	| { type: 'user_typing'; channel_id: string; user_id: string }
	| { type: 'user_stopped_typing'; channel_id: string; user_id: string }
	| { type: 'rtc_offer'; from_user_id: string; session_id: string; sdp: string }
	| { type: 'rtc_answer'; from_user_id: string; session_id: string; sdp: string }
	| { type: 'rtc_ice_candidate'; from_user_id: string; session_id: string; candidate: string }
	| { type: 'voice_state_update'; channel_id: string; participants: string[] }
	| { type: 'user_joined_voice'; channel_id: string; user_id: string }
	| { type: 'user_left_voice'; channel_id: string; user_id: string }
	| { type: 'kicked_from_voice'; channel_id: string; user_id: string; kicked_by: string }
	| { type: 'reaction_added'; message_id: string; user_id: string; emoji: string }
	| { type: 'reaction_removed'; message_id: string; user_id: string; emoji: string }
	| { type: 'member_kicked'; channel_id: string; user_id: string; kicked_by: string }
	| { type: 'member_banned'; channel_id: string; user_id: string; banned_by: string }
	| { type: 'member_role_updated'; channel_id: string; user_id: string; role: string }
	| { type: 'new_dm_channel'; channel_id: string; channel_name: string | null; created_at: string; other_user_id: string; other_user_username: string; other_user_display_name: string | null; other_user_avatar_url: string | null }
	| { type: 'message_pinned'; message_id: string; channel_id: string; pinned_by: string; pinned_at: string }
	| { type: 'message_unpinned'; message_id: string; channel_id: string }
	| { type: 'sender_key_updated'; channel_id: string; user_id: string; chain_id: number; distribution: object }
	| { type: 'sender_key_rotation_required'; channel_id: string; reason: string }
	| { type: 'user_timed_out'; channel_id: string; user_id: string; expires_at: string; reason: string | null }
	| { type: 'poll_created'; poll_id: string; channel_id: string; created_by: string; question: string }
	| { type: 'poll_voted'; poll_id: string; channel_id: string; option_index: number; voter_id: string | null }
	| { type: 'poll_vote_removed'; poll_id: string; channel_id: string; option_index: number; voter_id: string | null }
	| { type: 'poll_closed'; poll_id: string; channel_id: string }
	| { type: 'user_warned'; channel_id: string; user_id: string; reason: string; warning_count: number }
	| { type: 'user_profile_updated'; user_id: string; display_name: string; avatar_url: string | null; banner_url: string | null; custom_status: string | null; bio: string | null; pronouns: string | null }
	| { type: 'channel_updated'; channel_id: string; name: string | null; topic: string | null; read_only: boolean; slow_mode_seconds: number; archived: boolean; voice_background: string | null }
	| { type: 'group_updated'; group_id: string; name: string; description: string | null; icon_url: string | null; banner_url: string | null; accent_color: string | null; visibility: string }
	| { type: 'community_updated'; community_id: string; name: string; description: string | null; icon_url: string | null; banner_url: string | null; community_theme: object | null; welcome_message: string | null }
	| { type: 'channel_deleted'; channel_id: string }
	| { type: 'group_deleted'; group_id: string }
	| { type: 'announcement'; id: string; title: string; body: string; created_by: string; created_at: string }
	| { type: 'read_receipt'; channel_id: string; user_id: string; message_id: string; timestamp: string }
	| { type: 'error'; code: string; message: string }
	| { type: 'pong'; timestamp: number }
	| { type: 'keys_low'; remaining: number };
