use crate::components::add_vehicle_button::AddVehicleButton;
use crate::components::vehicle_dashboard::VehicleDashboard;
use crate::components::vehicle_detail::VehicleDetail;
use crate::components::vehicle_list::{fetch_vehicles, Vehicle_list};

//use crate::models::Vehicle;
use common::Vehicle;
use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;

#[component]
pub fn MainPage() -> impl IntoView {
    let (vehicles, set_vehicles) = create_signal(vec![]);

    let navigate = use_navigate();
    let (is_authenticated, set_is_authenticated) = create_signal(false);

    let (selected_vehicle_id, set_selected_vehicle_id) = create_signal(Option::<uuid::Uuid>::None);

    // 1. Clone pour l'effet de vérification de session
    let navigate_effect = navigate.clone();
    create_effect(move |_| {
        let token = if let Ok(Some(storage)) = leptos::window().local_storage() {
            storage.get_item("jwt_token").unwrap_or(None)
        } else {
            None
        };

        if let Some(token) = token {
            set_is_authenticated.set(true);

            // Appel API pour récupérer les véhicules
            spawn_local(async move {
                match fetch_vehicles(&token).await {
                    Ok(data) => set_vehicles.set(data),
                    Err(e) => leptos::logging::error!("Erreur fetch véhicules : {:?}", e),
                }
            });
        } else {
            navigate_effect("/", NavigateOptions::default());
        }
    });

    // 2. On prépare le clone pour la vue
    let navigate_view = navigate.clone();

    view! {
        <Show
            when=move || is_authenticated.get()
            fallback=move || view! {
                <div class="min-h-screen flex items-center justify-center bg-gray-50">
                    <p class="text-gray-500 animate-pulse">"Vérification de l'authentification..."</p>
                </div>
            }
        >
            <div class="min-h-screen bg-gray-100">
                <nav class="bg-white shadow-sm border-b border-gray-200">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
                        <span class="text-xl font-bold text-indigo-600">odo.io</span>

                        // L'astuce magique : on clone au moment où la closure parente s'exécute
                        <button
                            on:click={
                                let navigate_click = navigate_view.clone();
                                move |_| {
                                    if let Ok(Some(storage)) = leptos::window().local_storage() {
                                        let _ = storage.remove_item("jwt_token");
                                    }
                                    navigate_click("/", NavigateOptions::default());
                                }
                            }
                            class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 transition duration-150"
                        >
                            "Déconnexion"
                        </button>
                    </div>
                </nav>

            // Ligne du milieu : deux colonnes
            <div class="flex flex-1 gap-4 p-4 overflow-hidden min-h-0">
           <aside class="w-1/4 flex flex-col overflow-auto gap-3 p-2">
              <Vehicle_list vehicles=vehicles set_selected=set_selected_vehicle_id />
               <AddVehicleButton set_vehicles=set_vehicles />
            </aside>

                <main class="max-w-7xl flex flex-col mx-auto py-12 px-4 sm:px-6 lg:px-8">
                    <div class="bg-white p-8 rounded-xl shadow-md border border-gray-100 text-center space-y-4">
                         <VehicleDashboard selected_id=selected_vehicle_id />
                    </div>
                </main>

            </div>

            // Module 4 — bas
            <footer class="shrink-0 bg-white border-t border-gray-200 p-4">
    //            <Module4 />
            </footer>
            </div>
        </Show>
    }
}
