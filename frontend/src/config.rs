pub const API_BASE: &str = "https://api.limtrack.app";
pub const CONTACT_EMAIL: &str = "thierry.soulie@tsodev.fr";

/// Retourne true si l'app tourne dans Tauri (iOS/desktop).
/// Tauri injecte window.__TAURI__ automatiquement au démarrage.
pub fn is_tauri() -> bool {
    js_sys::Reflect::has(
        &leptos::window(),
        &wasm_bindgen::JsValue::from_str("__TAURI__"),
    )
    .unwrap_or(false)
}
