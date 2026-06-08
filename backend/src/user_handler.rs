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
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
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

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

// ─── Réponses ────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub is_ios: bool,
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

const MAX_LEN_USERNAME: usize = 50;
const MAX_LEN_EMAIL: usize = 254;
const MAX_LEN_PASSWORD: usize = 1000; // évite le bcrypt DoS sur entrées géantes

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
        "SELECT id, password_hash FROM public.users WHERE username = $1 OR LOWER(email) = LOWER($1)",
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
    if payload.username.len() > MAX_LEN_USERNAME {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("username : {MAX_LEN_USERNAME} caractères max")).into_response();
    }
    if payload.email.len() > MAX_LEN_EMAIL {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("email : {MAX_LEN_EMAIL} caractères max")).into_response();
    }
    if payload.password.len() > MAX_LEN_PASSWORD {
        return err(StatusCode::UNPROCESSABLE_ENTITY, "Mot de passe trop long").into_response();
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
        "SELECT id, username, email, is_admin, is_ios FROM public.users WHERE id = $1",
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
                is_ios: u.is_ios,
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

// ─── POST /api/user/forgot-password ──────────────────────────────

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> impl IntoResponse {
    let email = payload.email.trim().to_lowercase();

    let user = sqlx::query!(
        "SELECT id, username FROM public.users WHERE LOWER(email) = $1",
        email
    )
    .fetch_optional(&state.db)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        // Ne pas révéler si l'email existe ou non
        Ok(None) => return StatusCode::OK.into_response(),
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    let raw_token = Uuid::new_v4().to_string();
    let token_hash = format!("{:x}", Sha256::digest(raw_token.as_bytes()));
    let expires_at = Utc::now() + Duration::hours(1);

    let update = sqlx::query!(
        "UPDATE public.users SET password_reset_token = $1, password_reset_expires_at = $2 WHERE id = $3",
        token_hash,
        expires_at,
        user.id
    )
    .execute(&state.db)
    .await;

    if update.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response();
    }

    if !state.resend_api_key.is_empty() {
        let reset_link = format!("https://limtrack.app/reset-password?token={}", raw_token);
        let html = build_reset_email_html(&user.username, &reset_link);
        let _ = Client::new()
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", state.resend_api_key))
            .json(&json!({
                "from": "LimTrack <noreply@limtrack.app>",
                "to": [&email],
                "subject": "Réinitialisation de votre mot de passe LimTrack",
                "html": html,
            }))
            .send()
            .await;
    }

    StatusCode::OK.into_response()
}

// ─── POST /api/user/reset-password ───────────────────────────────

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    let token_hash = format!("{:x}", Sha256::digest(payload.token.as_bytes()));

    let user = sqlx::query!(
        r#"
        SELECT id, username, email
        FROM public.users
        WHERE password_reset_token = $1
          AND password_reset_expires_at > NOW()
        "#,
        token_hash
    )
    .fetch_optional(&state.db)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => return err(StatusCode::BAD_REQUEST, "Lien invalide ou expiré").into_response(),
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    if let Err(msg) = check_password_strength(
        &payload.new_password,
        &[&user.username, &user.email],
    ) {
        return err(StatusCode::UNPROCESSABLE_ENTITY, msg).into_response();
    }

    let hashed = match hash(&payload.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur de hachage").into_response(),
    };

    match sqlx::query!(
        r#"
        UPDATE public.users
        SET password_hash = $1,
            password_reset_token = NULL,
            password_reset_expires_at = NULL
        WHERE id = $2
        "#,
        hashed,
        user.id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur mise à jour").into_response(),
    }
}

fn build_reset_email_html(username: &str, reset_link: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width,initial-scale=1.0"/>
  <title>Réinitialisation — LimTrack</title>
</head>
<body style="margin:0;padding:0;background-color:#f8fafc;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#f8fafc;padding:40px 16px;">
    <tr><td align="center">
      <table width="560" cellpadding="0" cellspacing="0" style="max-width:560px;width:100%;">
        <tr>
          <td style="background:linear-gradient(135deg,#4f46e5 0%,#7c3aed 100%);border-radius:12px 12px 0 0;padding:32px 40px;text-align:center;">
            <p style="margin:0;font-size:28px;font-weight:800;color:#ffffff;letter-spacing:-0.5px;">LimTrack</p>
            <p style="margin:8px 0 0;font-size:13px;color:#c4b5fd;">Gestion de flotte kilométrique</p>
          </td>
        </tr>
        <tr>
          <td style="background:#ffffff;padding:40px;border-left:1px solid #e2e8f0;border-right:1px solid #e2e8f0;">
            <p style="margin:0 0 8px;font-size:18px;font-weight:700;color:#1e293b;">Bonjour {username},</p>
            <p style="margin:0 0 24px;font-size:15px;color:#64748b;line-height:1.6;">
              Vous avez demandé la réinitialisation de votre mot de passe LimTrack.<br/>
              Cliquez sur le bouton ci-dessous pour choisir un nouveau mot de passe.<br/>
              Ce lien est valable <strong>1 heure</strong>.
            </p>
            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;">
              <tr>
                <td align="center">
                  <a href="{reset_link}"
                     style="display:inline-block;background:linear-gradient(135deg,#4f46e5,#7c3aed);color:#ffffff;font-size:15px;font-weight:600;text-decoration:none;padding:14px 32px;border-radius:8px;">
                    Réinitialiser mon mot de passe →
                  </a>
                </td>
              </tr>
            </table>
            <p style="margin:0 0 16px;font-size:13px;color:#94a3b8;line-height:1.6;">
              Si vous n'avez pas demandé cette réinitialisation, ignorez cet email.<br/>
              Votre mot de passe ne sera pas modifié.
            </p>
            <p style="margin:0;font-size:12px;color:#cbd5e1;word-break:break-all;">
              Lien : {reset_link}
            </p>
          </td>
        </tr>
        <tr>
          <td style="background:#f1f5f9;border:1px solid #e2e8f0;border-top:none;border-radius:0 0 12px 12px;padding:20px 40px;text-align:center;">
            <p style="margin:0;font-size:12px;color:#94a3b8;">
              LimTrack · <a href="https://limtrack.app" style="color:#6366f1;text-decoration:none;">limtrack.app</a>
            </p>
          </td>
        </tr>
      </table>
    </td></tr>
  </table>
</body>
</html>"#,
        username = username,
        reset_link = reset_link,
    )
}
