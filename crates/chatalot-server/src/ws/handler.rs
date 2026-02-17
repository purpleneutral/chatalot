use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use chatalot_common::ws_messages::{ClientMessage, MessageType, ServerMessage};
use chatalot_db::models::channel::ChannelType;
use chatalot_db::repos::{
    block_repo, channel_repo, community_repo, message_repo, reaction_repo, timeout_repo,
    unread_repo, user_repo, voice_repo,
};

use crate::permissions;

use crate::app_state::AppState;
use crate::ws::connection_manager::SessionHandle;

/// Handle an authenticated WebSocket connection.
pub async fn handle_socket(socket: WebSocket, user_id: Uuid, state: Arc<AppState>) {
    let conn_mgr = &state.connections;
    let session_id = Uuid::new_v4();
    let (mut ws_sink, mut ws_stream) = socket.split();

    // Channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Register the session (enforces per-user connection limit)
    let handle = SessionHandle {
        session_id,
        user_id,
        tx: tx.clone(),
    };
    if !conn_mgr.add_session(handle) {
        tracing::warn!(%user_id, "WebSocket rejected: too many concurrent sessions");
        let _ = tx.send(ServerMessage::Error {
            code: "too_many_sessions".to_string(),
            message: "too many concurrent connections, close another device first".to_string(),
        });
        return;
    }

    // Broadcast presence: this user is online
    broadcast_presence(&state.db, conn_mgr, user_id, "online").await;

    tracing::info!(%user_id, %session_id, "WebSocket connected");

    // Writer task: forwards messages from the mpsc channel to the WebSocket
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(text) => {
                    if ws_sink.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!(%user_id, "failed to serialize outgoing WS message: {e}");
                }
            }
        }
    });

    // Heartbeat: server sends ping every 15 seconds (keeps proxies/tunnels alive)
    let heartbeat_tx = tx.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            let timestamp = chrono::Utc::now().timestamp();
            if heartbeat_tx
                .send(ServerMessage::Pong { timestamp })
                .is_err()
            {
                break;
            }
        }
    });

    // Track spawned subscription tasks so we can abort them on disconnect
    let mut subscription_tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    // Per-connection reaction cooldown tracker
    let mut last_reaction_time = tokio::time::Instant::now() - tokio::time::Duration::from_secs(1);

    // Maximum incoming WebSocket message size (1 MB)
    const MAX_WS_MESSAGE_SIZE: usize = 1_048_576;

    // Per-connection message rate limiter (token bucket: 10 msg/s burst, refills at 5/s)
    const RATE_LIMIT_BURST: f64 = 10.0;
    const RATE_LIMIT_REFILL: f64 = 5.0;
    let mut tokens: f64 = RATE_LIMIT_BURST;
    let mut last_refill = tokio::time::Instant::now();

    // Reader task: processes incoming WebSocket messages
    while let Some(Ok(msg)) = ws_stream.next().await {
        match msg {
            Message::Text(text) => {
                // Reject oversized messages
                if text.len() > MAX_WS_MESSAGE_SIZE {
                    let _ = tx.send(ServerMessage::Error {
                        code: "validation_error".to_string(),
                        message: "message too large".to_string(),
                    });
                    continue;
                }

                // Refill tokens
                let now = tokio::time::Instant::now();
                let elapsed = now.duration_since(last_refill).as_secs_f64();
                tokens = (tokens + elapsed * RATE_LIMIT_REFILL).min(RATE_LIMIT_BURST);
                last_refill = now;

                if tokens < 1.0 {
                    let _ = tx.send(ServerMessage::Error {
                        code: "rate_limited".to_string(),
                        message: "too many messages, slow down".to_string(),
                    });
                    continue;
                }
                tokens -= 1.0;

                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        handle_client_message(
                            client_msg,
                            user_id,
                            &state,
                            &tx,
                            &mut subscription_tasks,
                            &mut last_reaction_time,
                        )
                        .await;
                    }
                    Err(e) => {
                        tracing::warn!(%user_id, error = %e, "malformed WebSocket message");
                        let _ = tx.send(ServerMessage::Error {
                            code: "invalid_message".to_string(),
                            message: "malformed message".to_string(),
                        });
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Cleanup
    conn_mgr.remove_session(user_id, session_id);
    write_task.abort();
    heartbeat_task.abort();

    // Abort all channel subscription tasks
    for task in &subscription_tasks {
        task.abort();
    }

    // Clean up typing state — broadcast stop-typing for all channels this user was typing in
    let typing_channels = conn_mgr.clear_all_typing_for_user(user_id);
    for channel_id in typing_channels {
        conn_mgr.broadcast_to_channel(
            channel_id,
            ServerMessage::UserStoppedTyping {
                channel_id,
                user_id,
            },
        );
    }

    // Clean up voice sessions — delay cleanup to allow quick reconnects (e.g. tunnel flaps).
    // Record the disconnect time so we only clean up sessions that were active BEFORE this
    // disconnect. If the user reconnects and rejoins voice (getting a new joined_at), their
    // new session is preserved even if this timer fires.
    {
        let state = Arc::clone(&state);
        let disconnect_time = chrono::Utc::now();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;

            // If user reconnected in the meantime, skip voice cleanup entirely
            if state.connections.is_online(&user_id) {
                tracing::debug!(%user_id, "Skipping voice cleanup — user reconnected");
                return;
            }

            // Only leave sessions the user joined before the disconnect.
            // If they rejoined during the grace period (new joined_at > disconnect_time),
            // those sessions are preserved.
            match voice_repo::leave_sessions_joined_before(&state.db, user_id, disconnect_time).await {
                Ok(left_sessions) => {
                    for (voice_session_id, channel_id) in &left_sessions {
                        state.connections.broadcast_to_channel(
                            *channel_id,
                            ServerMessage::UserLeftVoice {
                                channel_id: *channel_id,
                                user_id,
                            },
                        );

                        if let Ok(participants) =
                            voice_repo::get_participants(&state.db, *voice_session_id).await
                        {
                            state.connections.broadcast_to_channel(
                                *channel_id,
                                ServerMessage::VoiceStateUpdate {
                                    channel_id: *channel_id,
                                    participants: participants.clone(),
                                },
                            );

                            if participants.is_empty() {
                                let _ =
                                    voice_repo::end_session(&state.db, *voice_session_id).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(%user_id, "Failed to clean up voice sessions: {e}");
                }
            }
        });
    }

    // If no more sessions for this user, broadcast offline (checked after grace period
    // by the voice cleanup task above — also check immediately for non-voice users)
    if !conn_mgr.is_online(&user_id) {
        broadcast_presence(&state.db, conn_mgr, user_id, "offline").await;
    }

    tracing::info!(%user_id, %session_id, "WebSocket disconnected");
}

async fn handle_client_message(
    msg: ClientMessage,
    user_id: Uuid,
    state: &AppState,
    tx: &mpsc::UnboundedSender<ServerMessage>,
    subscription_tasks: &mut Vec<tokio::task::JoinHandle<()>>,
    last_reaction_time: &mut tokio::time::Instant,
) {
    let conn_mgr = &state.connections;
    match msg {
        ClientMessage::Ping { timestamp } => {
            let _ = tx.send(ServerMessage::Pong { timestamp });
        }

        ClientMessage::SendMessage {
            channel_id,
            ciphertext,
            nonce,
            message_type,
            reply_to,
            sender_key_id,
            thread_id,
        } => {
            // Reject empty or oversized ciphertext (64 KiB limit)
            const MAX_CIPHERTEXT_SIZE: usize = 65_536;
            if ciphertext.is_empty() || ciphertext.len() > MAX_CIPHERTEXT_SIZE {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: if ciphertext.is_empty() {
                        "message cannot be empty".to_string()
                    } else {
                        "message too large".to_string()
                    },
                });
                return;
            }

            // Verify membership
            match channel_repo::is_member(&state.db, channel_id, user_id).await {
                Ok(true) => {}
                _ => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "forbidden".to_string(),
                        message: "not a member of this channel".to_string(),
                    });
                    return;
                }
            }

            // Fetch channel for permission checks
            let channel = match channel_repo::get_channel(&state.db, channel_id).await {
                Ok(Some(ch)) => ch,
                Ok(None) => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "not_found".to_string(),
                        message: "channel not found".to_string(),
                    });
                    return;
                }
                Err(e) => {
                    tracing::error!("Failed to fetch channel: {e}");
                    return;
                }
            };

            // Check role for non-DM channels (admins/owners exempt from slow mode, read-only)
            let is_privileged = if channel.channel_type != ChannelType::Dm {
                let role = channel_repo::get_member_role(&state.db, channel_id, user_id)
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                matches!(role.as_str(), "owner" | "admin")
            } else {
                false
            };

            if channel.channel_type != ChannelType::Dm {
                if channel.archived {
                    let _ = tx.send(ServerMessage::Error {
                        code: "archived".to_string(),
                        message: "this channel is archived".to_string(),
                    });
                    return;
                }

                if channel.read_only && !is_privileged {
                    let _ = tx.send(ServerMessage::Error {
                        code: "read_only".to_string(),
                        message: "this channel is read-only".to_string(),
                    });
                    return;
                }

                if channel.slow_mode_seconds > 0
                    && !is_privileged
                    && let Ok(Some(last_sent)) =
                        channel_repo::get_slowmode_last_sent(&state.db, channel_id, user_id).await
                {
                    let elapsed = (chrono::Utc::now() - last_sent).num_seconds().max(0);
                    if elapsed < channel.slow_mode_seconds as i64 {
                        let wait = (channel.slow_mode_seconds as i64 - elapsed).max(0);
                        let _ = tx.send(ServerMessage::Error {
                            code: "slow_mode".to_string(),
                            message: format!("slow mode: wait {wait} seconds"),
                        });
                        return;
                    }
                }

                // Check for active timeout
                if !is_privileged
                    && let Ok(Some(timeout)) =
                        timeout_repo::get_active_timeout(&state.db, user_id, channel_id).await
                {
                    let remaining = (timeout.expires_at - chrono::Utc::now())
                        .num_seconds()
                        .max(0);
                    let _ = tx.send(ServerMessage::Error {
                        code: "timed_out".to_string(),
                        message: format!("you are timed out for {remaining} more seconds"),
                    });
                    return;
                }
            }

            // For DM channels, check blocks and shared community
            if channel.channel_type == ChannelType::Dm {
                // Find the other user in this DM
                if let Ok(members) = channel_repo::list_members(&state.db, channel_id).await {
                    for member in &members {
                        if member.user_id != user_id {
                            // Check if either user has blocked the other
                            if let Ok(true) = block_repo::is_blocked_either_way(
                                &state.db,
                                user_id,
                                member.user_id,
                            )
                            .await
                            {
                                let _ = tx.send(ServerMessage::Error {
                                    code: "blocked".to_string(),
                                    message: "cannot send messages to this user".to_string(),
                                });
                                return;
                            }

                            match community_repo::shares_community(
                                &state.db,
                                user_id,
                                member.user_id,
                            )
                            .await
                            {
                                Ok(false) => {
                                    let _ = tx.send(ServerMessage::Error {
                                        code: "forbidden".to_string(),
                                        message: "you no longer share a community with this user"
                                            .to_string(),
                                    });
                                    return;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to check shared community: {e}");
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            // Validate reply_to references a message in the same channel
            if let Some(reply_id) = reply_to {
                match message_repo::get_message_by_id(&state.db, reply_id).await {
                    Ok(Some(reply_msg)) if reply_msg.channel_id == channel_id => {}
                    Ok(Some(_)) => {
                        let _ = tx.send(ServerMessage::Error {
                            code: "validation_error".to_string(),
                            message: "cannot reply to message from another channel".to_string(),
                        });
                        return;
                    }
                    _ => {
                        let _ = tx.send(ServerMessage::Error {
                            code: "validation_error".to_string(),
                            message: "replied-to message not found".to_string(),
                        });
                        return;
                    }
                }
            }

            // Validate and resolve thread_id — must be in same channel.
            // If the target message is itself a thread reply, follow to the real root.
            let resolved_thread_id = if let Some(tid) = thread_id {
                match message_repo::get_message_by_id(&state.db, tid).await {
                    Ok(Some(root_msg)) if root_msg.channel_id == channel_id => {
                        // If the "root" is itself in a thread, follow to the real root
                        Some(root_msg.thread_id.unwrap_or(tid))
                    }
                    Ok(Some(_)) => {
                        let _ = tx.send(ServerMessage::Error {
                            code: "validation_error".to_string(),
                            message: "thread root must be in the same channel".to_string(),
                        });
                        return;
                    }
                    _ => {
                        let _ = tx.send(ServerMessage::Error {
                            code: "validation_error".to_string(),
                            message: "thread root message not found".to_string(),
                        });
                        return;
                    }
                }
            } else {
                None
            };

            let msg_type_str = match message_type {
                MessageType::Text => "text",
                MessageType::File => "file",
                MessageType::System => "system",
                MessageType::Webhook => "webhook",
            };

            let message_id = Uuid::now_v7();

            // Defensive: prevent a message from referencing itself as its own thread root
            if resolved_thread_id == Some(message_id) {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: "message cannot be its own thread root".to_string(),
                });
                return;
            }

            // Compute expires_at if channel has a TTL configured
            let expires_at = channel
                .message_ttl_seconds
                .map(|ttl| chrono::Utc::now() + chrono::Duration::seconds(ttl as i64));

            // Persist the ciphertext
            match message_repo::create_message(
                &state.db,
                message_id,
                channel_id,
                user_id,
                &ciphertext,
                &nonce,
                msg_type_str,
                sender_key_id,
                reply_to,
                None,
                expires_at,
                resolved_thread_id,
            )
            .await
            {
                Ok(stored) => {
                    // Confirm to the sender
                    let _ = tx.send(ServerMessage::MessageSent {
                        id: message_id,
                        channel_id,
                        created_at: stored.created_at.to_rfc3339(),
                        thread_id: resolved_thread_id,
                    });

                    let new_msg = ServerMessage::NewMessage {
                        id: message_id,
                        channel_id,
                        sender_id: user_id,
                        ciphertext,
                        nonce,
                        message_type,
                        reply_to,
                        sender_key_id,
                        created_at: stored.created_at.to_rfc3339(),
                        thread_id: resolved_thread_id,
                    };

                    // For DM channels, deliver directly to the other member
                    // to avoid race conditions with subscription timing.
                    // For group channels, use broadcast as normal.
                    let is_dm = channel.channel_type == ChannelType::Dm;

                    if is_dm {
                        if let Ok(members) = channel_repo::list_members(&state.db, channel_id).await
                        {
                            // If this is the first message, notify the other user
                            // about the DM channel so it appears in their sidebar.
                            let is_first = message_repo::count_messages(&state.db, channel_id)
                                .await
                                .unwrap_or(0)
                                == 1;

                            for member in &members {
                                if member.user_id != user_id {
                                    if is_first
                                        && let Ok(Some(sender)) =
                                            user_repo::find_by_id(&state.db, user_id).await
                                        && let Ok(Some(ch)) =
                                            channel_repo::get_channel(&state.db, channel_id).await
                                    {
                                        conn_mgr.send_to_user(
                                            &member.user_id,
                                            &ServerMessage::NewDmChannel {
                                                channel_id,
                                                channel_name: ch.name.clone(),
                                                created_at: ch.created_at.to_rfc3339(),
                                                other_user_id: sender.id,
                                                other_user_username: sender.username.clone(),
                                                other_user_display_name: Some(
                                                    sender.display_name.clone(),
                                                ),
                                                other_user_avatar_url: sender.avatar_url.clone(),
                                            },
                                        );
                                    }
                                    conn_mgr.send_to_user(&member.user_id, &new_msg);
                                }
                            }
                        }
                    } else {
                        conn_mgr.broadcast_to_channel(channel_id, new_msg);
                    }

                    // Update slow mode tracker after successful send (skip for exempt users)
                    if channel.slow_mode_seconds > 0 && !is_privileged {
                        let _ =
                            channel_repo::update_slowmode_last_sent(&state.db, channel_id, user_id)
                                .await;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to persist message: {e}");
                    let _ = tx.send(ServerMessage::Error {
                        code: "internal_error".to_string(),
                        message: "failed to send message".to_string(),
                    });
                }
            }
        }

        ClientMessage::DeleteMessage { message_id } => {
            // Look up the message first to get channel_id for broadcast
            let msg_record = match message_repo::get_message_by_id(&state.db, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "not_found".to_string(),
                        message: "message not found".to_string(),
                    });
                    return;
                }
                Err(e) => {
                    tracing::error!("Failed to look up message: {e}");
                    return;
                }
            };

            let deleted = if msg_record.sender_id == Some(user_id) {
                // Own message — no role check needed
                message_repo::delete_message(&state.db, message_id, user_id).await
            } else {
                // Not own message — check mod permissions
                let role = channel_repo::get_member_role(&state.db, msg_record.channel_id, user_id)
                    .await
                    .ok()
                    .flatten();
                match role {
                    Some(ref r) if permissions::can_delete_others_messages(r) => {
                        message_repo::delete_message_as_mod(&state.db, message_id).await
                    }
                    _ => {
                        let _ = tx.send(ServerMessage::Error {
                            code: "forbidden".to_string(),
                            message: "you don't have permission to delete this message".to_string(),
                        });
                        return;
                    }
                }
            };

            match deleted {
                Ok(true) => {
                    conn_mgr.broadcast_to_channel(
                        msg_record.channel_id,
                        ServerMessage::MessageDeleted { message_id },
                    );
                }
                Ok(false) => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "not_found".to_string(),
                        message: "message not found or already deleted".to_string(),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to delete message: {e}");
                }
            }
        }

        ClientMessage::Typing { channel_id } => {
            if channel_repo::is_member(&state.db, channel_id, user_id)
                .await
                .unwrap_or(false)
            {
                // set_typing returns false if a broadcast was sent within the last 3 seconds
                if conn_mgr.set_typing(channel_id, user_id) {
                    conn_mgr.broadcast_to_channel(
                        channel_id,
                        ServerMessage::UserTyping {
                            channel_id,
                            user_id,
                        },
                    );
                }
            }
        }

        ClientMessage::StopTyping { channel_id } => {
            if channel_repo::is_member(&state.db, channel_id, user_id)
                .await
                .unwrap_or(false)
            {
                conn_mgr.clear_typing(channel_id, user_id);
                conn_mgr.broadcast_to_channel(
                    channel_id,
                    ServerMessage::UserStoppedTyping {
                        channel_id,
                        user_id,
                    },
                );
            }
        }

        ClientMessage::UpdatePresence { status } => {
            let status_str = format!("{:?}", status).to_lowercase();
            broadcast_presence(&state.db, conn_mgr, user_id, &status_str).await;
        }

        ClientMessage::Subscribe { channel_ids } => {
            // Subscribe this session to channel broadcasts.
            // Verify membership for each channel before subscribing.
            const MAX_SUBSCRIPTION_TASKS: usize = 500;
            if subscription_tasks.len() + channel_ids.len() > MAX_SUBSCRIPTION_TASKS {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: "too many active subscriptions".to_string(),
                });
                return;
            }
            if channel_ids.len() > 200 {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: "cannot subscribe to more than 200 channels at once".to_string(),
                });
                return;
            }
            for channel_id in channel_ids {
                let is_member = channel_repo::is_member(&state.db, channel_id, user_id)
                    .await
                    .unwrap_or(false);
                if !is_member {
                    continue;
                }

                // Send current voice state if there's an active voice session
                if let Ok(Some(session)) =
                    voice_repo::get_active_session(&state.db, channel_id).await
                    && let Ok(participants) =
                        voice_repo::get_participants(&state.db, session.id).await
                    && !participants.is_empty()
                {
                    let _ = tx.send(ServerMessage::VoiceStateUpdate {
                        channel_id,
                        participants,
                    });
                }

                let mut rx = conn_mgr.subscribe_channel(channel_id);
                let tx = tx.clone();
                let uid = user_id;
                subscription_tasks.push(tokio::spawn(async move {
                    loop {
                        match rx.recv().await {
                            Ok(msg) => {
                                // Don't echo messages back to the sender
                                if let ServerMessage::NewMessage { sender_id, .. } = &msg
                                    && *sender_id == uid
                                {
                                    continue;
                                }
                                if tx.send(msg).is_err() {
                                    break;
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                                tracing::warn!(%uid, %channel_id, skipped = n, "broadcast subscriber lagged");
                                let _ = tx.send(ServerMessage::Error {
                                    code: "out_of_sync".to_string(),
                                    message: "connection lagged, please refresh".to_string(),
                                });
                                break;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }));
            }
        }

        // Forward WebRTC signaling messages to the target user (validated)
        ClientMessage::RtcOffer {
            target_user_id,
            session_id,
            sdp,
        } => {
            if sdp.len() > 32_768 {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: "SDP too large".to_string(),
                });
                return;
            }
            // Validate both users are in the same voice session
            match voice_repo::are_in_same_session(&state.db, user_id, target_user_id).await {
                Ok(true) => {
                    tracing::info!(%user_id, %target_user_id, "RtcOffer: forwarding (same session confirmed)");
                }
                other => {
                    tracing::warn!(%user_id, %target_user_id, result = ?other, "RtcOffer: REJECTED (not in same session)");
                    let _ = tx.send(ServerMessage::Error {
                        code: "forbidden".to_string(),
                        message: "target user not in your voice session".to_string(),
                    });
                    return;
                }
            }
            conn_mgr.send_to_user(
                &target_user_id,
                &ServerMessage::RtcOffer {
                    from_user_id: user_id,
                    session_id,
                    sdp,
                },
            );
        }
        ClientMessage::RtcAnswer {
            target_user_id,
            session_id,
            sdp,
        } => {
            if sdp.len() > 32_768 {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: "SDP too large".to_string(),
                });
                return;
            }
            match voice_repo::are_in_same_session(&state.db, user_id, target_user_id).await {
                Ok(true) => {
                    tracing::info!(%user_id, %target_user_id, "RtcAnswer: forwarding (same session confirmed)");
                }
                other => {
                    tracing::warn!(%user_id, %target_user_id, result = ?other, "RtcAnswer: REJECTED (not in same session)");
                    let _ = tx.send(ServerMessage::Error {
                        code: "forbidden".to_string(),
                        message: "target user not in your voice session".to_string(),
                    });
                    return;
                }
            }
            conn_mgr.send_to_user(
                &target_user_id,
                &ServerMessage::RtcAnswer {
                    from_user_id: user_id,
                    session_id,
                    sdp,
                },
            );
        }
        ClientMessage::RtcIceCandidate {
            target_user_id,
            session_id,
            candidate,
        } => {
            if candidate.len() > 2048 {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: "ICE candidate too large".to_string(),
                });
                return;
            }
            match voice_repo::are_in_same_session(&state.db, user_id, target_user_id).await {
                Ok(true) => {}
                _ => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "forbidden".to_string(),
                        message: "target user not in your voice session".to_string(),
                    });
                    return;
                }
            }
            conn_mgr.send_to_user(
                &target_user_id,
                &ServerMessage::RtcIceCandidate {
                    from_user_id: user_id,
                    session_id,
                    candidate,
                },
            );
        }

        ClientMessage::JoinVoice { channel_id } => {
            // Verify membership
            match channel_repo::is_member(&state.db, channel_id, user_id).await {
                Ok(true) => {}
                _ => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "forbidden".to_string(),
                        message: "not a member of this channel".to_string(),
                    });
                    return;
                }
            }

            // Check participant cap (allow reconnection if already in the call)
            if let Ok(Some(session)) =
                voice_repo::get_active_session(&state.db, channel_id).await
            && let Ok(participants) =
                voice_repo::get_participants(&state.db, session.id).await
            && participants.len() >= 25
            && !participants.contains(&user_id)
            {
                let _ = tx.send(ServerMessage::Error {
                    code: "voice_full".to_string(),
                    message: "voice channel is full (max 25 participants)"
                        .to_string(),
                });
                return;
            }

            match voice_repo::get_or_create_session(&state.db, channel_id, user_id).await {
                Ok(session) => {
                    if let Err(e) = voice_repo::join_session(&state.db, session.id, user_id).await {
                        tracing::error!("Failed to join voice session: {e}");
                        let _ = tx.send(ServerMessage::Error {
                            code: "voice_error".to_string(),
                            message: "failed to join voice channel".to_string(),
                        });
                        return;
                    }

                    // Get current participants and broadcast
                    if let Ok(participants) =
                        voice_repo::get_participants(&state.db, session.id).await
                    {
                        tracing::info!(
                            %user_id, %channel_id,
                            participants = ?participants,
                            "JoinVoice: sending VoiceStateUpdate"
                        );

                        // Send directly to the joining user — they may not have
                        // a channel subscription yet (e.g. during WS reconnect
                        // where join_voice is sent before subscribe).
                        let _ = tx.send(ServerMessage::VoiceStateUpdate {
                            channel_id,
                            participants: participants.clone(),
                        });

                        // Broadcast full participant list to everyone in the channel
                        // so all clients can establish missing peer connections
                        conn_mgr.broadcast_to_channel(
                            channel_id,
                            ServerMessage::VoiceStateUpdate {
                                channel_id,
                                participants: participants.clone(),
                            },
                        );

                        // Also broadcast join event (for UI updates, sounds, etc.)
                        conn_mgr.broadcast_to_channel(
                            channel_id,
                            ServerMessage::UserJoinedVoice {
                                channel_id,
                                user_id,
                            },
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get/create voice session: {e}");
                    let _ = tx.send(ServerMessage::Error {
                        code: "voice_error".to_string(),
                        message: "failed to join voice channel".to_string(),
                    });
                }
            }
        }

        ClientMessage::LeaveVoice { channel_id } => {
            if let Ok(Some(session)) = voice_repo::get_active_session(&state.db, channel_id).await {
                let _ = voice_repo::leave_session(&state.db, session.id, user_id).await;

                // Broadcast leave
                conn_mgr.broadcast_to_channel(
                    channel_id,
                    ServerMessage::UserLeftVoice {
                        channel_id,
                        user_id,
                    },
                );

                // Get remaining participants and broadcast authoritative state
                if let Ok(participants) = voice_repo::get_participants(&state.db, session.id).await
                {
                    conn_mgr.broadcast_to_channel(
                        channel_id,
                        ServerMessage::VoiceStateUpdate {
                            channel_id,
                            participants: participants.clone(),
                        },
                    );

                    // If no participants left, end the session
                    if participants.is_empty() {
                        let _ = voice_repo::end_session(&state.db, session.id).await;
                    }
                }
            }
        }

        ClientMessage::KickFromVoice {
            channel_id,
            user_id: target_user_id,
        } => {
            // Check permissions: actor must outrank target
            let actor_role = channel_repo::get_member_role(&state.db, channel_id, user_id).await;
            let target_role =
                channel_repo::get_member_role(&state.db, channel_id, target_user_id).await;

            let allowed = match (actor_role, target_role) {
                (Ok(Some(a)), Ok(Some(t))) => permissions::can_moderate(&a, &t),
                _ => false,
            };

            if !allowed {
                let _ = tx.send(ServerMessage::Error {
                    code: "forbidden".to_string(),
                    message: "insufficient permissions to kick from voice".to_string(),
                });
                return;
            }

            if let Ok(Some(session)) = voice_repo::get_active_session(&state.db, channel_id).await {
                let _ = voice_repo::leave_session(&state.db, session.id, target_user_id).await;

                // Notify the kicked user
                conn_mgr.broadcast_to_channel(
                    channel_id,
                    ServerMessage::KickedFromVoice {
                        channel_id,
                        user_id: target_user_id,
                        kicked_by: user_id,
                    },
                );

                // Broadcast leave event
                conn_mgr.broadcast_to_channel(
                    channel_id,
                    ServerMessage::UserLeftVoice {
                        channel_id,
                        user_id: target_user_id,
                    },
                );

                // Broadcast updated participant list
                if let Ok(participants) = voice_repo::get_participants(&state.db, session.id).await
                {
                    conn_mgr.broadcast_to_channel(
                        channel_id,
                        ServerMessage::VoiceStateUpdate {
                            channel_id,
                            participants: participants.clone(),
                        },
                    );

                    if participants.is_empty() {
                        let _ = voice_repo::end_session(&state.db, session.id).await;
                    }
                }
            }
        }

        // Auth message not expected over an already-authenticated connection
        ClientMessage::Authenticate { .. } => {}
        ClientMessage::Unsubscribe { .. } => {
            // Abort all channel subscription tasks and clear the list.
            // Clients re-subscribe when navigating to a new channel set.
            for task in subscription_tasks.drain(..) {
                task.abort();
            }
        }
        ClientMessage::EditMessage {
            message_id,
            ciphertext,
            nonce,
        } => {
            // Validate ciphertext size
            const MAX_CIPHERTEXT_SIZE: usize = 65_536;
            if ciphertext.is_empty() || ciphertext.len() > MAX_CIPHERTEXT_SIZE {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: if ciphertext.is_empty() {
                        "message cannot be empty".to_string()
                    } else {
                        "message too large".to_string()
                    },
                });
                return;
            }

            // Look up the message first to get channel_id for broadcast
            let msg_record = match message_repo::get_message_by_id(&state.db, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "not_found".to_string(),
                        message: "message not found".to_string(),
                    });
                    return;
                }
                Err(e) => {
                    tracing::error!("Failed to look up message: {e}");
                    return;
                }
            };

            // Verify ownership — only the sender can edit their message
            if msg_record.sender_id != Some(user_id) {
                let _ = tx.send(ServerMessage::Error {
                    code: "forbidden".to_string(),
                    message: "you can only edit your own messages".to_string(),
                });
                return;
            }

            // Check for active timeout (timed-out users cannot edit)
            if let Ok(Some(timeout)) =
                timeout_repo::get_active_timeout(&state.db, user_id, msg_record.channel_id).await
            {
                let remaining = (timeout.expires_at - chrono::Utc::now())
                    .num_seconds()
                    .max(0);
                let _ = tx.send(ServerMessage::Error {
                    code: "timed_out".to_string(),
                    message: format!("you are timed out for {remaining} more seconds"),
                });
                return;
            }

            // Enforce 15-minute edit window
            const EDIT_WINDOW_SECONDS: i64 = 900;
            let age = (chrono::Utc::now() - msg_record.created_at).num_seconds();
            if age > EDIT_WINDOW_SECONDS {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: "messages can only be edited within 15 minutes".to_string(),
                });
                return;
            }

            // Save old content for edit history
            if let Err(e) = message_repo::save_edit_history(
                &state.db,
                message_id,
                &msg_record.ciphertext,
                &msg_record.nonce,
            )
            .await
            {
                tracing::warn!("Failed to save edit history: {e}");
            }

            match message_repo::edit_message(&state.db, message_id, user_id, &ciphertext, &nonce)
                .await
            {
                Ok(true) => {
                    conn_mgr.broadcast_to_channel(
                        msg_record.channel_id,
                        ServerMessage::MessageEdited {
                            message_id,
                            channel_id: msg_record.channel_id,
                            sender_id: user_id,
                            ciphertext,
                            nonce,
                            edited_at: chrono::Utc::now().to_rfc3339(),
                        },
                    );
                }
                Ok(false) => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "not_found".to_string(),
                        message: "message not found or not yours".to_string(),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to edit message: {e}");
                }
            }
        }

        ClientMessage::AddReaction { message_id, emoji } => {
            // Per-connection reaction cooldown (200ms between reactions)
            let now = tokio::time::Instant::now();
            if now.duration_since(*last_reaction_time) < tokio::time::Duration::from_millis(200) {
                let _ = tx.send(ServerMessage::Error {
                    code: "rate_limited".to_string(),
                    message: "adding reactions too quickly".to_string(),
                });
                return;
            }
            *last_reaction_time = now;

            // Validate emoji length
            if emoji.is_empty() || emoji.len() > 32 {
                let _ = tx.send(ServerMessage::Error {
                    code: "validation_error".to_string(),
                    message: "invalid emoji".to_string(),
                });
                return;
            }

            // Look up the message to get the channel_id
            let msg_record = match message_repo::get_message_by_id(&state.db, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "not_found".to_string(),
                        message: "message not found".to_string(),
                    });
                    return;
                }
                Err(e) => {
                    tracing::error!("Failed to look up message for reaction: {e}");
                    return;
                }
            };

            // Verify channel membership
            if !channel_repo::is_member(&state.db, msg_record.channel_id, user_id)
                .await
                .unwrap_or(false)
            {
                let _ = tx.send(ServerMessage::Error {
                    code: "forbidden".to_string(),
                    message: "not a member of this channel".to_string(),
                });
                return;
            }

            // Limit unique reactions per message (max 20)
            const MAX_UNIQUE_REACTIONS: i64 = 20;
            match reaction_repo::count_unique_reactions(&state.db, message_id).await {
                Ok(count) if count >= MAX_UNIQUE_REACTIONS => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "validation_error".to_string(),
                        message: "maximum number of reactions reached".to_string(),
                    });
                    return;
                }
                Err(e) => {
                    tracing::error!("Failed to count reactions: {e}");
                    return;
                }
                _ => {}
            }

            match reaction_repo::add_reaction(&state.db, message_id, user_id, &emoji).await {
                Ok(_) => {
                    conn_mgr.broadcast_to_channel(
                        msg_record.channel_id,
                        ServerMessage::ReactionAdded {
                            message_id,
                            user_id,
                            emoji,
                        },
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to add reaction: {e}");
                    let _ = tx.send(ServerMessage::Error {
                        code: "internal_error".to_string(),
                        message: "failed to add reaction".to_string(),
                    });
                }
            }
        }

        ClientMessage::RemoveReaction { message_id, emoji } => {
            let msg_record = match message_repo::get_message_by_id(&state.db, message_id).await {
                Ok(Some(m)) => m,
                Ok(None) => {
                    let _ = tx.send(ServerMessage::Error {
                        code: "not_found".to_string(),
                        message: "message not found".to_string(),
                    });
                    return;
                }
                Err(e) => {
                    tracing::error!("Failed to look up message for reaction removal: {e}");
                    return;
                }
            };

            // Verify channel membership
            if !channel_repo::is_member(&state.db, msg_record.channel_id, user_id)
                .await
                .unwrap_or(false)
            {
                let _ = tx.send(ServerMessage::Error {
                    code: "forbidden".to_string(),
                    message: "not a member of this channel".to_string(),
                });
                return;
            }

            match reaction_repo::remove_reaction(&state.db, message_id, user_id, &emoji).await {
                Ok(true) => {
                    conn_mgr.broadcast_to_channel(
                        msg_record.channel_id,
                        ServerMessage::ReactionRemoved {
                            message_id,
                            user_id,
                            emoji,
                        },
                    );
                }
                Ok(false) => {} // reaction didn't exist, no-op
                Err(e) => {
                    tracing::error!("Failed to remove reaction: {e}");
                    let _ = tx.send(ServerMessage::Error {
                        code: "internal_error".to_string(),
                        message: "failed to remove reaction".to_string(),
                    });
                }
            }
        }

        ClientMessage::MarkRead {
            channel_id,
            message_id,
        } => {
            // Only allow marking read if the user is actually a member of the channel
            if channel_repo::is_member(&state.db, channel_id, user_id)
                .await
                .unwrap_or(false)
            {
                let _ = unread_repo::mark_read(&state.db, user_id, channel_id, message_id).await;

                // Broadcast read receipt to channel if user hasn't opted out
                if !unread_repo::is_read_receipts_disabled(&state.db, user_id)
                    .await
                    .unwrap_or(false)
                {
                    conn_mgr.broadcast_to_channel(
                        channel_id,
                        ServerMessage::ReadReceipt {
                            channel_id,
                            user_id,
                            message_id,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    );
                }
            }
        }

        ClientMessage::MarkAllRead => {
            let _ = unread_repo::mark_all_read(&state.db, user_id).await;
        }
    }
}

async fn broadcast_presence(
    db: &sqlx::PgPool,
    conn_mgr: &crate::ws::connection_manager::ConnectionManager,
    user_id: Uuid,
    status: &str,
) {
    let presence_status = match status {
        "online" => chatalot_common::ws_messages::PresenceStatus::Online,
        "idle" => chatalot_common::ws_messages::PresenceStatus::Idle,
        "dnd" => chatalot_common::ws_messages::PresenceStatus::Dnd,
        "invisible" => chatalot_common::ws_messages::PresenceStatus::Invisible,
        _ => chatalot_common::ws_messages::PresenceStatus::Offline,
    };

    let msg = ServerMessage::PresenceUpdate {
        user_id,
        status: presence_status,
    };

    // Only send to users who share a community with this user
    match community_repo::get_community_mates(db, user_id).await {
        Ok(mates) => {
            for uid in mates {
                if conn_mgr.is_online(&uid) {
                    conn_mgr.send_to_user(&uid, &msg);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to get community mates for presence broadcast: {e}");
        }
    }
}
