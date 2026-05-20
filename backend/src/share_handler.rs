// src/share_handler.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::Serialize;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;
use common::{CreateShareCodePayload, ShareCode, UseShareCodePayload};

// ─── Erreur unifiée ──────────────────────────────────────────────

#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

// ─── Helper : génère un code lisible format XXX-XXX-XXX ──────────

fn generate_code() -> String {
    let part = || -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(3)
            .map(|c| c as char) // ← cast u8 → char
            .map(|c| c.to_ascii_uppercase()) // ← puis majuscule
            .collect()
    };
    format!("{}-{}-{}", part(), part(), part())
}
// ─── Helper : vérifie que l'utilisateur est owner ────────────────

async fn require_owner(
    db: &sqlx::PgPool,
    vehicle_id: Uuid,
    user_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let role = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access
         WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données"))?;

    match role.as_deref() {
        Some("owner") => Ok(()),
        Some(_) => Err(err(
            StatusCode::FORBIDDEN,
            "Réservé au propriétaire du véhicule",
        )),
        None => Err(err(
            StatusCode::NOT_FOUND,
            "Véhicule introuvable ou accès refusé",
        )),
    }
}

// ─── POST /api/vehicles/:id/share ────────────────────────────────
// Génère un code de partage à usage unique (owner uniquement)

pub async fn create_share_code(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateShareCodePayload>,
) -> impl IntoResponse {
    // 1. Vérifie le rôle
    if let Err(e) = require_owner(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    // 2. Valide le rôle demandé
    if !matches!(payload.role.as_str(), "editor" | "viewer") {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Rôle invalide — doit être 'editor' ou 'viewer'",
        )
        .into_response();
    }

    // 3. Génère un code unique (retry si collision)
    let mut code = generate_code();
    for _ in 0..5 {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM public.vehicle_share_codes WHERE code = $1)",
            code
        )
        .fetch_one(&state.db)
        .await
        .unwrap_or(Some(false))
        .unwrap_or(false);

        if !exists {
            break;
        }
        code = generate_code();
    }

    // 4. Expiration dans 24h
    let expires_at = Utc::now() + chrono::Duration::hours(24);

    // 5. Insertion
    let result = sqlx::query!(
        r#"
        INSERT INTO public.vehicle_share_codes
            (vehicle_id, role, code, created_by, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        vehicle_id,
        payload.role,
        code,
        user_id,
        expires_at,
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (
            StatusCode::CREATED,
            Json(ShareCode {
                code,
                role: payload.role,
                expires_at,
            }),
        )
            .into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur génération code : {}", e),
        )
        .into_response(),
    }
}

// ─── POST /api/vehicles/join ─────────────────────────────────────
// Utilise un code de partage pour rejoindre un véhicule

pub async fn join_with_code(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<UseShareCodePayload>,
) -> impl IntoResponse {
    // 1. Recherche le code
    let share = sqlx::query!(
        r#"
        SELECT id, vehicle_id, role, used, expires_at
        FROM public.vehicle_share_codes
        WHERE code = $1
        "#,
        payload.code
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données"));

    let share = match share {
        Ok(Some(s)) => s,
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "Code invalide ou introuvable").into_response()
        }
        Err(e) => return e.into_response(),
    };

    // 2. Vérifie que le code n'est pas déjà utilisé
    if share.used {
        return err(StatusCode::GONE, "Ce code a déjà été utilisé").into_response();
    }

    // 3. Vérifie l'expiration
    if share.expires_at < Utc::now() {
        return err(StatusCode::GONE, "Ce code a expiré").into_response();
    }

    // 4. Vérifie que l'utilisateur n'a pas déjà accès
    let already_has_access = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM public.vehicle_access
         WHERE vehicle_id = $1 AND user_id = $2)",
        share.vehicle_id,
        user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(false))
    .unwrap_or(false);

    if already_has_access {
        return err(StatusCode::CONFLICT, "Vous avez déjà accès à ce véhicule").into_response();
    }

    // 5. Transaction : insère l'accès + marque le code comme utilisé
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur de transaction"));

    let mut tx = match tx {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };

    // Insère l'accès
    if let Err(e) = sqlx::query!(
        "INSERT INTO public.vehicle_access (vehicle_id, user_id, role) VALUES ($1, $2, $3)",
        share.vehicle_id,
        user_id,
        share.role,
    )
    .execute(&mut *tx)
    .await
    {
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur insertion accès : {}", e),
        )
        .into_response();
    }

    // Marque le code comme utilisé
    if let Err(e) = sqlx::query!(
        "UPDATE public.vehicle_share_codes
         SET used = true, used_by = $1, used_at = now()
         WHERE id = $2",
        user_id,
        share.id,
    )
    .execute(&mut *tx)
    .await
    {
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur mise à jour code : {}", e),
        )
        .into_response();
    }

    // Commit
    if let Err(e) = tx.commit().await {
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur commit : {}", e),
        )
        .into_response();
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "vehicle_id": share.vehicle_id,
            "role": share.role,
        })),
    )
        .into_response()
}
