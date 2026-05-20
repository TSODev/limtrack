// src/user_handler.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Claims};
use crate::state::AppState;

// ─── Payloads ────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// ─── Réponses ────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct SharedUser {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct OwnedVehicleAccesses {
    pub vehicle_id: Uuid,
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub accesses: Vec<SharedUser>,
}

#[derive(Serialize)]
pub struct SharedVehicle {
    pub vehicle_id: Uuid,
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct ProfileShares {
    pub owned: Vec<OwnedVehicleAccesses>,
    pub shared_with_me: Vec<SharedVehicle>,
}

// ─── Erreur unifiée ──────────────────────────────────────────────

#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

// ─── POST /login ─────────────────────────────────────────────────

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = sqlx::query!(
        "SELECT id, password_hash FROM public.users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&state.db)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => return err(StatusCode::UNAUTHORIZED, "Identifiants invalides").into_response(),
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    };

    let is_valid = verify(&payload.password, &user.password_hash).unwrap_or(false);
    if !is_valid {
        return err(StatusCode::UNAUTHORIZED, "Identifiants invalides").into_response();
    }

    let secret = match std::env::var("JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erreur de configuration serveur",
            )
            .into_response()
        }
    };

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("timestamp valide")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur génération token")
                .into_response()
        }
    };

    (StatusCode::OK, Json(LoginResponse { token })).into_response()
}

// ─── POST /api/user/register ─────────────────────────────────────

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    if payload.username.trim().is_empty()
        || payload.email.trim().is_empty()
        || payload.password.is_empty()
    {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "username, email et password sont requis",
        )
        .into_response();
    }

    let hashed = match hash(&payload.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur de hachage").into_response()
        }
    };

    let result = sqlx::query!(
        "INSERT INTO public.users (username, email, password_hash) VALUES ($1, $2, $3)",
        payload.username.trim(),
        payload.email.trim(),
        hashed,
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(sqlx::Error::Database(e)) if e.constraint() == Some("users_username_key") => {
            err(StatusCode::CONFLICT, "Ce nom d'utilisateur est déjà pris").into_response()
        }
        Err(sqlx::Error::Database(e)) if e.constraint() == Some("users_email_key") => {
            err(StatusCode::CONFLICT, "Cet email est déjà utilisé").into_response()
        }
        Err(e) => err(StatusCode::BAD_REQUEST, format!("Erreur création : {}", e)).into_response(),
    }
}

// ─── GET /api/profile ────────────────────────────────────────────

pub async fn get_profile(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user = sqlx::query!(
        "SELECT id, username, email FROM public.users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    match user {
        Ok(Some(u)) => (
            StatusCode::OK,
            Json(UserProfile {
                id: u.id,
                username: u.username,
                email: u.email,
            }),
        )
            .into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "Utilisateur introuvable").into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── POST /api/profile/password ──────────────────────────────────

pub async fn change_password(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    if payload.new_password.len() < 8 {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Le nouveau mot de passe doit contenir au moins 8 caractères",
        )
        .into_response();
    }

    let user = sqlx::query!(
        "SELECT password_hash FROM public.users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => return err(StatusCode::NOT_FOUND, "Utilisateur introuvable").into_response(),
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    };

    let is_valid = verify(&payload.current_password, &user.password_hash).unwrap_or(false);
    if !is_valid {
        return err(StatusCode::UNAUTHORIZED, "Mot de passe actuel incorrect").into_response();
    }

    let hashed = match hash(&payload.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur de hachage").into_response()
        }
    };

    match sqlx::query!(
        "UPDATE public.users SET password_hash = $1 WHERE id = $2",
        hashed,
        user_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur mise à jour").into_response(),
    }
}

// ─── GET /api/profile/shares ─────────────────────────────────────

pub async fn get_shares(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 1. Véhicules que je possède — récupère séparément les accès partagés
    let owned_vehicles = sqlx::query!(
        r#"
        SELECT v.id AS vehicle_id, v.make, v.model, v.plate_number
        FROM public.vehicles v
        JOIN public.vehicle_access va
            ON va.vehicle_id = v.id
            AND va.user_id = $1
            AND va.role = 'owner'
        ORDER BY v.make, v.model
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await;

    let owned_vehicles = match owned_vehicles {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    };

    // Pour chaque véhicule possédé, récupère les utilisateurs ayant accès
    let mut owned: Vec<OwnedVehicleAccesses> = Vec::new();
    for v in owned_vehicles {
        let accesses_rows = sqlx::query!(
            r#"
            SELECT u.id AS user_id, u.username, va.role
            FROM public.vehicle_access va
            JOIN public.users u ON u.id = va.user_id
            WHERE va.vehicle_id = $1
              AND va.user_id != $2
              AND va.role != 'owner'
            ORDER BY u.username
            "#,
            v.vehicle_id,
            user_id
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        let accesses = accesses_rows
            .into_iter()
            .map(|r| SharedUser {
                user_id: r.user_id,
                username: r.username,
                role: r.role,
            })
            .collect();

        owned.push(OwnedVehicleAccesses {
            vehicle_id: v.vehicle_id,
            make: v.make,
            model: v.model,
            plate_number: v.plate_number,
            accesses,
        });
    }

    // 2. Véhicules partagés avec moi
    let shared_rows = sqlx::query!(
        r#"
        SELECT v.id AS vehicle_id, v.make, v.model, v.plate_number, va.role
        FROM public.vehicles v
        JOIN public.vehicle_access va
            ON va.vehicle_id = v.id
            AND va.user_id = $1
            AND va.role != 'owner'
        ORDER BY v.make, v.model
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await;

    let shared_with_me = match shared_rows {
        Ok(rows) => rows
            .into_iter()
            .map(|r| SharedVehicle {
                vehicle_id: r.vehicle_id,
                make: r.make,
                model: r.model,
                plate_number: r.plate_number,
                role: r.role,
            })
            .collect(),
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    };

    (
        StatusCode::OK,
        Json(ProfileShares {
            owned,
            shared_with_me,
        }),
    )
        .into_response()
}

// ─── DELETE /api/vehicles/:id/access/:user_id ────────────────────

pub async fn revoke_access(
    AuthenticatedUser(requester_id): AuthenticatedUser,
    Path((vehicle_id, target_user_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Vérifie que le demandeur est owner
    let role = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        requester_id
    )
    .fetch_optional(&state.db)
    .await;

    match role {
        Ok(Some(r)) if r == "owner" => {}
        Ok(Some(_)) => {
            return err(StatusCode::FORBIDDEN, "Réservé au propriétaire").into_response()
        }
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                "Véhicule introuvable ou accès refusé",
            )
            .into_response()
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    }

    // Ne peut pas révoquer le propriétaire
    if target_user_id == requester_id {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Impossible de révoquer votre propre accès",
        )
        .into_response();
    }

    match sqlx::query!(
        "DELETE FROM public.vehicle_access WHERE vehicle_id = $1 AND user_id = $2 AND role != 'owner'",
        vehicle_id,
        target_user_id,
    )
    .execute(&state.db)
    .await {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, "Accès introuvable").into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur suppression").into_response(),
    }
}

// ─── DELETE /api/vehicles/:id/leave ──────────────────────────────

pub async fn leave_vehicle(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let role = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    match role {
        Ok(Some(r)) if r == "owner" => {
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Le propriétaire ne peut pas quitter son véhicule",
            )
            .into_response();
        }
        Ok(None) => return err(StatusCode::NOT_FOUND, "Accès introuvable").into_response(),
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
        _ => {}
    }

    match sqlx::query!(
        "DELETE FROM public.vehicle_access WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur suppression").into_response(),
    }
}
