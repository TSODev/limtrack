// src/components/maintenance/maintenance_widget.rs
use crate::api_client::api_get;
use crate::components::ui::{format_date_fr, get_token};
use common::MaintenanceStatus;
use leptos::*;
use uuid::Uuid;

#[component]
pub fn MaintenanceWidget(vehicle_id: ReadSignal<Option<Uuid>>, on_navigate: Callback<()>) -> impl IntoView {
    let (statuses, set_statuses) = create_signal(Vec::<MaintenanceStatus>::new());
    let (loading, set_loading) = create_signal(false);

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_statuses.set(Vec::new());
            set_loading.set(true);
            spawn_local(async move {
                let Some(token) = get_token() else { return };
                let result = api_get::<Vec<MaintenanceStatus>>(
                    &format!("{}/api/vehicles/{}/maintenance-status", crate::config::API_BASE, id),
                    &token,
                )
                .await
                .unwrap_or_default();
                set_statuses.set(result);
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
                "Entretien"
                <svg class="h-3.5 w-3.5 opacity-50" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                </svg>
            </button>

            <Show when=move || loading.get() fallback=|| ()>
                <p class="text-xs text-gray-400 animate-pulse">"Chargement..."</p>
            </Show>

            <Show when=move || !loading.get() fallback=|| ()>
                {move || {
                    let list = statuses.get();

                    // Priorité : en retard d'abord, puis la prochaine échéance la plus proche
                    let most_urgent = list.iter()
                        .filter(|s| s.overdue || s.next_due_date.is_some())
                        .min_by(|a, b| {
                            match (a.overdue, b.overdue) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.next_due_date.cmp(&b.next_due_date),
                            }
                        })
                        .cloned();

                    let Some(s) = most_urgent else {
                        return view! {
                            <p class="text-xs text-gray-400 italic">
                                "Aucune échéance à venir — ajoutez un type d'entretien pour être prévenu."
                            </p>
                        }.into_view();
                    };

                    let date_text = s.next_due_date.map(format_date_fr).unwrap_or_else(|| "date inconnue".to_string());

                    view! {
                        <div class=format!(
                            "flex items-center gap-1.5 text-xs font-medium px-2.5 py-1.5 rounded-lg {}",
                            if s.overdue { "bg-red-50 text-red-700" } else { "bg-green-50 text-green-700" }
                        )>
                            {if s.overdue { "⚠ " } else { "" }}
                            {s.label.clone()}
                            {if s.overdue { " en retard".to_string() } else { format!(" — {}", date_text) }}
                        </div>
                    }.into_view()
                }}
            </Show>
        </div>
    }
}
