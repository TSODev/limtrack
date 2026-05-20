use common::{AccessRole, VehicleWithAccess};
use leptos::*;

#[component]
pub fn VehicleHeader(vehicle: ReadSignal<Option<VehicleWithAccess>>) -> impl IntoView {
    view! {
        <Show when=move || vehicle.get().is_some() fallback=|| ()>
            {move || vehicle.get().map(|v| {
                let role_label = match v.my_role {
                    AccessRole::Owner  => ("Propriétaire", "bg-indigo-100 text-indigo-700"),
                    AccessRole::Editor => ("Éditeur",      "bg-amber-100 text-amber-700"),
                    AccessRole::Viewer => ("Lecteur",      "bg-gray-100 text-gray-600"),
                };

                view! {
                    <div class="flex items-center justify-between px-6 py-4 bg-white border-b border-gray-100">
                        // Infos véhicule
                        <div class="flex items-center gap-4">
                            // Icône
                            <div class="w-12 h-12 rounded-xl bg-indigo-50 flex items-center justify-center shrink-0">
                                <svg class="w-7 h-7 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                                </svg>
                            </div>

                            // Texte
                            <div>
                                <div class="flex items-center gap-2">
                                    <h2 class="text-lg font-bold text-gray-900">
                                        {format!("{} {}", v.make, v.model)}
                                    </h2>
                                    <span class=format!(
                                        "text-xs font-medium px-2 py-0.5 rounded-full {}",
                                        role_label.1
                                    )>
                                        {role_label.0}
                                    </span>
                                </div>
                                <p class="text-sm font-mono font-semibold text-indigo-600 tracking-widest mt-0.5">
                                    {v.plate_number}
                                </p>
                            </div>
                        </div>
                    </div>
                }
            })}
        </Show>
    }
}
