// notify-expiry — Déclenchement manuel des notifications d'expiration de licence

use backend::{notifier::run_notifications, secrets::load_secrets};
use clap::Parser;
use sqlx::PgPool;
use std::env;

#[derive(Parser)]
#[command(
    name = "notify-expiry",
    about = "Envoie les notifications d'expiration de licence par email (Resend)",
    long_about = "Envoie un email aux utilisateurs dont la licence expire dans 7, 15 ou 30 jours.\n\
                  Anti-doublon 24h : un utilisateur ne reçoit pas deux emails le même jour.\n\n\
                  Variables d'environnement requises :\n\
                    DATABASE_URL    — connexion NeonDB\n\
                    RESEND_API_KEY  — clé API Resend"
)]
struct Args {}

#[tokio::main]
async fn main() {
    load_secrets().await;
    let _args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let api_key = env::var("RESEND_API_KEY").unwrap_or_default();

    if api_key.is_empty() {
        eprintln!("RESEND_API_KEY absente — vérifiez votre .env ou Infisical");
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
