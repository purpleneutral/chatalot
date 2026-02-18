use sqlx::PgPool;
use uuid::Uuid;
use web_push::{
    ContentEncoding, IsahcWebPushClient, PartialVapidSignatureBuilder, SubscriptionInfo,
    VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

use chatalot_db::repos::push_subscription_repo;

/// Notification payload sent via Web Push (metadata only — never message content).
#[derive(serde::Serialize)]
pub struct PushPayload {
    pub notification_type: String,
    pub sender_name: String,
    pub channel_id: String,
    pub channel_name: String,
}

pub struct PushService {
    client: IsahcWebPushClient,
    /// Pre-parsed VAPID signing key (reused across all sends).
    vapid_builder: PartialVapidSignatureBuilder,
    /// Contact info for VAPID (mailto: or https:// URL).
    contact: String,
}

impl PushService {
    /// Create a new PushService. Returns Err if the VAPID key is invalid.
    pub fn new(vapid_private_key: &str, public_url: Option<&str>) -> Result<Self, String> {
        let vapid_builder = VapidSignatureBuilder::from_base64_no_sub(
            vapid_private_key,
            base64::URL_SAFE_NO_PAD,
        )
        .map_err(|e| format!("invalid VAPID private key: {e}"))?;

        let client = IsahcWebPushClient::new()
            .map_err(|e| format!("failed to create push client: {e}"))?;

        let contact = public_url
            .map(|u| u.to_string())
            .unwrap_or_else(|| "mailto:admin@localhost".to_string());

        Ok(Self {
            client,
            vapid_builder,
            contact,
        })
    }

    /// Send a push notification to all subscriptions for a user.
    /// Handles failure tracking and dead subscription cleanup.
    pub async fn send_to_user(&self, pool: &PgPool, user_id: Uuid, payload: &PushPayload) {
        let subscriptions =
            match push_subscription_repo::get_subscriptions_for_user(pool, user_id).await {
                Ok(subs) => subs,
                Err(e) => {
                    tracing::warn!("Failed to get push subscriptions for {user_id}: {e}");
                    return;
                }
            };

        if subscriptions.is_empty() {
            return;
        }

        let payload_json = match serde_json::to_string(payload) {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("Failed to serialize push payload: {e}");
                return;
            }
        };

        for sub in subscriptions {
            let sub_info = SubscriptionInfo::new(&sub.endpoint, &sub.p256dh_key, &sub.auth_key);

            let mut builder = self.vapid_builder.clone().add_sub_info(&sub_info);
            builder.add_claim("sub", serde_json::Value::String(self.contact.clone()));

            let sig = match builder.build() {
                Ok(sig) => sig,
                Err(e) => {
                    tracing::warn!("Failed to build VAPID sig for {}: {e}", sub.id);
                    let _ = push_subscription_repo::increment_failure_count(pool, sub.id).await;
                    continue;
                }
            };

            let mut msg_builder = WebPushMessageBuilder::new(&sub_info);
            msg_builder.set_payload(ContentEncoding::Aes128Gcm, payload_json.as_bytes());
            msg_builder.set_vapid_signature(sig);

            let message = match msg_builder.build() {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::warn!("Failed to build push message for {}: {e}", sub.id);
                    let _ = push_subscription_repo::increment_failure_count(pool, sub.id).await;
                    continue;
                }
            };

            match self.client.send(message).await {
                Ok(()) => {
                    let _ = push_subscription_repo::mark_used(pool, sub.id).await;
                }
                Err(e) => {
                    let err_str = format!("{e}");
                    // EndpointNotValid / EndpointNotFound → subscription is dead
                    if err_str.contains("410")
                        || err_str.contains("404")
                        || err_str.contains("NotFound")
                        || err_str.contains("NotValid")
                    {
                        tracing::info!("Removing expired push subscription {}", sub.id);
                        let _ = push_subscription_repo::delete_by_endpoint(
                            pool,
                            sub.user_id,
                            &sub.endpoint,
                        )
                        .await;
                    } else {
                        tracing::warn!("Push delivery failed for {}: {e}", sub.id);
                        let _ =
                            push_subscription_repo::increment_failure_count(pool, sub.id).await;
                    }
                }
            }
        }
    }
}
