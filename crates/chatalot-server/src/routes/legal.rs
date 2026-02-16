use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::routing::get;
use axum::Router;
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Serialize)]
pub struct LegalDocument {
    pub title: String,
    pub body: String,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/legal/privacy", get(privacy_policy))
        .route("/legal/terms", get(terms_of_service))
}

async fn privacy_policy(State(state): State<Arc<AppState>>) -> Json<LegalDocument> {
    let body = load_legal_file("data/privacy-policy.md", &state)
        .unwrap_or_else(|| default_privacy_policy(&state));
    Json(LegalDocument {
        title: "Privacy Policy".to_string(),
        body,
    })
}

async fn terms_of_service(State(state): State<Arc<AppState>>) -> Json<LegalDocument> {
    let body = load_legal_file("data/terms-of-service.md", &state)
        .unwrap_or_else(|| default_terms_of_service(&state));
    Json(LegalDocument {
        title: "Terms of Service".to_string(),
        body,
    })
}

fn load_legal_file(path: &str, _state: &AppState) -> Option<String> {
    std::fs::read_to_string(path).ok().filter(|s| !s.trim().is_empty())
}

fn instance_name(state: &AppState) -> &str {
    state.config.public_url.as_deref().unwrap_or("this Chatalot instance")
}

fn default_privacy_policy(state: &AppState) -> String {
    let name = instance_name(state);
    format!(
        r#"# Privacy Policy

## What This Instance Collects

**{name}** is a self-hosted Chatalot instance. The following data is stored:

- **Account information**: username, email, display name, and avatar
- **Messages**: encrypted message content (the server cannot read your messages)
- **Files**: encrypted file uploads
- **Session data**: device name, IP address (for active session management)
- **Audit logs**: login attempts and security events (IP addresses, timestamps)

## End-to-End Encryption

Chatalot uses end-to-end encryption (E2E) based on the Signal Protocol. This means:

- Your messages are encrypted on your device before being sent
- The server stores only encrypted ciphertext — it **cannot** read message content
- Your password is never stored — only a one-way Argon2id hash

## What the Server Can See vs. Cannot See

| Can See | Cannot See |
|---------|------------|
| Who sent a message and when | Message content |
| File sizes and metadata | File contents |
| Your username and email | Your password |
| When you're online | What you're typing |
| Channel membership | Private conversations |

## Data Retention

- Messages and files are retained until deleted by users or administrators
- Audit logs are retained for security purposes
- Account data is deleted when you delete your account

## Third-Party Services

This instance does not share your data with third parties. No analytics, tracking, or telemetry is collected by the Chatalot software.

## Your Rights

- You can export or delete your account data at any time
- You can view and revoke active sessions
- You can regenerate your recovery code
- Contact the instance administrator for any privacy concerns

## Instance Responsibility

This privacy policy applies to **{name}**. The instance administrator is responsible for how this server is operated, maintained, and secured. The Chatalot software developers are not responsible for individual instance operations.

*Last updated: This is a default policy. Instance administrators can customize it.*"#
    )
}

fn default_terms_of_service(state: &AppState) -> String {
    let name = instance_name(state);
    format!(
        r#"# Terms of Service

## Self-Hosted Software Notice

**{name}** runs Chatalot, an open-source self-hosted chat platform licensed under GPL-3.0. This instance is operated independently by its administrator.

## Acceptance

By creating an account, you agree to these terms and the Privacy Policy.

## Your Responsibilities

- Keep your account credentials secure
- Save your recovery code — it's the only way to reset your password without admin help
- Do not use this service for illegal activities
- Respect other users and follow any community guidelines set by the administrator

## Host Responsibilities

The instance administrator is responsible for:

- Server maintenance and security
- Data backups and availability
- Enforcing acceptable use policies
- Responding to legal requests in their jurisdiction

## Software Disclaimer

Chatalot is provided "as is" without warranty of any kind. The software developers:

- Are **not responsible** for content hosted on any instance
- Do **not operate** or control individual instances
- Provide **no guarantee** of uptime, security, or data integrity
- Are **not liable** for any damages arising from use of the software

## Account Termination

- You can delete your account at any time from Settings
- The administrator may suspend or delete accounts that violate these terms
- Deleted accounts cannot be recovered

## Trust & Verification

Before sharing sensitive information, verify that you trust the administrator of this instance. End-to-end encryption protects message content, but the server administrator controls the infrastructure.

## Changes

These terms may be updated by the instance administrator. Continued use constitutes acceptance of any changes.

*Last updated: This is a default policy. Instance administrators can customize it.*"#
    )
}
