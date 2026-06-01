// notify-expiry — Wrapper CLI pour lancement manuel des notifications
//
// Usage : cargo run --bin notify-expiry
// Variables requises : DATABASE_URL, RESEND_API_KEY

use backend::notifier::run_notifications;
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let api_key = env::var("RESEND_API_KEY").expect("RESEND_API_KEY manquante");

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Connexion NeonDB impossible");

    run_notifications(&pool, &api_key).await;
}
