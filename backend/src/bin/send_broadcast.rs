// send-broadcast — Envoi d'un message broadcast à tous les utilisateurs
//
// Usage :
//   cargo run --bin send-broadcast -- --message "Maintenance prévue samedi 23h-01h UTC"
//   cargo run --bin send-broadcast -- --message "..." --days 7
//   cargo run --bin send-broadcast -- --message "..." --days 3 --exclude-ios
//
// --exclude-ios : masque le message pour les comptes iOS App Store
//                (utile pour les messages de dons, Ko-fi, etc. — règle Apple 3.1.1)

use backend::secrets::load_secrets;
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() {
    load_secrets().await;

    let args: Vec<String> = env::args().collect();

    let message = parse_str(&args, "--message").unwrap_or_else(|| {
        eprintln!("Usage: --message \"<texte>\" [--days <n>] [--exclude-ios]");
        std::process::exit(1);
    });

    let days: Option<i64> = parse_i64(&args, "--days");
    let exclude_ios = args.contains(&"--exclude-ios".to_string());

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Impossible de se connecter à la base");

    let expires_at = days.map(|d| chrono::Utc::now() + chrono::Duration::days(d));

    let id = sqlx::query_scalar!(
        "INSERT INTO public.broadcasts (message, expires_at, exclude_ios)
         VALUES ($1, $2, $3)
         RETURNING id",
        message,
        expires_at,
        exclude_ios,
    )
    .fetch_one(&pool)
    .await
    .expect("Erreur lors de l'insertion du broadcast");

    println!("✓ Broadcast créé");
    println!("  ID         : {}", id);
    println!("  Message    : {}", message);
    match days {
        Some(d) => println!("  Expire dans : {} jour(s)", d),
        None => println!("  Expiration  : aucune"),
    }
    println!(
        "  Exclure iOS : {}",
        if exclude_ios { "OUI" } else { "non" }
    );
}

fn parse_str(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

fn parse_i64(args: &[String], flag: &str) -> Option<i64> {
    parse_str(args, flag)?.parse().ok()
}
