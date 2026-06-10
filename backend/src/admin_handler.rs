// src/admin_handler.rs — Dashboard administrateur

use axum::{
    async_trait,
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;
use uuid::Uuid;

use crate::auth::Claims;
use crate::state::AppState;
use jsonwebtoken::{decode, DecodingKey, Validation};

// ─── Extracteur AdminUser ─────────────────────────────────────

pub struct AdminUser(pub Uuid);

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let err = |msg: &str| {
            (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": msg})),
            )
        };

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| err("Token manquant"))?;

        let secret = std::env::var("JWT_SECRET")
            .map_err(|_| err("Erreur configuration"))?;

        let token_data = decode::<Claims>(
            auth_header,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| err("Token invalide"))?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| err("ID invalide"))?;

        let is_admin = sqlx::query_scalar!(
            "SELECT is_admin FROM public.users WHERE id = $1",
            user_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|_| err("Erreur base de données"))?
        .unwrap_or(false);

        if !is_admin {
            return Err(err("Accès refusé — admin uniquement"));
        }

        Ok(AdminUser(user_id))
    }
}

// ─── Types réponse ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub trial: i64,
    pub active: i64,
    pub expired: i64,
    pub total_vehicles: i64,
    pub total_license_requests: i64,
}

#[derive(Serialize)]
pub struct AdminUser_ {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub is_ios: bool,
    pub license_type: String,
    pub created_at: chrono::DateTime<Utc>,
    pub trial_ends_at: chrono::DateTime<Utc>,
    pub access_expires_at: Option<chrono::DateTime<Utc>>,
    pub status: String,
}

