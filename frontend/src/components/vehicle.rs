// src/components/vehicle_card.rs
use common::Vehicle;
use leptos::*;
use uuid::Uuid;

fn make_avatar_style(make: &str) -> &'static str {
    // Palette froide uniquement — évite rouge/amber/vert utilisés par les badges statut contrats
    let styles = [
        "background-color:#dbeafe;color:#1d4ed8", // blue
        "background-color:#e0e7ff;color:#4338ca", // indigo
        "background-color:#ede9fe;color:#6d28d9", // violet
        "background-color:#f3e8ff;color:#7e22ce", // purple
        "background-color:#fdf4ff;color:#a21caf", // fuchsia
        "background-color:#cffafe;color:#0e7490", // cyan
        "background-color:#ccfbf1;color:#0f766e", // teal
        "background-color:#e0f2fe;color:#0369a1", // sky
        "background-color:#f1f5f9;color:#475569", // slate
    ];
    let idx = make.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize)) % styles.len();
    styles[idx]
}

#[component]
pub fn VehicleCard(vehicle: Vehicle, set_selected: WriteSignal<Option<Uuid>>) -> impl IntoView {
    let id = vehicle.id;

    let initial = vehicle
        .make
        .chars()
        .next()
        .unwrap_or('?')
        .to_ascii_uppercase()
        .to_string();
    let avatar_style = make_avatar_style(&vehicle.make);

    view! {
        <button
            type="button"
            on:click=move |_| set_selected.set(Some(id))
            class="w-full bg-white rounded-xl border border-gray-100
                   px-4 py-3 flex items-center gap-3 text-left
                   cursor-pointer hover:border-indigo-300 hover:shadow-sm
                   transition-all duration-150"
        >
            // Initiale de la marque
            <div class="shrink-0 w-9 h-9 rounded-lg flex items-center justify-center" style=avatar_style>
                <span class="text-sm font-bold">{initial}</span>
            </div>

            // Contenu
            <div class="flex-1 min-w-0">
                <p class="text-xs text-gray-400 truncate">{vehicle.make}</p>
                <p class="text-xs text-gray-400 truncate">{vehicle.model}</p>
                <p class="text-sm font-semibold text-gray-900 tracking-wide mt-0.5">
                    {vehicle.plate_number}
                </p>
            </div>

            // Indicateur statut contrats
            {match vehicle.contract_status.as_deref() {
                Some("danger") => view! {
                    <span class="shrink-0 inline-flex items-center gap-1 text-xs font-medium text-red-700 bg-red-100 border border-red-200 rounded-full px-2 py-0.5">
                        <span class="w-1.5 h-1.5 rounded-full bg-red-500" />
                        "Dépassé"
                    </span>
                }.into_view(),
                Some("warning") => view! {
                    <span class="shrink-0 inline-flex items-center gap-1 text-xs font-medium text-amber-700 bg-amber-100 border border-amber-200 rounded-full px-2 py-0.5">
                        <span class="w-1.5 h-1.5 rounded-full bg-amber-400" />
                        "Risque"
                    </span>
                }.into_view(),
                Some("ok") => view! {
                    <span class="shrink-0 inline-flex items-center gap-1 text-xs font-medium text-green-700 bg-green-100 border border-green-200 rounded-full px-2 py-0.5">
                        <span class="w-1.5 h-1.5 rounded-full bg-green-500" />
                        "Actif"
                    </span>
                }.into_view(),
                _ => view! { <span /> }.into_view(),
            }}

            // Chevron
            <svg class="shrink-0 w-4 h-4 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
            </svg>
        </button>
    }
}
