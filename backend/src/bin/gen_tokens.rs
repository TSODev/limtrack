// gen-tokens — Outil CLI de génération de jetons de licence
//
// Usage :
//   cargo run --bin gen-tokens -- --count 10 --days 30
//   cargo run --bin gen-tokens -- --count 1 --days 365
//   cargo run --bin gen-tokens -- --count 1 --lifetime
//
// Les jetons sont insérés en base (token_hash) et affichés en clair UNE SEULE FOIS.

use dotenvy::dotenv;
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let count = parse_arg(&args, "--count").unwrap_or(1usize);
    let lifetime = args.contains(&"--lifetime".to_string());

    // 36 500 jours ≈ 100 ans : sentinelle "illimité"
    let days: i32 = if lifetime {
        36500
    } else {
        let d = parse_arg::<i32>(&args, "--days").unwrap_or(30);
        let valid_durations = [30, 90, 180, 365];
        if !valid_durations.contains(&d) {
            eprintln!(
                "Durée invalide : {}. Valeurs autorisées : {:?}",
                d, valid_durations
            );
            std::process::exit(1);
        }
        d
    };

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Connexion NeonDB impossible");

    let label = if lifetime {
        "∞ (lifetime)".to_string()
    } else {
        format!("{} jour(s)", days)
    };
    println!("Génération de {} jeton(s) {}...\n", count, label);
    println!("{:<30}  {:>14}  {}", "Jeton (en clair)", "Durée", "Statut");
    println!("{}", "-".repeat(62));

    for _ in 0..count {
        let token = generate_token();
        let hash = hash_token(&token);

        let result = sqlx::query!(
            "INSERT INTO public.license_tokens (token_hash, duration_days) VALUES ($1, $2)",
            hash,
            days
        )
        .execute(&pool)
        .await;

        let dur_label = if lifetime { "∞ lifetime".to_string() } else { format!("{} j", days) };
        match result {
            Ok(_) => println!("{:<30}  {:>14}  OK", token, dur_label),
            Err(e) => println!("{:<30}  {:>14}  ERREUR: {}", token, dur_label, e),
        }
    }

    println!("\nConservez ces jetons en lieu sûr. Ils ne peuvent pas être récupérés depuis la base.");
}

/// Génère un jeton format XXXX-XXXX-XXXX-XXXX (lettres majuscules + chiffres)
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

/// SHA-256 du jeton normalisé (sans tirets, majuscules)
fn hash_token(token: &str) -> String {
    let normalized = token.replace('-', "").to_uppercase();
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}

fn parse_arg<T: std::str::FromStr>(args: &[String], flag: &str) -> Option<T> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse().ok())
}
