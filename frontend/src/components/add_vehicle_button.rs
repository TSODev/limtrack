use common::Vehicle;
use leptos::*;
use wasm_bindgen::JsCast;

fn format_plate(raw: &str, country: &str) -> String {
    match country {
        "BE" => {
            // 0-AAA-000 : 1 chiffre + 3 lettres + 3 chiffres
            let cleaned: String = raw
                .chars()
                .filter(|c| c.is_alphanumeric())
                .map(|c| c.to_ascii_uppercase())
                .take(7)
                .collect();
            match cleaned.len() {
                0..=1 => cleaned,
                2..=4 => format!("{}-{}", &cleaned[..1], &cleaned[1..]),
                _ => format!("{}-{}-{}", &cleaned[..1], &cleaned[1..4], &cleaned[4..]),
            }
        }
        "LU" => {
            // AA 0000 : 2 lettres + 4 chiffres
            let cleaned: String = raw
                .chars()
                .filter(|c| c.is_alphanumeric())
                .map(|c| c.to_ascii_uppercase())
                .take(6)
                .collect();
            match cleaned.len() {
                0..=2 => cleaned,
                _ => format!("{} {}", &cleaned[..2], &cleaned[2..]),
            }
        }
        "CH" => {
            // AA 000000 : 2 lettres canton + 1-6 chiffres
            let cleaned: String = raw
                .chars()
                .filter(|c| c.is_alphanumeric())
                .map(|c| c.to_ascii_uppercase())
                .take(8)
                .collect();
            match cleaned.len() {
                0..=2 => cleaned,
                _ => format!("{} {}", &cleaned[..2], &cleaned[2..]),
            }
        }
        _ => {
            // FR (défaut) : AA-000-AA
            let cleaned: String = raw
                .chars()
                .filter(|c| c.is_alphanumeric())
                .map(|c| c.to_ascii_uppercase())
                .take(7)
                .collect();
            match cleaned.len() {
                0..=2 => cleaned,
                3..=5 => format!("{}-{}", &cleaned[..2], &cleaned[2..]),
                _ => format!("{}-{}-{}", &cleaned[..2], &cleaned[2..5], &cleaned[5..]),
            }
        }
    }
}

