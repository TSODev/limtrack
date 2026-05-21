use crate::components::notification_bell::NotificationBell;
use crate::components::vehicle_dashboard::VehicleDashboard;
use crate::components::vehicle_list::{fetch_vehicles, Vehicle_list};
use common::Vehicle;
use leptos::*;
use leptos_router::*;

#[component]
pub fn MainPage() -> impl IntoView {
    let (vehicles, set_vehicles) = create_signal(vec![]);
    let navigate = use_navigate();
    let (is_authenticated, set_is_authenticated) = create_signal(false);
    let (selected_vehicle_id, set_selected_vehicle_id) = create_signal(Option::<uuid::Uuid>::None);

    let navigate_effect = navigate.clone();
    create_effect(move |_| {
        let token = if let Ok(Some(storage)) = leptos::window().local_storage() {
            storage.get_item("jwt_token").unwrap_or(None)
        } else {
            None
        };

        if let Some(token) = token {
            set_is_authenticated.set(true);
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
            <div class="min-h-screen bg-gray-100 flex flex-col">
                // Navbar
                <nav class="bg-white shadow-sm border-b border-gray-200 shrink-0">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
                        <span class="text-xl font-bold text-indigo-600">"odo.io"</span>

                        <div class="flex items-center gap-3">
                            // Cloche de notifications
                            <NotificationBell vehicles=vehicles />

                            // Lien profil
                            <A
                                href="/profile"
                                class="flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium text-gray-600 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                            >
                                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z" />
                                </svg>
                                "Mon profil"
                            </A>

                            // Déconnexion
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
                    </div>
                </nav>

                // Contenu principal
                <div class="flex flex-1 gap-4 p-4 overflow-hidden min-h-0">
                    <aside class="w-1/4 flex flex-col overflow-auto gap-3 p-2">
                        <Vehicle_list
                            vehicles=vehicles
                            set_vehicles=set_vehicles
                            set_selected=set_selected_vehicle_id
                        />
                    </aside>

                    <main class="flex-1 flex flex-col min-h-0 py-4 pr-4">
                        <VehicleDashboard selected_id=selected_vehicle_id />
                    </main>
                </div>

                // Footer
                <footer class="shrink-0 bg-white border-t border-gray-200 p-4" />
            </div>
        </Show>
    }
}
