use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use chatalot_common::ws_messages::{ClientMessage, MessageType, ServerMessage};
use chatalot_db::models::channel::ChannelType;
use chatalot_db::repos::{channel_repo, community_repo, message_repo, reaction_repo, unread_repo, user_repo, voice_repo};

use crate::permissions;

use crate::app_state::AppState;
use crate::ws::connection_manager::SessionHandle;

/// Handle an authenticated WebSocket connection.
pub async fn handle_socket(
    socket: WebSocket,
    user_id: Uuid,
    state: Arc<AppState>,
) {
    let conn_mgr = &state.connections;
    let session_id = Uuid::new_v4();
    let (mut ws_sink, mut ws_stream) = socket.split();

    // Channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Register the session
    let handle = SessionHandle {
        session_id,
        user_id,
        tx: tx.clone(),
    };
    conn_mgr.add_session(handle);

    // Broadcast presence: this user is online
    broadcast_presence(conn_mgr, user_id, "online");

    tracing::info!(%user_id, %session_id, "WebSocket connected");

    // Writer task: forwards messages from the mpsc channel to the WebSocket
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg)
                && ws_sink.send(Message::Text(text.into())).await.is_err()
            {
                break;
            }
        }
    });

    // Heartbeat: server sends ping every 30 seconds
    let heartbeat_tx = tx.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
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

    // Reader task: processes incoming WebSocket messages
    while let Some(Ok(msg)) = ws_stream.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    handle_client_message(
                        client_msg,
                        user_id,
                        &state,
                        &tx,
                    )
                    .await;
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

    // If no more sessions for this user, broadcast offline
    if !conn_mgr.is_online(&user_id) {
        broadcast_presence(conn_mgr, user_id, "offline");
    }

    tracing::info!(%user_id, %session_id, "WebSocket disconnected");
}

