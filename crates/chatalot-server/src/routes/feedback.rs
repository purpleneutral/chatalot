use std::sync::Arc;

use axum::extract::{Multipart, State};
use axum::routing::post;
use axum::{Extension, Json, Router};
use chatalot_common::api_types::FeedbackResponse;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/feedback", post(submit_feedback))
}

const MAX_SCREENSHOT_SIZE: usize = 5 * 1024 * 1024; // 5MB
const MAX_TITLE_LEN: usize = 200;
const MAX_DESCRIPTION_LEN: usize = 5000;
const VALID_CATEGORIES: &[&str] = &["bug", "feature", "ui", "other"];

async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    mut multipart: Multipart,
) -> Result<Json<FeedbackResponse>, AppError> {
    let mut title = String::new();
    let mut description = String::new();
    let mut category = String::new();
    let mut screenshot: Option<(String, Vec<u8>)> = None; // (filename, bytes)

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("invalid form data: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "title" => {
                title = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("invalid title: {e}")))?;
            }
            "description" => {
                description = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("invalid description: {e}")))?;
            }
            "category" => {
                category = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("invalid category: {e}")))?;
            }
            "screenshot" => {
                let filename = field.file_name().unwrap_or("screenshot.png").to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("failed to read screenshot: {e}")))?;
                if data.len() > MAX_SCREENSHOT_SIZE {
                    return Err(AppError::Validation(
                        "screenshot must be under 5MB".to_string(),
                    ));
                }
                if !data.is_empty() {
                    screenshot = Some((filename, data.to_vec()));
                }
            }
            _ => {}
        }
    }

    // Validate
    let title = title.trim();
    if title.is_empty() || title.len() > MAX_TITLE_LEN {
        return Err(AppError::Validation(
            format!("title must be 1-{MAX_TITLE_LEN} characters"),
        ));
    }
    let description = description.trim();
    if description.is_empty() || description.len() > MAX_DESCRIPTION_LEN {
        return Err(AppError::Validation(
            format!("description must be 1-{MAX_DESCRIPTION_LEN} characters"),
        ));
    }
    let category = category.trim();
    if !VALID_CATEGORIES.contains(&category) {
        return Err(AppError::Validation(
            "category must be one of: bug, feature, ui, other".to_string(),
        ));
    }

    // Check GitHub config
    let (api_token, repo_owner, repo_name) = match (
        &state.config.github_api_token,
        &state.config.github_repo_owner,
        &state.config.github_repo_name,
    ) {
        (Some(token), Some(owner), Some(name)) => (token, owner, name),
        _ => {
            return Err(AppError::Internal(
                "Feedback is not configured on this server".to_string(),
            ));
        }
    };

    // Build issue body
    let category_label = match category {
        "bug" => "Bug Report",
        "feature" => "Feature Request",
        "ui" => "UI/UX",
        _ => "Other",
    };

    let mut body = format!(
        "**Category:** {}\n**Submitted by:** {}\n\n---\n\n{}",
        category_label, claims.username, description
    );

    // Embed screenshot as base64 in a collapsible section
    if let Some((filename, data)) = &screenshot {
        let mime = if filename.ends_with(".png") {
            "image/png"
        } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
            "image/jpeg"
        } else if filename.ends_with(".webp") {
            "image/webp"
        } else {
            "image/png"
        };
        let b64 = base64::encode(data);
        body.push_str(&format!(
            "\n\n<details>\n<summary>Screenshot</summary>\n\n![screenshot](data:{mime};base64,{b64})\n\n</details>"
        ));
    }

    // Map category to GitHub label
    let label = match category {
        "bug" => "bug",
        "feature" => "enhancement",
        "ui" => "ui",
        _ => "feedback",
    };

    // Create GitHub issue via API
    let issues_url =
        format!("https://api.github.com/repos/{repo_owner}/{repo_name}/issues");

    let response = state
        .http_client
        .post(&issues_url)
        .header("Authorization", format!("Bearer {api_token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "chatalot-server")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&serde_json::json!({
            "title": format!("[{}] {}", category_label, title),
            "body": body,
            "labels": [label],
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to contact GitHub: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!("GitHub API error: {} - {}", status, body);
        return Err(AppError::Internal(
            "Failed to create feedback issue".to_string(),
        ));
    }

    let issue: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Invalid response from GitHub: {e}")))?;

    let issue_number = issue["number"].as_u64();

    Ok(Json(FeedbackResponse {
        success: true,
        issue_number,
        message: "Feedback submitted successfully. Thank you!".to_string(),
    }))
}