#[derive(Serialize)]
pub struct AdminLicenseRequest {
    pub email: String,
    pub requested_at: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct GenerateTokenPayload {
    pub email: Option<String>,
    pub days: i32,
    pub license_type: String,
}

#[derive(Serialize)]
pub struct GenerateTokenResponse {
    pub token: String,
    pub assigned_to: Option<String>,
}

#[derive(Serialize)]
pub struct GrowthPoint {
    pub week: String,
    pub count: i64,
}

#[derive(Deserialize)]
pub struct PatchUserPayload {
    pub username: Option<String>,
    pub email: Option<String>,
    pub is_admin: Option<bool>,
    pub is_ios: Option<bool>,
    pub license_type: Option<String>,
    pub access_expires_at: Option<chrono::DateTime<Utc>>,
}

// ─── Helpers ────────────────────────────────────────────────────

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let charset: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    (0..4)
        .map(|_| {
            (0..4)
                .map(|_| charset[rng.gen_range(0..charset.len())])
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn hash_token(token: &str) -> String {
    let normalized = token.replace('-', "").to_uppercase();
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}

fn license_status(
    trial_ends_at: chrono::DateTime<Utc>,
    access_expires_at: Option<chrono::DateTime<Utc>>,
) -> String {
    let now = Utc::now();
    if access_expires_at.map_or(false, |e| e > now) {
        "active".to_string()
    } else if trial_ends_at > now {
        "trial".to_string()
    } else {
        "expired".to_string()
    }
}

// ─── GET /api/admin/stats ──────────────────────────────────────

pub async fn get_stats(
    AdminUser(_): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let now = Utc::now();

    let total_users = sqlx::query_scalar!("SELECT COUNT(*) FROM public.users")
        .fetch_one(&state.db)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);

    let active = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.users WHERE access_expires_at > $1",
        now
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    let trial = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.users WHERE trial_ends_at > $1 AND (access_expires_at IS NULL OR access_expires_at <= $1)",
        now
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    let expired = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.users WHERE trial_ends_at <= $1 AND (access_expires_at IS NULL OR access_expires_at <= $1)",
        now
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    let total_vehicles = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.vehicles WHERE archived_at IS NULL"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    let total_license_requests =
        sqlx::query_scalar!("SELECT COUNT(*) FROM public.license_requests")
            .fetch_one(&state.db)
            .await
            .unwrap_or(Some(0))
            .unwrap_or(0);

    Json(AdminStats {
        total_users,
        trial,
        active,
        expired,
        total_vehicles,
        total_license_requests,
    })
}

// ─── GET /api/admin/users ──────────────────────────────────────

pub async fn list_users(
    AdminUser(_): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rows = sqlx::query!(
        "SELECT id, username, email, is_admin, is_ios, license_type, created_at, trial_ends_at, access_expires_at
         FROM public.users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let users: Vec<AdminUser_> = rows
                .into_iter()
                .map(|r| {
                    let status = license_status(r.trial_ends_at, r.access_expires_at);
                    AdminUser_ {
                        id: r.id,
                        username: r.username,
                        email: r.email,
                        is_admin: r.is_admin,
                        is_ios: r.is_ios,
                        license_type: r.license_type,
                        created_at: r.created_at.unwrap_or_else(Utc::now),
                        trial_ends_at: r.trial_ends_at,
                        access_expires_at: r.access_expires_at,
                        status,
                    }
                })
                .collect();
            Json(users).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur base de données"})),
        )
            .into_response(),
    }
}

// ─── PATCH /api/admin/users/:id ────────────────────────────────

pub async fn patch_user_admin(
    AdminUser(_): AdminUser,
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<PatchUserPayload>,
) -> impl IntoResponse {
    if let Some(ref lt) = payload.license_type {
        if lt != "personal" && lt != "fleet" {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "license_type invalide (personal | fleet)"})),
            )
                .into_response();
        }
    }

    let current = sqlx::query!(
        "SELECT username, email, is_admin, is_ios, access_expires_at, license_type
         FROM public.users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    let current = match current {
        Ok(Some(r)) => r,
        Ok(None) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Utilisateur introuvable"})),
        ).into_response(),
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur base de données"})),
        ).into_response(),
    };

    if let Some(ref new_username) = payload.username {
        let exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM public.users WHERE username = $1 AND id != $2",
            new_username,
            user_id
        )
        .fetch_one(&state.db)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);
        if exists > 0 {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "Ce nom d'utilisateur est déjà pris"})),
            )
                .into_response();
        }
    }

    if let Some(ref new_email) = payload.email {
        let exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM public.users WHERE email = $1 AND id != $2",
            new_email,
            user_id
        )
        .fetch_one(&state.db)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);
        if exists > 0 {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "Cet email est déjà utilisé"})),
            )
                .into_response();
        }
    }

    let username = payload.username.as_deref().unwrap_or(&current.username);
    let email = payload.email.as_deref().unwrap_or(&current.email);
    let is_admin = payload.is_admin.unwrap_or(current.is_admin);
    let is_ios = payload.is_ios.unwrap_or(current.is_ios);
    let license_type = payload.license_type.as_deref().unwrap_or(&current.license_type);
    let access_expires_at = if payload.access_expires_at.is_some() {
        payload.access_expires_at
    } else {
        current.access_expires_at
    };

    match sqlx::query!(
        "UPDATE public.users
         SET username = $2, email = $3, is_admin = $4, is_ios = $5, license_type = $6, access_expires_at = $7
         WHERE id = $1",
        user_id,
        username,
        email,
        is_admin,
        is_ios,
        license_type,
        access_expires_at
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur mise à jour"})),
        )
            .into_response(),
    }
}

// ─── GET /api/admin/growth ─────────────────────────────────────

pub async fn get_growth(
    AdminUser(_): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user_rows = sqlx::query!(
        r#"
        SELECT date_trunc('week', created_at)::date::text AS "week!",
               COUNT(*) AS "count!"
        FROM public.users
        WHERE created_at >= NOW() - INTERVAL '11 weeks'
        GROUP BY date_trunc('week', created_at)
        ORDER BY 1 DESC
        "#
    )
    .fetch_all(&state.db)
    .await;

    let vehicle_rows = sqlx::query!(
        r#"
        SELECT date_trunc('week', created_at)::date::text AS "week!",
               COUNT(*) AS "count!"
        FROM public.vehicles
        WHERE created_at >= NOW() - INTERVAL '11 weeks'
        GROUP BY date_trunc('week', created_at)
        ORDER BY 1 DESC
        "#
    )
    .fetch_all(&state.db)
    .await;

    match (user_rows, vehicle_rows) {
        (Ok(u), Ok(v)) => {
            let result = serde_json::json!({
                "users": u.into_iter().map(|r| serde_json::json!({"week": r.week, "count": r.count})).collect::<Vec<_>>(),
                "vehicles": v.into_iter().map(|r| serde_json::json!({"week": r.week, "count": r.count})).collect::<Vec<_>>(),
            });
            (StatusCode::OK, Json(result)).into_response()
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur base de données"})),
        )
            .into_response(),
    }
}

// ─── GET /api/admin/license-requests ──────────────────────────

