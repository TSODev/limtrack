use bcrypt::{hash, DEFAULT_COST};
use common::{AccessRole, User, Vehicle, VehicleWithAccess};

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
    State(pool): State<PgPool>,
    // En production, on ajouterait ici notre extracteur ClaimsUser
) -> Result<Json<Vec<VehicleWithAccess>>, StatusCode> {
    // Simulation d'un ID utilisateur (à remplacer par user.0.sub plus tard)
    let mock_user_id = Uuid::parse_str("a7985c6d-7acd-4384-ade3-c5764dd8edf0").unwrap();

    let vehicles = sqlx::query_as!(
        VehicleWithAccess,
        r#"
        SELECT v.id as "id!", v.make, v.model, v.plate_number, v.owner_id, va.role as "my_role: AccessRole"
        FROM public.vehicles v
        JOIN public.vehicle_access va ON v.id = va.vehicle_id
        WHERE va.user_id = $1
        "#,
        mock_user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("Erreur SQL: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(vehicles))
}
