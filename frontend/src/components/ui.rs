pub fn input_class() -> &'static str {
    "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm \
     placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 \
     sm:text-sm transition duration-150"
}

pub fn get_token() -> Option<String> {
    leptos::window()
        .local_storage()
        .ok()?
        .and_then(|s| s.get_item("jwt_token").ok()?)
}

pub async fn parse_error_response(resp: web_sys::Response) -> String {
    let status = resp.status();
    if let Ok(promise) = resp.text() {
        if let Ok(val) = wasm_bindgen_futures::JsFuture::from(promise).await {
            if let Some(text) = val.as_string() {
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(msg) = obj.get("error").and_then(|v| v.as_str()) {
                        return msg.to_string();
                    }
                }
            }
        }
    }
    match status {
        409 => "Un contrat existe déjà sur cette période.".to_string(),
        402 => "Accès en lecture seule — licence expirée.".to_string(),
        403 => "Action non autorisée.".to_string(),
        404 => "Ressource introuvable.".to_string(),
        429 => "Trop de requêtes, réessayez dans quelques secondes.".to_string(),
        _ => format!("Erreur inattendue (HTTP {}).", status),
    }
}

pub fn format_date_fr(d: chrono::NaiveDate) -> String {
    let months = [
        "jan.", "fév.", "mars", "avr.", "mai", "juin",
        "juil.", "août", "sept.", "oct.", "nov.", "déc.",
    ];
    format!("{} {} {}", d.day(), months[d.month0() as usize], d.year())
}

pub fn format_km(km: i32) -> String {
    let s = km.to_string();
    let chars: Vec<char> = s.chars().collect();
    let formatted = chars
        .rchunks(3)
        .rev()
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\u{202F}");
    format!("{} km", formatted)
}
