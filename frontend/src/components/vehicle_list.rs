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
) -> impl IntoView {
    view! {
        <div class="h-full flex flex-col bg-white rounded-xl border border-gray-100">
            // En-tête
            <div class="shrink-0 px-4 py-3 border-b border-gray-100">
                <h2 class="text-sm font-medium text-gray-700">"Véhicules"</h2>
            </div>

            // Liste scrollable
            <div class="flex-1 overflow-y-auto p-3 flex flex-col gap-2">
                <For
                    each=move || vehicles.get()
                    key=|v| v.id
                    children=move |v| view! {
                        <VehicleCard vehicle=v set_selected=set_selected />
                    }
                />
            </div>

            // Boutons en bas
            <div class="shrink-0 p-3 flex flex-col gap-2 border-t border-gray-100">
                <AddVehicleButton set_vehicles=set_vehicles />
                <JoinVehicleButton set_vehicles=set_vehicles />
            </div>
        </div>
    }
}

pub async fn fetch_vehicles(token: &str) -> Result<Vec<Vehicle>, String> {
    let url = "/api/vehicles";

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
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;

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
