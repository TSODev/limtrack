// src/components/trips/trip_widget.rs
use crate::api_client::api_get;
use crate::components::ui::{format_date_fr, format_km, get_token};
use common::UsageForecast;
use leptos::*;
use uuid::Uuid;

#[component]
pub fn TripsWidget(vehicle_id: ReadSignal<Option<Uuid>>, on_navigate: Callback<()>) -> impl IntoView {
    let (forecast, set_forecast) = create_signal(Option::<UsageForecast>::None);
    let (loading, set_loading) = create_signal(false);

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_forecast.set(None);
            set_loading.set(true);
            spawn_local(async move {
                let Some(token) = get_token() else { return };
                let result = api_get::<UsageForecast>(
                    &format!("{}/api/vehicles/{}/usage-forecast", crate::config::API_BASE, id),
                    &token,
                )
                .await
                .ok();
                set_forecast.set(result);
                set_loading.set(false);
            });
        }
    });

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 flex flex-col gap-3 md:gap-4">
            <button
                on:click=move |_| on_navigate.call(())
                class="flex items-center gap-1 text-sm font-semibold text-gray-700 uppercase tracking-wide hover:text-indigo-600 transition-colors duration-150 text-left"
            >
                "Capacité kilométrique"
                <svg class="h-3.5 w-3.5 opacity-50" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                </svg>
            </button>

            <Show when=move || loading.get() fallback=|| ()>
                <p class="text-xs text-gray-400 animate-pulse">"Chargement..."</p>
            </Show>

            <Show when=move || !loading.get() fallback=|| ()>
                {move || {
                    let Some(f) = forecast.get() else {
                        return view! {
                            <p class="text-xs text-gray-400 italic">
                                "Aucun contrat actif — rien à projeter pour l'instant."
                            </p>
                        }.into_view();
                    };

                    let Some(km_per_day) = f.km_per_day_available else {
                        return view! {
                            <p class="text-xs text-gray-400 italic">
                                "Aucun contrat actif — rien à projeter pour l'instant."
                            </p>
                        }.into_view();
                    };

                    let is_negative = km_per_day < 0.0;
                    let value_cls = if is_negative { "text-red-600" } else { "text-gray-800" };
                    let km_per_week = km_per_day * 7.0;
                    let km_per_month = km_per_day * 30.0;

                    view! {
                        <div class="space-y-3">
                            <div class="grid grid-cols-3 gap-3">
                                <div class="bg-gray-50 rounded-lg p-3 text-center">
                                    <p class="text-xs text-gray-400 mb-1">"Par jour"</p>
                                    <p class=format!("text-sm font-bold {}", value_cls)>{format_km(km_per_day.round() as i32)}</p>
                                </div>
                                <div class="bg-gray-50 rounded-lg p-3 text-center">
                                    <p class="text-xs text-gray-400 mb-1">"Par semaine"</p>
                                    <p class=format!("text-sm font-bold {}", value_cls)>{format_km(km_per_week.round() as i32)}</p>
                                </div>
                                <div class="bg-gray-50 rounded-lg p-3 text-center">
                                    <p class="text-xs text-gray-400 mb-1">"Par mois"</p>
                                    <p class=format!("text-sm font-bold {}", value_cls)>{format_km(km_per_month.round() as i32)}</p>
                                </div>
                            </div>
                            <p class="text-xs text-gray-400 text-center">
                                {if is_negative { "Dépassement déjà prévisible (rythme actuel + voyages planifiés)" } else { "Rythme actuel + voyages planifiés" }}
                            </p>

                            {f.unavailable_from.map(|date| {
                                let until_line = f.unavailable_until.map(|until| {
                                    format!("Du {} au {} (fin de contrat, voyages inclus)", format_date_fr(date), format_date_fr(until))
                                }).unwrap_or_else(|| format!("À partir du {}", format_date_fr(date)));

                                let reduction_line = f.recommended_daily_reduction_km.map(|km| {
                                    format!("Rouler {} km/jour de moins sur le reste de la période pour rester dans les clous", km)
                                });
                                let days_off_line = f.recommended_days_off.map(|days| {
                                    format!("...ou l'équivalent de {} jour(s) sans utiliser le véhicule, au rythme actuel", days)
                                });

                                view! {
                                    <div class="space-y-1.5 text-xs font-medium px-2.5 py-1.5 rounded-lg bg-amber-50 text-amber-700">
                                        <div class="flex items-center gap-1.5">"⚠ Indisponible : "{until_line}</div>
                                        {reduction_line.map(|l| view! { <p class="font-normal text-amber-600">{l}</p> })}
                                        {days_off_line.map(|l| view! { <p class="font-normal text-amber-600">{l}</p> })}
                                    </div>
                                }
                            })}
                        </div>
                    }.into_view()
                }}
            </Show>
        </div>
    }
}
