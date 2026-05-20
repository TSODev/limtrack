// src/user_handler.rs

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::auth::Claims;
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
    // 1. Récupérer l'utilisateur
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

    // 2. Vérifier le mot de passe
    let is_valid = verify(&payload.password, &user.password_hash).unwrap_or(false);

    if !is_valid {
        return err(StatusCode::UNAUTHORIZED, "Identifiants invalides").into_response();
    }

    // 3. Lire JWT_SECRET
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

    // 4. Générer le JWT
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
    // Validation minimale
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

    // 1. Hacher le mot de passe
    let hashed = match hash(&payload.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur de hachage").into_response()
        }
    };

    // 2. Insérer en base
    let result = sqlx::query!(
        r#"
        INSERT INTO public.users (username, email, password_hash)
        VALUES ($1, $2, $3)
        "#,
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
