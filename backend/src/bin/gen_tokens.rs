// gen-tokens — Génération de jetons de licence LimTrack

use backend::secrets::load_secrets;
use clap::Parser;
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::env;

#[derive(Parser)]
#[command(
    name = "gen-tokens",
    about = "Génère des jetons de licence LimTrack et les insère en base",
    long_about = "Génère un ou plusieurs jetons de licence (format XXXX-XXXX-XXXX-XXXX),\n\
                  les insère en base (token_hash SHA-256) et les affiche en clair UNE SEULE FOIS."
)]
struct Args {
    /// Nombre de jetons à générer
    #[arg(long, default_value_t = 1)]
    count: usize,

    /// Durée en jours — valeurs autorisées : 30, 90, 180, 365 (ignoré si --lifetime)
    #[arg(long, default_value_t = 30)]
    days: i32,

    /// Jeton illimité (≈ 100 ans). Remplace --days
    #[arg(long)]
    lifetime: bool,

    /// Type fleet (gestion de flotte). Par défaut : personal
    #[arg(long)]
    fleet: bool,
}

#[tokio::main]
async fn main() {
    load_secrets().await;

    let args = Args::parse();

    let license_type = if args.fleet { "fleet" } else { "personal" };

    let days: i32 = if args.lifetime {
        36500
    } else {
        let valid = [30, 90, 180, 365];
        if !valid.contains(&args.days) {
            eprintln!(
                "Durée invalide : {}. Valeurs autorisées : {:?}",
                args.days, valid
            );
            std::process::exit(1);
        }
        args.days
    };

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Connexion NeonDB impossible");

    let dur_label = if args.lifetime {
        "∞ lifetime".to_string()
    } else {
        format!("{} j", days)
    };

    println!(
        "Génération de {} jeton(s) {} [{}]...\n",
        args.count, dur_label, license_type
    );
    println!(
        "{:<30}  {:>14}  {:>10}  {}",
        "Jeton (en clair)", "Durée", "Type", "Statut"
    );
    println!("{}", "-".repeat(72));

    for _ in 0..args.count {
        let token = generate_token();
        let hash = hash_token(&token);

        let result = sqlx::query!(
            "INSERT INTO public.license_tokens (token_hash, duration_days, license_type) VALUES ($1, $2, $3)",
            hash,
            days,
            license_type
        )
        .execute(&pool)
        .await;

        match result {
            Ok(_) => println!("{:<30}  {:>14}  {:>10}  OK", token, dur_label, license_type),
            Err(e) => println!(
                "{:<30}  {:>14}  {:>10}  ERREUR: {}",
                token, dur_label, license_type, e
            ),
        }
    }

    println!("\nConservez ces jetons en lieu sûr. Ils ne peuvent pas être récupérés depuis la base.");
}

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let charset: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    (0..4)
        .map(|_| {
            (0..4)
                .map(|_| charset[rng.gen_range(0..charset.len())])
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn hash_token(token: &str) -> String {
    let normalized = token.replace('-', "").to_uppercase();
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}
