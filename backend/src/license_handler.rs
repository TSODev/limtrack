// src/license_handler.rs

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::auth::AuthenticatedUser;
use crate::state::AppState;
use common::{LicenseStatus, RedeemTokenPayload};

#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

fn hash_token(token: &str) -> String {
    let normalized = token.replace('-', "").to_uppercase();
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}

// ─── GET /api/profile/license ─────────────────────────────────

pub async fn get_license(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let row = sqlx::query!(
        "SELECT trial_ends_at, access_expires_at FROM public.users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return err(StatusCode::NOT_FOUND, "Utilisateur introuvable").into_response(),
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    };

    let now = Utc::now();
    let status = if row.access_expires_at.map_or(false, |e| e > now) {
        "active"
    } else if row.trial_ends_at > now {
        "trial"
    } else {
        "expired"
    };

    (
        StatusCode::OK,
        Json(LicenseStatus {
            status: status.to_string(),
            trial_ends_at: row.trial_ends_at,
            access_expires_at: row.access_expires_at,
        }),
    )
        .into_response()
}

// ─── POST /api/profile/redeem ─────────────────────────────────

pub async fn redeem_token(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<RedeemTokenPayload>,
) -> impl IntoResponse {
    let token_hash = hash_token(&payload.token);

    let token = sqlx::query!(
        "SELECT id, duration_days, used_at FROM public.license_tokens WHERE token_hash = $1",
        token_hash
    )
    .fetch_optional(&state.db)
    .await;

    let token = match token {
        Ok(Some(t)) => t,
        Ok(None) => return err(StatusCode::NOT_FOUND, "Jeton invalide ou inexistant").into_response(),
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    };

    if token.used_at.is_some() {
        return err(StatusCode::CONFLICT, "Ce jeton a déjà été utilisé").into_response();
    }

    // Calculer la nouvelle date d'expiration :
    // on part de MAX(maintenant, access_expires_at actuel) + duration_days
    let current_expiry = sqlx::query_scalar!(
        "SELECT access_expires_at FROM public.users WHERE id = $1",
        user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(None);

    let base = current_expiry
        .filter(|e| *e > Utc::now())
        .unwrap_or_else(Utc::now);

    let new_expiry = base + chrono::Duration::days(token.duration_days as i64);

    // Marquer le jeton comme utilisé
    let _ = sqlx::query!(
        "UPDATE public.license_tokens SET used_at = NOW(), used_by = $1 WHERE id = $2",
        user_id,
        token.id
    )
    .execute(&state.db)
    .await;

    // Étendre l'accès de l'utilisateur
    match sqlx::query!(
        "UPDATE public.users SET access_expires_at = $1 WHERE id = $2",
        new_expiry,
        user_id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(LicenseStatus {
                status: "active".to_string(),
                trial_ends_at: Utc::now(), // recalculé côté client via GET /license
                access_expires_at: Some(new_expiry),
            }),
        )
            .into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur mise à jour").into_response(),
    }
}
