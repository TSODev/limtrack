// src/components/trips/trip_list.rs
use crate::api_client::{api_delete, api_get, api_patch, api_post};
use crate::components::ui::{format_date_fr, get_token, input_class};
use common::PlannedTrip;
use leptos::*;
use uuid::Uuid;

const WEEKDAY_LABELS: [&str; 7] = ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"];

fn recurrence_summary(trip: &PlannedTrip) -> String {
    match trip.recurrence.as_str() {
        "daily" => {
            if trip.recurrence_interval <= 1 {
                "Tous les jours".to_string()
            } else {
                format!("Tous les {} jours", trip.recurrence_interval)
            }
        }
        "weekly" => {
            let days = trip.days_of_week.clone().unwrap_or_default();
            let list = days
                .iter()
                .filter_map(|&d| WEEKDAY_LABELS.get(d as usize))
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            if trip.recurrence_interval <= 1 {
                format!("Chaque semaine ({})", list)
            } else {
                format!("Toutes les {} semaines ({})", trip.recurrence_interval, list)
            }
        }
        "monthly" => {
            let dom = trip.day_of_month.unwrap_or(1);
            if trip.recurrence_interval <= 1 {
                format!("Chaque mois (jour {})", dom)
            } else {
                format!("Tous les {} mois (jour {})", trip.recurrence_interval, dom)
            }
        }
        _ => "Ponctuel".to_string(),
    }
}

#[component]
pub fn TripList(vehicle_id: ReadSignal<Option<Uuid>>, can_manage_trips: Memo<bool>) -> impl IntoView {
    let (trips, set_trips) = create_signal(Vec::<PlannedTrip>::new());
    let (loading, set_loading) = create_signal(false);
    let (show_modal, set_show_modal) = create_signal(false);
    let (editing, set_editing) = create_signal(Option::<PlannedTrip>::None);

    let load_trips = move |id: Uuid| {
        set_loading.set(true);
        spawn_local(async move {
            let Some(token) = get_token() else { return };
            let result = api_get::<Vec<PlannedTrip>>(
                &format!("{}/api/vehicles/{}/trips", crate::config::API_BASE, id),
                &token,
            )
            .await
            .unwrap_or_default();
            set_trips.set(result);
            set_loading.set(false);
        });
    };

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_trips.set(Vec::new());
            load_trips(id);
        }
    });

    let on_saved = move || {
        if let Some(id) = vehicle_id.get() {
            load_trips(id);
        }
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-lg font-bold text-gray-900">"Voyages planifiés"</h2>
                    <p class="text-xs text-gray-400 mt-0.5">
                        "Anticipez vos trajets à venir pour projeter votre usage kilométrique futur."
                    </p>
                </div>
                <Show when=move || can_manage_trips.get() fallback=|| ()>
                    <button
                        on:click=move |_| { set_editing.set(None); set_show_modal.set(true); }
                        class="text-sm px-4 py-2 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 font-medium transition duration-150 whitespace-nowrap"
                    >
                        "+ Voyage"
                    </button>
                </Show>
            </div>

            <Show when=move || loading.get() fallback=|| ()>
                <p class="text-sm text-gray-400 animate-pulse">"Chargement..."</p>
            </Show>

            <Show when=move || !loading.get() && trips.get().is_empty() fallback=|| ()>
                <div class="bg-white rounded-xl border border-dashed border-gray-200 p-8 text-center space-y-2">
                    <p class="text-sm font-medium text-gray-600">"Aucun voyage planifié."</p>
                    <p class="text-xs text-gray-400 max-w-sm mx-auto">
                        "Ajoutez un trajet ponctuel ou récurrent pour que l'app anticipe votre usage "
                        "kilométrique futur par rapport à vos plafonds LOA/assurance."
                    </p>
                </div>
            </Show>

            <Show when=move || !trips.get().is_empty() fallback=|| ()>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {move || trips.get().into_iter().map(|trip| {
                        let can_manage = can_manage_trips.get();
                        let on_deleted = Callback::new(move |_: ()| on_saved());
                        let trip_for_edit = trip.clone();
                        let on_edit = Callback::new(move |_: ()| {
                            set_editing.set(Some(trip_for_edit.clone()));
                            set_show_modal.set(true);
                        });
                        view! { <TripCard trip=trip can_manage=can_manage on_deleted=on_deleted on_edit=on_edit /> }
                    }).collect_view()}
                </div>
            </Show>
        </div>

        <Show when=move || show_modal.get() fallback=|| ()>
            <TripModal
                vehicle_id=vehicle_id
                existing=editing.get()
                on_close=Callback::new(move |_| set_show_modal.set(false))
                on_saved=Callback::new(move |_| on_saved())
            />
        </Show>
    }
}

