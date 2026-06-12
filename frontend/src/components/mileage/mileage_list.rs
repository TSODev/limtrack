// src/components/mileage/mileage_list.rs
use crate::api_client::{api_delete, api_get, api_post};
use crate::components::ui::{format_date_fr, format_km, get_token, input_class};
use common::MileageLog;
use leptos::*;
use uuid::Uuid;

#[component]
pub fn MileageList(vehicle_id: ReadSignal<Option<Uuid>>, can_edit: Memo<bool>) -> impl IntoView {
    let (entries, set_entries) = create_signal(Vec::<MileageLog>::new());
    let (loading, set_loading) = create_signal(false);
    let (show_modal, set_show_modal) = create_signal(false);
    let (confirm_delete_id, set_confirm_delete_id) = create_signal(Option::<Uuid>::None);

    let load_mileage = move |id: Uuid| {
        set_loading.set(true);
        spawn_local(async move {
            let Some(token) = get_token() else { return };
            let data =
                api_get::<Vec<MileageLog>>(&format!("{}/api/vehicles/{}/mileage", crate::config::API_BASE, id), &token)
                    .await
                    .unwrap_or_default();
            set_entries.set(data);
            set_loading.set(false);
        });
    };

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_entries.set(vec![]);
            load_mileage(id);
        }
    });

    let on_created = move || {
        if let Some(id) = vehicle_id.get() {
            load_mileage(id);
        }
    };

    let delete_entry = move |entry_id: Uuid| {
        let vid = vehicle_id.get();
        spawn_local(async move {
            let Some(id) = vid else { return };
            let Some(token) = get_token() else { return };
            let url = format!(
                "{}/api/vehicles/{}/mileage/{}",
                crate::config::API_BASE, id, entry_id
            );
            if api_delete(&url, &token).await.is_ok() {
                load_mileage(id);
            }
            set_confirm_delete_id.set(None);
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex items-center justify-between">
                <h2 class="text-lg font-bold text-gray-900">"Historique kilométrique"</h2>
                // Bouton visible uniquement pour owner et editor
                <Show when=move || can_edit.get() fallback=|| ()>
                    <button
                        on:click=move |_| set_show_modal.set(true)
                        class="text-sm px-4 py-2 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 font-medium transition duration-150"
                    >
                        "+ Saisir un relevé"
                    </button>
                </Show>
            </div>

            <Show when=move || loading.get() fallback=|| ()>
                <p class="text-sm text-gray-400 animate-pulse">"Chargement..."</p>
            </Show>

            <Show when=move || !loading.get() fallback=|| ()>
                {move || {
                    let data = entries.get();
                    if data.is_empty() {
                        let can = can_edit.get();
                        return view! {
                            <div class="bg-white rounded-xl border border-dashed border-gray-200 p-8 text-center space-y-2">
                                <p class="text-sm font-medium text-gray-600">"Aucun relevé kilométrique enregistré."</p>
                                {if can {
                                    view! {
                                        <p class="text-xs text-gray-400 max-w-sm mx-auto">
                                            "Saisissez votre premier relevé kilométrique pour alimenter vos contrats LOA et assurance. "
                                            "Le relevé est automatiquement partagé entre tous les contrats actifs du véhicule."
                                        </p>
                                    }.into_view()
                                } else {
                                    view! { <></> }.into_view()
                                }}
                            </div>
                        }.into_view();
                    }

                    let last = data.first().unwrap().clone();
                    let first_entry = data.last().unwrap().clone();
                    let total_entries = data.len();
                    let km_parcourus = last.value - first_entry.value;

                    view! {
                        <div class="flex flex-col gap-4">
                            <div class="grid grid-cols-3 gap-4">
                                <div class="bg-white rounded-xl border border-gray-100 p-4 shadow-sm text-center">
                                    <p class="text-xs text-gray-400 uppercase tracking-wide mb-2">"Compteur actuel"</p>
                                    <p class="text-2xl font-extrabold text-indigo-600">{format_km(last.value)}</p>
                                    <p class="text-xs text-gray-400 mt-1">"au "{format_date_fr(last.recorded_at)}</p>
                                </div>
                                <div class="bg-white rounded-xl border border-gray-100 p-4 shadow-sm text-center">
                                    <p class="text-xs text-gray-400 uppercase tracking-wide mb-2">"Km parcourus"</p>
                                    <p class="text-2xl font-extrabold text-gray-800">{format_km(km_parcourus)}</p>
                                    <p class="text-xs text-gray-400 mt-1">"depuis le premier relevé"</p>
                                </div>
                                <div class="bg-white rounded-xl border border-gray-100 p-4 shadow-sm text-center">
                                    <p class="text-xs text-gray-400 uppercase tracking-wide mb-2">"Relevés"</p>
                                    <p class="text-2xl font-extrabold text-gray-800">{total_entries}</p>
                                    <p class="text-xs text-gray-400 mt-1">"enregistrés"</p>
                                </div>
                            </div>

                            <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
                                <div class="overflow-y-auto max-h-[280px]">
                                    <table class="w-full text-sm">
                                        <thead class="sticky top-0 bg-gray-50 z-10">
                                            <tr class="border-b border-gray-100">
                                                <th class="text-left px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wide">"Date"</th>
                                                <th class="text-right px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wide">"Compteur"</th>
                                                <th class="text-right px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wide">"Écart"</th>
                                                <th class="text-center px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wide">"Source"</th>
                                                <Show when=move || can_edit.get() fallback=|| ()>
                                                    <th class="w-8" />
                                                </Show>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {data.iter().enumerate().map(|(i, entry)| {
                                                let entry = entry.clone();
                                                let entry_id = entry.id;
                                                let next = data.get(i + 1).cloned();
                                                let ecart = next.map(|n| entry.value - n.value);
                                                let source_label = match entry.source.as_str() {
                                                    "manual" => ("Manuelle", "bg-gray-100 text-gray-600"),
                                                    "import" => ("Import",   "bg-blue-100 text-blue-600"),
                                                    "api"    => ("API",      "bg-purple-100 text-purple-600"),
                                                    _        => ("—",        "bg-gray-100 text-gray-400"),
                                                };
                                                let is_confirming = move || confirm_delete_id.get() == Some(entry_id);
                                                view! {
                                                    <tr class="border-b border-gray-50 hover:bg-gray-50 transition-colors duration-100">
                                                        <td class="px-4 py-3 text-gray-700">{format_date_fr(entry.recorded_at)}</td>
                                                        <td class="px-4 py-3 text-right font-semibold text-gray-900">{format_km(entry.value)}</td>
                                                        <td class="px-4 py-3 text-right text-gray-500">
                                                            {ecart.map(|e| format!("+ {}", format_km(e))).unwrap_or("—".to_string())}
                                                        </td>
                                                        <td class="px-4 py-3 text-center">
                                                            <span class=format!("text-xs font-medium px-2 py-0.5 rounded-full {}", source_label.1)>
                                                                {source_label.0}
                                                            </span>
                                                        </td>
                                                        <Show when=move || can_edit.get() fallback=|| ()>
                                                            <td class="px-2 py-2 text-right whitespace-nowrap">
                                                                <Show when=is_confirming fallback=move || view! {
                                                                    <button
                                                                        on:click=move |_| set_confirm_delete_id.set(Some(entry_id))
                                                                        title="Supprimer ce relevé"
                                                                        class="text-gray-400 hover:text-red-500 hover:bg-red-50 transition-colors duration-100 p-1 rounded"
                                                                    >
                                                                        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                                                                        </svg>
                                                                    </button>
                                                                }>
                                                                    <div class="flex items-center gap-1 justify-end">
                                                                        <button
                                                                            on:click=move |_| set_confirm_delete_id.set(None)
                                                                            class="text-xs px-1.5 py-0.5 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 transition duration-150"
                                                                        >"Non"</button>
                                                                        <button
                                                                            on:click=move |_| delete_entry(entry_id)
                                                                            class="text-xs px-1.5 py-0.5 rounded border border-red-200 bg-red-50 text-red-600 hover:bg-red-100 font-medium transition duration-150"
                                                                        >"Oui, supprimer"</button>
                                                                    </div>
                                                                </Show>
                                                            </td>
                                                        </Show>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                }}
            </Show>
        </div>

        <Show when=move || show_modal.get() fallback=|| ()>
            <MileageModal
                vehicle_id=vehicle_id
                on_close=Callback::new(move |_| set_show_modal.set(false))
                on_created=Callback::new(move |_| on_created())
            />
        </Show>
    }
}

#[component]
fn MileageModal(
    vehicle_id: ReadSignal<Option<Uuid>>,
    on_close: Callback<()>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (value, set_value) = create_signal(String::new());
    let (recorded_at, set_recorded_at) = create_signal(today_str());
    let (error, set_error) = create_signal(String::new());

    let submit = create_action(move |(vid, val, date): &(Uuid, String, String)| {
        let (vid, val, date) = (*vid, val.clone(), date.clone());
        async move {
            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({ "value": val.parse::<i32>().unwrap_or(0), "recorded_at": date, "source": "manual" });
            match api_post(&format!("{}/api/vehicles/{}/mileage", crate::config::API_BASE, vid), &token, &body).await {
                Ok(_) => {
                    on_created.call(());
                    on_close.call(());
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(id) = vehicle_id.get() else { return };
        set_error.set(String::new());
        submit.dispatch((id, value.get(), recorded_at.get()));
    };

    view! {
        <button type="button" class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm w-full cursor-default" on:click=move |_| on_close.call(()) />
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
            <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-sm p-8 space-y-6">
                <div class="flex items-center justify-between">
                    <div>
                        <h2 class="text-xl font-bold text-gray-900">"Nouveau relevé"</h2>
                        <p class="text-xs text-gray-400 mt-1">"Ce relevé sera appliqué à tous les contrats actifs."</p>
                    </div>
                    <button on:click=move |_| on_close.call(()) class="text-gray-400 hover:text-gray-600 text-xl font-light">"✕"</button>
                </div>
                <form on:submit=on_submit class="space-y-4">
                    <div class="space-y-1">
                        <label class="text-sm font-medium text-gray-700 block">"Kilométrage au compteur"</label>
                        <input type="number" min="0" required prop:value=value
                            on:input=move |ev| set_value.set(event_target_value(&ev))
                            placeholder="ex: 48500" class=input_class() />
                    </div>
                    <div class="space-y-1">
                        <label class="text-sm font-medium text-gray-700 block">"Date du relevé"</label>
                        <input type="date" required prop:value=recorded_at
                            on:input=move |ev| set_recorded_at.set(event_target_value(&ev))
                            class=input_class() />
                    </div>
                    <Show when=move || !error.get().is_empty() fallback=|| ()>
                        <p class="text-sm text-center text-red-600">{move || error.get()}</p>
                    </Show>
                    <div class="flex gap-3 pt-2">
                        <button type="button" on:click=move |_| on_close.call(())
                            class="flex-1 py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150">
                            "Annuler"
                        </button>
                        <button type="submit" prop:disabled=move || submit.pending().get()
                            class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150">
                            {move || if submit.pending().get() { "Envoi..." } else { "Enregistrer" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

fn today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

