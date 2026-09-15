// src/components/maintenance/maintenance_list.rs
use crate::api_client::{api_delete, api_get, api_patch, api_post, api_post_response};
use crate::components::maintenance::catalog::{GenericTemplate, CATEGORY_ORDER, GENERIC_CATALOG};
use crate::components::ui::{format_date_fr, format_km, get_token, input_class};
use common::{MaintenanceAttachment, MaintenanceEntry, MaintenanceStatus, MaintenanceType};
use leptos::*;
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

fn interval_summary(t: &MaintenanceType) -> String {
    match (t.interval_km, t.interval_months) {
        (Some(km), Some(m)) => format!("Tous les {} ou {} mois", format_km(km), m),
        (Some(km), None) => format!("Tous les {}", format_km(km)),
        (None, Some(m)) => format!("Tous les {} mois", m),
        (None, None) => "—".to_string(),
    }
}

fn status_badge(status: Option<&MaintenanceStatus>) -> (&'static str, &'static str, String) {
    let Some(s) = status else {
        return ("bg-gray-100 text-gray-500", "Jamais fait", String::new());
    };
    if s.last_performed_at.is_none() {
        return ("bg-gray-100 text-gray-500", "Jamais fait", String::new());
    }
    if s.overdue {
        return ("bg-red-100 text-red-700", "En retard", String::new());
    }
    if let Some(date) = s.next_due_date {
        let today = chrono::Local::now().date_naive();
        let days = (date - today).num_days();
        if days <= 30 {
            return ("bg-amber-100 text-amber-700", "Bientôt", format!(" ({})", format_date_fr(date)));
        }
        return ("bg-green-100 text-green-700", "À jour", format!(" (jusqu'au {})", format_date_fr(date)));
    }
    ("bg-green-100 text-green-700", "À jour", String::new())
}

#[derive(Clone)]
struct MaintenanceData {
    types: Vec<MaintenanceType>,
    statuses: Vec<MaintenanceStatus>,
    entries: Vec<MaintenanceEntry>,
}

