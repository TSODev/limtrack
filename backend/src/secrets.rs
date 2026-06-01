// Chargement des secrets via Infisical Service Token.
// Un seul appel GET suffit — le service token gère E2EE côté serveur.
//
// Variables Railway en production :
//   INFISICAL_TOKEN       — Service Token Infisical (commence par "st.")
//   INFISICAL_PROJECT_ID  — ID du projet Infisical
//   INFISICAL_ENVIRONMENT — slug d'environnement (défaut : "prod")
//   INFISICAL_URL         — URL de base pour self-hosted (défaut : https://eu.infisical.com)
//
// En local : pas de INFISICAL_TOKEN → fallback .env automatique.
// Les noms des secrets dans Infisical = noms des variables d'env.

use reqwest::Client;
use serde::Deserialize;

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
    let Ok(token) = std::env::var("INFISICAL_TOKEN") else {
        dotenvy::dotenv().ok();
        return;
    };

    let project_id = std::env::var("INFISICAL_PROJECT_ID")
        .expect("INFISICAL_PROJECT_ID manquante");
    let environment = std::env::var("INFISICAL_ENVIRONMENT")
        .unwrap_or_else(|_| "prod".to_string());
    let base_url = std::env::var("INFISICAL_URL")
        .unwrap_or_else(|_| "https://eu.infisical.com".to_string());

    println!("Chargement des secrets depuis Infisical ({})...", environment);

    let response = Client::new()
        .get(format!(
            "{}/api/v3/secrets/raw?workspaceId={}&environment={}&secretPath=/",
            base_url, project_id, environment
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Impossible de contacter Infisical");

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        panic!("Infisical erreur {}: {}", status, body);
    }

    let data: SecretsResponse = response.json().await.expect("Réponse Infisical invalide");

    for secret in &data.secrets {
        std::env::set_var(&secret.key, &secret.value);
    }

    println!("  ✓ {} secrets chargés", data.secrets.len());
}
