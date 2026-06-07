// src/broadcast_handler.rs
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::{auth::AuthenticatedUser, state::AppState};

#[derive(Serialize)]
pub struct BroadcastResponse {
    pub id: uuid::Uuid,
    pub message: String,
}

pub async fn get_active_broadcast(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> impl IntoResponse {
    let is_ios: bool = sqlx::query_scalar!(
        "SELECT is_ios FROM public.users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .unwrap_or(false);

    let row = sqlx::query!(
        r#"
        SELECT id, message
        FROM public.broadcasts
        WHERE (expires_at IS NULL OR expires_at > NOW())
          AND (NOT exclude_ios OR NOT $1)
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        is_ios
    )
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(r)) => (
            StatusCode::OK,
            Json(BroadcastResponse {
                id: r.id,
                message: r.message,
            }),
        )
            .into_response(),
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("broadcast fetch error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