pub async fn list_license_requests(
    AdminUser(_): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rows = sqlx::query!(
        "SELECT email, requested_at FROM public.license_requests ORDER BY requested_at DESC"
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let requests: Vec<AdminLicenseRequest> = rows
                .into_iter()
                .map(|r| AdminLicenseRequest {
                    email: r.email,
                    requested_at: r.requested_at,
                })
                .collect();
            Json(requests).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur base de données"})),
        )
            .into_response(),
    }
}

// ─── POST /api/admin/generate-token ───────────────────────────

pub async fn generate_token_handler(
    AdminUser(admin_id): AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<GenerateTokenPayload>,
) -> impl IntoResponse {
    let valid_days = [30, 90, 180, 365, 36500];
    if !valid_days.contains(&payload.days) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Durée invalide (30, 90, 180, 365, 36500)"})),
        )
            .into_response();
    }
    if payload.license_type != "personal" && payload.license_type != "fleet" {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Type invalide (personal, fleet)"})),
        )
            .into_response();
    }

    let token = generate_token();
    let hash = hash_token(&token);

    if let Err(_) = sqlx::query!(
        "INSERT INTO public.license_tokens (token_hash, duration_days, license_type) VALUES ($1, $2, $3)",
        hash,
        payload.days,
        payload.license_type
    )
    .execute(&state.db)
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur création jeton"})),
        )
            .into_response();
    }

    let mut assigned_to: Option<String> = None;
    if let Some(ref email) = payload.email {
        let email = email.trim().to_lowercase();
        if !email.is_empty() {
            let user = sqlx::query!(
                "SELECT id, access_expires_at FROM public.users WHERE email = $1",
                email
            )
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

            if let Some(user) = user {
                let base = user
                    .access_expires_at
                    .filter(|e| *e > Utc::now())
                    .unwrap_or_else(Utc::now);
                let new_expiry = base + chrono::Duration::days(payload.days as i64);

                let _ = sqlx::query!(
                    "UPDATE public.license_tokens SET used_at = NOW(), used_by = $1 WHERE token_hash = $2",
                    user.id,
                    hash
                )
                .execute(&state.db)
                .await;

                let _ = sqlx::query!(
                    "UPDATE public.users SET access_expires_at = $1, license_type = $2 WHERE id = $3",
                    new_expiry,
                    payload.license_type,
                    user.id
                )
                .execute(&state.db)
                .await;

                assigned_to = Some(email.clone());
                info!("admin {}: jeton {} assigné à {}", admin_id, token, email);
            }
        }
    }

    Json(GenerateTokenResponse { token, assigned_to }).into_response()
}

// ─── POST /api/admin/assign-license ───────────────────────────

#[derive(Deserialize)]
pub struct AssignLicensePayload {
    pub email: String,
    pub token: String,
}

pub async fn assign_license_handler(
    AdminUser(admin_id): AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<AssignLicensePayload>,
) -> impl IntoResponse {
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || payload.token.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "email et token requis"})),
        )
            .into_response();
    }

    let token_hash = hash_token(payload.token.trim());

    let token = sqlx::query!(
        "SELECT id, duration_days, license_type, used_at FROM public.license_tokens WHERE token_hash = $1",
        token_hash
    )
    .fetch_optional(&state.db)
    .await;

    let token = match token {
        Ok(Some(t)) => t,
        Ok(None) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Jeton invalide ou inexistant"})),
        ).into_response(),
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur base de données"})),
        ).into_response(),
    };

    if token.used_at.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "Ce jeton a déjà été utilisé"})),
        ).into_response();
    }

    let user = sqlx::query!(
        "SELECT id, access_expires_at FROM public.users WHERE email = $1",
        email
    )
    .fetch_optional(&state.db)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Utilisateur introuvable"})),
        ).into_response(),
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur base de données"})),
        ).into_response(),
    };

    let base = user.access_expires_at.filter(|e| *e > Utc::now()).unwrap_or_else(Utc::now);
    let new_expiry = base + chrono::Duration::days(token.duration_days as i64);

    let _ = sqlx::query!(
        "UPDATE public.license_tokens SET used_at = NOW(), used_by = $1 WHERE id = $2",
        user.id,
        token.id
    )
    .execute(&state.db)
    .await;

    match sqlx::query!(
        "UPDATE public.users SET access_expires_at = $1, license_type = $2 WHERE id = $3",
        new_expiry,
        token.license_type,
        user.id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            info!("admin {}: jeton assigné à {}", admin_id, email);
            (StatusCode::OK, Json(serde_json::json!({
                "ok": true,
                "email": email,
                "new_expires_at": new_expiry.format("%Y-%m-%d").to_string(),
                "license_type": token.license_type,
            })))
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur mise à jour"})),
        )
            .into_response(),
    }
}

