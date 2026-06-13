// assign-license — Assigne un jeton de licence à un utilisateur

use backend::secrets::load_secrets;
use chrono::Utc;
use clap::Parser;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::env;

#[derive(Parser)]
#[command(
    name = "assign-license",
    about = "Assigne un jeton de licence à un utilisateur (manuel ou batch CSV)",
    long_about = "Mode manuel : --email + --token\n\
                  Mode batch  : --file <chemin.csv>\n\n\
                  Format CSV (sans en-tête) :\n\
                    email,token\n\
                    ami@example.com,XXXX-XXXX-XXXX-XXXX"
)]
struct Args {
    /// Email de l'utilisateur (mode manuel, requis avec --token)
    #[arg(long)]
    email: Option<String>,

    /// Jeton de licence (mode manuel, requis avec --email)
    #[arg(long)]
    token: Option<String>,

    /// Fichier CSV à traiter en batch (format : email,token — sans en-tête)
    #[arg(long)]
    file: Option<String>,
}

#[tokio::main]
async fn main() {
    load_secrets().await;

    let args = Args::parse();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Connexion PostgreSQL impossible");

    match (&args.file, &args.email, &args.token) {
        (Some(file), _, _) => {
            run_batch(&pool, file).await;
        }
        (None, Some(email), Some(token)) => {
            let result = assign(&pool, email, token).await;
            print_result(email, token, result);
        }
        _ => {
            eprintln!("Usage :");
            eprintln!("  Mode manuel : --email <email> --token <XXXX-XXXX-XXXX-XXXX>");
            eprintln!("  Mode batch  : --file <chemin.csv>");
            eprintln!("  Aide        : --help");
            std::process::exit(1);
        }
    }
}

async fn run_batch(pool: &PgPool, path: &str) {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .unwrap_or_else(|e| panic!("Impossible de lire {} : {}", path, e));

    println!("{:<35}  {:<25}  {}", "Email", "Token", "Résultat");
    println!("{}", "-".repeat(80));

    for record in rdr.records() {
        let record = match record {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Ligne invalide : {}", e);
                continue;
            }
        };
        if record.len() < 2 {
            eprintln!(
                "Ligne ignorée (format attendu : email,token) : {:?}",
                record
            );
            continue;
        }
        let email = record[0].trim().to_string();
        let token = record[1].trim().to_string();
        let result = assign(pool, &email, &token).await;
        print_result(&email, &token, result);
    }
}

async fn assign(pool: &PgPool, email: &str, token: &str) -> Result<String, String> {
    let user = sqlx::query!("SELECT id FROM public.users WHERE email = $1", email)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Erreur DB : {}", e))?
        .ok_or_else(|| "Utilisateur introuvable".to_string())?;

    let user_id = user.id;

    let hash = hash_token(token);
    let tok = sqlx::query!(
        "SELECT id, duration_days, used_at FROM public.license_tokens WHERE token_hash = $1",
        hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Erreur DB : {}", e))?
    .ok_or_else(|| "Jeton invalide ou inexistant".to_string())?;

    if tok.used_at.is_some() {
        return Err("Jeton déjà utilisé".to_string());
    }

    let current_expiry = sqlx::query_scalar!(
        "SELECT access_expires_at FROM public.users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
    .unwrap_or(None);

    let base = current_expiry
        .filter(|e| *e > Utc::now())
        .unwrap_or_else(Utc::now);

    let new_expiry = base + chrono::Duration::days(tok.duration_days as i64);

    sqlx::query!(
        "UPDATE public.license_tokens SET used_at = NOW(), used_by = $1 WHERE id = $2",
        user_id,
        tok.id
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Erreur marquage jeton : {}", e))?;

    sqlx::query!(
        "UPDATE public.users SET access_expires_at = $1 WHERE id = $2",
        new_expiry,
        user_id
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Erreur mise à jour utilisateur : {}", e))?;

    let days = tok.duration_days;
    let label = if days >= 36500 {
        "∞ lifetime".to_string()
    } else {
        format!("{} j", days)
    };
    Ok(format!(
        "OK — {} — expire le {}",
        label,
        new_expiry.format("%Y-%m-%d")
    ))
}

fn print_result(email: &str, token: &str, result: Result<String, String>) {
    let display_token = if token.len() > 19 {
        &token[..19]
    } else {
        token
    };
    match result {
        Ok(msg) => println!("{:<35}  {:<25}  {}", email, display_token, msg),
        Err(err) => println!("{:<35}  {:<25}  ERREUR: {}", email, display_token, err),
    }
}

fn hash_token(token: &str) -> String {
    let normalized = token.replace('-', "").to_uppercase();
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}
