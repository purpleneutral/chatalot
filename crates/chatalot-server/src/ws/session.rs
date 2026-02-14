use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::StreamExt;

use crate::app_state::AppState;
use crate::middleware::auth::AccessClaims;
use crate::ws::handler;

/// WebSocket upgrade handler.
///
/// Authentication is done via the first message (ClientMessage::Authenticate)
/// rather than via headers, since WebSocket headers are unreliable across browsers.
pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_ws_auth(socket, state))
}

/// First stage: wait for authentication message, then hand off to the main handler.
async fn handle_ws_auth(
    mut socket: WebSocket,
    state: Arc<AppState>,
) {
    // Wait for the first message which must be an Authenticate message
    let auth_timeout = tokio::time::Duration::from_secs(10);

    let user_id = match tokio::time::timeout(auth_timeout, socket.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => {
            match serde_json::from_str::<chatalot_common::ws_messages::ClientMessage>(&text) {
                Ok(chatalot_common::ws_messages::ClientMessage::Authenticate { token }) => {
                    // Validate the JWT
                    match validate_token(&state, &token) {
                        Some(claims) => {
                            // Send authenticated confirmation
                            let confirm = chatalot_common::ws_messages::ServerMessage::Authenticated {
                                user_id: claims.sub,
                                server_version: state.client_version.clone(),
                            };
                            if let Ok(json) = serde_json::to_string(&confirm) {
                                let _ = socket.send(Message::Text(json.into())).await;
                            }
                            claims.sub
                        }
                        None => {
                            let err = chatalot_common::ws_messages::ServerMessage::Error {
                                code: "unauthorized".to_string(),
                                message: "invalid token".to_string(),
                            };
                            if let Ok(json) = serde_json::to_string(&err) {
                                let _ = socket.send(Message::Text(json.into())).await;
                            }
                            return;
                        }
                    }
                }
                _ => return, // First message must be Authenticate
            }
        }
        _ => return, // Timeout or error
    };

    // Hand off to the main handler
    handler::handle_socket(socket, user_id, state).await;
}

fn validate_token(state: &AppState, token: &str) -> Option<AccessClaims> {
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::EdDSA);
    validation.validate_exp = true;

    jsonwebtoken::decode::<AccessClaims>(token, &state.jwt_decoding_key, &validation)
        .ok()
        .map(|data| data.claims)
}
