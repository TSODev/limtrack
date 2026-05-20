//use crate::models::Vehicle;
use common::Vehicle;
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn AddVehicleButton(set_vehicles: WriteSignal<Vec<Vehicle>>) -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);
    let (make, set_make) = create_signal(String::new());
    let (model, set_model) = create_signal(String::new());
    let (plate_number, set_plate_number) = create_signal(String::new());
    let (status, set_status) = create_signal(String::new());

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

            let url = "/api/vehicles";
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

            let request = web_sys::Request::new_with_str_and_init(url, &opts).expect("requête");

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

        // Ne garde que lettres et chiffres, met en majuscules
        let cleaned: String = raw
            .chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| c.to_ascii_uppercase())
            .take(7) // AA + 3 chiffres + AA = 7 caractères utiles
            .collect();

        // Formate en AA-111-AA
        let formatted = match cleaned.len() {
            0..=2 => cleaned.clone(),
            3..=5 => format!("{}-{}", &cleaned[..2], &cleaned[2..]),
            _ => format!("{}-{}-{}", &cleaned[..2], &cleaned[2..5], &cleaned[5..]),
        };

        set_plate_number.set(formatted.clone());

        // Met à jour la valeur affichée dans l'input
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
                            <input
                                type="text"
                                placeholder="ex: AB-123-CD"
                                prop:value=plate_number
                                on:input=on_plate_input
                                required
                                maxlength="10" // AA-111-AA + marge
                                pattern="[A-Z]{2}-[0-9]{3}-[A-Z]{2}"
                                title="Format attendu : AA-111-AA (ex: AB-123-CD)"
                                class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            />
                            <p class="text-xs text-gray-400">"Format : AA-111-AA (ex: AB-123-CD)"</p>
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
