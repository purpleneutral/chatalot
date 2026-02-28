use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{Algorithm, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;

/// Expected JWT audience claim.
pub const JWT_AUDIENCE: &str = "chatalot";

/// JWT access token claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    /// Subject (user ID)
    pub sub: Uuid,
    pub username: String,
    /// Whether the user is a site-wide admin
    #[serde(default)]
    pub is_admin: bool,
    /// Whether the user is the instance owner (god role)
    #[serde(default)]
    pub is_owner: bool,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration (Unix timestamp)
    pub exp: i64,
    /// JWT ID (unique token identifier for revocation)
    pub jti: Uuid,
    /// Audience
    #[serde(default)]
    pub aud: String,
}

/// Extract and validate JWT from the Authorization header.
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.validate_exp = true;
    validation.leeway = 60; // 60 seconds clock skew tolerance
    validation.set_audience(&[JWT_AUDIENCE]);

    let token_data =
        jsonwebtoken::decode::<AccessClaims>(token, &state.jwt_decoding_key, &validation)
            .map_err(|_| AppError::Unauthorized)?;

    // Reject tokens from suspended users immediately
    if state.suspended_users.contains(&token_data.claims.sub) {
        return Err(AppError::Unauthorized);
    }

    // Insert claims into request extensions for downstream handlers
    request.extensions_mut().insert(token_data.claims);
    Ok(next.run(request).await)
}
