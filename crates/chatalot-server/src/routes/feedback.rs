use std::sync::Arc;

use axum::extract::State;
use axum::routing::post;
use axum::{Extension, Json, Router};
use chatalot_common::api_types::{CreateFeedbackRequest, FeedbackResponse};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/feedback", post(submit_feedback))
}

async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<CreateFeedbackRequest>,
) -> Result<Json<FeedbackResponse>, AppError> {
    // Validate
    let title = req.title.trim();
    if title.is_empty() || title.len() > 200 {
        return Err(AppError::Validation(
            "title must be 1-200 characters".to_string(),
        ));
    }
    let description = req.description.trim();
    if description.is_empty() || description.len() > 5000 {
        return Err(AppError::Validation(
            "description must be 1-5000 characters".to_string(),
        ));
    }
    let valid_categories = ["bug", "feature", "ui", "other"];
    if !valid_categories.contains(&req.category.as_str()) {
        return Err(AppError::Validation(
            "category must be one of: bug, feature, ui, other".to_string(),
        ));
    }

    // Check Forgejo config
    let (api_url, api_token, repo_owner, repo_name) = match (
        &state.config.forgejo_api_url,
        &state.config.forgejo_api_token,
        &state.config.forgejo_repo_owner,
        &state.config.forgejo_repo_name,
    ) {
        (Some(url), Some(token), Some(owner), Some(name)) => (url, token, owner, name),
        _ => {
            return Err(AppError::Internal(
                "Feedback is not configured on this server".to_string(),
            ));
        }
    };

    // Build issue body
    let category_label = match req.category.as_str() {
        "bug" => "Bug Report",
        "feature" => "Feature Request",
        "ui" => "UI/UX",
        _ => "Other",
    };
    let body = format!(
        "**Category:** {}\n**Submitted by:** {}\n\n---\n\n{}",
        category_label, claims.username, description
    );

    // Create Forgejo issue via API
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/v1/repos/{}/{}/issues",
        api_url.trim_end_matches('/'),
        repo_owner,
        repo_name
    );

    let response = client
        .post(&url)
        .header("Authorization", format!("token {api_token}"))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "title": format!("[{}] {}", category_label, title),
            "body": body,
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to contact issue tracker: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!("Forgejo API error: {} - {}", status, body);
        return Err(AppError::Internal(
            "Failed to create feedback issue".to_string(),
        ));
    }

    let issue: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Invalid response from issue tracker: {e}")))?;

    let issue_number = issue["number"].as_u64();

    Ok(Json(FeedbackResponse {
        success: true,
        issue_number,
        message: "Feedback submitted successfully. Thank you!".to_string(),
    }))
}