async fn handle_client_message(
    msg: ClientMessage,
    user_id: Uuid,
    state: &AppState,
    tx: &mpsc::UnboundedSender<ServerMessage>,
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
        } => {
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

            // For DM channels, block messages if users no longer share a community
            if let Ok(Some(ch)) = channel_repo::get_channel(&state.db, channel_id).await
                && ch.channel_type == ChannelType::Dm
            {
                // Find the other user in this DM
                if let Ok(members) = channel_repo::list_members(&state.db, channel_id).await {
                    for member in &members {
                        if member.user_id != user_id {
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
                                        message:
                                            "you no longer share a community with this user"
                                                .to_string(),
                                    });
                                    return;
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to check shared community: {e}"
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            let msg_type_str = match message_type {
                MessageType::Text => "text",
                MessageType::File => "file",
                MessageType::System => "system",
            };

            let message_id = Uuid::now_v7();

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
            )
            .await
            {
                Ok(stored) => {
                    // Confirm to the sender
                    let _ = tx.send(ServerMessage::MessageSent {
                        id: message_id,
                        channel_id,
                        created_at: stored.created_at.to_rfc3339(),
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
                    };

                    // For DM channels, deliver directly to the other member
                    // to avoid race conditions with subscription timing.
                    // For group channels, use broadcast as normal.
                    let is_dm = channel_repo::get_channel(&state.db, channel_id)
                        .await
                        .ok()
                        .flatten()
                        .is_some_and(|ch| ch.channel_type == ChannelType::Dm);

                    if is_dm {
                        if let Ok(members) =
                            channel_repo::list_members(&state.db, channel_id).await
                        {
                            // If this is the first message, notify the other user
                            // about the DM channel so it appears in their sidebar.
                            let is_first = message_repo::count_messages(&state.db, channel_id)
                                .await
                                .unwrap_or(0)
                                == 1;

                            for member in &members {
                                if member.user_id != user_id {
                                    if is_first {
                                        if let Ok(Some(sender)) =
                                            user_repo::find_by_id(&state.db, user_id).await
                                        {
                                            if let Ok(Some(ch)) =
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
                                                        other_user_avatar_url: sender
                                                            .avatar_url
                                                            .clone(),
                                                    },
                                                );
                                            }
                                        }
                                    }
                                    conn_mgr.send_to_user(&member.user_id, &new_msg);
                                }
                            }
                        }
                    } else {
                        conn_mgr.broadcast_to_channel(channel_id, new_msg);
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
                let role = channel_repo::get_member_role(
                    &state.db,
                    msg_record.channel_id,
                    user_id,
                )
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
                            message: "you don't have permission to delete this message"
                                .to_string(),
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
                conn_mgr.broadcast_to_channel(
                    channel_id,
                    ServerMessage::UserTyping {
                        channel_id,
                        user_id,
                    },
                );
            }
        }

        ClientMessage::StopTyping { channel_id } => {
            if channel_repo::is_member(&state.db, channel_id, user_id)
                .await
                .unwrap_or(false)
            {
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
            broadcast_presence(conn_mgr, user_id, &status_str);
        }

        ClientMessage::Subscribe { channel_ids } => {
            // Subscribe this session to channel broadcasts.
            // Verify membership for each channel before subscribing.
            for channel_id in channel_ids {
                let is_member = channel_repo::is_member(&state.db, channel_id, user_id)
                    .await
                    .unwrap_or(false);
                if !is_member {
                    continue;
                }

                let mut rx = conn_mgr.subscribe_channel(channel_id);
                let tx = tx.clone();
                let uid = user_id;
                tokio::spawn(async move {
                    while let Ok(msg) = rx.recv().await {
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
                });
            }
        }

        // Forward WebRTC signaling messages to the target user
        ClientMessage::RtcOffer {
            target_user_id,
            session_id,
            sdp,
        } => {
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

            match voice_repo::get_or_create_session(&state.db, channel_id, user_id).await {
                Ok(session) => {
                    if let Err(e) = voice_repo::join_session(&state.db, session.id, user_id).await {
                        tracing::error!("Failed to join voice session: {e}");
                        return;
                    }

                    // Get current participants and broadcast
                    if let Ok(participants) = voice_repo::get_participants(&state.db, session.id).await {
                        // Tell the joiner who's already in the call
                        let _ = tx.send(ServerMessage::VoiceStateUpdate {
                            channel_id,
                            participants: participants.clone(),
                        });

                        // Tell everyone in the channel someone joined
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

                // If no participants left, end the session
                if let Ok(participants) = voice_repo::get_participants(&state.db, session.id).await
                    && participants.is_empty()
                {
                    let _ = voice_repo::end_session(&state.db, session.id).await;
                }
            }
        }

        // Auth message not expected over an already-authenticated connection
        ClientMessage::Authenticate { .. } => {}
        ClientMessage::Unsubscribe { .. } => {
            // Unsubscribe is implicit: when the spawned tasks drop, they stop receiving.
            // A proper implementation would track and abort them. Left as a TODO.
        }
        ClientMessage::EditMessage {
            message_id,
            ciphertext,
            nonce,
        } => {
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

            match message_repo::edit_message(
                &state.db,
                message_id,
                user_id,
                &ciphertext,
                &nonce,
            )
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
                _ => return,
            };

            // Verify channel membership
            if !channel_repo::is_member(&state.db, msg_record.channel_id, user_id)
                .await
                .unwrap_or(false)
            {
                return;
            }

            if reaction_repo::add_reaction(&state.db, message_id, user_id, &emoji)
                .await
                .is_ok()
            {
                conn_mgr.broadcast_to_channel(
                    msg_record.channel_id,
                    ServerMessage::ReactionAdded {
                        message_id,
                        user_id,
                        emoji,
                    },
                );
            }
        }

        ClientMessage::RemoveReaction { message_id, emoji } => {
            let msg_record = match message_repo::get_message_by_id(&state.db, message_id).await {
                Ok(Some(m)) => m,
                _ => return,
            };

            if reaction_repo::remove_reaction(&state.db, message_id, user_id, &emoji)
                .await
                .unwrap_or(false)
            {
                conn_mgr.broadcast_to_channel(
                    msg_record.channel_id,
                    ServerMessage::ReactionRemoved {
                        message_id,
                        user_id,
                        emoji,
                    },
                );
            }
        }

        ClientMessage::MarkRead {
            channel_id,
            message_id,
        } => {
            let _ = unread_repo::mark_read(&state.db, user_id, channel_id, message_id).await;
        }
    }
}

fn broadcast_presence(conn_mgr: &crate::ws::connection_manager::ConnectionManager, user_id: Uuid, status: &str) {
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

    // Send to all online users
    for uid in conn_mgr.online_users() {
        if uid != user_id {
            conn_mgr.send_to_user(&uid, &msg);
        }
    }
}
