// Chargement des secrets via Infisical Universal Auth.
// Étape 1 : échange client_id + client_secret → access token
// Étape 2 : récupération de tous les secrets du projet → injection dans l'env
//
// Variables Railway en production :
//   INFISICAL_CLIENT_ID     — Client ID du Machine Identity
//   INFISICAL_CLIENT_SECRET — Client Secret du Machine Identity
//   INFISICAL_PROJECT_ID    — ID du projet Infisical
//   INFISICAL_ENVIRONMENT   — slug d'environnement (défaut : "prod")
//   INFISICAL_URL           — URL de base pour self-hosted (défaut : https://app.infisical.com)
//
// En local : pas de variables INFISICAL_* → fallback .env automatique.
// Les noms des secrets dans Infisical = noms des variables d'env.

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AuthRequest<'a> {
    #[serde(rename = "clientId")]
    client_id: &'a str,
    #[serde(rename = "clientSecret")]
    client_secret: &'a str,
}

#[derive(Deserialize)]
struct AuthResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
}

#[derive(Deserialize)]
struct RawSecret {
    #[serde(rename = "secretKey")]
    key: String,
    #[serde(rename = "secretValue")]
    value: String,
}

#[derive(Deserialize)]
struct SecretsResponse {
    secrets: Vec<RawSecret>,
}

pub async fn load_secrets() {
    let Ok(client_id) = std::env::var("INFISICAL_CLIENT_ID") else {
        dotenvy::dotenv().ok();
        return;
    };

    let client_secret = std::env::var("INFISICAL_CLIENT_SECRET")
        .expect("INFISICAL_CLIENT_SECRET manquante");
    let project_id = std::env::var("INFISICAL_PROJECT_ID")
        .expect("INFISICAL_PROJECT_ID manquante");
    let environment = std::env::var("INFISICAL_ENVIRONMENT")
        .unwrap_or_else(|_| "prod".to_string());
    let base_url = std::env::var("INFISICAL_URL")
        .unwrap_or_else(|_| "https://app.infisical.com".to_string());

    println!("Chargement des secrets depuis Infisical ({})...", environment);

    let http = Client::new();

    // Étape 1 : obtenir l'access token
    let auth_resp = http
        .post(format!("{}/api/v1/auth/universal-auth/login", base_url))
        .json(&AuthRequest { client_id: &client_id, client_secret: &client_secret })
        .send()
        .await
        .expect("Impossible de contacter Infisical (auth)");

    if !auth_resp.status().is_success() {
        let status = auth_resp.status();
        let body = auth_resp.text().await.unwrap_or_default();
        panic!("Infisical auth erreur {}: {}", status, body);
    }

    let auth: AuthResponse = auth_resp.json().await.expect("Réponse auth Infisical invalide");

    // Étape 2 : récupérer les secrets
    let secrets_resp = http
        .get(format!(
            "{}/api/v4/secrets?projectId={}&environment={}&secretPath=/",
            base_url, project_id, environment
        ))
        .bearer_auth(&auth.access_token)
        .send()
        .await
        .expect("Impossible de contacter Infisical (secrets)");

    if !secrets_resp.status().is_success() {
        let status = secrets_resp.status();
        let body = secrets_resp.text().await.unwrap_or_default();
        panic!("Infisical secrets erreur {}: {}", status, body);
    }

    let data: SecretsResponse = secrets_resp.json().await.expect("Réponse secrets Infisical invalide");

    for secret in &data.secrets {
        std::env::set_var(&secret.key, &secret.value);
    }

    println!("  ✓ {} secrets chargés", data.secrets.len());
}