// ─── POST /api/admin/notify-expiry ────────────────────────────

pub async fn trigger_notify_expiry(
    AdminUser(_): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if state.resend_api_key.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "RESEND_API_KEY non configurée — notifications désactivées"})),
        )
            .into_response();
    }
    crate::notifier::run_notifications(&state.db, &state.resend_api_key).await;
    (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response()
}

// ─── POST /api/admin/broadcasts ───────────────────────────────

#[derive(Deserialize)]
pub struct CreateBroadcastPayload {
    pub message: String,
    pub days: Option<i64>,
    pub exclude_ios: bool,
}

pub async fn create_broadcast(
    AdminUser(_): AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateBroadcastPayload>,
) -> impl IntoResponse {
    let message = payload.message.trim().to_string();
    if message.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "message requis"})),
        )
            .into_response();
    }

    let days = payload.days.unwrap_or(7);
    let expires_at = Utc::now() + chrono::Duration::days(days);

    match sqlx::query!(
        "INSERT INTO public.broadcasts (message, expires_at, exclude_ios) VALUES ($1, $2, $3)",
        message,
        expires_at,
        payload.exclude_ios
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur création broadcast"})),
        )
            .into_response(),
    }
}

// ─── GET /api/admin/companies ─────────────────────────────────

#[derive(Serialize)]
pub struct AdminCompanyMember {
    pub username: String,
    pub email: String,
    pub fleet_role: Option<String>,
}

#[derive(Serialize)]
pub struct AdminCompanyVehicle {
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub org_name: Option<String>,
}

#[derive(Serialize)]
pub struct AdminCompanyOrg {
    pub name: String,
    pub vehicle_count: i64,
}

#[derive(Serialize)]
pub struct AdminCompany {
    pub id: Uuid,
    pub name: String,
    pub siret: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub members: Vec<AdminCompanyMember>,
    pub vehicles: Vec<AdminCompanyVehicle>,
    pub organizations: Vec<AdminCompanyOrg>,
}

pub async fn list_companies_admin(
    AdminUser(_): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let companies = sqlx::query!(
        "SELECT id, name, siret, created_at FROM public.companies ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await;

    let companies = match companies {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Erreur base de données"}))).into_response(),
    };

    let mut result: Vec<AdminCompany> = Vec::new();

    for c in companies {
        let members = sqlx::query!(
            r#"SELECT u.username, u.email,
               (SELECT fr.role FROM public.fleet_roles fr
                WHERE fr.user_id = u.id AND fr.company_id = $1 AND fr.org_id IS NULL
                LIMIT 1) AS fleet_role
               FROM public.company_members cm
               JOIN public.users u ON u.id = cm.user_id
               WHERE cm.company_id = $1
               ORDER BY u.username"#,
            c.id
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| AdminCompanyMember {
            username: r.username,
            email: r.email,
            fleet_role: r.fleet_role,
        })
        .collect();

        let vehicles = sqlx::query!(
            r#"SELECT v.make, v.model, v.plate_number, o.name AS "org_name?"
               FROM public.vehicles v
               LEFT JOIN public.vehicle_access va_fleet
                 ON va_fleet.vehicle_id = v.id
               LEFT JOIN public.organizations o ON o.id = v.company_id
               WHERE v.company_id = $1
               ORDER BY v.make, v.model"#,
            c.id
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| AdminCompanyVehicle {
            make: r.make,
            model: r.model,
            plate_number: r.plate_number,
            org_name: r.org_name,
        })
        .collect();

        let organizations = sqlx::query!(
            r#"SELECT o.name,
               (SELECT COUNT(*) FROM public.vehicles v WHERE v.company_id = $1) AS "vehicle_count!"
               FROM public.organizations o
               WHERE o.company_id = $1
               ORDER BY o.name"#,
            c.id
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| AdminCompanyOrg {
            name: r.name,
            vehicle_count: r.vehicle_count,
        })
        .collect();

        result.push(AdminCompany {
            id: c.id,
            name: c.name,
            siret: c.siret,
            created_at: c.created_at,
            members,
            vehicles,
            organizations,
        });
    }

    Json(result).into_response()
}
