// notify-expiry — Wrapper CLI pour lancement manuel des notifications
//
// Usage : cargo run --bin notify-expiry
// Variables requises : DATABASE_URL, RESEND_API_KEY

use backend::{notifier::run_notifications, secrets::load_secrets};
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() {
    load_secrets().await;
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let api_key = env::var("RESEND_API_KEY").unwrap_or_default();

    if api_key.is_empty() {
        println!("❌ RESEND_API_KEY absente — vérifiez votre .env");
        std::process::exit(1);
    }
    println!("✓ RESEND_API_KEY présente");

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Connexion NeonDB impossible");
    println!("✓ Connexion NeonDB OK");

    run_notifications(&pool, &api_key).await;
    println!("✓ Terminé");
}
