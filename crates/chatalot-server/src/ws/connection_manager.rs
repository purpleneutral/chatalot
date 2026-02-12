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
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
            channel_senders: DashMap::new(),
        }
    }

    /// Register a new WebSocket session.
    pub fn add_session(&self, handle: SessionHandle) {
        self.connections
            .entry(handle.user_id)
            .or_default()
            .push(handle);
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
}
