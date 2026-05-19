// src/components/vehicle_card.rs
use crate::models::vehicle::Vehicle;
use leptos::*;

#[component]
pub fn VehicleCard(vehicle: Vehicle) -> impl IntoView {
    view! {
        <div class="
            bg-white rounded-xl border border-gray-100
            px-4 py-3 flex items-center gap-3
            cursor-pointer hover:border-gray-300
            transition-colors duration-150
        ">
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

            // Chevron
            <svg class="shrink-0 w-4 h-4 text-gray-300" /* chevron droit */ />
        </div>
    }
}

fn format_km(km: u32) -> String {
    // Formate 47320 → "47 320 km"
    let s = km.to_string();
    let chars: Vec<char> = s.chars().collect();
    let formatted = chars
        .rchunks(3)
        .rev()
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\u{202F}"); // espace fine insécable
    format!("{} km", formatted)
}
