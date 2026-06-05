// src/ios_handler.rs — Activation automatique pour les utilisateurs iOS (App Store)
// Appelé une seule fois au premier lancement, accorde un accès lifetime.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use serde::Deserialize;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct IosActivatePayload {
    pub key: String,
}

pub async fn ios_activate(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<IosActivatePayload>,
) -> impl IntoResponse {
    let expected_key = std::env::var("IOS_ACTIVATION_KEY").unwrap_or_default();

    if expected_key.is_empty() || payload.key != expected_key {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Clé invalide"})),
        )
            .into_response();
    }

    // Vérifier si l'utilisateur a déjà un accès lifetime (>3650 jours restants)
    let current = sqlx::query_scalar!(
        "SELECT access_expires_at FROM public.users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None)
    .flatten();

    let now = Utc::now();
    if let Some(exp) = current {
        if (exp - now).num_days() > 3650 {
            return (
                StatusCode::OK,
                Json(serde_json::json!({"message": "Déjà activé", "activated": false})),
            )
                .into_response();
        }
    }

    // Accorder un accès lifetime (~100 ans)
    let lifetime = now + chrono::Duration::days(36500);
    match sqlx::query!(
        "UPDATE public.users SET access_expires_at = $1 WHERE id = $2",
        lifetime,
        user_id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"message": "Activation iOS réussie", "activated": true})),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur base de données"})),
        )
            .into_response(),
    }
}