#[component]
pub fn MaintenanceList(
    vehicle_id: ReadSignal<Option<Uuid>>,
    can_manage_maintenance: Memo<bool>,
    vehicle_fuel_type: Signal<Option<String>>,
) -> impl IntoView {
    let (data, set_data) = create_signal(Option::<MaintenanceData>::None);
    let (loading, set_loading) = create_signal(false);
    let (show_type_modal, set_show_type_modal) = create_signal(false);
    let (editing_type, set_editing_type) = create_signal(Option::<MaintenanceType>::None);
    let (show_entry_modal, set_show_entry_modal) = create_signal(false);
    let (confirm_delete, set_confirm_delete) = create_signal(Option::<(String, String)>::None); // (label, kind:type|entry)
    let (viewing_attachments, set_viewing_attachments) = create_signal(Option::<(Uuid, String)>::None); // (entry_id, entry_label)

    let load = move |id: Uuid| {
        set_loading.set(true);
        spawn_local(async move {
            let Some(token) = get_token() else { return };
            let types = api_get::<Vec<MaintenanceType>>(
                &format!("{}/api/vehicles/{}/maintenance-types", crate::config::API_BASE, id), &token,
            ).await.unwrap_or_default();
            let statuses = api_get::<Vec<MaintenanceStatus>>(
                &format!("{}/api/vehicles/{}/maintenance-status", crate::config::API_BASE, id), &token,
            ).await.unwrap_or_default();
            let entries = api_get::<Vec<MaintenanceEntry>>(
                &format!("{}/api/vehicles/{}/maintenance-entries", crate::config::API_BASE, id), &token,
            ).await.unwrap_or_default();
            set_data.set(Some(MaintenanceData { types, statuses, entries }));
            set_loading.set(false);
        });
    };

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_data.set(None);
            load(id);
        }
    });

    let on_saved = move || {
        if let Some(id) = vehicle_id.get() {
            load(id);
        }
    };

    view! {
        <div class="flex flex-col gap-8">
            // ─── Types de maintenance ───────────────────────────
            <div class="flex flex-col gap-4">
                <div class="flex items-center justify-between">
                    <h2 class="text-lg font-bold text-gray-900">"Types de maintenance"</h2>
                    <Show when=move || can_manage_maintenance.get() fallback=|| ()>
                        <button
                            on:click=move |_| { set_editing_type.set(None); set_show_type_modal.set(true); }
                            class="text-sm px-4 py-2 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 font-medium transition duration-150"
                        >
                            "+ Type"
                        </button>
                    </Show>
                </div>

                <Show when=move || loading.get() fallback=|| ()>
                    <p class="text-sm text-gray-400 animate-pulse">"Chargement..."</p>
                </Show>

                <Show when=move || !loading.get() && data.get().map(|d| d.types.is_empty()).unwrap_or(false) fallback=|| ()>
                    <div class="bg-white rounded-xl border border-dashed border-gray-200 p-8 text-center">
                        <p class="text-sm font-medium text-gray-600">"Aucun type d'entretien configuré."</p>
                    </div>
                </Show>

                <Show when=move || data.get().map(|d| !d.types.is_empty()).unwrap_or(false) fallback=|| ()>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {move || {
                            let Some(d) = data.get() else { return view! { <div /> }.into_view() };
                            let can_manage = can_manage_maintenance.get();
                            d.types.into_iter().map(|t| {
                                let status = d.statuses.iter().find(|s| s.type_id == t.id).cloned();
                                let (badge_bg, badge_label, badge_suffix) = status_badge(status.as_ref());
                                let label = t.label.clone();
                                let summary = interval_summary(&t);
                                let t_for_edit = t.clone();
                                let t_for_delete = t.clone();
                                view! {
                                    <div class="bg-white rounded-xl border border-gray-100 p-4 space-y-2 shadow-sm">
                                        <div class="flex items-center justify-between">
                                            <span class="text-sm font-bold text-gray-800">{label.clone()}</span>
                                            <span class=format!("text-xs font-medium px-2.5 py-1 rounded-full {}", badge_bg)>
                                                {badge_label}{badge_suffix}
                                            </span>
                                        </div>
                                        <p class="text-xs text-gray-400">{summary}</p>
                                        <Show when=move || can_manage fallback=|| ()>
                                            <div class="flex items-center justify-end gap-1.5 pt-1 border-t border-gray-50">
                                                <button
                                                    on:click={let t = t_for_edit.clone(); move |_| { set_editing_type.set(Some(t.clone())); set_show_type_modal.set(true); }}
                                                    class="text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                                                >
                                                    "Modifier"
                                                </button>
                                                <button
                                                    on:click={let t = t_for_delete.clone(); move |_| set_confirm_delete.set(Some((t.label.clone(), t.id.to_string())))}
                                                    class="text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-red-50 hover:text-red-600 transition duration-150"
                                                >
                                                    "Supprimer"
                                                </button>
                                            </div>
                                        </Show>
                                    </div>
                                }
                            }).collect_view().into_view()
                        }}
                    </div>
                </Show>
            </div>

            // ─── Historique ─────────────────────────────────────
            <div class="flex flex-col gap-4">
                <div class="flex items-center justify-between">
                    <h2 class="text-lg font-bold text-gray-900">"Historique"</h2>
                    <Show when=move || can_manage_maintenance.get() fallback=|| ()>
                        <button
                            on:click=move |_| set_show_entry_modal.set(true)
                            class="text-sm px-4 py-2 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 font-medium transition duration-150"
                        >
                            "+ Entretien"
                        </button>
                    </Show>
                </div>

                <Show when=move || data.get().map(|d| d.entries.is_empty()).unwrap_or(false) fallback=|| ()>
                    <div class="bg-white rounded-xl border border-dashed border-gray-200 p-8 text-center">
                        <p class="text-sm font-medium text-gray-600">"Aucun entretien enregistré."</p>
                    </div>
                </Show>

                <Show when=move || data.get().map(|d| !d.entries.is_empty()).unwrap_or(false) fallback=|| ()>
                    <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
                        <div class="overflow-y-auto max-h-[320px]">
                            <table class="w-full text-sm">
                                <thead class="sticky top-0 bg-gray-50 z-10">
                                    <tr class="border-b border-gray-100">
                                        <th class="text-left px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wide">"Date"</th>
                                        <th class="text-left px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wide">"Type"</th>
                                        <th class="text-right px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wide">"Km"</th>
                                        <th class="text-right px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wide">"Coût"</th>
                                        <th class="text-left px-4 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wide">"Garage"</th>
                                        <th class="px-4 py-3"></th>
                                        <th class="px-4 py-3"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        let Some(d) = data.get() else { return view! { <tr /> }.into_view() };
                                        let can_manage = can_manage_maintenance.get();
                                        d.entries.into_iter().map(|e| {
                                            let label = e.label.clone();
                                            let entry_id = e.id;
                                            let entry_label = label.clone();
                                            let attachment_count = e.attachment_count;
                                            view! {
                                                <tr class="border-b border-gray-50 last:border-0">
                                                    <td class="px-4 py-3 text-gray-600 whitespace-nowrap">{format_date_fr(e.performed_at)}</td>
                                                    <td class="px-4 py-3 text-gray-800 font-medium">{label.clone()}</td>
                                                    <td class="px-4 py-3 text-right text-gray-600 whitespace-nowrap">{format_km(e.km_at_service)}</td>
                                                    <td class="px-4 py-3 text-right text-gray-600 whitespace-nowrap">
                                                        {e.cost.map(|c| format!("{:.2} €", c)).unwrap_or_else(|| "—".to_string())}
                                                    </td>
                                                    <td class="px-4 py-3 text-gray-500">{e.provider.clone().unwrap_or_else(|| "—".to_string())}</td>
                                                    <td class="px-4 py-3 text-right">
                                                        {(attachment_count > 0).then(|| view! {
                                                            <button
                                                                on:click=move |_| set_viewing_attachments.set(Some((entry_id, entry_label.clone())))
                                                                class="text-xs text-gray-500 hover:text-indigo-600 transition duration-150"
                                                                title="Voir les pièces jointes"
                                                            >
                                                                "📎 "{attachment_count}
                                                            </button>
                                                        })}
                                                    </td>
                                                    <td class="px-4 py-3 text-right">
                                                        <Show when=move || can_manage fallback=|| ()>
                                                            <button
                                                                on:click={let label = label.clone(); let id = e.id; move |_| set_confirm_delete.set(Some((label.clone(), format!("entry:{}", id))))}
                                                                class="text-xs text-gray-400 hover:text-red-600 transition duration-150"
                                                            >
                                                                "Supprimer"
                                                            </button>
                                                        </Show>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view().into_view()
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </Show>
            </div>
        </div>

        <Show when=move || show_type_modal.get() fallback=|| ()>
            <TypeModal
                vehicle_id=vehicle_id
                existing=editing_type.get()
                on_close=Callback::new(move |_| set_show_type_modal.set(false))
                on_saved=Callback::new(move |_| on_saved())
            />
        </Show>

        <Show when=move || show_entry_modal.get() fallback=|| ()>
            <EntryModal
                vehicle_id=vehicle_id
                types=data.get().map(|d| d.types).unwrap_or_default()
                vehicle_fuel_type=vehicle_fuel_type.get()
                on_close=Callback::new(move |_| set_show_entry_modal.set(false))
                on_saved=Callback::new(move |_| on_saved())
            />
        </Show>

        <Show when=move || viewing_attachments.get().is_some() fallback=|| ()>
            {move || viewing_attachments.get().map(|(entry_id, entry_label)| {
                let can_manage = can_manage_maintenance.get();
                view! {
                    <AttachmentsModal
                        vehicle_id=vehicle_id
                        entry_id=entry_id
                        entry_label=entry_label
                        can_manage=can_manage
                        on_close=Callback::new(move |_| set_viewing_attachments.set(None))
                        on_changed=Callback::new(move |_| on_saved())
                    />
                }
            })}
        </Show>

        <Show when=move || confirm_delete.get().is_some() fallback=|| ()>
            {move || confirm_delete.get().map(|(label, key)| {
                let vid = vehicle_id;
                let key_for_confirm = key.clone();
                view! {
                    <ConfirmDeleteModal
                        label=label
                        on_cancel=Callback::new(move |_| set_confirm_delete.set(None))
                        on_confirm=Callback::new(move |_| {
                            let key = key_for_confirm.clone();
                            spawn_local(async move {
                                let Some(id) = vid.get() else { return };
                                let Some(token) = get_token() else { return };
                                let url = if let Some(entry_id) = key.strip_prefix("entry:") {
                                    format!("{}/api/vehicles/{}/maintenance-entries/{}", crate::config::API_BASE, id, entry_id)
                                } else {
                                    format!("{}/api/vehicles/{}/maintenance-types/{}", crate::config::API_BASE, id, key)
                                };
                                let _ = api_delete(&url, &token).await;
                                set_confirm_delete.set(None);
                                on_saved();
                            });
                        })
                    />
                }
            })}
        </Show>
    }
}

#[component]
fn ConfirmDeleteModal(label: String, on_cancel: Callback<()>, on_confirm: Callback<()>) -> impl IntoView {
    view! {
        <button type="button"
            class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm w-full cursor-default"
            on:click=move |_| on_cancel.call(()) />
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
            <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-sm p-8 space-y-6">
                <div class="flex items-center gap-3">
                    <div class="flex-shrink-0 w-10 h-10 rounded-full bg-red-50 flex items-center justify-center">
                        <svg class="w-5 h-5 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
                        </svg>
                    </div>
                    <div>
                        <h2 class="text-lg font-bold text-gray-900">"Confirmer la suppression"</h2>
                        <p class="text-sm text-gray-500 mt-0.5">"Cette action est irréversible."</p>
                    </div>
                </div>
                <p class="text-sm text-gray-600">
                    "Voulez-vous vraiment supprimer "
                    <span class="font-semibold">{label}</span>
                    " ?"
                </p>
                <div class="flex gap-3">
                    <button type="button" on:click=move |_| on_cancel.call(())
                        class="flex-1 py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150">
                        "Annuler"
                    </button>
                    <button type="button"
                        on:click=move |_| on_confirm.call(())
                        class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-red-600 hover:bg-red-700 transition duration-150">
                        "Supprimer"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TypeModal(
    vehicle_id: ReadSignal<Option<Uuid>>,
    existing: Option<MaintenanceType>,
    on_close: Callback<()>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let is_edit = existing.is_some();
    let type_id = existing.as_ref().map(|t| t.id);

    let (label, set_label) = create_signal(existing.as_ref().map(|t| t.label.clone()).unwrap_or_default());
    let (interval_km, set_interval_km) = create_signal(
        existing.as_ref().and_then(|t| t.interval_km).map(|v| v.to_string()).unwrap_or_default(),
    );
    let (interval_months, set_interval_months) = create_signal(
        existing.as_ref().and_then(|t| t.interval_months).map(|v| v.to_string()).unwrap_or_default(),
    );
    let (error, set_error) = create_signal(String::new());

    let submit = create_action(move |_: &()| {
        let vid = vehicle_id.get();
        let label_v = label.get();
        let km = interval_km.get().trim().parse::<i32>().ok();
        let months = interval_months.get().trim().parse::<i32>().ok();

        async move {
            let Some(vid) = vid else { return };
            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({
                "label": label_v,
                "interval_km": km,
                "interval_months": months,
            });

            let result = if let Some(tid) = type_id {
                api_patch(&format!("{}/api/vehicles/{}/maintenance-types/{}", crate::config::API_BASE, vid, tid), &token, &body).await
            } else {
                api_post(&format!("{}/api/vehicles/{}/maintenance-types", crate::config::API_BASE, vid), &token, &body).await
            };

            match result {
                Ok(_) => { on_saved.call(()); on_close.call(()); }
                Err(e) => set_error.set(e),
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_error.set(String::new());
        submit.dispatch(());
    };

    view! {
        <Modal title=if is_edit { "Modifier le type" } else { "Nouveau type d'entretien" } on_close=on_close>
            <form on:submit=on_submit class="space-y-4">
                <Field label="Nom">
                    <input type="text" required prop:value=label
                        on:input=move |ev| set_label.set(event_target_value(&ev))
                        placeholder="ex: Vidange" class=input_class() />
                </Field>
                <div class="grid grid-cols-2 gap-3">
                    <Field label="Intervalle (km)">
                        <input type="number" min="1" prop:value=interval_km
                            on:input=move |ev| set_interval_km.set(event_target_value(&ev))
                            placeholder="ex: 15000" class=input_class() />
                    </Field>
                    <Field label="Intervalle (mois)">
                        <input type="number" min="1" prop:value=interval_months
                            on:input=move |ev| set_interval_months.set(event_target_value(&ev))
                            placeholder="ex: 12" class=input_class() />
                    </Field>
                </div>
                <p class="text-xs text-gray-400">"Au moins un des deux intervalles est requis — le plus proche des deux déclenche le rappel."</p>
                <ModalActions
                    pending=submit.pending().into()
                    on_cancel=Callback::new(move |_| on_close.call(()))
                    label_submit=if is_edit { "Enregistrer" } else { "Créer le type" }
                    error=error
                />
            </form>
        </Modal>
    }
}

#[component]
fn EntryModal(
    vehicle_id: ReadSignal<Option<Uuid>>,
    types: Vec<MaintenanceType>,
    vehicle_fuel_type: Option<String>,
    on_close: Callback<()>,
    on_saved: Callback<()>,
) -> impl IntoView {
    // Items du catalogue générique pas encore instanciés pour ce véhicule (dédoublonnage
    // insensible à la casse), filtrés par motorisation si elle est renseignée.
    let available_generic: Vec<(usize, &'static GenericTemplate)> = GENERIC_CATALOG
        .iter()
        .enumerate()
        .filter(|(_, tpl)| !types.iter().any(|t| t.label.eq_ignore_ascii_case(tpl.label)))
        .filter(|(_, tpl)| match (&vehicle_fuel_type, tpl.fuel_type) {
            (Some(vft), Some(tft)) => vft == tft,
            _ => true,
        })
        .collect();
    let show_fuel_tag = vehicle_fuel_type.is_none();

    // Sélection multiple (une "révision" peut couvrir plusieurs types) — Vec plutôt que
    // HashSet pour conserver l'ordre de sélection, réutilisé pour construire le libellé auto.
    let (selected, set_selected) = create_signal(Vec::<String>::new());
    let (custom_label, set_custom_label) = create_signal(String::new());
    let (performed_at, set_performed_at) = create_signal(chrono::Local::now().date_naive().to_string());
    let (km_at_service, set_km_at_service) = create_signal(String::new());
    let (cost, set_cost) = create_signal(String::new());
    let (provider, set_provider) = create_signal(String::new());
    let (notes, set_notes) = create_signal(String::new());
    let (files, set_files) = create_signal(Vec::<web_sys::File>::new());
    let (compressing, set_compressing) = create_signal(false);
    let (error, set_error) = create_signal(String::new());
    let file_input_ref = create_node_ref::<leptos::html::Input>();

    // Le libellé n'est requis que si aucun type n'est coché (entrée libre, ex: "Autre")
    let label_required = move || selected.get().is_empty();

    let toggle_selected = move |key: String| {
        set_selected.update(|s| {
            if let Some(pos) = s.iter().position(|k| k == &key) {
                s.remove(pos);
            } else {
                s.push(key);
            }
        });
    };

    // Compresse les photos côté client avant l'envoi (redimensionnement + ré-encodage JPEG) —
    // réduit une photo de smartphone de plusieurs Mo à quelques centaines de Ko sans passer
    // par un format intermédiaire type PDF (qui n'apporterait aucun gain). Les PDF (factures
    // scannées) passent inchangés.
    let on_files_change = move |ev: web_sys::Event| {
        let input = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let mut list = Vec::new();
        if let Some(file_list) = input.files() {
            for i in 0..file_list.length() {
                if let Some(f) = file_list.get(i) {
                    list.push(f);
                }
            }
        }
        set_compressing.set(true);
        spawn_local(async move {
            let mut compressed = Vec::with_capacity(list.len());
            for file in list {
                if file.type_().starts_with("image/") {
                    match compress_image(&file).await {
                        Ok(f) => compressed.push(f),
                        Err(_) => compressed.push(file),
                    }
                } else {
                    compressed.push(file);
                }
            }
            set_files.set(compressed);
            set_compressing.set(false);
        });
    };

    let types_for_submit = types.clone();
    let submit = create_action(move |_: &()| {
        let types = types_for_submit.clone();
        let vid = vehicle_id.get();
        let selection = selected.get();
        let date_v = performed_at.get();
        let km_v = km_at_service.get().trim().parse::<i32>().unwrap_or(0);
        let cost_v = cost.get().trim().parse::<f64>().ok();
        let provider_v = provider.get();
        let notes_v = notes.get();
        let custom_label_v = custom_label.get().trim().to_string();
        let files_v = files.get();

        async move {
            let Some(vid) = vid else { return };
            let token = get_token().unwrap_or_default();

            if selection.is_empty() && custom_label_v.is_empty() {
                set_error.set("Sélectionnez au moins un type, ou saisissez un libellé".to_string());
                return;
            }

            // Résout maintenance_type_ids + libellés selon la sélection (plusieurs types
            // possibles pour une révision) : types existants, items générique (instanciés en
            // type si périodique, sinon libellé seul), ou entrée libre.
            let mut type_ids: Vec<Uuid> = Vec::new();
            let mut labels: Vec<String> = Vec::new();

            for key in &selection {
                if let Some(uuid_str) = key.strip_prefix("type:") {
                    let Some(id) = Uuid::parse_str(uuid_str).ok() else { continue };
                    let label = types.iter().find(|t| t.id == id).map(|t| t.label.clone()).unwrap_or_default();
                    type_ids.push(id);
                    labels.push(label);
                } else if let Some(idx_str) = key.strip_prefix("generic:") {
                    let Some(tpl) = idx_str.parse::<usize>().ok().and_then(|i| GENERIC_CATALOG.get(i)) else {
                        set_error.set("Élément du catalogue introuvable".to_string());
                        return;
                    };
                    if tpl.is_periodic() {
                        let create_type_body = serde_json::json!({
                            "label": tpl.label,
                            "interval_km": tpl.interval_km,
                            "interval_months": tpl.interval_months,
                        });
                        match api_post_response::<serde_json::Value>(
                            &format!("{}/api/vehicles/{}/maintenance-types", crate::config::API_BASE, vid),
                            &token, &create_type_body,
                        ).await {
                            Ok(v) => {
                                if let Some(new_id) = v["id"].as_str().and_then(|s| Uuid::parse_str(s).ok()) {
                                    type_ids.push(new_id);
                                }
                                labels.push(tpl.label.to_string());
                            }
                            Err(e) => { set_error.set(e); return; }
                        }
                    } else {
                        labels.push(tpl.label.to_string());
                    }
                }
            }

            let label = if !custom_label_v.is_empty() {
                Some(custom_label_v)
            } else if !labels.is_empty() {
                Some(labels.join(", "))
            } else {
                None
            };

            let body = serde_json::json!({
                "maintenance_type_ids": type_ids,
                "label": label,
                "performed_at": date_v,
                "km_at_service": km_v,
                "cost": cost_v,
                "provider": if provider_v.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(provider_v) },
                "notes": if notes_v.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(notes_v) },
            });

            match api_post_response::<serde_json::Value>(
                &format!("{}/api/vehicles/{}/maintenance-entries", crate::config::API_BASE, vid),
                &token, &body,
            ).await {
                Ok(created) => {
                    if !files_v.is_empty() {
                        if let Some(entry_id) = created["id"].as_str().and_then(|s| Uuid::parse_str(s).ok()) {
                            if let Err(e) = upload_attachment_files(vid, entry_id, &files_v).await {
                                // L'entretien est déjà créé — on prévient sans annuler la création
                                set_error.set(format!("Entretien créé, mais échec de l'envoi des pièces jointes : {e}"));
                                on_saved.call(());
                                return;
                            }
                        }
                    }
                    on_saved.call(());
                    on_close.call(());
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_error.set(String::new());
        submit.dispatch(());
    };

    view! {
        <Modal title="Nouvel entretien" on_close=on_close>
            <form on:submit=on_submit class="space-y-4">
                <Field label="Types concernés (un ou plusieurs — ex: révision = vidange + filtres)">
                    <div class="space-y-3 border border-gray-200 rounded-lg p-3">
                        {(!types.is_empty()).then(|| {
                            let items = types.clone();
                            view! {
                                <div class="space-y-1">
                                    <p class="text-xs font-semibold text-gray-400 uppercase tracking-wide">"Vos types"</p>
                                    {items.into_iter().map(|t| {
                                        let value = format!("type:{}", t.id);
                                        let value_for_checked = value.clone();
                                        let value_for_toggle = value.clone();
                                        let label = t.label.clone();
                                        view! {
                                            <label class="flex items-center gap-2 text-sm text-gray-700 py-0.5 cursor-pointer">
                                                <input type="checkbox"
                                                    prop:checked=move || selected.get().contains(&value_for_checked)
                                                    on:change=move |_| toggle_selected(value_for_toggle.clone())
                                                    class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
                                                {label}
                                            </label>
                                        }
                                    }).collect_view()}
                                </div>
                            }
                        })}
                        {CATEGORY_ORDER.iter().map(|cat| {
                            let items: Vec<_> = available_generic.iter().filter(|(_, tpl)| tpl.category == *cat).collect();
                            if items.is_empty() {
                                return view! { <></> }.into_view();
                            }
                            view! {
                                <div class="space-y-1">
                                    <p class="text-xs font-semibold text-gray-400 uppercase tracking-wide">{*cat}</p>
                                    {items.into_iter().map(|(i, tpl)| {
                                        let value = format!("generic:{}", i);
                                        let value_for_checked = value.clone();
                                        let value_for_toggle = value.clone();
                                        let tag = if show_fuel_tag {
                                            match tpl.fuel_type {
                                                Some("thermique") => " · ⛽ thermique",
                                                Some("electrique") => " · 🔋 électrique",
                                                _ => "",
                                            }
                                        } else { "" };
                                        let label = format!("{}{}", tpl.label, tag);
                                        view! {
                                            <label class="flex items-center gap-2 text-sm text-gray-700 py-0.5 cursor-pointer">
                                                <input type="checkbox"
                                                    prop:checked=move || selected.get().contains(&value_for_checked)
                                                    on:change=move |_| toggle_selected(value_for_toggle.clone())
                                                    class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500" />
                                                {label}
                                            </label>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_view()
                        }).collect_view()}
                    </div>
                </Field>
                <Field label="Libellé (optionnel si un type est sélectionné ci-dessus)">
                    <input type="text" prop:required=label_required prop:value=custom_label
                        on:input=move |ev| set_custom_label.set(event_target_value(&ev))
                        placeholder="ex: Révision 30 000 km" class=input_class() />
                </Field>
                <div class="grid grid-cols-2 gap-3">
                    <Field label="Date">
                        <input type="date" required prop:value=performed_at
                            on:input=move |ev| set_performed_at.set(event_target_value(&ev)) class=input_class() />
                    </Field>
                    <Field label="Km au compteur">
                        <input type="number" min="0" required prop:value=km_at_service
                            on:input=move |ev| set_km_at_service.set(event_target_value(&ev))
                            placeholder="ex: 45000" class=input_class() />
                    </Field>
                </div>
                <div class="grid grid-cols-2 gap-3">
                    <Field label="Coût (€, optionnel)">
                        <input type="number" min="0" step="0.01" prop:value=cost
                            on:input=move |ev| set_cost.set(event_target_value(&ev))
                            placeholder="ex: 89.90" class=input_class() />
                    </Field>
                    <Field label="Garage (optionnel)">
                        <input type="text" prop:value=provider
                            on:input=move |ev| set_provider.set(event_target_value(&ev))
                            placeholder="ex: Norauto" class=input_class() />
                    </Field>
                </div>
                <Field label="Notes (optionnel)">
                    <textarea prop:value=notes
                        on:input=move |ev| set_notes.set(event_target_value(&ev))
                        rows="2" class=input_class() />
                </Field>
                <Field label="Facture (photo ou fichier, optionnel)">
                    <input type="file"
                        node_ref=file_input_ref
                        accept="image/jpeg,image/png,image/webp,application/pdf"
                        capture="environment"
                        multiple
                        on:change=on_files_change
                        class="hidden" />
                    <button type="button"
                        on:click=move |_| {
                            // Différé via set_timeout : appeler .click() de façon synchrone ici
                            // réentre dans le closure d'event delegation de Leptos (encore en
                            // cours d'exécution pour CE click) → panique wasm-bindgen
                            // "closure invoked recursively or after being dropped".
                            set_timeout(move || {
                                if let Some(input) = file_input_ref.get() { input.click(); }
                            }, std::time::Duration::ZERO);
                        }
                        class="text-sm px-4 py-2 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 font-medium transition duration-150"
                    >
                        "📎 Ajouter une photo ou un document"
                    </button>
                    <Show when=move || compressing.get() fallback=|| ()>
                        <p class="text-xs text-gray-400 animate-pulse">"Optimisation des photos..."</p>
                    </Show>
                    <Show when=move || !compressing.get() && !files.get().is_empty() fallback=|| ()>
                        <p class="text-xs text-gray-400">{move || format!("{} fichier(s) sélectionné(s)", files.get().len())}</p>
                    </Show>
                </Field>
                <ModalActions
                    pending=Signal::derive(move || submit.pending().get() || compressing.get())
                    on_cancel=Callback::new(move |_| on_close.call(()))
                    label_submit="Enregistrer"
                    error=error
                />
            </form>
        </Modal>
    }
}

// Redimensionne (max 1920px de long côté) et ré-encode en JPEG qualité 0.75 via un
// <canvas> hors-DOM — réduit typiquement une photo de smartphone de plusieurs Mo à
// quelques centaines de Ko avant l'upload. Ne touche pas aux fichiers déjà petits
// (< 300 Ko) ni ne dégrade si la compression n'apporte aucun gain (garde l'original).
const COMPRESS_MAX_DIMENSION: f64 = 1920.0;
const COMPRESS_QUALITY: f64 = 0.75;
const COMPRESS_MIN_SIZE_BYTES: f64 = 300.0 * 1024.0;

async fn compress_image(file: &web_sys::File) -> Result<web_sys::File, String> {
    if file.size() < COMPRESS_MIN_SIZE_BYTES {
        return Ok(file.clone());
    }

    let window = leptos::window();
    let obj_url = web_sys::Url::create_object_url_with_blob(file).map_err(|e| format!("{:?}", e))?;

    let img = web_sys::HtmlImageElement::new().map_err(|e| format!("{:?}", e))?;
    let img_for_events = img.clone();
    let load_promise = js_sys::Promise::new(&mut |resolve, reject| {
        let resolve = resolve.clone();
        let onload = Closure::once(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        img_for_events.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();

        let reject = reject.clone();
        let onerror = Closure::once(move || {
            let _ = reject.call0(&JsValue::NULL);
        });
        img_for_events.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();
    });
    img.set_src(&obj_url);
    let load_result = wasm_bindgen_futures::JsFuture::from(load_promise).await;
    web_sys::Url::revoke_object_url(&obj_url).ok();
    load_result.map_err(|_| "échec de lecture de l'image".to_string())?;

    let (w, h) = (img.natural_width() as f64, img.natural_height() as f64);
    if w <= 0.0 || h <= 0.0 {
        return Ok(file.clone());
    }
    let scale = (COMPRESS_MAX_DIMENSION / w.max(h)).min(1.0);
    let (target_w, target_h) = (w * scale, h * scale);

    let document = window.document().ok_or("pas de document")?;
    let canvas: web_sys::HtmlCanvasElement = document
        .create_element("canvas")
        .map_err(|e| format!("{:?}", e))?
        .dyn_into()
        .map_err(|_| "échec de création du canvas".to_string())?;
    canvas.set_width(target_w as u32);
    canvas.set_height(target_h as u32);

    let ctx: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("pas de contexte 2d")?
        .dyn_into()
        .map_err(|_| "échec de cast du contexte 2d".to_string())?;
    ctx.draw_image_with_html_image_element_and_dw_and_dh(&img, 0.0, 0.0, target_w, target_h)
        .map_err(|e| format!("{:?}", e))?;

    let blob_promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let resolve = resolve.clone();
        let callback = Closure::once(move |blob: Option<web_sys::Blob>| {
            let _ = resolve.call1(&JsValue::NULL, &blob.into());
        });
        let _ = canvas.to_blob_with_type_and_encoder_options(
            callback.as_ref().unchecked_ref(),
            "image/jpeg",
            &JsValue::from_f64(COMPRESS_QUALITY),
        );
        callback.forget();
    });
    let blob_value = wasm_bindgen_futures::JsFuture::from(blob_promise)
        .await
        .map_err(|e| format!("{:?}", e))?;
    let blob: web_sys::Blob = blob_value.dyn_into().map_err(|_| "échec de cast du blob".to_string())?;

    if blob.size() >= file.size() {
        return Ok(file.clone());
    }

    let original_name = file.name();
    let new_name = match original_name.rsplit_once('.') {
        Some((stem, _ext)) => format!("{stem}.jpg"),
        None => format!("{original_name}.jpg"),
    };
    let parts = js_sys::Array::new();
    parts.push(&blob);
    let mut opts = web_sys::FilePropertyBag::new();
    opts.type_("image/jpeg");
    web_sys::File::new_with_blob_sequence_and_options(&parts, &new_name, &opts)
        .map_err(|e| format!("{:?}", e))
}

// Upload multipart des pièces jointes vers une entrée déjà créée.
async fn upload_attachment_files(vehicle_id: Uuid, entry_id: Uuid, files: &[web_sys::File]) -> Result<(), String> {
    let token = get_token().unwrap_or_default();
    let url = format!("{}/api/vehicles/{}/maintenance-entries/{}/attachments", crate::config::API_BASE, vehicle_id, entry_id);

    let form = web_sys::FormData::new().map_err(|e| format!("{:?}", e))?;
    for (i, file) in files.iter().enumerate() {
        form.append_with_blob(&format!("file{i}"), file).map_err(|e| format!("{:?}", e))?;
    }

    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    opts.body(Some(form.as_ref()));
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers.set("Authorization", &format!("Bearer {}", token)).ok();
    opts.headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&request))
        .await
        .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Erreur HTTP {}", resp.status()))
    }
}

// Ouvre le fichier dans un nouvel onglet — fetch authentifié (Authorization: Bearer)
// requis car un simple <a href> n'enverrait pas le token, contrairement à un window.open
// classique sur une URL publique.
fn open_attachment(vehicle_id: Uuid, attachment_id: Uuid) {
    spawn_local(async move {
        let Some(token) = get_token() else { return };
        let url = format!("{}/api/vehicles/{}/attachments/{}", crate::config::API_BASE, vehicle_id, attachment_id);

        let mut opts = web_sys::RequestInit::new();
        opts.method("GET");
        let Ok(headers) = web_sys::Headers::new() else { return };
        let _ = headers.set("Authorization", &format!("Bearer {}", token));
        opts.headers(&headers);

        let Ok(request) = web_sys::Request::new_with_str_and_init(&url, &opts) else { return };
        let Ok(resp_value) = wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&request)).await else { return };
        let Ok(resp) = resp_value.dyn_into::<web_sys::Response>() else { return };
        if !resp.ok() { return; }
        let Ok(blob_promise) = resp.blob() else { return };
        let Ok(blob_value) = wasm_bindgen_futures::JsFuture::from(blob_promise).await else { return };
        let Ok(blob) = blob_value.dyn_into::<web_sys::Blob>() else { return };
        if let Ok(obj_url) = web_sys::Url::create_object_url_with_blob(&blob) {
            let _ = leptos::window().open_with_url_and_target(&obj_url, "_blank");
        }
    });
}

#[component]
fn AttachmentsModal(
    vehicle_id: ReadSignal<Option<Uuid>>,
    entry_id: Uuid,
    entry_label: String,
    can_manage: bool,
    on_close: Callback<()>,
    on_changed: Callback<()>,
) -> impl IntoView {
    let (attachments, set_attachments) = create_signal(Vec::<MaintenanceAttachment>::new());
    let (loading, set_loading) = create_signal(true);

    let load = move || {
        let Some(vid) = vehicle_id.get_untracked() else { return };
        set_loading.set(true);
        spawn_local(async move {
            let Some(token) = get_token() else { return };
            let list = api_get::<Vec<MaintenanceAttachment>>(
                &format!("{}/api/vehicles/{}/maintenance-entries/{}/attachments", crate::config::API_BASE, vid, entry_id),
                &token,
            ).await.unwrap_or_default();
            set_attachments.set(list);
            set_loading.set(false);
        });
    };

    create_effect(move |_| load());

    let delete_attachment = move |attachment_id: Uuid| {
        let Some(vid) = vehicle_id.get_untracked() else { return };
        spawn_local(async move {
            let Some(token) = get_token() else { return };
            let url = format!("{}/api/vehicles/{}/attachments/{}", crate::config::API_BASE, vid, attachment_id);
            if api_delete(&url, &token).await.is_ok() {
                load();
                on_changed.call(());
            }
        });
    };

    view! {
        <Modal title="Pièces jointes" on_close=on_close>
            <p class="text-sm text-gray-500 -mt-2">{entry_label}</p>
            <Show when=move || loading.get() fallback=|| ()>
                <p class="text-sm text-gray-400 animate-pulse">"Chargement..."</p>
            </Show>
            <Show when=move || !loading.get() fallback=|| ()>
                <div class="space-y-2">
                    {move || attachments.get().into_iter().map(|a| {
                        let att_id = a.id;
                        let vid_for_open = vehicle_id;
                        let size_kb = a.size_bytes / 1024;
                        view! {
                            <div class="flex items-center justify-between gap-2 bg-gray-50 rounded-lg px-3 py-2">
                                <button
                                    on:click=move |_| {
                                        if let Some(vid) = vid_for_open.get_untracked() {
                                            open_attachment(vid, att_id);
                                        }
                                    }
                                    class="text-sm text-indigo-600 hover:underline text-left truncate"
                                >
                                    "📎 "{a.original_filename.clone()}
                                </button>
                                <div class="flex items-center gap-2 flex-shrink-0">
                                    <span class="text-xs text-gray-400">{size_kb}" Ko"</span>
                                    <Show when=move || can_manage fallback=|| ()>
                                        <button
                                            on:click=move |_| delete_attachment(att_id)
                                            class="text-xs text-gray-400 hover:text-red-600 transition duration-150"
                                        >
                                            "Supprimer"
                                        </button>
                                    </Show>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </Show>
            <button
                type="button"
                on:click=move |_| on_close.call(())
                class="w-full py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150"
            >
                "Fermer"
            </button>
        </Modal>
    }
}

#[component]
fn Modal(title: &'static str, on_close: Callback<()>, children: Children) -> impl IntoView {
    view! {
        <button type="button" class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm w-full cursor-default" on:click=move |_| on_close.call(()) />
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
            <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md p-8 space-y-6 max-h-[90vh] overflow-y-auto">
                <div class="flex items-center justify-between">
                    <h2 class="text-xl font-bold text-gray-900">{title}</h2>
                    <button on:click=move |_| on_close.call(()) class="text-gray-400 hover:text-gray-600 text-xl font-light">"✕"</button>
                </div>
                {children()}
            </div>
        </div>
    }
}

#[component]
fn Field(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="space-y-1">
            <label class="text-sm font-medium text-gray-700 block">{label}</label>
            {children()}
        </div>
    }
}

#[component]
fn ModalActions(
    pending: Signal<bool>,
    on_cancel: Callback<()>,
    label_submit: &'static str,
    error: ReadSignal<String>,
) -> impl IntoView {
    view! {
        <Show when=move || !error.get().is_empty() fallback=|| ()>
            <p class="text-sm text-center text-red-600">{move || error.get()}</p>
        </Show>
        <div class="flex gap-3 pt-2">
            <button type="button" on:click=move |_| on_cancel.call(()) class="flex-1 py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150">"Annuler"</button>
            <button type="submit" prop:disabled=move || pending.get() class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150">
                {move || if pending.get() { "Envoi..." } else { label_submit }}
            </button>
        </div>
    }
}
