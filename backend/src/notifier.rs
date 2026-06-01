// Notifications email d'expiration de licence — logique partagée
// Appelé depuis main.rs (tâche de fond tokio) et le binaire notify-expiry.

use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use sqlx::PgPool;
use tracing::{info, warn};

macro_rules! log {
    ($($arg:tt)*) => {
        let msg = format!($($arg)*);
        info!("{}", msg);
        println!("{}", msg);
    };
}

pub async fn run_notifications(pool: &PgPool, api_key: &str) {
    let http = Client::new();
    let now = Utc::now();

    let users = sqlx::query!(
        r#"
        SELECT
            u.id,
            u.email,
            u.username,
            u.trial_ends_at,
            u.access_expires_at,
            u.expiry_notif_sent_at,
            lt.duration_days,
            lt.license_type
        FROM public.users u
        LEFT JOIN LATERAL (
            SELECT duration_days, license_type
            FROM public.license_tokens
            WHERE used_by = u.id AND used_at IS NOT NULL
            ORDER BY used_at DESC
            LIMIT 1
        ) lt ON true
        WHERE
            u.access_expires_at > NOW()
            OR u.trial_ends_at > NOW()
        "#
    )
    .fetch_all(pool)
    .await;

    let users = match users {
        Ok(u) => u,
        Err(e) => { warn!("notify: erreur lecture utilisateurs : {}", e); println!("❌ Erreur DB : {}", e); return; }
    };

    log!("notify: vérification de {} utilisateur(s)", users.len());
    let mut sent = 0u32;

    for user in users {
        let is_active = user.access_expires_at.map_or(false, |e| e > now);
        let expiry = if is_active { user.access_expires_at.unwrap() } else { user.trial_ends_at };
        let days_remaining = (expiry - now).num_days();

        println!("  → {} <{}> : {} jours restants (actif={})", user.username, user.email, days_remaining, is_active);

        if days_remaining > 3650 { println!("    ignoré : lifetime"); continue; }

        let threshold: i64 = if is_active {
            match user.duration_days {
                Some(d) if d <= 30 => 7,
                Some(d) if d <= 95 => 15,
                _                  => 30,
            }
        } else {
            15
        };

        println!("    seuil={} jours", threshold);
        if days_remaining > threshold { println!("    ignoré : hors seuil"); continue; }

        // Anti-doublon 24h
        if let Some(sent_at) = user.expiry_notif_sent_at {
            if (now - sent_at).num_hours() < 24 {
                println!("    ignoré : déjà notifié il y a moins de 24h");
                continue;
            }
        }

        let is_trial = !is_active;
        let license_type = user.license_type.as_deref().unwrap_or("personal");
        let html = build_email_html(&user.username, days_remaining, is_trial, license_type);

        let subject = if days_remaining <= 1 {
            "⚠️ Votre licence odo.io expire aujourd'hui".to_string()
        } else {
            format!("Votre licence odo.io expire dans {} jours", days_remaining)
        };

        let res = http
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "from": "odo.io <noreply@tsodev.fr>",
                "to": [&user.email],
                "subject": subject,
                "html": html,
            }))
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => {
                sqlx::query!(
                    "UPDATE public.users SET expiry_notif_sent_at = NOW() WHERE id = $1",
                    user.id
                )
                .execute(pool)
                .await
                .ok();
                log!("notify: ✓ {} <{}> J-{}", user.username, user.email, days_remaining);
                sent += 1;
            }
            Ok(r) => {
                let status = r.status();
                let body = r.text().await.unwrap_or_default();
                warn!("notify: ✗ {} — HTTP {}", user.email, status);
                println!("    ❌ HTTP {} : {}", status, body);
            }
            Err(e) => { warn!("notify: ✗ {} — {}", user.email, e); println!("    ❌ Réseau : {}", e); }
        }
    }

    log!("notify: {} email(s) envoyé(s)", sent);
}

fn build_email_html(username: &str, days: i64, is_trial: bool, license_type: &str) -> String {
    let license_label = if is_trial {
        "période d'essai"
    } else if license_type == "fleet" {
        "licence Flotte"
    } else {
        "licence Personnelle"
    };

    let urgency_color = if days <= 3 { "#dc2626" } else { "#d97706" };

    let days_text = if days <= 1 {
        "expire <strong>aujourd'hui</strong>".to_string()
    } else {
        format!("expire dans <strong>{} jours</strong>", days)
    };

    format!(r#"<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width,initial-scale=1.0"/>
  <title>Expiration de licence — odo.io</title>
</head>
<body style="margin:0;padding:0;background-color:#f8fafc;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#f8fafc;padding:40px 16px;">
    <tr><td align="center">
      <table width="560" cellpadding="0" cellspacing="0" style="max-width:560px;width:100%;">
        <tr>
          <td style="background:linear-gradient(135deg,#4f46e5 0%,#7c3aed 100%);border-radius:12px 12px 0 0;padding:32px 40px;text-align:center;">
            <p style="margin:0;font-size:28px;font-weight:800;color:#ffffff;letter-spacing:-0.5px;">odo.io</p>
            <p style="margin:8px 0 0;font-size:13px;color:#c4b5fd;">Gestion de flotte kilométrique</p>
          </td>
        </tr>
        <tr>
          <td style="background:#ffffff;padding:40px;border-left:1px solid #e2e8f0;border-right:1px solid #e2e8f0;">
            <p style="margin:0 0 8px;font-size:18px;font-weight:700;color:#1e293b;">Bonjour {username},</p>
            <p style="margin:0 0 24px;font-size:15px;color:#64748b;line-height:1.6;">
              Votre <strong>{license_label}</strong> {days_text}.
            </p>
            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;">
              <tr>
                <td style="background:#fefce8;border:1px solid {urgency_color};border-left:4px solid {urgency_color};border-radius:8px;padding:16px 20px;">
                  <p style="margin:0;font-size:14px;color:#1e293b;line-height:1.5;">
                    Sans renouvellement, vous perdrez l'accès aux fonctionnalités d'écriture
                    (ajout de relevés, modification de contrats). Vos données restent conservées.
                  </p>
                </td>
              </tr>
            </table>
            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;">
              <tr>
                <td align="center">
                  <a href="https://odo.tsodev.fr/profile"
                     style="display:inline-block;background:linear-gradient(135deg,#4f46e5,#7c3aed);color:#ffffff;font-size:15px;font-weight:600;text-decoration:none;padding:14px 32px;border-radius:8px;">
                    Renouveler ma licence →
                  </a>
                </td>
              </tr>
            </table>
            <p style="margin:0;font-size:13px;color:#94a3b8;line-height:1.6;">
              Rendez-vous dans <strong>Profil → Licence</strong> pour activer un nouveau jeton.<br/>
              Si vous avez des questions, répondez directement à cet email.
            </p>
          </td>
        </tr>
        <tr>
          <td style="background:#f1f5f9;border:1px solid #e2e8f0;border-top:none;border-radius:0 0 12px 12px;padding:20px 40px;text-align:center;">
            <p style="margin:0;font-size:12px;color:#94a3b8;">
              odo.io · <a href="https://odo.tsodev.fr" style="color:#6366f1;text-decoration:none;">odo.tsodev.fr</a><br/>
              Vous recevez cet email car votre licence approche de son échéance.
            </p>
          </td>
        </tr>
      </table>
    </td></tr>
  </table>
</body>
</html>"#,
        username = username,
        license_label = license_label,
        days_text = days_text,
        urgency_color = urgency_color,
    )
}
