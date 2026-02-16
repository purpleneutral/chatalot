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
    if title.is_empty() || title.len() > 200 {
        return Err(AppError::Validation(
            "title must be 1-200 characters".to_string(),
        ));
    }
    let description = description.trim();
    if description.is_empty() || description.len() > 5000 {
        return Err(AppError::Validation(
            "description must be 1-5000 characters".to_string(),
        ));
    }
    let category = category.trim();
    let valid_categories = ["bug", "feature", "ui", "other"];
    if !valid_categories.contains(&category) {
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
    let category_label = match category {
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
    let base_url = api_url.trim_end_matches('/');
    let issues_url = format!("{base_url}/api/v1/repos/{repo_owner}/{repo_name}/issues");

    let response = client
        .post(&issues_url)
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

    // Upload screenshot as issue attachment if provided
    if let (Some((filename, data)), Some(issue_idx)) = (screenshot, issue_number) {
        let mime = if filename.ends_with(".png") {
            "image/png"
        } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
            "image/jpeg"
        } else if filename.ends_with(".webp") {
            "image/webp"
        } else {
            "image/png"
        };

        let part = reqwest::multipart::Part::bytes(data)
            .file_name(filename)
            .mime_str(mime)
            .unwrap_or_else(|_| reqwest::multipart::Part::bytes(vec![]));

        let form = reqwest::multipart::Form::new().part("attachment", part);

        let attach_url =
            format!("{base_url}/api/v1/repos/{repo_owner}/{repo_name}/issues/{issue_idx}/assets");

        let attach_resp = client
            .post(&attach_url)
            .header("Authorization", format!("token {api_token}"))
            .multipart(form)
            .send()
            .await;

        match attach_resp {
            Ok(resp) if !resp.status().is_success() => {
                tracing::warn!(
                    "Failed to attach screenshot to issue #{}: {}",
                    issue_idx,
                    resp.status()
                );
            }
            Err(e) => {
                tracing::warn!("Failed to attach screenshot to issue: {e}");
            }
            _ => {}
        }
    }

    Ok(Json(FeedbackResponse {
        success: true,
        issue_number,
        message: "Feedback submitted successfully. Thank you!".to_string(),
    }))
}
