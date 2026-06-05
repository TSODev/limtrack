// src/components/vehicle_list.rs
use crate::components::add_vehicle_button::AddVehicleButton;
use crate::components::join_vehicle_button::JoinVehicleButton;
use crate::components::vehicle::VehicleCard;
use common::Vehicle;
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Vehicle_list(
    vehicles: ReadSignal<Vec<Vehicle>>,
    set_vehicles: WriteSignal<Vec<Vehicle>>,
    set_selected: WriteSignal<Option<uuid::Uuid>>,
    #[prop(optional)] hide_actions: bool,
) -> impl IntoView {
    view! {
        <div class="h-full flex flex-col bg-white rounded-xl border border-gray-100">
            // En-tête
            <div class="shrink-0 px-4 py-3 border-b border-gray-100">
                <h2 class="text-sm font-medium text-gray-700">"Véhicules"</h2>
            </div>

            // Liste scrollable
            <div class="flex-1 overflow-y-auto p-3 flex flex-col gap-2">
                <Show
                    when=move || !vehicles.get().is_empty()
                    fallback=|| view! {
                        <div class="flex flex-col items-center justify-center py-6 text-center">
                            <svg class="w-10 h-10 text-gray-200 mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                <path stroke-linecap="round" stroke-linejoin="round"
                                    d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                            </svg>
                            <p class="text-sm text-gray-400 italic">"Aucun véhicule"</p>
                            <p class="text-xs text-gray-300 mt-1">"Ajoutez ou rejoignez un véhicule ci-dessous"</p>
                        </div>
                    }
                >
                    <For
                        each=move || vehicles.get()
                        key=|v| v.id
                        children=move |v| view! {
                            <VehicleCard vehicle=v set_selected=set_selected />
                        }
                    />
                </Show>
            </div>

            // Boutons en bas (masqués si hide_actions=true)
            <Show when=move || !hide_actions fallback=|| ()>
                <div class="shrink-0 p-3 flex flex-row gap-2 border-t border-gray-100">
                    <AddVehicleButton set_vehicles=set_vehicles />
                    <JoinVehicleButton set_vehicles=set_vehicles />
                </div>
            </Show>
        </div>
    }
}

pub async fn fetch_vehicles(token: &str) -> Result<Vec<Vehicle>, String> {
    let url = format!("{}/api/vehicles", crate::config::API_BASE);

    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");

    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .map_err(|e| format!("{:?}", e))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("{:?}", e))?;
    headers
        .set("Cache-Control", "no-cache")
        .map_err(|e| format!("{:?}", e))?;
    opts.headers(&headers);

    let request =
        web_sys::Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;

    let window = leptos::window();
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
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
