// send-broadcast — Envoi d'un message broadcast à tous les utilisateurs

use backend::secrets::load_secrets;
use clap::Parser;
use sqlx::PgPool;
use std::env;

#[derive(Parser)]
#[command(
    name = "send-broadcast",
    about = "Envoie un message broadcast affiché à tous les utilisateurs après connexion",
    long_about = "Le message s'affiche une seule fois par utilisateur (suivi en localStorage).\n\
                  Seul le broadcast le plus récent non expiré est affiché.\n\n\
                  Regle Apple 3.1.1 : utilisez --exclude-ios pour les messages\n\
                  contenant des liens de dons (Ko-fi, GitHub Sponsors, etc.)."
)]
struct Args {
    /// Message à afficher aux utilisateurs
    #[arg(long)]
    message: String,

    /// Durée d'affichage en jours (sans cette option : pas d'expiration)
    #[arg(long)]
    days: Option<i64>,

    /// Masquer le message pour les comptes iOS App Store (règle Apple 3.1.1)
    #[arg(long)]
    exclude_ios: bool,
}

#[tokio::main]
async fn main() {
    load_secrets().await;

    let args = Args::parse();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Impossible de se connecter à la base");

    let expires_at = args
        .days
        .map(|d| chrono::Utc::now() + chrono::Duration::days(d));

    let id = sqlx::query_scalar!(
        "INSERT INTO public.broadcasts (message, expires_at, exclude_ios)
         VALUES ($1, $2, $3)
         RETURNING id",
        args.message,
        expires_at,
        args.exclude_ios,
    )
    .fetch_one(&pool)
    .await
    .expect("Erreur lors de l'insertion du broadcast");

    println!("✓ Broadcast créé");
    println!("  ID          : {}", id);
    println!("  Message     : {}", args.message);
    match args.days {
        Some(d) => println!("  Expire dans : {} jour(s)", d),
        None => println!("  Expiration  : aucune"),
    }
    println!(
        "  Exclure iOS : {}",
        if args.exclude_ios { "OUI" } else { "non" }
    );
}
