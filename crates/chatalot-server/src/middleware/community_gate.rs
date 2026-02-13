use std::sync::Arc;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use chatalot_db::repos::community_repo;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

/// Pre-validated community context inserted into request extensions by the gate middleware.
#[derive(Debug, Clone)]
pub struct CommunityContext {
    pub community_id: Uuid,
    pub role: String,
}

impl CommunityContext {
    /// Owner, admin, or instance admin.
    pub fn can_manage(&self) -> bool {
        matches!(
            self.role.as_str(),
            "owner" | "admin" | "instance_admin"
        )
    }

    /// Owner, admin, moderator, or instance admin.
    pub fn can_moderate(&self) -> bool {
        matches!(
            self.role.as_str(),
            "owner" | "admin" | "moderator" | "instance_admin"
        )
    }

    /// Owner or instance admin.
    pub fn is_owner(&self) -> bool {
        matches!(self.role.as_str(), "owner" | "instance_admin")
    }
}

/// Middleware that validates community membership for all `/api/communities/{community_id}/...` routes.
///
/// Extracts `community_id` from the URL path, checks the caller's membership and ban status,
/// then inserts a `CommunityContext` into request extensions.
///
/// Instance admins bypass all checks.
pub async fn community_gate_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract community_id from the path.
    // Routes are structured as /api/communities/{cid}/...
    // By the time this middleware runs, the path is already matched by the router.
    let community_id = extract_community_id(&request)?;

    let claims = request
        .extensions()
        .get::<AccessClaims>()
        .ok_or(AppError::Unauthorized)?
        .clone();

    // Instance admin bypasses all checks
    if claims.is_admin {
        request.extensions_mut().insert(CommunityContext {
            community_id,
            role: "instance_admin".to_string(),
        });
        return Ok(next.run(request).await);
    }

    // Check if banned
    if community_repo::is_banned_from_community(&state.db, community_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    // Check membership and get role
    let role = community_repo::get_community_member_role(&state.db, community_id, claims.sub)
        .await?
        .ok_or(AppError::Forbidden)?;

    request
        .extensions_mut()
        .insert(CommunityContext { community_id, role });

    Ok(next.run(request).await)
}

/// Extract community_id UUID from the request path.
/// Expects paths like `/api/communities/{uuid}/...` or `/communities/{uuid}/...`.
fn extract_community_id(request: &Request) -> Result<Uuid, AppError> {
    let path = request.uri().path();

    // Find "communities/" in the path, then extract the next segment
    let segments: Vec<&str> = path.split('/').collect();
    for (i, segment) in segments.iter().enumerate() {
        if *segment == "communities"
            && let Some(id_str) = segments.get(i + 1)
        {
            return id_str
                .parse::<Uuid>()
                .map_err(|_| AppError::Validation("invalid community ID".to_string()));
        }
    }

    Err(AppError::Validation(
        "missing community ID in path".to_string(),
    ))
}