#[component]
fn TripCard(
    trip: PlannedTrip,
    can_manage: bool,
    on_deleted: Callback<()>,
    on_edit: Callback<()>,
) -> impl IntoView {
    let (show_confirm_delete, set_show_confirm_delete) = create_signal(false);
    let trip_id = trip.id;
    let vehicle_id = trip.vehicle_id;
    let delete_action = create_action(move |_: &()| async move {
        let token = get_token().unwrap_or_default();
        let url = format!(
            "{}/api/vehicles/{}/trips/{}",
            crate::config::API_BASE,
            vehicle_id,
            trip_id
        );
        if api_delete(&url, &token).await.is_ok() {
            on_deleted.call(());
        }
        set_show_confirm_delete.set(false);
    });

    let summary = recurrence_summary(&trip);
    let label = trip.label.clone();

    view! {
        <div class=format!(
            "bg-white rounded-xl border border-gray-100 p-5 space-y-3 shadow-sm {}",
            if trip.active { "" } else { "opacity-60" }
        )>
            <div class="flex items-center justify-between">
                <span class="text-sm font-bold text-gray-800">{label.clone()}</span>
                <span class="text-xs font-medium px-2.5 py-1 rounded-full bg-indigo-50 text-indigo-600">
                    {summary}
                </span>
            </div>
            <div class="flex items-center justify-between text-xs text-gray-400">
                <span>"Du "{format_date_fr(trip.start_date)}" au "{format_date_fr(trip.end_date)}</span>
                <span class="font-semibold text-gray-600">{trip.estimated_km}" km"</span>
            </div>
            {(!trip.active).then(|| view! {
                <span class="text-xs font-medium px-2 py-0.5 rounded bg-gray-100 text-gray-500">"Inactif"</span>
            })}
            <Show when=move || can_manage fallback=|| ()>
                <div class="flex items-center justify-end gap-1.5 pt-1 border-t border-gray-50">
                    <button
                        on:click=move |_| on_edit.call(())
                        class="text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                    >
                        "Modifier"
                    </button>
                    <button
                        on:click=move |_| set_show_confirm_delete.set(true)
                        class="text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-red-50 hover:text-red-600 transition duration-150"
                    >
                        "Supprimer"
                    </button>
                </div>
            </Show>
        </div>

        <Show when=move || show_confirm_delete.get() fallback=|| ()>
            <ConfirmDeleteTripModal
                label=label.clone()
                on_cancel=Callback::new(move |_| set_show_confirm_delete.set(false))
                on_confirm=Callback::new(move |_| delete_action.dispatch(()))
                pending=delete_action.pending()
            />
        </Show>
    }
}

