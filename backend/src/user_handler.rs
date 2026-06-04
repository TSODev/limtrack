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
use common::{UpdatePreferencesPayload, UserPreferences};
use zxcvbn::zxcvbn;

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
    pub is_admin: bool,
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

fn check_password_strength(password: &str, user_inputs: &[&str]) -> Result<(), String> {
    let estimate = zxcvbn(password, user_inputs);
    if u8::from(estimate.score()) < 3 {
        let msg = estimate
            .feedback()
            .as_ref()
            .and_then(|f| f.warning())
            .map(|w| w.to_string())
            .unwrap_or_else(|| "Mot de passe trop faible.".to_string());
        return Err(msg);
    }
    Ok(())
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

    if let Err(msg) = check_password_strength(
        &payload.password,
        &[payload.username.trim(), payload.email.trim()],
    ) {
        return err(StatusCode::UNPROCESSABLE_ENTITY, msg).into_response();
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
        "SELECT id, username, email, is_admin FROM public.users WHERE id = $1",
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
                is_admin: u.is_admin,
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
    let user = sqlx::query!(
        "SELECT username, email, password_hash FROM public.users WHERE id = $1",
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

    if let Err(msg) = check_password_strength(
        &payload.new_password,
        &[&user.username, &user.email],
    ) {
        return err(StatusCode::UNPROCESSABLE_ENTITY, msg).into_response();
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

// ─── GET /api/profile/preferences ────────────────────────────────

pub async fn get_preferences(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let prefs = sqlx::query!(
        "SELECT notif_days_before, notif_km_percent, updated_once
         FROM public.user_preferences WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    match prefs {
        Ok(Some(p)) => (
            StatusCode::OK,
            Json(UserPreferences {
                notif_days_before: p.notif_days_before,
                notif_km_percent: p.notif_km_percent,
                updated_once: p.updated_once,
            }),
        )
            .into_response(),
        // Pas encore de préférences → retourne les valeurs par défaut
        Ok(None) => (
            StatusCode::OK,
            Json(UserPreferences {
                notif_days_before: 30,
                notif_km_percent: 80,
                updated_once: false,
            }),
        )
            .into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── PUT /api/profile/preferences ────────────────────────────────

pub async fn update_preferences(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<UpdatePreferencesPayload>,
) -> impl IntoResponse {
    // Validation
    if payload.notif_days_before < 1 || payload.notif_days_before > 365 {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "notif_days_before doit être entre 1 et 365",
        )
        .into_response();
    }
    if payload.notif_km_percent < 1 || payload.notif_km_percent > 100 {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "notif_km_percent doit être entre 1 et 100",
        )
        .into_response();
    }

    // UPSERT — crée ou met à jour
    let result = sqlx::query!(
        r#"
        INSERT INTO public.user_preferences (user_id, notif_days_before, notif_km_percent, updated_once)
        VALUES ($1, $2, $3, true)
        ON CONFLICT (user_id) DO UPDATE SET
            notif_days_before = EXCLUDED.notif_days_before,
            notif_km_percent  = EXCLUDED.notif_km_percent,
            updated_once      = true
        "#,
        user_id,
        payload.notif_days_before,
        payload.notif_km_percent,
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(UserPreferences {
                notif_days_before: payload.notif_days_before,
                notif_km_percent: payload.notif_km_percent,
                updated_once: true,
            }),
        )
            .into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur mise à jour").into_response(),
    }
}

// ─── GET /api/profile/shares ─────────────────────────────────────

pub async fn get_shares(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
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
    .await
    {
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

// ─── DELETE /api/profile ─────────────────────────────────────────

pub async fn delete_account(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
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
        return err(StatusCode::UNAUTHORIZED, "Mot de passe incorrect").into_response();
    }

    // Pour chaque entreprise créée par l'utilisateur :
    // - s'il existe un autre admin → lui transférer created_by
    // - sinon → supprimer l'entreprise (cascade : orgs, membres, rôles)
    let owned_companies = sqlx::query_scalar!(
        "SELECT id FROM public.companies WHERE created_by = $1",
        user_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for company_id in owned_companies {
        let other_admin = sqlx::query_scalar!(
            r#"
            SELECT user_id FROM public.fleet_roles
            WHERE company_id = $1
              AND user_id != $2
              AND role = 'admin'
              AND org_id IS NULL
            LIMIT 1
            "#,
            company_id,
            user_id
        )
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        match other_admin {
            Some(new_owner) => {
                let _ = sqlx::query!(
                    "UPDATE public.companies SET created_by = $1 WHERE id = $2",
                    new_owner,
                    company_id
                )
                .execute(&state.db)
                .await;
            }
            None => {
                let _ = sqlx::query!(
                    "DELETE FROM public.companies WHERE id = $1",
                    company_id
                )
                .execute(&state.db)
                .await;
            }
        }
    }

    // Supprimer les rôles fleet dans toutes les entreprises
    let _ = sqlx::query!(
        "DELETE FROM public.fleet_roles WHERE user_id = $1",
        user_id
    )
    .execute(&state.db)
    .await;

    // Supprimer les memberships dans toutes les entreprises
    let _ = sqlx::query!(
        "DELETE FROM public.company_members WHERE user_id = $1",
        user_id
    )
    .execute(&state.db)
    .await;

    // Supprimer les véhicules owned (cascade : contrats, km, accès, codes)
    let vehicle_ids = sqlx::query_scalar!(
        r#"
        SELECT v.id FROM public.vehicles v
        JOIN public.vehicle_access va ON va.vehicle_id = v.id
        WHERE va.user_id = $1 AND va.role = 'owner'
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for vid in vehicle_ids {
        let _ = sqlx::query!("DELETE FROM public.vehicles WHERE id = $1", vid)
            .execute(&state.db)
            .await;
    }

    // Supprimer le compte (cascade : préférences, accès partagés)
    match sqlx::query!("DELETE FROM public.users WHERE id = $1", user_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur suppression").into_response(),
    }
}
