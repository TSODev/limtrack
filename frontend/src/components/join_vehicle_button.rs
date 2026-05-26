// src/components/join_vehicle_button.rs
// Bouton "Rejoindre un véhicule" — affiché sous AddVehicleButton dans vehicle_list

use common::Vehicle;
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn JoinVehicleButton(set_vehicles: WriteSignal<Vec<Vehicle>>) -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);

    view! {
        <button
            on:click=move |_| set_show_modal.set(true)
            class="w-full flex items-center justify-center gap-2 px-4 py-2 border border-dashed border-gray-300 rounded-lg text-sm font-medium text-gray-500 hover:bg-gray-50 hover:border-gray-400 transition duration-150"
        >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                <path stroke-linecap="round" stroke-linejoin="round"
                    d="M13.19 8.688a4.5 4.5 0 0 1 1.242 7.244l-4.5 4.5a4.5 4.5 0 0 1-6.364-6.364l1.757-1.757m13.35-.622 1.757-1.757a4.5 4.5 0 0 0-6.364-6.364l-4.5 4.5a4.5 4.5 0 0 0 1.242 7.244" />
            </svg>
            "Rejoindre un véhicule"
        </button>

        <Show when=move || show_modal.get() fallback=|| ()>
            <JoinModal
                set_vehicles=set_vehicles
                on_close=Callback::new(move |_| set_show_modal.set(false))
            />
        </Show>
    }
}

// ─── Modal rejoindre ─────────────────────────────────────────────

#[component]
fn JoinModal(set_vehicles: WriteSignal<Vec<Vehicle>>, on_close: Callback<()>) -> impl IntoView {
    let (code, set_code) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());
    let (success, set_success) = create_signal(String::new());

    let submit = create_action(move |code: &String| {
        let code = code.clone();
        async move {
            set_error.set(String::new());
            set_success.set(String::new());

            // Validation basique du format XXX-XXX-XXX
            let trimmed = code.trim().to_uppercase();
            let parts: Vec<&str> = trimmed.split('-').collect();
            if parts.len() != 3 || parts.iter().any(|p| p.len() != 3) {
                set_error.set("Code invalide — format attendu : XXX-XXX-XXX".to_string());
                return;
            }

            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({ "code": trimmed });

            match post_json(
                &format!("{}/api/vehicles/join", crate::config::API_BASE),
                &token,
                &body,
            )
            .await
            {
                Ok(_) => {
                    set_success
                        .set("Accès accordé ! Le véhicule apparaît dans votre liste.".to_string());
                    if let Ok(vehicles) = fetch_vehicles(&token).await {
                        set_vehicles.set(vehicles);
                    }
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        submit.dispatch(code.get());
    };

    view! {
        <div
            class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm"
            on:click=move |_| on_close.call(())
        />
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
            <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md p-8 space-y-6">

                // En-tête
                <div class="flex items-center justify-between">
                    <div>
                        <h2 class="text-xl font-bold text-gray-900">"Rejoindre un véhicule"</h2>
                        <p class="text-sm text-gray-500 mt-1">
                            "Entrez le code partagé par le propriétaire"
                        </p>
                    </div>
                    <button
                        on:click=move |_| on_close.call(())
                        class="text-gray-400 hover:text-gray-600 text-xl font-light"
                    >"✕"</button>
                </div>

                // Formulaire
                <form on:submit=on_submit class="space-y-4">
                    <div class="space-y-1">
                        <label class="text-sm font-medium text-gray-700 block">
                            "Code de partage"
                        </label>
                        <input
                            type="text"
                            required
                            prop:value=code
                            on:input=move |ev| set_code.set(event_target_value(&ev))
                            placeholder="ex: XK7-M2P-9QR"
                            maxlength="11"
                            class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm font-mono tracking-widest text-center transition duration-150"
                        />
                        <p class="text-xs text-gray-400 text-center">
                            "Format : XXX-XXX-XXX — communiqué par le propriétaire"
                        </p>
                    </div>

                    <Show when=move || !error.get().is_empty() fallback=|| ()>
                        <p class="text-sm text-center text-red-600">{move || error.get()}</p>
                    </Show>
                    <Show when=move || !success.get().is_empty() fallback=|| ()>
                        <p class="text-sm text-center text-green-600 font-medium">
                            {move || success.get()}
                        </p>
                    </Show>

                    <div class="flex gap-3 pt-2">
                        <button
                            type="button"
                            on:click=move |_| on_close.call(())
                            class="flex-1 py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150"
                        >
                            "Annuler"
                        </button>
                        <button
                            type="submit"
                            prop:disabled=move || submit.pending().get()
                            class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                        >
                            {move || if submit.pending().get() { "Vérification..." } else { "Rejoindre" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

// ─── Helpers ─────────────────────────────────────────────────────

fn get_token() -> Option<String> {
    leptos::window()
        .local_storage()
        .ok()?
        .and_then(|s| s.get_item("jwt_token").ok()?)
}

async fn fetch_vehicles(token: &str) -> Result<Vec<Vehicle>, String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
    headers.set("Cache-Control", "no-cache").ok();
    opts.headers(&headers);
    let req = web_sys::Request::new_with_str_and_init(
        &format!("{}/api/vehicles", crate::config::API_BASE),
        &opts,
    )
    .map_err(|e| format!("{:?}", e))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
            .await
            .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;
    if !resp.ok() {
        return Err(format!("Erreur HTTP : {}", resp.status()));
    }
    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
        .await
        .map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(json).map_err(|e| format!("{:?}", e))
}

async fn post_json(url: &str, token: &str, body: &serde_json::Value) -> Result<(), String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
    headers.set("Content-Type", "application/json").ok();
    opts.headers(&headers);
    opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));
    let req =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
            .await
            .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;
    if resp.ok() || resp.status() == 201 {
        Ok(())
    } else {
        let json =
            wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
                .await
                .ok();
        let msg = json
            .and_then(|j| serde_wasm_bindgen::from_value::<serde_json::Value>(j).ok())
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| e.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| format!("Erreur HTTP : {}", resp.status()));
        Err(msg)
    }
}