#[component]
fn ConfirmDeleteTripModal(
    label: String,
    on_cancel: Callback<()>,
    on_confirm: Callback<()>,
    pending: ReadSignal<bool>,
) -> impl IntoView {
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
                        prop:disabled=move || pending.get()
                        class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-red-600 hover:bg-red-700 disabled:opacity-50 transition duration-150">
                        {move || if pending.get() { "Suppression..." } else { "Supprimer" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TripModal(
    vehicle_id: ReadSignal<Option<Uuid>>,
    existing: Option<PlannedTrip>,
    on_close: Callback<()>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let is_edit = existing.is_some();
    let trip_id = existing.as_ref().map(|t| t.id);

    let (label, set_label) = create_signal(existing.as_ref().map(|t| t.label.clone()).unwrap_or_default());
    let (estimated_km, set_estimated_km) =
        create_signal(existing.as_ref().map(|t| t.estimated_km.to_string()).unwrap_or_default());
    let (start_date, set_start_date) =
        create_signal(existing.as_ref().map(|t| t.start_date.to_string()).unwrap_or_default());
    let (end_date, set_end_date) =
        create_signal(existing.as_ref().map(|t| t.end_date.to_string()).unwrap_or_default());
    let (recurrence, set_recurrence) =
        create_signal(existing.as_ref().map(|t| t.recurrence.clone()).unwrap_or_else(|| "none".to_string()));
    let (recurrence_interval, set_recurrence_interval) = create_signal(
        existing.as_ref().map(|t| t.recurrence_interval.to_string()).unwrap_or_else(|| "1".to_string()),
    );
    let (days_of_week, set_days_of_week) =
        create_signal(existing.as_ref().and_then(|t| t.days_of_week.clone()).unwrap_or_default());
    let (day_of_month, set_day_of_month) = create_signal(
        existing
            .as_ref()
            .and_then(|t| t.day_of_month)
            .map(|d| d.to_string())
            .unwrap_or_default(),
    );
    let (recurrence_end_date, set_recurrence_end_date) = create_signal(
        existing
            .as_ref()
            .and_then(|t| t.recurrence_end_date)
            .map(|d| d.to_string())
            .unwrap_or_default(),
    );
    let (active, set_active) = create_signal(existing.as_ref().map(|t| t.active).unwrap_or(true));
    let (error, set_error) = create_signal(String::new());

    let toggle_day = move |d: i16| {
        set_days_of_week.update(|days| {
            if let Some(pos) = days.iter().position(|&x| x == d) {
                days.remove(pos);
            } else {
                days.push(d);
            }
        });
    };

    let submit = create_action(move |_: &()| {
        let vid = vehicle_id.get();
        let label_v = label.get();
        let km_v = estimated_km.get().parse::<i32>().unwrap_or(0);
        let sd = start_date.get();
        let ed = end_date.get();
        let rec = recurrence.get();
        let interval = recurrence_interval.get().parse::<i32>().unwrap_or(1);
        let dow: Vec<i32> = days_of_week.get().into_iter().map(|d| d as i32).collect();
        let dom = day_of_month.get().parse::<i32>().ok();
        let rec_end = recurrence_end_date.get();
        let is_active = active.get();
        let trip_id = trip_id;

        async move {
            let Some(vid) = vid else { return };
            let token = get_token().unwrap_or_default();

            // "active" est ignoré silencieusement par le backend lors d'une création
            // (champ absent de CreateTripPayload) — inclus systématiquement pour simplifier.
            let body = serde_json::json!({
                "label": label_v,
                "estimated_km": km_v,
                "start_date": sd,
                "end_date": ed,
                "recurrence": rec,
                "recurrence_interval": interval,
                "days_of_week": if rec == "weekly" { serde_json::json!(dow) } else { serde_json::Value::Null },
                "day_of_month": if rec == "monthly" { dom } else { None },
                "recurrence_end_date": if rec_end.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(rec_end) },
                "active": is_active,
            });

            let result = if let Some(tid) = trip_id {
                api_patch(
                    &format!("{}/api/vehicles/{}/trips/{}", crate::config::API_BASE, vid, tid),
                    &token,
                    &body,
                )
                .await
            } else {
                api_post(
                    &format!("{}/api/vehicles/{}/trips", crate::config::API_BASE, vid),
                    &token,
                    &body,
                )
                .await
            };

            match result {
                Ok(_) => {
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
        <Modal title=if is_edit { "Modifier le voyage" } else { "Nouveau voyage planifié" } on_close=on_close>
            <form on:submit=on_submit class="space-y-4">
                <Field label="Nom du voyage">
                    <input type="text" required prop:value=label
                        on:input=move |ev| set_label.set(event_target_value(&ev))
                        placeholder="ex: Vacances d'été" class=input_class() />
                </Field>
                <Field label="Kilométrage estimé">
                    <input type="number" min="1" required prop:value=estimated_km
                        on:input=move |ev| set_estimated_km.set(event_target_value(&ev))
                        placeholder="ex: 800" class=input_class() />
                </Field>
                <div class=move || if recurrence.get() == "none" { "grid grid-cols-2 gap-3" } else { "grid grid-cols-1 gap-3" }>
                    <Field label="Date de début">
                        <input type="date" required prop:value=start_date
                            on:input=move |ev| set_start_date.set(event_target_value(&ev)) class=input_class() />
                    </Field>
                    <Show when=move || recurrence.get() == "none" fallback=|| ()>
                        <Field label="Date de fin">
                            <input type="date" required prop:value=end_date
                                on:input=move |ev| set_end_date.set(event_target_value(&ev)) class=input_class() />
                        </Field>
                    </Show>
                </div>
                <Show when=move || recurrence.get() != "none" fallback=|| ()>
                    <p class="text-xs text-gray-400 -mt-2">
                        "Chaque occurrence dure 1 jour — utilisez \"Fin de la récurrence\" ci-dessous pour arrêter la répétition."
                    </p>
                </Show>
                <Field label="Récurrence">
                    <select
                        prop:value=recurrence
                        on:change=move |ev| set_recurrence.set(event_target_value(&ev))
                        class=input_class()
                    >
                        <option value="none">"Ponctuel"</option>
                        <option value="daily">"Quotidien"</option>
                        <option value="weekly">"Hebdomadaire"</option>
                        <option value="monthly">"Mensuel"</option>
                    </select>
                </Field>

                <Show when=move || recurrence.get() != "none" fallback=|| ()>
                    <Field label="Tous les combien ?">
                        <input type="number" min="1" prop:value=recurrence_interval
                            on:input=move |ev| set_recurrence_interval.set(event_target_value(&ev))
                            class=input_class() />
                    </Field>
                </Show>

                <Show when=move || recurrence.get() == "weekly" fallback=|| ()>
                    <Field label="Jours de la semaine">
                        <div class="flex gap-1.5 flex-wrap">
                            {(0..7i16).map(|d| {
                                let is_checked = move || days_of_week.get().contains(&d);
                                view! {
                                    <button type="button"
                                        on:click=move |_| toggle_day(d)
                                        class=move || format!(
                                            "text-xs px-2.5 py-1.5 rounded-md border transition duration-150 {}",
                                            if is_checked() { "bg-indigo-600 text-white border-indigo-600" } else { "border-gray-200 text-gray-600 hover:bg-gray-50" }
                                        )
                                    >
                                        {WEEKDAY_LABELS[d as usize]}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </Field>
                </Show>

                <Show when=move || recurrence.get() == "monthly" fallback=|| ()>
                    <Field label="Jour du mois">
                        <input type="number" min="1" max="31" prop:value=day_of_month
                            on:input=move |ev| set_day_of_month.set(event_target_value(&ev))
                            placeholder="ex: 15" class=input_class() />
                    </Field>
                </Show>

                <Show when=move || recurrence.get() != "none" fallback=|| ()>
                    <Field label="Fin de la récurrence (optionnel)">
                        <input type="date" prop:value=recurrence_end_date
                            on:input=move |ev| set_recurrence_end_date.set(event_target_value(&ev))
                            class=input_class() />
                    </Field>
                </Show>

                {is_edit.then(|| view! {
                    <label class="flex items-center gap-3 cursor-pointer select-none py-1">
                        <input type="checkbox" class="sr-only peer"
                            prop:checked=move || active.get()
                            on:change=move |ev| set_active.set(event_target_checked(&ev)) />
                        <div class="relative w-9 h-5 bg-gray-200 rounded-full peer-checked:bg-indigo-500 transition-colors after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:w-4 after:h-4 after:bg-white after:rounded-full after:transition-all peer-checked:after:translate-x-4" />
                        <span class="text-sm text-gray-700">"Voyage actif"</span>
                    </label>
                })}

                <ModalActions
                    pending=submit.pending()
                    on_cancel=Callback::new(move |_| on_close.call(()))
                    label_submit=if is_edit { "Enregistrer" } else { "Créer le voyage" }
                    error=error
                />
            </form>
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
    pending: ReadSignal<bool>,
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
