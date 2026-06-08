// src/main.rs

mod admin_handler;
mod auth;
mod broadcast_handler;
mod ios_handler;
mod company_handler;
mod contracts_handler;
mod license_handler;
mod license_middleware;
mod mileage_handler;
mod notifier;
mod request_license_handler;
mod secrets;
mod share_handler;
mod state;
mod user_handler;
mod vehicles_handler;

use crate::contracts_handler::{
    create_insurance, create_loa, delete_insurance, delete_loa, list_insurance, list_loa,
    renew_insurance, run_insurance_renewals, update_insurance, update_loa,
};
use crate::mileage_handler::{create_mileage, delete_mileage, list_mileage};
use crate::share_handler::{create_share_code, join_with_code};
use crate::state::AppState;
use crate::user_handler::{
    change_password,
    delete_account,
    forgot_password,
    get_preferences,
    get_profile,
    get_shares,
    leave_vehicle,
    login,
    register,
    reset_password,
    revoke_access,
    update_preferences,
};
use crate::admin_handler::{generate_token_handler, get_stats, list_companies_admin, list_license_requests, list_users};
use crate::broadcast_handler::get_active_broadcast;
use crate::ios_handler::ios_activate;
use crate::license_handler::{get_license, redeem_token};
use crate::request_license_handler::request_license;
use crate::company_handler::{
    add_member, assign_fleet_role, assign_vehicle_to_fleet, create_company, create_organization,
    delete_company, delete_organization, fleet_report, get_company, list_companies,
    list_fleet_roles, list_fleet_vehicles, list_members, list_org_vehicles, list_organizations,
    remove_member, remove_vehicle_from_fleet, revoke_fleet_role,
};
use crate::vehicles_handler::{
    archive_vehicle, create_vehicle, delete_vehicle, get_vehicle, list_archived_vehicles,
    list_vehicles, unarchive_vehicle, update_vehicle,
};

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;

use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    secrets::load_secrets().await;

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL manquante");

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Impossible de se connecter à NeonDB");

    let resend_api_key = std::env::var("RESEND_API_KEY").unwrap_or_default();
    let state = AppState { db: pool.clone(), resend_api_key: resend_api_key.clone() };
    let notif_pool = pool;

    info!("Le backend démarre...");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
        .allow_headers(Any);

    let app = Router::new()
        // Auth (user_handler — State<AppState>)
        .route("/login", post(login))
        .route("/api/user/register", post(register))
        .route("/api/user/forgot-password", post(forgot_password))
        .route("/api/user/reset-password", post(reset_password))
        // Vehicles (vehicles_handler — State<AppState>)
        .route("/api/vehicles", get(list_vehicles))
        .route("/api/vehicles", post(create_vehicle))
        .route("/api/vehicles/archived", get(list_archived_vehicles))
        // Contrats LOA
        .route(
            "/api/vehicles/:vehicle_id/contracts/loa",
            get(list_loa).post(create_loa),
        )
        .route(
            "/api/vehicles/:vehicle_id/contracts/loa/:contract_id",
            axum::routing::patch(update_loa).delete(delete_loa),
        )
        // Contrats Assurance
        .route(
            "/api/vehicles/:vehicle_id/contracts/insurance",
            get(list_insurance).post(create_insurance),
        )
        .route(
            "/api/vehicles/:vehicle_id/contracts/insurance/:contract_id",
            axum::routing::patch(update_insurance).delete(delete_insurance),
        )
        .route(
            "/api/vehicles/:vehicle_id/contracts/insurance/:contract_id/renew",
            post(renew_insurance),
        )
        .route(
            "/api/vehicles/:vehicle_id/mileage",
            get(list_mileage).post(create_mileage),
        )
        .route(
            "/api/vehicles/:vehicle_id/mileage/:entry_id",
            axum::routing::delete(delete_mileage),
        )
        .route("/api/vehicles/:id/share", post(create_share_code))
        .route("/api/vehicles/join", post(join_with_code))
        .route("/api/vehicles/:id", get(get_vehicle).delete(delete_vehicle))
        .route("/api/vehicles/:id/archive", axum::routing::patch(archive_vehicle))
        .route("/api/vehicles/:id/unarchive", axum::routing::patch(unarchive_vehicle))
        .route("/api/profile", get(get_profile).delete(delete_account))
        .route("/api/profile/password", post(change_password))
        .route("/api/profile/shares", get(get_shares))
        .route(
            "/api/profile/preferences",
            get(get_preferences).put(update_preferences),
        )
        .route("/api/profile/license", get(get_license))
        .route("/api/profile/redeem", post(redeem_token))
        .route("/api/license/request", post(request_license))
        .route("/api/ios/activate", post(ios_activate))
        .route("/api/broadcasts/active", get(get_active_broadcast))
        // Admin
        .route("/api/admin/stats", get(get_stats))
        .route("/api/admin/users", get(list_users))
        .route("/api/admin/license-requests", get(list_license_requests))
        .route("/api/admin/generate-token", post(generate_token_handler))
        .route("/api/admin/companies", get(list_companies_admin))
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
        .route("/api/companies/:id/fleet-report", get(fleet_report))
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
    println!("🚀 Backend LimTrack lancé sur http://{}", addr);
    info!("Connexion à NeonDB réussie !");

    // Tâche de fond : notifications email d'expiration + renouvellements assurance, à 8h UTC
    let notif_api_key = resend_api_key;
    let renewal_pool = notif_pool.clone();
    if notif_api_key.is_empty() {
        info!("RESEND_API_KEY absente — notifications email désactivées");
    } else {
        tokio::spawn(async move {
            loop {
                let now = chrono::Utc::now();
                let next_8h = {
                    let today_8h = now.date_naive().and_hms_opt(8, 0, 0).unwrap().and_utc();
                    if now < today_8h { today_8h } else { today_8h + chrono::Duration::days(1) }
                };
                let delay_secs = (next_8h - now).num_seconds().max(0) as u64;
                tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                notifier::run_notifications(&notif_pool, &notif_api_key).await;
            }
        });
    }

    // Tâche de fond : renouvellements automatiques des contrats assurance à 8h UTC
    tokio::spawn(async move {
        loop {
            let now = chrono::Utc::now();
            let next_8h = {
                let today_8h = now.date_naive().and_hms_opt(8, 0, 0).unwrap().and_utc();
                if now < today_8h { today_8h } else { today_8h + chrono::Duration::days(1) }
            };
            let delay_secs = (next_8h - now).num_seconds().max(0) as u64;
            tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
            run_insurance_renewals(&renewal_pool).await;
        }
    });

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
