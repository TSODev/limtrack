use bcrypt::{hash, DEFAULT_COST};
use common::ApiStatus;
use common::{AccessRole, User, Vehicle, VehicleWithAccess}; // Utilisation du contrat commun

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    response::Response,
    routing::get,
    Json, Router,
};

use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use regex::Regex;
use serde::Serialize;

use crate::auth::AuthenticatedUser;

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
pub struct CreateVehicleRequest {
    pub make: String,
    pub model: String,
    pub plate_number: String,
}

pub async fn get_status() -> Json<ApiStatus> {
    Json(ApiStatus {
        version: "0.1.0-alpha".to_string(),
        online: true,
        message: Some("Le serveur est opérationnel".to_string()),
    })
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Récupérer l'utilisateur en base
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erreur base de données".into(),
        )
    })?
    .ok_or((StatusCode::UNAUTHORIZED, "Identifiants invalides".into()))?;

    // 2. Vérifier le mot de passe
    let is_valid = verify(payload.password, &user.password_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erreur de vérification".into(),
        )
    })?;

    if !is_valid {
        return Err((StatusCode::UNAUTHORIZED, "Identifiants invalides".into()));
    }

    // 3. Créer le JWT
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = crate::auth::Claims {
        sub: user.id.to_string(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erreur génération token".into(),
        )
    })?;

    Ok(Json(LoginResponse { token }))
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Hacher le mot de passe
    let hashed = hash(payload.password, DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erreur de hachage".into(),
        )
    })?;

    // 2. Insérer dans la base
    sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        "#,
        payload.username,
        payload.email,
        hashed
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Erreur lors de la création : {}", e),
        )
    })?;

    Ok(StatusCode::CREATED)
}

// --- Handler pour lister les véhicules ---
pub async fn list_vehicles(
    auth: AuthenticatedUser, // <--- Le garde-barrière est ici !
    State(pool): State<PgPool>,
    // En production, on ajouterait ici notre extracteur ClaimsUser
) -> Result<Json<Vec<VehicleWithAccess>>, StatusCode> {
    let user_id = auth.0; // Voici l'ID de l'utilisateur qui fait la requête

    tracing::info!("L'utilisateur {} demande ses véhicules", user_id);

    let vehicles = sqlx::query_as!(
        VehicleWithAccess,
        r#"
        SELECT v.id as "id!", v.make, v.model, v.plate_number, v.owner_id, va.role as "my_role: AccessRole"
        FROM public.vehicles v
        JOIN public.vehicle_access va ON v.id = va.vehicle_id
        WHERE va.user_id = $1
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("Erreur SQL: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(vehicles))
}

pub async fn create_vehicle(
    auth: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateVehicleRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = auth.0;

    let re = Regex::new(r"^[A-Z]{2}-\d{3}-[A-Z]{2}$").unwrap();
    if !re.is_match(&payload.plate_number) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Format d'immatriculation invalide (attendu : AA-111-AA)".into(),
        ));
    }

    // 1. Démarrer une transaction
    let mut tx = pool.begin().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erreur de transaction".into(),
        )
    })?;

    // 2. Insérer le véhicule
    let vehicle_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO vehicles (id, owner_id, make, model, plate_number)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        vehicle_id,
        user_id,
        payload.make,
        payload.model,
        payload.plate_number
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Erreur création véhicule : {}", e),
        )
    })?;

    // 3. Insérer le droit d'accès "owner" dans la table pivot
    sqlx::query!(
        r#"
        INSERT INTO vehicle_access (vehicle_id, user_id, role)
        VALUES ($1, $2, 'owner')
        "#,
        vehicle_id,
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur création accès : {}", e),
        )
    })?;

    // 4. Valider la transaction
    tx.commit()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Échec du commit".into()))?;

    Ok((StatusCode::CREATED, Json(vehicle_id)))
}
