// src/license_middleware.rs — Vérification d'accès (trial ou licence active)

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::Claims;
use crate::state::AppState;

#[derive(Serialize)]
struct ApiError {
    error: String,
}

// Routes exemptées de la vérification de licence
const EXEMPT_PATHS: &[&str] = &[
    "/login",
    "/api/user/register",
    "/api/profile/license",
    "/api/profile/redeem",
];

pub async fn check_license(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();

    // Laisser passer les routes exemptées
    if EXEMPT_PATHS.iter().any(|p| path == *p) {
        return next.run(request).await;
    }

    // Extraire l'user_id depuis le JWT (sans bloquer si absent — AuthenticatedUser s'en charge)
    let user_id = extract_user_id(&request);
    let user_id = match user_id {
        Some(id) => id,
        None => return next.run(request).await, // pas de JWT → AuthenticatedUser rejetera
    };

    // Vérifier l'accès en base
    let row = sqlx::query!(
        "SELECT trial_ends_at, access_expires_at FROM public.users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        _ => return next.run(request).await, // en cas d'erreur, on laisse passer
    };

    let now = Utc::now();
    let has_access = row.trial_ends_at > now
        || row.access_expires_at.map_or(false, |e| e > now);

    if !has_access && request.method() != axum::http::Method::GET {
        return (
            StatusCode::PAYMENT_REQUIRED,
            Json(ApiError {
                error: "Licence expirée. Veuillez activer un jeton pour continuer.".to_string(),
            }),
        )
            .into_response();
    }

    next.run(request).await
}

fn extract_user_id(request: &Request<Body>) -> Option<Uuid> {
    let auth_header = request
        .headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;

    let secret = std::env::var("JWT_SECRET").ok()?;
    let token_data = decode::<Claims>(
        auth_header,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .ok()?;

    Uuid::parse_str(&token_data.claims.sub).ok()
}
