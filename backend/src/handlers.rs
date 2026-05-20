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
