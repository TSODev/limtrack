// src/main.rs

mod auth;
mod company_handler;
mod contracts_handler;
mod license_handler;
mod license_middleware;
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
    update_preferences,
    delete_account,
};
use crate::license_handler::{get_license, redeem_token};
use crate::company_handler::{
    add_member, assign_fleet_role, assign_vehicle_to_fleet, create_company, create_organization,
    delete_company, delete_organization, get_company, list_companies, list_fleet_roles,
    list_fleet_vehicles, list_members, list_org_vehicles, list_organizations, remove_member,
    remove_vehicle_from_fleet, revoke_fleet_role,
};
use crate::vehicles_handler::{
    create_vehicle, delete_vehicle, get_vehicle, list_vehicles, update_vehicle,
};

use axum::{
    middleware,
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
        .route("/api/profile", get(get_profile).delete(delete_account))
        .route("/api/profile/password", post(change_password))
        .route("/api/profile/shares", get(get_shares))
        .route(
            "/api/profile/preferences",
            get(get_preferences).put(update_preferences),
        )
        .route("/api/profile/license", get(get_license))
        .route("/api/profile/redeem", post(redeem_token))
        .route("/api/vehicles/:id/access/:user_id", delete(revoke_access))
        .route("/api/vehicles/:id/leave", delete(leave_vehicle))
        // Fleet : véhicule → entreprise
        .route(
            "/api/vehicles/:id/fleet",
            post(assign_vehicle_to_fleet).delete(remove_vehicle_from_fleet),
        )
        // Entreprises
        .route("/api/companies", get(list_companies).post(create_company))
        .route(
            "/api/companies/:id",
            get(get_company).delete(delete_company),
        )
        // Organisations
        .route(
            "/api/companies/:id/organizations",
            get(list_organizations).post(create_organization),
        )
        .route(
            "/api/companies/:id/organizations/:oid",
            delete(delete_organization),
        )
        // Membres
        .route(
            "/api/companies/:id/members",
            get(list_members).post(add_member),
        )
        .route(
            "/api/companies/:id/members/:uid",
            delete(remove_member),
        )
        // Rôles fleet
        .route(
            "/api/companies/:id/fleet-roles",
            get(list_fleet_roles).post(assign_fleet_role),
        )
        .route(
            "/api/companies/:id/fleet-roles/:role_id",
            delete(revoke_fleet_role),
        )
        // Vue flotte
        .route("/api/companies/:id/vehicles", get(list_fleet_vehicles))
        .route(
            "/api/companies/:id/organizations/:oid/vehicles",
            get(list_org_vehicles),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            license_middleware::check_license,
        ))
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

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("🚀 Backend ODO lancé sur http://{}", addr);
    info!("Connexion à NeonDB réussie !");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
