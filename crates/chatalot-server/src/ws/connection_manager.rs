use dashmap::DashMap;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use chatalot_common::ws_messages::ServerMessage;

/// Handle to a connected WebSocket session.
#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub tx: mpsc::UnboundedSender<ServerMessage>,
}

/// Manages all active WebSocket connections and channel subscriptions.
pub struct ConnectionManager {
    /// user_id -> list of active sessions (supports multi-device)
    connections: DashMap<Uuid, Vec<SessionHandle>>,
    /// channel_id -> broadcast sender for real-time messages
    channel_senders: DashMap<Uuid, broadcast::Sender<ServerMessage>>,
    /// (channel_id, user_id) -> last typing timestamp (for timeout cleanup)
    typing_state: DashMap<(Uuid, Uuid), tokio::time::Instant>,
}

/// Maximum concurrent WebSocket sessions per user (multi-device support).
const MAX_SESSIONS_PER_USER: usize = 8;

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
            channel_senders: DashMap::new(),
            typing_state: DashMap::new(),
        }
    }

    /// Register a new WebSocket session.
    /// Returns false if the user has too many active sessions.
    pub fn add_session(&self, handle: SessionHandle) -> bool {
        let mut entry = self.connections.entry(handle.user_id).or_default();
        if entry.len() >= MAX_SESSIONS_PER_USER {
            return false;
        }
        entry.push(handle);
        true
    }

    /// Remove a WebSocket session.
    pub fn remove_session(&self, user_id: Uuid, session_id: Uuid) {
        if let Some(mut sessions) = self.connections.get_mut(&user_id) {
            sessions.retain(|s| s.session_id != session_id);
            if sessions.is_empty() {
                drop(sessions);
                self.connections.remove(&user_id);
            }
        }
    }

    /// Check if a user has any active connections.
    pub fn is_online(&self, user_id: &Uuid) -> bool {
        self.connections.contains_key(user_id)
    }

    /// Get all online user IDs.
    pub fn online_users(&self) -> Vec<Uuid> {
        self.connections.iter().map(|entry| *entry.key()).collect()
    }

    /// Send a message directly to all sessions of a specific user.
    pub fn send_to_user(&self, user_id: &Uuid, message: &ServerMessage) {
        if let Some(sessions) = self.connections.get(user_id) {
            for session in sessions.iter() {
                let _ = session.tx.send(message.clone());
            }
        }
    }

    /// Get or create a broadcast channel for a chat channel.
    pub fn get_channel_sender(
        &self,
        channel_id: Uuid,
    ) -> broadcast::Sender<ServerMessage> {
        self.channel_senders
            .entry(channel_id)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                tx
            })
            .clone()
    }

    /// Subscribe to a channel's broadcast.
    pub fn subscribe_channel(
        &self,
        channel_id: Uuid,
    ) -> broadcast::Receiver<ServerMessage> {
        self.get_channel_sender(channel_id).subscribe()
    }

    /// Broadcast a message to all subscribers of a channel.
    pub fn broadcast_to_channel(&self, channel_id: Uuid, message: ServerMessage) {
        let sender = self.get_channel_sender(channel_id);
        // Ignore error if no receivers (no one subscribed)
        let _ = sender.send(message);
    }

    /// Broadcast a message to all connected users.
    pub fn broadcast_all(&self, message: ServerMessage) {
        for entry in self.connections.iter() {
            for session in entry.value().iter() {
                let _ = session.tx.send(message.clone());
            }
        }
    }

    /// Record that a user started typing in a channel.
    pub fn set_typing(&self, channel_id: Uuid, user_id: Uuid) {
        self.typing_state.insert((channel_id, user_id), tokio::time::Instant::now());
    }

    /// Clear typing state for a user in a channel.
    pub fn clear_typing(&self, channel_id: Uuid, user_id: Uuid) {
        self.typing_state.remove(&(channel_id, user_id));
    }

    /// Clear all typing state for a user (on disconnect).
    pub fn clear_all_typing_for_user(&self, user_id: Uuid) -> Vec<Uuid> {
        let mut channels = Vec::new();
        self.typing_state.retain(|(ch_id, uid), _| {
            if *uid == user_id {
                channels.push(*ch_id);
                false
            } else {
                true
            }
        });
        channels
    }

    /// Remove typing entries older than the timeout and return them.
    pub fn expire_typing(&self, timeout: std::time::Duration) -> Vec<(Uuid, Uuid)> {
        let now = tokio::time::Instant::now();
        let mut expired = Vec::new();
        self.typing_state.retain(|(ch_id, uid), instant| {
            if now.duration_since(*instant) > timeout {
                expired.push((*ch_id, *uid));
                false
            } else {
                true
            }
        });
        expired
    }
}
