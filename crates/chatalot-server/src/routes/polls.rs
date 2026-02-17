use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use uuid::Uuid;

use chatalot_common::api_types::{
    CreatePollRequest, PollOptionVotes, PollResponse, VotePollRequest,
};
use chatalot_common::ws_messages::ServerMessage;
use chatalot_db::repos::{channel_repo, poll_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/channels/{id}/polls", post(create_poll).get(list_polls))
        .route("/polls/{id}", get(get_poll))
        .route("/polls/{id}/vote", post(vote_poll))
        .route("/polls/{id}/vote/{idx}", delete(remove_vote))
        .route("/polls/{id}/close", post(close_poll))
}

async fn create_poll(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<CreatePollRequest>,
) -> Result<Json<PollResponse>, AppError> {
    if !channel_repo::is_member(&state.db, channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let question = req.question.trim();
    if question.is_empty() || question.len() > 500 {
        return Err(AppError::Validation(
            "question must be 1-500 characters".into(),
        ));
    }

    if req.options.len() < 2 || req.options.len() > 10 {
        return Err(AppError::Validation("polls must have 2-10 options".into()));
    }

    let options: Vec<String> = req.options.iter().map(|o| o.trim().to_string()).collect();
    for opt in &options {
        if opt.is_empty() || opt.len() > 200 {
            return Err(AppError::Validation(
                "each option must be 1-200 characters".into(),
            ));
        }
    }

    if let Some(m) = req.expires_in_minutes
        && !(1..=10080).contains(&m)
    {
        return Err(AppError::Validation(
            "expires_in_minutes must be 1-10080".into(),
        ));
    }

    let id = Uuid::now_v7();
    let options_json = serde_json::to_value(&options)
        .map_err(|_| AppError::Validation("invalid options".into()))?;

    let expires_at = req
        .expires_in_minutes
        .and_then(chrono::Duration::try_minutes)
        .map(|d| chrono::Utc::now() + d);

    let poll = poll_repo::create(
        &state.db,
        id,
        channel_id,
        claims.sub,
        question,
        &options_json,
        req.multi_select,
        req.anonymous,
        expires_at,
    )
    .await?;

    state.connections.broadcast_to_channel(
        channel_id,
        ServerMessage::PollCreated {
            poll_id: id,
            channel_id,
            created_by: claims.sub,
            question: question.to_string(),
        },
    );

    Ok(Json(poll_to_response(&poll, &[])))
}

async fn get_poll(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(poll_id): Path<Uuid>,
) -> Result<Json<PollResponse>, AppError> {
    let poll = poll_repo::get(&state.db, poll_id)
        .await?
        .ok_or_else(|| AppError::NotFound("poll not found".into()))?;

    if !channel_repo::is_member(&state.db, poll.channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let votes = poll_repo::get_votes(&state.db, poll_id).await?;
    Ok(Json(poll_to_response(&poll, &votes)))
}

async fn list_polls(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<Vec<PollResponse>>, AppError> {
    if !channel_repo::is_member(&state.db, channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let polls = poll_repo::list_for_channel(&state.db, channel_id).await?;
    let poll_ids: Vec<Uuid> = polls.iter().map(|p| p.id).collect();
    let all_votes = poll_repo::get_votes_for_polls(&state.db, &poll_ids).await?;

    // Group votes by poll_id
    let mut votes_by_poll: std::collections::HashMap<Uuid, Vec<_>> =
        std::collections::HashMap::new();
    for vote in all_votes {
        votes_by_poll.entry(vote.poll_id).or_default().push(vote);
    }

    let responses: Vec<_> = polls
        .iter()
        .map(|poll| {
            let votes = votes_by_poll
                .get(&poll.id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            poll_to_response(poll, votes)
        })
        .collect();
    Ok(Json(responses))
}

async fn vote_poll(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(poll_id): Path<Uuid>,
    Json(req): Json<VotePollRequest>,
) -> Result<(), AppError> {
    let poll = poll_repo::get(&state.db, poll_id)
        .await?
        .ok_or_else(|| AppError::NotFound("poll not found".into()))?;

    if poll.closed {
        return Err(AppError::Validation("poll is closed".into()));
    }

    if let Some(expires_at) = poll.expires_at
        && expires_at < chrono::Utc::now()
    {
        return Err(AppError::Validation("poll has expired".into()));
    }

    if !channel_repo::is_member(&state.db, poll.channel_id, claims.sub).await? {
        return Err(AppError::Forbidden);
    }

    let options: Vec<String> = serde_json::from_value(poll.options.clone()).unwrap_or_default();
    if req.option_index < 0 || req.option_index as usize >= options.len() {
        return Err(AppError::Validation("invalid option index".into()));
    }

    // For single-select, remove previous votes first
    if !poll.multi_select {
        poll_repo::remove_all_votes_for_user(&state.db, poll_id, claims.sub).await?;
    }

    let vote_id = Uuid::now_v7();
    poll_repo::vote(&state.db, vote_id, poll_id, claims.sub, req.option_index).await?;

    let voter_id = if poll.anonymous {
        None
    } else {
        Some(claims.sub)
    };
    state.connections.broadcast_to_channel(
        poll.channel_id,
        ServerMessage::PollVoted {
            poll_id,
            channel_id: poll.channel_id,
            option_index: req.option_index,
            voter_id,
        },
    );

    Ok(())
}

async fn remove_vote(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path((poll_id, idx)): Path<(Uuid, i32)>,
) -> Result<(), AppError> {
    let poll = poll_repo::get(&state.db, poll_id)
        .await?
        .ok_or_else(|| AppError::NotFound("poll not found".into()))?;

    if poll.closed {
        return Err(AppError::Validation("poll is closed".into()));
    }

    if let Some(expires_at) = poll.expires_at
        && expires_at < chrono::Utc::now()
    {
        return Err(AppError::Validation("poll has expired".into()));
    }

    poll_repo::remove_vote(&state.db, poll_id, claims.sub, idx).await?;
    Ok(())
}

async fn close_poll(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(poll_id): Path<Uuid>,
) -> Result<(), AppError> {
    let poll = poll_repo::get(&state.db, poll_id)
        .await?
        .ok_or_else(|| AppError::NotFound("poll not found".into()))?;

    // Creator or channel admin/owner can close
    if poll.created_by != claims.sub {
        let role = channel_repo::get_member_role(&state.db, poll.channel_id, claims.sub)
            .await?
            .ok_or(AppError::Forbidden)?;

        if !crate::permissions::can_manage_roles(&role) {
            return Err(AppError::Forbidden);
        }
    }

    poll_repo::close(&state.db, poll_id).await?;

    state.connections.broadcast_to_channel(
        poll.channel_id,
        ServerMessage::PollClosed {
            poll_id,
            channel_id: poll.channel_id,
        },
    );

    Ok(())
}

fn poll_to_response(
    poll: &chatalot_db::models::poll::Poll,
    votes: &[chatalot_db::models::poll::PollVote],
) -> PollResponse {
    let options: Vec<String> = serde_json::from_value(poll.options.clone()).unwrap_or_default();

    // Aggregate votes per option
    let mut option_votes: Vec<PollOptionVotes> = options
        .iter()
        .enumerate()
        .map(|(i, _)| PollOptionVotes {
            option_index: i as i32,
            count: 0,
            voter_ids: Vec::new(),
        })
        .collect();

    for v in votes {
        if let Some(ov) = option_votes.get_mut(v.option_index as usize) {
            ov.count += 1;
            if !poll.anonymous {
                ov.voter_ids.push(v.user_id);
            }
        }
    }

    PollResponse {
        id: poll.id,
        channel_id: poll.channel_id,
        created_by: poll.created_by,
        question: poll.question.clone(),
        options,
        multi_select: poll.multi_select,
        anonymous: poll.anonymous,
        closed: poll.closed,
        expires_at: poll.expires_at.map(|t| t.to_rfc3339()),
        created_at: poll.created_at.to_rfc3339(),
        votes: option_votes,
    }
}