#[component]
pub fn AddVehicleButton(set_vehicles: WriteSignal<Vec<Vehicle>>) -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);
    let (make, set_make) = create_signal(String::new());
    let (model, set_model) = create_signal(String::new());
    let (plate_number, set_plate_number) = create_signal(String::new());
    let (country, set_country) = create_signal("FR".to_string());
    let (status, set_status) = create_signal(String::new());

    let plate_placeholder = create_memo(move |_| match country.get().as_str() {
        "BE" => "1-ABC-234",
        "LU" => "AB 1234",
        "CH" => "GE 123456",
        _ => "AB-123-CD",
    });

    let plate_hint = create_memo(move |_| match country.get().as_str() {
        "BE" => "Format : 0-AAA-000 (ex: 1-ABC-234)",
        "LU" => "Format : AA 0000 (ex: AB 1234)",
        "CH" => "Format : AA 000000 (ex: GE 123456)",
        _ => "Format : AA-000-AA (ex: AB-123-CD)",
    });

    let plate_maxlength = create_memo(move |_| match country.get().as_str() {
        "LU" => "7",
        _ => "9",
    });

    let plate_pattern = create_memo(move |_| match country.get().as_str() {
        "BE" => r"[0-9]-[A-Z]{3}-[0-9]{3}",
        "LU" => r"[A-Z]{2} [0-9]{4}",
        "CH" => r"[A-Z]{2} [0-9]{1,6}",
        _ => r"[A-Z]{2}-[0-9]{3}-[A-Z]{2}",
    });

    let create_action = create_action(move |(mk, mo, plate): &(String, String, String)| {
        let mk = mk.clone();
        let mo = mo.clone();
        let plate = plate.clone();

        async move {
            let token = if let Ok(Some(storage)) = leptos::window().local_storage() {
                storage.get_item("jwt_token").unwrap_or(None)
            } else {
                None
            };

            let Some(token) = token else {
                set_status.set("Non authentifié.".to_string());
                return;
            };

            let url = format!("{}/api/vehicles", crate::config::API_BASE);
            let mut opts = web_sys::RequestInit::new();
            opts.method("POST");

            let headers = web_sys::Headers::new().expect("headers");
            headers
                .set("Authorization", &format!("Bearer {}", token))
                .ok();
            headers.set("Content-Type", "application/json").ok();
            opts.headers(&headers);

            let body = serde_json::json!({
                "make": mk,
                "model": mo,
                "plate_number": plate,
            });
            opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));

            let request = web_sys::Request::new_with_str_and_init(&url, &opts).expect("requête");

            match wasm_bindgen_futures::JsFuture::from(
                leptos::window().fetch_with_request(&request),
            )
            .await
            {
                Ok(resp_value) => {
                    let resp: web_sys::Response = resp_value.dyn_into().expect("réponse");

                    if resp.ok() || resp.status() == 201 {
                        set_status.set(String::new());
                        set_show_modal.set(false);
                        set_make.set(String::new());
                        set_model.set(String::new());
                        set_plate_number.set(String::new());

                        if let Ok(Some(storage)) = leptos::window().local_storage() {
                            if let Ok(Some(t)) = storage.get_item("jwt_token") {
                                if let Ok(vehicles) =
                                    crate::components::vehicle_list::fetch_vehicles(&t).await
                                {
                                    set_vehicles.set(vehicles);
                                }
                            }
                        }
                    } else {
                        set_status.set(format!("Erreur : {}", resp.status()));
                    }
                }
                Err(_) => set_status.set("Serveur inaccessible.".to_string()),
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_status.set(String::new());
        create_action.dispatch((make.get(), model.get(), plate_number.get()));
    };

    let on_plate_input = move |ev: web_sys::Event| {
        let raw = event_target_value(&ev.clone().dyn_into::<web_sys::InputEvent>().unwrap());
        let formatted = format_plate(&raw, &country.get());
        set_plate_number.set(formatted.clone());

        let input = ev
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();
        input.set_value(&formatted);
    };

    view! {
        // Bouton déclencheur
        <button
            on:click=move |_| {
                set_status.set(String::new());
                set_show_modal.set(true);
            }
            class="w-full flex items-center justify-center gap-2 px-4 py-2 border border-dashed border-indigo-300 rounded-lg text-sm font-medium text-indigo-600 hover:bg-indigo-50 transition duration-150"
        >
            "+ Ajouter un véhicule"
        </button>

        // Overlay modal
        <Show when=move || show_modal.get() fallback=|| ()>
            // Fond semi-transparent
            <div
                class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm"
                on:click=move |_| set_show_modal.set(false)
            />

            // Fenêtre modale
            <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
                <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md p-8 space-y-6">

                    // En-tête modal
                    <div class="flex items-center justify-between">
                        <div>
                            <h2 class="text-xl font-bold text-gray-900">"Nouveau véhicule"</h2>
                            <p class="text-sm text-gray-500 mt-1">"Renseignez les informations du véhicule"</p>
                        </div>
                        <button
                            on:click=move |_| set_show_modal.set(false)
                            class="text-gray-400 hover:text-gray-600 transition duration-150 text-xl font-light"
                        >
                            "✕"
                        </button>
                    </div>

                    // Formulaire
                    <form on:submit=on_submit class="space-y-4">

                        <div class="space-y-1">
                            <label class="text-sm font-medium text-gray-700 block">"Marque"</label>
                            <input
                                type="text"
                                placeholder="ex: Renault"
                                prop:value=make
                                on:input=move |ev| set_make.set(event_target_value(&ev))
                                required
                                class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            />
                        </div>

                        <div class="space-y-1">
                            <label class="text-sm font-medium text-gray-700 block">"Modèle"</label>
                            <input
                                type="text"
                                placeholder="ex: Mégane IV 1.5 dCi"
                                prop:value=model
                                on:input=move |ev| set_model.set(event_target_value(&ev))
                                required
                                class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            />
                        </div>

                        <div class="space-y-1">
                            <label class="text-sm font-medium text-gray-700 block">"Immatriculation"</label>
                            <div class="flex gap-2">
                                <select
                                    on:change=move |ev| {
                                        set_country.set(event_target_value(&ev));
                                        set_plate_number.set(String::new());
                                    }
                                    class="px-2 py-2 border border-gray-300 rounded-md shadow-sm text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 bg-white"
                                >
                                    <option value="FR">"🇫🇷 FR"</option>
                                    <option value="BE">"🇧🇪 BE"</option>
                                    <option value="LU">"🇱🇺 LU"</option>
                                    <option value="CH">"🇨🇭 CH"</option>
                                </select>
                                <input
                                    type="text"
                                    attr:placeholder=move || plate_placeholder.get()
                                    prop:value=plate_number
                                    on:input=on_plate_input
                                    required
                                    attr:maxlength=move || plate_maxlength.get()
                                    attr:pattern=move || plate_pattern.get()
                                    attr:title=move || plate_hint.get()
                                    class="appearance-none block flex-1 px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm font-mono tracking-widest transition duration-150"
                                />
                            </div>
                            <p class="text-xs text-gray-400">{move || plate_hint.get()}</p>
                        </div>

                        // Message d'erreur
                        <Show when=move || !status.get().is_empty() fallback=|| ()>
                            <p class="text-sm text-center text-red-600">
                                {move || status.get()}
                            </p>
                        </Show>

                        // Actions
                        <div class="flex gap-3 pt-2">
                            <button
                                type="button"
                                on:click=move |_| set_show_modal.set(false)
                                class="flex-1 py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 transition duration-150"
                            >
                                "Annuler"
                            </button>
                            <button
                                type="submit"
                                prop:disabled=create_action.pending()
                                class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                            >
                                {move || if create_action.pending().get() { "Création..." } else { "Créer" }}
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </Show>
    }
}
