use crate::components::notification_bell::NotificationBell;
use crate::components::vehicle_dashboard::VehicleDashboard;
use crate::components::vehicle_list::{fetch_vehicles, Vehicle_list};
use crate::pages::fleet::fetch_companies_count;
use leptos::*;
use leptos_router::*;

#[component]
pub fn MainPage() -> impl IntoView {
    let (vehicles, set_vehicles) = create_signal(vec![]);
    let navigate = use_navigate();
    let (is_authenticated, set_is_authenticated) = create_signal(false);
    let (selected_vehicle_id, set_selected_vehicle_id) = create_signal(Option::<uuid::Uuid>::None);
    let (sheet_open, set_sheet_open) = create_signal(false);
    let (has_fleet, set_has_fleet) = create_signal(false);

    let navigate_effect = navigate.clone();
    create_effect(move |_| {
        let token = if let Ok(Some(storage)) = leptos::window().local_storage() {
            storage.get_item("jwt_token").unwrap_or(None)
        } else {
            None
        };

        if let Some(token) = token {
            set_is_authenticated.set(true);
            let token_fleet = token.clone();
            spawn_local(async move {
                match fetch_vehicles(&token).await {
                    Ok(data) => set_vehicles.set(data),
                    Err(e) => leptos::logging::error!("Erreur fetch véhicules : {:?}", e),
                }
            });
            // Vérification silencieuse — pas d'impact si 0 entreprises
            spawn_local(async move {
                let count = fetch_companies_count(&token_fleet).await;
                set_has_fleet.set(count > 0);
            });
        } else {
            navigate_effect("/", NavigateOptions::default());
        }
    });

    let navigate_view = navigate.clone();

    // Véhicule sélectionné pour l'affichage dans la bottom bar
    let selected_vehicle = create_memo(move |_| {
        let id = selected_vehicle_id.get()?;
        vehicles.get().into_iter().find(|v| v.id == id)
    });

    // Ferme la sheet quand un véhicule est sélectionné
    create_effect(move |prev: Option<Option<uuid::Uuid>>| {
        let id = selected_vehicle_id.get();
        if prev.is_some() && id.is_some() {
            set_sheet_open.set(false);
        }
        id
    });

    view! {
        <Show
            when=move || is_authenticated.get()
            fallback=move || view! {
                <div class="min-h-screen flex items-center justify-center bg-gray-50">
                    <p class="text-gray-500 animate-pulse">"Vérification de l'authentification..."</p>
                </div>
            }
        >
            <div class="min-h-screen bg-gray-100 flex flex-col" style="padding-top: env(safe-area-inset-top)">

                // ─── Navbar ──────────────────────────────────────────
                <nav class="bg-white/90 backdrop-blur-md border-b border-gray-100 shadow-sm shrink-0 z-20">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-14 md:h-16 flex items-center justify-between">
                        <span class="text-lg md:text-xl font-bold text-indigo-600">"LimTrack"</span>

                        <div class="flex items-center gap-2 md:gap-3">
                            // Cloche
                            <NotificationBell vehicles=vehicles />

                            // Flotte — visible uniquement si l'utilisateur a des entreprises
                            <Show when=move || has_fleet.get() fallback=|| ()>
                                <A
                                    href="/fleet"
                                    class="hidden md:flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium text-gray-600 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                                >
                                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                            d="M3.75 21h16.5M4.5 3h15M5.25 3v18m13.5-18v18M9 6.75h1.5m-1.5 3h1.5m-1.5 3h1.5m3-6H15m-1.5 3H15m-1.5 3H15M9 21v-3.375c0-.621.504-1.125 1.125-1.125h3.75c.621 0 1.125.504 1.125 1.125V21" />
                                    </svg>
                                    "Flotte"
                                </A>
                                // Icône seule sur mobile
                                <A
                                    href="/fleet"
                                    class="md:hidden p-2 rounded-lg text-gray-500 hover:bg-gray-50 transition duration-150"
                                >
                                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                            d="M3.75 21h16.5M4.5 3h15M5.25 3v18m13.5-18v18M9 6.75h1.5m-1.5 3h1.5m-1.5 3h1.5m3-6H15m-1.5 3H15m-1.5 3H15M9 21v-3.375c0-.621.504-1.125 1.125-1.125h3.75c.621 0 1.125.504 1.125 1.125V21" />
                                    </svg>
                                </A>
                            </Show>

                            // À propos texte — visible md+
                            <A
                                href="/about"
                                class="hidden md:flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium text-gray-600 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                            >
                                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" />
                                </svg>
                                "À propos"
                            </A>

                            // À propos icône — mobile uniquement
                            <A
                                href="/about"
                                class="md:hidden p-2 rounded-lg text-gray-500 hover:bg-gray-50 transition duration-150"
                            >
                                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" />
                                </svg>
                            </A>

                            // Profil texte — visible md+
                            <A
                                href="/profile"
                                class="hidden md:flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium text-gray-600 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                            >
                                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z" />
                                </svg>
                                "Mon profil"
                            </A>

                            // Profil icône — mobile uniquement
                            <A
                                href="/profile"
                                class="md:hidden p-2 rounded-lg text-gray-500 hover:bg-gray-50 transition duration-150"
                            >
                                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z" />
                                </svg>
                            </A>

                            // Déconnexion — masquée sur mobile
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
                                class="hidden md:block px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 transition duration-150"
                            >
                                "Déconnexion"
                            </button>
                        </div>
                    </div>
                </nav>

                // ─── Layout Desktop (md+) ────────────────────────────
                <div class="hidden md:flex flex-1 gap-4 p-4 overflow-hidden min-h-0">
                    <aside class="w-1/4 flex flex-col overflow-auto gap-3 p-2">
                        <Vehicle_list
                            vehicles=vehicles
                            set_vehicles=set_vehicles
                            set_selected=set_selected_vehicle_id
                        />
                    </aside>
                    <main class="flex-1 flex flex-col min-h-0 py-4 pr-4">
                        <VehicleDashboard
                            selected_id=selected_vehicle_id
                            set_selected_id=set_selected_vehicle_id
                            set_vehicles=set_vehicles
                        />
                    </main>
                </div>

                // ─── Layout Mobile (< md) ────────────────────────────
                <div class="flex md:hidden flex-1 flex-col overflow-hidden min-h-0 relative">

                    // Dashboard — prend tout l'écran, pb pour laisser place à la bottom bar
                    <main class="flex-1 flex flex-col min-h-0 p-3 pb-24">
                        <VehicleDashboard
                            selected_id=selected_vehicle_id
                            set_selected_id=set_selected_vehicle_id
                            set_vehicles=set_vehicles
                        />
                    </main>

                    // ── Pill flottante ────────────────────────────────
                    <button
                        type="button"
                        class="fixed left-3 right-3 z-30 cursor-pointer w-auto"
                        style="bottom: calc(env(safe-area-inset-bottom) + 0.75rem)"
                        on:click=move |_| set_sheet_open.set(true)
                    >
                        <div class="bg-white/95 backdrop-blur-sm rounded-2xl shadow-xl border border-gray-100 flex items-center justify-between px-4 py-3">
                            {move || match selected_vehicle.get() {
                                Some(v) => view! {
                                    <div class="flex items-center gap-3">
                                        <div class="w-9 h-9 rounded-xl bg-indigo-100 flex items-center justify-center shrink-0">
                                            <svg class="w-5 h-5 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                                <path stroke-linecap="round" stroke-linejoin="round"
                                                    d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                                            </svg>
                                        </div>
                                        <div>
                                            <p class="text-sm font-semibold text-gray-900">
                                                {format!("{} {}", v.make, v.model)}
                                            </p>
                                            <p class="text-xs font-mono text-indigo-500">{v.plate_number}</p>
                                        </div>
                                    </div>
                                }.into_view(),
                                None => view! {
                                    <div class="flex items-center gap-3">
                                        <div class="w-9 h-9 rounded-xl bg-gray-100 flex items-center justify-center shrink-0">
                                            <svg class="w-5 h-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                                <path stroke-linecap="round" stroke-linejoin="round"
                                                    d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                                            </svg>
                                        </div>
                                        <p class="text-sm text-gray-400 font-medium">"Sélectionner un véhicule"</p>
                                    </div>
                                }.into_view(),
                            }}
                            <svg class="w-5 h-5 text-gray-300 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 15.75 7.5-7.5 7.5 7.5" />
                            </svg>
                        </div>
                    </button>

                    // ── Bottom Sheet ──────────────────────────────────
                    <Show when=move || sheet_open.get() fallback=|| ()>
                        // Overlay
                        <button
                            type="button"
                            class="fixed inset-0 z-40 bg-black bg-opacity-40 w-full cursor-default"
                            on:click=move |_| set_sheet_open.set(false)
                        />

                        // Panneau
                        <div class="fixed bottom-0 left-0 right-0 z-50 bg-white rounded-t-2xl shadow-2xl max-h-[80vh] flex flex-col">
                            // Handle
                            <div class="flex justify-center pt-3 pb-2 shrink-0">
                                <div class="w-12 h-1.5 bg-gray-200 rounded-full" />
                            </div>

                            // En-tête sheet
                            <div class="flex items-center justify-between px-4 pb-3 shrink-0 border-b border-gray-100">
                                <h2 class="text-base font-bold text-gray-900">"Mes véhicules"</h2>
                                <button
                                    on:click=move |_| set_sheet_open.set(false)
                                    class="text-gray-400 hover:text-gray-600 p-1"
                                >
                                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                                    </svg>
                                </button>
                            </div>

                            // Liste scrollable
                            <div class="flex-1 overflow-y-auto p-4">
                                <Vehicle_list
                                    vehicles=vehicles
                                    set_vehicles=set_vehicles
                                    set_selected=set_selected_vehicle_id
                                />
                            </div>
                        </div>
                    </Show>
                </div>

                // Footer desktop uniquement
                <footer class="hidden md:block shrink-0 bg-white border-t border-gray-200 p-4" />
            </div>
        </Show>
    }
}
