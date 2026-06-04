// src/request_license_handler.rs — POST /api/license/request (public, sans auth)

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct LicenseRequestPayload {
    pub email: String,
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

fn build_email_html(token: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width,initial-scale=1.0"/>
  <title>Votre licence LimTrack</title>
</head>
<body style="margin:0;padding:0;background-color:#f8fafc;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#f8fafc;padding:40px 16px;">
    <tr><td align="center">
      <table width="560" cellpadding="0" cellspacing="0" style="max-width:560px;width:100%;">
        <tr>
          <td style="background:linear-gradient(135deg,#4f46e5 0%,#7c3aed 100%);border-radius:12px 12px 0 0;padding:32px 40px;text-align:center;">
            <p style="margin:0;font-size:28px;font-weight:800;color:#ffffff;letter-spacing:-0.5px;">LimTrack</p>
            <p style="margin:8px 0 0;font-size:13px;color:#c4b5fd;">Gestion de flotte kilométrique</p>
          </td>
        </tr>
        <tr>
          <td style="background:#ffffff;padding:40px;border-left:1px solid #e2e8f0;border-right:1px solid #e2e8f0;">
            <p style="margin:0 0 16px;font-size:18px;font-weight:700;color:#1e293b;">Bienvenue sur LimTrack !</p>
            <p style="margin:0 0 24px;font-size:15px;color:#64748b;line-height:1.6;">
              Voici votre jeton de licence gratuite valable <strong>365 jours</strong>.
            </p>
            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;">
              <tr>
                <td style="background:#f0fdf4;border:1px solid #16a34a;border-left:4px solid #16a34a;border-radius:8px;padding:20px;text-align:center;">
                  <p style="margin:0 0 8px;font-size:13px;color:#64748b;">Votre jeton de licence</p>
                  <p style="margin:0;font-size:22px;font-weight:800;color:#1e293b;letter-spacing:2px;font-family:monospace;">{token}</p>
                </td>
              </tr>
            </table>
            <p style="margin:0 0 12px;font-size:14px;color:#475569;line-height:1.6;">Pour activer votre licence :</p>
            <ol style="margin:0 0 28px;padding-left:24px;font-size:14px;color:#475569;line-height:2.2;">
              <li>Connectez-vous sur <a href="https://limtrack.app" style="color:#6366f1;">limtrack.app</a></li>
              <li>Allez dans <strong>Profil → Licence</strong></li>
              <li>Saisissez le jeton ci-dessus</li>
            </ol>
            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;">
              <tr>
                <td align="center">
                  <a href="https://limtrack.app/profile"
                     style="display:inline-block;background:linear-gradient(135deg,#4f46e5,#7c3aed);color:#ffffff;font-size:15px;font-weight:600;text-decoration:none;padding:14px 32px;border-radius:8px;">
                    Activer ma licence →
                  </a>
                </td>
              </tr>
            </table>
            <p style="margin:0;font-size:13px;color:#94a3b8;line-height:1.6;">
              LimTrack est un projet open source gratuit. Si vous souhaitez soutenir son développement,
              vous pouvez faire un don sur <a href="https://ko-fi.com/limtrack" style="color:#6366f1;">Ko-fi</a>
              ou <a href="https://github.com/sponsors/TSODev" style="color:#6366f1;">GitHub Sponsors</a>.
            </p>
          </td>
        </tr>
        <tr>
          <td style="background:#f1f5f9;border:1px solid #e2e8f0;border-top:none;border-radius:0 0 12px 12px;padding:20px 40px;text-align:center;">
            <p style="margin:0;font-size:12px;color:#94a3b8;">
              LimTrack · <a href="https://limtrack.app" style="color:#6366f1;text-decoration:none;">limtrack.app</a> ·
              <a href="https://github.com/TSODev/limtrack" style="color:#6366f1;text-decoration:none;">GitHub</a>
            </p>
          </td>
        </tr>
      </table>
    </td></tr>
  </table>
</body>
</html>"#,
        token = token
    )
}

pub async fn request_license(
    State(state): State<AppState>,
    Json(payload): Json<LicenseRequestPayload>,
) -> impl IntoResponse {
    let email = payload.email.trim().to_lowercase();

    if email.is_empty() || !email.contains('@') {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Adresse email invalide"})),
        )
            .into_response();
    }

    // 1 jeton max par adresse email
    let existing = sqlx::query!(
        "SELECT id FROM public.license_requests WHERE email = $1",
        email
    )
    .fetch_optional(&state.db)
    .await;

    match existing {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({"error": "Un jeton a déjà été envoyé à cette adresse email"})),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Erreur base de données"})),
            )
                .into_response();
        }
        Ok(None) => {}
    }

    let token = generate_token();
    let hash = hash_token(&token);

    if let Err(_) = sqlx::query!(
        "INSERT INTO public.license_tokens (token_hash, duration_days, license_type) VALUES ($1, $2, $3)",
        hash,
        365i32,
        "personal"
    )
    .execute(&state.db)
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Erreur lors de la création du jeton"})),
        )
            .into_response();
    }

    // Envoyer le token par email
    let api_key = &state.resend_api_key;
    info!("license/request: envoi email à {} (api_key présente: {})", email, !api_key.is_empty());
    if !api_key.is_empty() {
        let html = build_email_html(&token);
        let http = Client::new();
        match http
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "from": "LimTrack <noreply@limtrack.app>",
                "to": [&email],
                "subject": "Votre licence LimTrack gratuite",
                "html": html,
            }))
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => info!("license/request: email envoyé à {}", email),
            Ok(r) => {
                let status = r.status();
                let body = r.text().await.unwrap_or_default();
                warn!("license/request: Resend erreur {} pour {} — {}", status, email, body);
            }
            Err(e) => warn!("license/request: Resend réseau erreur pour {} — {}", email, e),
        }
    } else {
        warn!("license/request: RESEND_API_KEY absente — email non envoyé à {}", email);
    }

    // Enregistrer la demande (anti-doublon)
    let _ = sqlx::query!(
        "INSERT INTO public.license_requests (email, token_hash) VALUES ($1, $2)",
        email,
        hash
    )
    .execute(&state.db)
    .await;

    (
        StatusCode::OK,
        Json(json!({"message": "Jeton envoyé à votre adresse email"})),
    )
        .into_response()
}
