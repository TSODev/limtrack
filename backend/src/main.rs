use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
    routing::post,
    Json, Router,
};
use common::{AccessRole, Vehicle, VehicleWithAccess};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;
use uuid::Uuid;

use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info, instrument};
use tracing_subscriber;

mod handlers; // Déclare le module handler

#[tokio::main]
async fn main() {
    // 1. Charge les variables du fichier .env dans l'environnement du processus
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .expect("La variable DATABASE_URL doit être définie dans le fichier .env");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Impossible de se connecter à la base de données Neon");

    // 1. Initialise le tracing
    tracing_subscriber::fmt()
        .pretty() // Ajoute des couleurs et indente le JSON/Structs
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        //        .add_directive(tracing::Level::DEBUG.into()))
        .init();

    info!("Le backend démarre...");

    // Configuration CORS
    let cors = CorsLayer::new()
        // Autorise ton frontend (ex: Tauri ou React)
        // .allow_origin("http://localhost:1420".parse::<HeaderValue>().unwrap())
        // Ou plus simple pour le dev :
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/vehicles", get(handlers::list_vehicles))
        .route("/api/user/register", post(handlers::register))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Backend ODO lancé sur http://{}", addr);
    info!("Connexion à la base de données Neon réussie !");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
