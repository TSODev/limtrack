// src/components/vehicle_card.rs
//use crate::models::vehicle::Vehicle;
use common::Vehicle;
use leptos::*;
use uuid::Uuid;

#[component]
pub fn VehicleCard(vehicle: Vehicle, set_selected: WriteSignal<Option<Uuid>>) -> impl IntoView {
    let id = vehicle.id;

    view! {
        <button
            type="button"
            on:click=move |_| set_selected.set(Some(id))
            class="w-full bg-white rounded-xl border border-gray-100
                   px-4 py-3 flex items-center gap-3 text-left
                   cursor-pointer hover:border-indigo-300 hover:shadow-sm
                   transition-all duration-150"
        >
            // Icône
            <div class="shrink-0 w-9 h-9 rounded-lg bg-blue-50 flex items-center justify-center">
                <svg class="w-5 h-5 text-blue-500" /* icône voiture */ />
            </div>

            // Contenu
            <div class="flex-1 min-w-0">
                // Ligne 1 — modèle (muted, tronqué si trop long)
                <p class="text-xs text-gray-400 truncate">
                    {vehicle.make}
                </p>
                <p class="text-xs text-gray-400 truncate">
                    {vehicle.model}
                </p>
                // Ligne 2 — immatriculation en gras
                <p class="text-sm font-semibold text-gray-900 tracking-wide mt-0.5">
                    {vehicle.plate_number}
                </p>
                // Ligne 3 — kilométrage
                //<p class="text-xs text-gray-400 mt-0.5">
                //    {format_km(vehicle.kilometrage)}
                //</p>
            </div>

            // Indicateur statut contrats
            {match vehicle.contract_status.as_deref() {
                Some("danger") => view! {
                    <span class="shrink-0 inline-flex items-center gap-1 text-xs font-medium text-red-700 bg-red-100 border border-red-200 rounded-full px-2 py-0.5">
                        <span class="w-1.5 h-1.5 rounded-full bg-red-500"></span>
                        "Dépassé"
                    </span>
                }.into_view(),
                Some("warning") => view! {
                    <span class="shrink-0 inline-flex items-center gap-1 text-xs font-medium text-amber-700 bg-amber-100 border border-amber-200 rounded-full px-2 py-0.5">
                        <span class="w-1.5 h-1.5 rounded-full bg-amber-400"></span>
                        "Risque"
                    </span>
                }.into_view(),
                Some("ok") => view! {
                    <span class="shrink-0 inline-flex items-center gap-1 text-xs font-medium text-green-700 bg-green-100 border border-green-200 rounded-full px-2 py-0.5">
                        <span class="w-1.5 h-1.5 rounded-full bg-green-500"></span>
                        "Actif"
                    </span>
                }.into_view(),
                _ => view! { <span /> }.into_view(),
            }}

            // Chevron
            <svg class="shrink-0 w-4 h-4 text-gray-300" /* chevron droit */ />
        </button>
    }
}

