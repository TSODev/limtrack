use common::VehicleWithAccess;
use leptos::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[component]
pub fn VehicleDetail(selected_id: ReadSignal<Option<Uuid>>) -> impl IntoView {
    let (detail, set_detail) = create_signal(Option::<VehicleWithAccess>::None);
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(String::new());

    // Se déclenche à chaque changement de sélection
    create_effect(move |_| {
        let Some(id) = selected_id.get() else {
            set_detail.set(None);
            return;
        };

        set_loading.set(true);
        set_error.set(String::new());

        spawn_local(async move {
            let token = if let Ok(Some(storage)) = leptos::window().local_storage() {
                storage.get_item("jwt_token").unwrap_or(None)
            } else {
                None
            };

            let Some(token) = token else {
                set_error.set("Non authentifié.".to_string());
                set_loading.set(false);
                return;
            };

            match fetch_vehicle_detail(&token, id).await {
                Ok(data) => set_detail.set(Some(data)),
                Err(e) => set_error.set(e),
            }
            set_loading.set(false);
        });
    });

    view! {
            <div class="bg-white p-8 rounded-xl shadow-md border border-gray-100">
                <Show
                    when=move || selected_id.get().is_some()
                    fallback=move || view! {
                        <div class="text-center space-y-4">
                            <h1 class="text-4xl font-extrabold text-gray-900 tracking-tight">
                                "Bienvenue dans votre espace !"
                            </h1>
                            <p class="text-lg text-gray-600">
                                "Sélectionnez un véhicule pour voir ses détails."
                            </p>
                        </div>
                    }
                >
                    <Show when=move || loading.get() fallback=|| ()>
                        <p class="text-gray-400 animate-pulse text-sm">"Chargement..."</p>
                    </Show>

                    <Show when=move || !error.get().is_empty() fallback=|| ()>
                        <p class="text-red-500 text-sm">{move || error.get()}</p>
                    </Show>

                    <Show when=move || detail.get().is_some() && !loading.get() fallback=|| ()>
                        {move || detail.get().map(|v| view! {
                            <div class="space-y-6">
                                <div>
                                    <h2 class="text-2xl font-bold text-gray-900">
                                        {format!("{} {}", v.make, v.model)}
                                    </h2>
                                    <p class="text-indigo-600 font-semibold tracking-widest mt-1">
                                        {v.plate_number.clone()}
                                    </p>
                                </div>
    //                            <div class="grid grid-cols-2 gap-4">
    //                                <div class="bg-gray-50 rounded-lg p-4">
    //                                    <p class="text-xs text-gray-400 uppercase tracking-wide">"Rôle"</p>
    //                                    <p class="text-sm font-semibold text-gray-800 mt-1">
    //                                        {format!("{:?}", v.my_role)}
    //                                    </p>
    //                                </div>
    //                                <div class="bg-gray-50 rounded-lg p-4">
    //                                    <p class="text-xs text-gray-400 uppercase tracking-wide">"Propriétaire"</p>
    //                                    <p class="text-sm font-semibold text-gray-800 mt-1">
    //                                        {v.owner_id.to_string()}
    //                                    </p>
    //                                </div>
    //                            </div>
                            </div>
                        })}
                    </Show>
                </Show>
            </div>
        }
}

async fn fetch_vehicle_detail(token: &str, id: Uuid) -> Result<VehicleWithAccess, String> {
    let url = format!("/api/vehicles/{}", id);

    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");

    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
    headers.set("Cache-Control", "no-cache").ok();
    opts.headers(&headers);

    let request =
        web_sys::Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;

    let resp_value =
        wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&request))
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
