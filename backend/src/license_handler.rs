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
    let is_active = row.access_expires_at.map_or(false, |e| e > now);
    let status = if is_active {
        "active"
    } else if row.trial_ends_at > now {
        "trial"
    } else {
        "expired"
    };

    // Calcul de la fenêtre d'alerte et du type de licence depuis le dernier jeton
    let days_until_expiry: (Option<i64>, String) = if status == "expired" {
        (None, "personal".to_string())
    } else {
        let expiry = if is_active {
            row.access_expires_at.unwrap()
        } else {
            row.trial_ends_at
        };
        let days_remaining = (expiry - now).num_days();

        // Jetons lifetime (≈ 100 ans) : jamais d'alerte
        if days_remaining > 3650 {
            (None, "fleet".to_string()) // lifetime = forcément fleet
        } else {
            // Seuil et type depuis le dernier jeton (ou défauts pour la période d'essai)
            let last_token = if is_active {
                sqlx::query!(
                    "SELECT duration_days, license_type FROM public.license_tokens
                     WHERE used_by = $1 AND used_at IS NOT NULL
                     ORDER BY used_at DESC LIMIT 1",
                    user_id
                )
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None)
            } else {
                None
            };

            let threshold: i64 = match last_token.as_ref().map(|t| t.duration_days) {
                Some(d) if d <= 30 => 7,
                Some(d) if d <= 95 => 15,
                _                  => if is_active { 30 } else { 15 },
            };

            (
                if days_remaining <= threshold { Some(days_remaining) } else { None },
                last_token.and_then(|t| Some(t.license_type)).unwrap_or_else(|| "personal".to_string()),
            )
        }
    };

    (
        StatusCode::OK,
        Json(LicenseStatus {
            status: status.to_string(),
            trial_ends_at: row.trial_ends_at,
            access_expires_at: row.access_expires_at,
            days_until_expiry: days_until_expiry.0,
            license_type: if status == "expired" { "personal".to_string() } else { days_until_expiry.1 },
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
                trial_ends_at: Utc::now(),    // recalculé côté client via GET /license
                access_expires_at: Some(new_expiry),
                days_until_expiry: None,       // recalculé côté client via GET /license
                license_type: "personal".to_string(), // recalculé côté client via GET /license
            }),
        )
            .into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur mise à jour").into_response(),
    }
}
