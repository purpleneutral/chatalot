use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages sent from client to server over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    // Authentication (first message after WS upgrade)
    Authenticate {
        token: String,
    },

    // Messaging
    SendMessage {
        channel_id: Uuid,
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
        message_type: MessageType,
        reply_to: Option<Uuid>,
        sender_key_id: Option<Uuid>,
        #[serde(default)]
        thread_id: Option<Uuid>,
    },
    EditMessage {
        message_id: Uuid,
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
    },
    DeleteMessage {
        message_id: Uuid,
    },

    // Presence
    UpdatePresence {
        status: PresenceStatus,
    },
    Typing {
        channel_id: Uuid,
    },
    StopTyping {
        channel_id: Uuid,
    },

    // Channel subscriptions
    Subscribe {
        channel_ids: Vec<Uuid>,
    },
    Unsubscribe {
        channel_ids: Vec<Uuid>,
    },

    // WebRTC signaling
    RtcOffer {
        target_user_id: Uuid,
        session_id: Uuid,
        sdp: String,
    },
    RtcAnswer {
        target_user_id: Uuid,
        session_id: Uuid,
        sdp: String,
    },
    RtcIceCandidate {
        target_user_id: Uuid,
        session_id: Uuid,
        candidate: String,
    },

    // Voice/video
    JoinVoice {
        channel_id: Uuid,
    },
    LeaveVoice {
        channel_id: Uuid,
    },
    KickFromVoice {
        channel_id: Uuid,
        user_id: Uuid,
    },

    // Reactions
    AddReaction {
        message_id: Uuid,
        emoji: String,
    },
    RemoveReaction {
        message_id: Uuid,
        emoji: String,
    },

    // Unread tracking
    MarkRead {
        channel_id: Uuid,
        message_id: Uuid,
    },
    MarkAllRead,

    // Keepalive
    Ping {
        timestamp: i64,
    },
}

/// Messages sent from server to client over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    // Auth response
    Authenticated {
        user_id: Uuid,
        server_version: String,
    },

    // Messaging
    NewMessage {
        id: Uuid,
        channel_id: Uuid,
        sender_id: Uuid,
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
        message_type: MessageType,
        reply_to: Option<Uuid>,
        sender_key_id: Option<Uuid>,
        created_at: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        thread_id: Option<Uuid>,
    },
    MessageEdited {
        message_id: Uuid,
        channel_id: Uuid,
        sender_id: Uuid,
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
        edited_at: String,
    },
    MessageDeleted {
        message_id: Uuid,
    },

    // Confirmations
    MessageSent {
        /// The client-facing ID so they can reconcile optimistic sends.
        id: Uuid,
        channel_id: Uuid,
        created_at: String,
    },

    // Presence
    PresenceUpdate {
        user_id: Uuid,
        status: PresenceStatus,
    },
    UserTyping {
        channel_id: Uuid,
        user_id: Uuid,
    },
    UserStoppedTyping {
        channel_id: Uuid,
        user_id: Uuid,
    },

    // WebRTC signaling
    RtcOffer {
        from_user_id: Uuid,
        session_id: Uuid,
        sdp: String,
    },
    RtcAnswer {
        from_user_id: Uuid,
        session_id: Uuid,
        sdp: String,
    },
    RtcIceCandidate {
        from_user_id: Uuid,
        session_id: Uuid,
        candidate: String,
    },

    // Voice/video
    VoiceStateUpdate {
        channel_id: Uuid,
        participants: Vec<Uuid>,
    },
    UserJoinedVoice {
        channel_id: Uuid,
        user_id: Uuid,
    },
    UserLeftVoice {
        channel_id: Uuid,
        user_id: Uuid,
    },
    KickedFromVoice {
        channel_id: Uuid,
        user_id: Uuid,
        kicked_by: Uuid,
    },

    // Reactions
    ReactionAdded {
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },
    ReactionRemoved {
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },

    // Read receipts
    ReadReceipt {
        channel_id: Uuid,
        user_id: Uuid,
        message_id: Uuid,
        timestamp: String,
    },

    // Channel moderation
    MemberKicked {
        channel_id: Uuid,
        user_id: Uuid,
        kicked_by: Uuid,
    },
    MemberBanned {
        channel_id: Uuid,
        user_id: Uuid,
        banned_by: Uuid,
    },
    MemberRoleUpdated {
        channel_id: Uuid,
        user_id: Uuid,
        role: String,
    },

    // Pinned messages
    MessagePinned {
        message_id: Uuid,
        channel_id: Uuid,
        pinned_by: Uuid,
        pinned_at: String,
    },
    MessageUnpinned {
        message_id: Uuid,
        channel_id: Uuid,
    },

    // DM notifications
    NewDmChannel {
        channel_id: Uuid,
        channel_name: Option<String>,
        created_at: String,
        other_user_id: Uuid,
        other_user_username: String,
        other_user_display_name: Option<String>,
        other_user_avatar_url: Option<String>,
    },

    // Sender Keys (Group E2E)
    SenderKeyUpdated {
        channel_id: Uuid,
        user_id: Uuid,
        chain_id: i32,
        distribution: serde_json::Value,
    },
    SenderKeyRotationRequired {
        channel_id: Uuid,
        reason: String,
    },

    // Polls
    PollCreated {
        poll_id: Uuid,
        channel_id: Uuid,
        created_by: Uuid,
        question: String,
    },
    PollVoted {
        poll_id: Uuid,
        channel_id: Uuid,
        option_index: i32,
        voter_id: Option<Uuid>,
    },
    PollClosed {
        poll_id: Uuid,
        channel_id: Uuid,
    },

    // Moderation
    UserTimedOut {
        channel_id: Uuid,
        user_id: Uuid,
        expires_at: String,
        reason: Option<String>,
    },
    UserWarned {
        channel_id: Uuid,
        user_id: Uuid,
        reason: String,
        warning_count: i64,
    },

    // User profile changes
    UserProfileUpdated {
        user_id: Uuid,
        display_name: String,
        avatar_url: Option<String>,
        banner_url: Option<String>,
        custom_status: Option<String>,
        bio: Option<String>,
        pronouns: Option<String>,
    },

    // Channel/group settings changes
    ChannelUpdated {
        channel_id: Uuid,
        name: Option<String>,
        topic: Option<String>,
        read_only: bool,
        slow_mode_seconds: i32,
        archived: bool,
        voice_background: Option<String>,
    },
    GroupUpdated {
        group_id: Uuid,
        name: String,
        description: Option<String>,
        icon_url: Option<String>,
        banner_url: Option<String>,
        accent_color: Option<String>,
        visibility: String,
    },
    CommunityUpdated {
        community_id: Uuid,
        name: String,
        description: Option<String>,
        icon_url: Option<String>,
        banner_url: Option<String>,
        community_theme: Option<serde_json::Value>,
        welcome_message: Option<String>,
    },
    ChannelDeleted {
        channel_id: Uuid,
    },
    GroupDeleted {
        group_id: Uuid,
    },

    // Announcements
    Announcement {
        id: Uuid,
        title: String,
        body: String,
        created_by: Uuid,
        created_at: String,
    },

    // System
    Error {
        code: String,
        message: String,
    },
    Pong {
        timestamp: i64,
    },
    KeysLow {
        remaining: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    File,
    System,
    Webhook,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    Online,
    Idle,
    Dnd,
    Invisible,
    Offline,
}
