use common::{AccessRole, VehicleWithAccess};
use leptos::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;

use crate::components::contracts::contract_list::ContractList;
use crate::components::contracts::contract_widget::ContractsWidget;
use crate::components::mileage::mileage_list::MileageList;
use crate::components::mileage::mileage_widget::MileageWidget;
use crate::components::vehicle_header::VehicleHeader;

#[derive(Clone, PartialEq)]
pub enum DashboardTab {
    Overview,
    Kilometrage,
    Contracts,
}

#[component]
pub fn VehicleDashboard(
    selected_id: ReadSignal<Option<Uuid>>,
    set_selected_id: WriteSignal<Option<Uuid>>,
    set_vehicles: WriteSignal<Vec<common::Vehicle>>,
) -> impl IntoView {
    let (tab, set_tab) = create_signal(DashboardTab::Overview);
    let (vehicle, set_vehicle) = create_signal(Option::<VehicleWithAccess>::None);
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(String::new());

    let can_edit = create_memo(move |_| {
        vehicle
            .get()
            .map(|v| matches!(v.my_role, AccessRole::Owner | AccessRole::Editor))
            .unwrap_or(false)
    });

    let can_manage_contracts = create_memo(move |_| {
        vehicle
            .get()
            .map(|v| matches!(v.my_role, AccessRole::Owner))
            .unwrap_or(false)
    });

    create_effect(move |_| {
        let Some(id) = selected_id.get() else {
            set_vehicle.set(None);
            return;
        };

        set_tab.set(DashboardTab::Overview);
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

            match fetch_vehicle(&token, id).await {
                Ok(data) => set_vehicle.set(Some(data)),
                Err(e) => set_error.set(e),
            }
            set_loading.set(false);
        });
    });

    // Callback appelé après suppression du véhicule
    let on_deleted = Callback::new(move |deleted_id: Uuid| {
        // Retire le véhicule de la liste
        set_vehicles.update(|list| list.retain(|v| v.id != deleted_id));
        // Réinitialise la sélection et le dashboard
        set_selected_id.set(None);
        set_vehicle.set(None);
    });

    view! {
        <div class="flex flex-col h-full bg-gray-50 rounded-xl border border-gray-100 shadow-sm overflow-hidden">
            <Show
                when=move || selected_id.get().is_some()
                fallback=|| view! {
                    <div class="flex flex-col items-center justify-center h-full text-center space-y-4 p-8">
                        <div class="w-16 h-16 rounded-2xl bg-indigo-50 flex items-center justify-center">
                            <svg class="w-9 h-9 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                <path stroke-linecap="round" stroke-linejoin="round"
                                    d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                            </svg>
                        </div>
                        <h1 class="text-2xl font-bold text-gray-900">"Bienvenue dans votre espace !"</h1>
                        <p class="text-gray-500 max-w-sm">
                            "Sélectionnez un véhicule dans la liste pour accéder à son tableau de bord."
                        </p>
                    </div>
                }
            >
                // Bandeau véhicule
                <VehicleHeader vehicle=vehicle on_deleted=on_deleted />

                // Chargement / erreur
                <Show when=move || loading.get() fallback=|| ()>
                    <div class="flex items-center justify-center py-4">
                        <p class="text-sm text-gray-400 animate-pulse">"Chargement..."</p>
                    </div>
                </Show>

                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <div class="px-6 py-3">
                        <p class="text-sm text-red-500">{move || error.get()}</p>
                    </div>
                </Show>

                // Onglets
                <div class="flex border-b border-gray-200 bg-white px-6">
                    <TabButton
                        label="Tableau de bord"
                        active=move || tab.get() == DashboardTab::Overview
                        on_click=move || set_tab.set(DashboardTab::Overview)
                    />
                    <TabButton
                        label="Kilométrage"
                        active=move || tab.get() == DashboardTab::Kilometrage
                        on_click=move || set_tab.set(DashboardTab::Kilometrage)
                    />
                    <TabButton
                        label="Contrats"
                        active=move || tab.get() == DashboardTab::Contracts
                        on_click=move || set_tab.set(DashboardTab::Contracts)
                    />
                </div>

                // Contenu de l'onglet
                <div class="flex-1 overflow-auto p-6">
                    {move || match tab.get() {
                        DashboardTab::Overview => view! {
                            <div class="grid grid-cols-2 gap-4">
                                <MileageWidget vehicle_id=selected_id />
                                <ContractsWidget
                                    vehicle_id=selected_id
                                    can_manage_contracts=can_manage_contracts
                                />
                            </div>
                        }.into_view(),
                        DashboardTab::Kilometrage => view! {
                            <MileageList
                                vehicle_id=selected_id
                                can_edit=can_edit
                            />
                        }.into_view(),
                        DashboardTab::Contracts => view! {
                            <ContractList
                                vehicle_id=selected_id
                                can_manage_contracts=can_manage_contracts
                            />
                        }.into_view(),
                    }}
                </div>
            </Show>
        </div>
    }
}

#[component]
fn TabButton(
    label: &'static str,
    active: impl Fn() -> bool + 'static,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_click()
            class=move || {
                if active() {
                    "px-4 py-3 text-sm font-medium text-indigo-600 border-b-2 border-indigo-600 -mb-px transition-colors duration-150"
                } else {
                    "px-4 py-3 text-sm font-medium text-gray-500 border-b-2 border-transparent hover:text-gray-700 transition-colors duration-150"
                }
            }
        >
            {label}
        </button>
    }
}

async fn fetch_vehicle(token: &str, id: Uuid) -> Result<VehicleWithAccess, String> {
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
