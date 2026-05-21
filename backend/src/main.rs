// src/main.rs

mod auth;
mod contracts_handler;
mod mileage_handler;
mod share_handler;
mod state;
mod user_handler;
mod vehicles_handler;

use crate::contracts_handler::{create_insurance, create_loa, list_insurance, list_loa};
use crate::mileage_handler::{create_mileage, list_mileage};
use crate::share_handler::{create_share_code, join_with_code};
use crate::state::AppState;
use crate::user_handler::{
    change_password,
    get_preferences,
    get_profile,
    get_shares,
    leave_vehicle,
    login,
    register,
    revoke_access,
    update_preferences, // <-- nouveaux
};
use crate::vehicles_handler::{
    create_vehicle, delete_vehicle, get_vehicle, list_vehicles, update_vehicle,
};

use axum::{
    routing::{delete, get, post},
    Router,
};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::net::SocketAddr;

use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL doit être définie dans .env");

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Impossible de se connecter à NeonDB");

    let state = AppState { db: pool };

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Le backend démarre...");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        // Auth (user_handler — State<AppState>)
        .route("/login", post(login))
        .route("/api/user/register", post(register))
        // Vehicles (vehicles_handler — State<AppState>)
        .route("/api/vehicles", get(list_vehicles))
        .route("/api/vehicles", post(create_vehicle))
        // Contrats LOA
        .route(
            "/api/vehicles/:vehicle_id/contracts/loa",
            get(list_loa).post(create_loa),
        )
        // Contrats Assurance
        .route(
            "/api/vehicles/:vehicle_id/contracts/insurance",
            get(list_insurance).post(create_insurance),
        )
        .route(
            "/api/vehicles/:vehicle_id/mileage",
            get(list_mileage).post(create_mileage),
        )
        .route("/api/vehicles/:id/share", post(create_share_code))
        .route("/api/vehicles/join", post(join_with_code))
        .route("/api/vehicles/:id", get(get_vehicle).delete(delete_vehicle))
        .route("/api/profile", get(get_profile))
        .route("/api/profile/password", post(change_password))
        .route("/api/profile/shares", get(get_shares))
        .route(
            "/api/profile/preferences",
            get(get_preferences).put(update_preferences),
        )
        .route("/api/vehicles/:id/access/:user_id", delete(revoke_access))
        .route("/api/vehicles/:id/leave", delete(leave_vehicle))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri    = %request.uri(),
                )
            }),
        )
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Backend ODO lancé sur http://{}", addr);
    info!("Connexion à NeonDB réussie !");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
