// src/components/contracts/contract_list.rs
use crate::components::ui::{format_km, get_token, input_class};
use common::{ContractInsurance, ContractLoa, MileageLog};
use leptos::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use js_sys;

#[derive(Clone)]
struct ContractsData {
    loa: Vec<ContractLoa>,
    insurance: Vec<ContractInsurance>,
}

#[component]
pub fn ContractList(
    vehicle_id: ReadSignal<Option<Uuid>>,
    can_manage_contracts: Memo<bool>,
) -> impl IntoView {
    let (data, set_data) = create_signal(Option::<ContractsData>::None);
    let (loading, set_loading) = create_signal(false);
    let (show_loa_modal, set_show_loa_modal) = create_signal(false);
    let (show_insurance_modal, set_show_insurance_modal) = create_signal(false);

    let load_contracts = move |id: Uuid| {
        set_loading.set(true);
        spawn_local(async move {
            let Some(token) = get_token() else { return };
            let loa = fetch_json::<Vec<ContractLoa>>(
                &format!("{}/api/vehicles/{}/contracts/loa", crate::config::API_BASE, id),
                &token,
            )
            .await
            .unwrap_or_default();
            let insurance = fetch_json::<Vec<ContractInsurance>>(
                &format!("{}/api/vehicles/{}/contracts/insurance", crate::config::API_BASE, id),
                &token,
            )
            .await
            .unwrap_or_default();
            set_data.set(Some(ContractsData { loa, insurance }));
            set_loading.set(false);
        });
    };

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_data.set(None);
            load_contracts(id);
        }
    });

    let on_created = move || {
        if let Some(id) = vehicle_id.get() {
            load_contracts(id);
        }
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex items-center justify-between">
                <h2 class="text-lg font-bold text-gray-900">"Contrats"</h2>
                // Boutons visibles uniquement pour owner
                <Show when=move || can_manage_contracts.get() fallback=|| ()>
                    <div class="flex gap-2">
                        <button
                            on:click=move |_| set_show_loa_modal.set(true)
                            class="text-sm px-4 py-2 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 font-medium transition duration-150"
                        >
                            "+ LOA"
                        </button>
                        <button
                            on:click=move |_| set_show_insurance_modal.set(true)
                            class="text-sm px-4 py-2 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 font-medium transition duration-150"
                        >
                            "+ Assurance"
                        </button>
                    </div>
                </Show>
            </div>

            <Show when=move || loading.get() fallback=|| ()>
                <p class="text-sm text-gray-400 animate-pulse">"Chargement..."</p>
            </Show>

            <Show when=move || !loading.get() && data.get().is_some() fallback=|| ()>
                {move || data.get().map(|d| {
                    let total = d.loa.len() + d.insurance.len();
                    let can_manage = can_manage_contracts.get();
                    if total == 0 {
                        return view! {
                            <div class="bg-white rounded-xl border border-dashed border-gray-200 p-12 text-center">
                                <p class="text-sm text-gray-400 italic">"Aucun contrat enregistré."</p>
                            </div>
                        }.into_view();
                    }
                    view! {
                        <div class="flex flex-col gap-4">
                            {if !d.loa.is_empty() { view! {
                                <div class="flex flex-col gap-3">
                                    <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-widest">"LOA"</h3>
                                    {d.loa.into_iter().map(|c| {
                                        let on_deleted = Callback::new(move |_| on_created());
                                        view! { <ContractLoaCard contract=c can_manage=can_manage on_deleted=on_deleted /> }
                                    }).collect_view()}
                                </div>
                            }.into_view() } else { view! { <div /> }.into_view() }}
                            {if !d.insurance.is_empty() { view! {
                                <div class="flex flex-col gap-3">
                                    <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-widest">"Assurance"</h3>
                                    {d.insurance.into_iter().map(|c| {
                                        let on_deleted  = Callback::new(move |_| on_created());
                                        let on_updated  = Callback::new(move |_| on_created());
                                        view! { <ContractInsuranceCard contract=c can_manage=can_manage on_deleted=on_deleted on_updated=on_updated /> }
                                    }).collect_view()}
                                </div>
                            }.into_view() } else { view! { <div /> }.into_view() }}
                        </div>
                    }.into_view()
                })}
            </Show>
        </div>

        <Show when=move || show_loa_modal.get() fallback=|| ()>
            <LoaModal
                vehicle_id=vehicle_id
                on_close=Callback::new(move |_| set_show_loa_modal.set(false))
                on_created=Callback::new(move |_| on_created())
            />
        </Show>

        <Show when=move || show_insurance_modal.get() fallback=|| ()>
            <InsuranceModal
                vehicle_id=vehicle_id
                on_close=Callback::new(move |_| set_show_insurance_modal.set(false))
                on_created=Callback::new(move |_| on_created())
            />
        </Show>
    }
}

#[component]
fn ContractLoaCard(contract: ContractLoa, can_manage: bool, on_deleted: Callback<()>) -> impl IntoView {
    let (show_edit, set_show_edit) = create_signal(false);
    let (show_confirm_delete, set_show_confirm_delete) = create_signal(false);
    let contract_id = contract.id;
    let vehicle_id = contract.vehicle_id;
    let delete_action = create_action(move |_: &()| async move {
        let token = get_token().unwrap_or_default();
        let url = format!(
            "{}/api/vehicles/{}/contracts/loa/{}",
            crate::config::API_BASE, vehicle_id, contract_id
        );
        let mut opts = web_sys::RequestInit::new();
        opts.method("DELETE");
        let headers = web_sys::Headers::new().unwrap();
        headers.set("Authorization", &format!("Bearer {}", token)).ok();
        opts.headers(&headers);
        let req = web_sys::Request::new_with_str_and_init(&url, &opts).unwrap();
        if let Ok(resp_val) = wasm_bindgen_futures::JsFuture::from(
            leptos::window().fetch_with_request(&req)
        ).await {
            let resp: web_sys::Response = resp_val.dyn_into().unwrap();
            if resp.ok() || resp.status() == 204 {
                on_deleted.call(());
            }
        }
        set_show_confirm_delete.set(false);
    });
    let pct =
        ((contract.km_consumed as f64 / contract.km_allowed as f64) * 100.0).min(100.0) as u32;
    let (bar_color, badge_color, badge_label) = match contract.status.as_str() {
        "exceeded" => ("bg-red-500", "bg-red-100 text-red-700", "Dépassé"),
        "closed" => ("bg-gray-400", "bg-gray-100 text-gray-600", "Clôturé"),
        _ => {
            if contract.overage_risk {
                (
                    "bg-amber-400",
                    "bg-amber-100 text-amber-700",
                    "Risque dépassement",
                )
            } else {
                ("bg-indigo-500", "bg-green-100 text-green-700", "Actif")
            }
        }
    };

    view! {
        <div class="bg-white rounded-xl border border-gray-100 p-5 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
                <span class="text-sm font-bold text-gray-800">"Contrat LOA"</span>
                <span class=format!("text-xs font-medium px-2.5 py-1 rounded-full {}", badge_color)>{badge_label}</span>
            </div>
            <div>
                <div class="flex justify-between text-xs text-gray-400 mb-1.5">
                    <span>{format_km(contract.km_consumed)}" consommés"</span>
                    <span>{format_km(contract.km_allowed)}" autorisés"</span>
                </div>
                <div class="w-full bg-gray-100 rounded-full h-2.5">
                    <div class=format!("h-2.5 rounded-full transition-all {}", bar_color) style=format!("width: {}%", pct) />
                </div>
                <p class="text-right text-xs text-gray-400 mt-1">{pct}"% utilisé"</p>
            </div>
            <div class="grid grid-cols-3 gap-3">
                <div class="bg-gray-50 rounded-lg p-3 text-center">
                    <p class="text-xs text-gray-400 mb-1">"Restant"</p>
                    <p class="text-sm font-bold text-gray-800">{format_km(contract.km_remaining)}</p>
                </div>
                <div class=format!("rounded-lg p-3 text-center {}", if contract.overage_risk { "bg-amber-50" } else { "bg-gray-50" })>
                    <p class="text-xs text-gray-400 mb-1">"Projection"</p>
                    <p class=format!("text-sm font-bold {}", if contract.overage_risk { "text-amber-600" } else { "text-gray-800" })>
                        {format_km(contract.forecast_km)}
                    </p>
                </div>
                <div class="bg-gray-50 rounded-lg p-3 text-center">
                    <p class="text-xs text-gray-400 mb-1">"Échéance"</p>
                    <p class="text-sm font-bold text-gray-800">{contract.days_remaining}" j"</p>
                </div>
            </div>
            {contract.estimated_limit_date.map(|d| view! {
                <p class=format!("text-xs {}", if contract.overage_risk { "text-amber-600" } else { "text-gray-400" })>
                    "📅 Limite estimée : "{d.to_string()}
                </p>
            })}
            {contract.price_per_extra_km.and_then(|price| {
                let extra_km = if contract.km_consumed > contract.km_allowed {
                    contract.km_consumed - contract.km_allowed
                } else if contract.forecast_km > contract.km_allowed {
                    contract.forecast_km - contract.km_allowed
                } else {
                    return None;
                };
                let cost = extra_km as f64 * price;
                let (label, cls) = if contract.km_consumed > contract.km_allowed {
                    (format!("💶 Coût dépassement : {:.2} €", cost), "text-red-600")
                } else {
                    (format!("💶 Coût projeté : {:.2} €", cost), "text-amber-600")
                };
                Some(view! { <p class=format!("text-xs font-semibold {}", cls)>{label}</p> })
            })}
            <div class="flex items-center justify-between pt-1 border-t border-gray-50">
                <div class="flex text-xs text-gray-400 gap-2">
                    <span>"Du "{contract.start_date.to_string()}</span>
                    <span>"→"</span>
                    <span>{contract.end_date.to_string()}</span>
                </div>
                <div class="flex gap-1.5">
                    // Bouton édition prix/km (owner uniquement)
                    <Show when=move || can_manage fallback=|| ()>
                        <button
                            on:click=move |_| set_show_edit.set(true)
                            title="Modifier le prix/km"
                            class="flex items-center gap-1 text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                        >
                            <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125" />
                            </svg>
                            "€/km"
                        </button>
                    </Show>
                    {
                        let c = contract.clone();
                        view! {
                            <button
                                on:click=move |_| export_loa_pdf(&c)
                                title="Exporter en PDF"
                                class="flex items-center gap-1 text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                            >
                                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                                </svg>
                                "PDF"
                            </button>
                        }
                    }
                    {
                        let vid         = contract.vehicle_id;
                        let km_start    = contract.km_start;
                        let km_total    = contract.km_allowed;
                        let start_date  = contract.start_date;
                        let end_date    = contract.end_date;
                        let csv_action = create_action(move |_: &()| async move {
                            if let Some(token) = get_token() {
                                if let Ok(entries) = fetch_json::<Vec<MileageLog>>(
                                    &format!("{}/api/vehicles/{}/mileage", crate::config::API_BASE, vid),
                                    &token,
                                ).await {
                                    download_mileage_csv(&entries, km_start, km_total, start_date, end_date, &format!("releves-loa-{}.csv", vid));
                                }
                            }
                        });
                        view! {
                            <button
                                on:click=move |_| { csv_action.dispatch(()); }
                                title="Exporter les relevés CSV"
                                class="flex items-center gap-1 text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 hover:text-green-600 transition duration-150"
                            >
                                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3" />
                                </svg>
                                "CSV"
                            </button>
                        }
                    }
                    // Bouton suppression (owner uniquement)
                    <Show when=move || can_manage fallback=|| ()>
                        <button
                            on:click=move |_| set_show_confirm_delete.set(true)
                            title="Supprimer ce contrat"
                            class="flex items-center gap-1 text-xs px-2 py-1 rounded border border-red-100 text-red-400 hover:bg-red-50 hover:text-red-600 transition duration-150"
                        >
                            <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                            </svg>
                        </button>
                    </Show>
                </div>
            </div>
        </div>

        // Modal confirmation suppression LOA
        <Show when=move || show_confirm_delete.get() fallback=|| ()>
            <ConfirmDeleteModal
                label="ce contrat LOA"
                on_cancel=Callback::new(move |_| set_show_confirm_delete.set(false))
                on_confirm=Callback::new(move |_| { delete_action.dispatch(()); })
                pending=delete_action.pending()
            />
        </Show>

        // Modal édition prix/km
        <Show when=move || show_edit.get() fallback=|| ()>
            <EditLoaPriceModal
                contract_id=contract.id
                vehicle_id=contract.vehicle_id
                current_price=contract.price_per_extra_km
                on_close=Callback::new(move |_| set_show_edit.set(false))
            />
        </Show>
    }
}

#[component]
fn EditLoaPriceModal(
    contract_id: uuid::Uuid,
    vehicle_id: uuid::Uuid,
    current_price: Option<f64>,
    on_close: Callback<()>,
) -> impl IntoView {
    let initial = current_price.map(|p| format!("{:.2}", p)).unwrap_or_default();
    let (price, set_price) = create_signal(initial);
    let (error, set_error) = create_signal(String::new());

    let submit = create_action(move |price_str: &String| {
        let price_val = price_str.trim().replace(',', ".").parse::<f64>().ok();
        async move {
            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({"price_per_extra_km": price_val});
            let url = format!(
                "{}/api/vehicles/{}/contracts/loa/{}",
                crate::config::API_BASE, vehicle_id, contract_id
            );
            let mut opts = web_sys::RequestInit::new();
            opts.method("PATCH");
            let headers = web_sys::Headers::new().unwrap();
            headers.set("Authorization", &format!("Bearer {}", token)).ok();
            headers.set("Content-Type", "application/json").ok();
            opts.headers(&headers);
            opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));
            use wasm_bindgen::JsCast;
            let req = web_sys::Request::new_with_str_and_init(&url, &opts).unwrap();
            match wasm_bindgen_futures::JsFuture::from(
                leptos::window().fetch_with_request(&req)
            ).await {
                Ok(r) => {
                    let resp: web_sys::Response = r.dyn_into().unwrap();
                    if resp.ok() || resp.status() == 204 {
                        on_close.call(());
                        leptos::window().location().reload().ok();
                    } else {
                        set_error.set(format!("Erreur HTTP {}", resp.status()));
                    }
                }
                Err(_) => set_error.set("Erreur réseau".to_string()),
            }
        }
    });

    view! {
        <button type="button"
            class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm w-full cursor-default"
            on:click=move |_| on_close.call(()) />
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
            <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-sm p-8 space-y-6">
                <div class="flex items-center justify-between">
                    <h2 class="text-xl font-bold text-gray-900">"Prix/km dépassement"</h2>
                    <button on:click=move |_| on_close.call(())
                        class="text-gray-400 hover:text-gray-600 text-xl font-light">"✕"</button>
                </div>
                <p class="text-sm text-gray-500">
                    "Renseignez le prix par kilomètre supplémentaire prévu dans votre contrat LOA."
                </p>
                <div class="space-y-1">
                    <label class="text-sm font-medium text-gray-700 block">"Prix (€ / km)"</label>
                    <input
                        type="text"
                        inputmode="decimal"
                        value=move || price.get()
                        on:input=move |ev| set_price.set(event_target_value(&ev))
                        placeholder="ex: 0.08"
                        class=input_class()
                    />
                </div>
                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-red-600">{move || error.get()}</p>
                </Show>
                <div class="flex gap-3">
                    <button type="button" on:click=move |_| on_close.call(())
                        class="flex-1 py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150">
                        "Annuler"
                    </button>
                    <button
                        on:click=move |_| submit.dispatch(price.get())
                        prop:disabled=move || submit.pending().get()
                        class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 transition duration-150">
                        {move || if submit.pending().get() { "Enregistrement..." } else { "Enregistrer" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ContractInsuranceCard(contract: ContractInsurance, can_manage: bool, on_deleted: Callback<()>, on_updated: Callback<()>) -> impl IntoView {
    let (show_confirm_delete, set_show_confirm_delete) = create_signal(false);
    let contract_id = contract.id;
    let vehicle_id = contract.vehicle_id;

    let (auto_renew, set_auto_renew) = create_signal(contract.auto_renew);

    let on_updated_toggle = on_updated.clone();
    let toggle_action = create_action(move |(vid, cid, val): &(Uuid, Uuid, bool)| {
        let (vid, cid, val) = (*vid, *cid, *val);
        let on_upd = on_updated_toggle.clone();
        async move {
            let token = get_token().unwrap_or_default();
            let result = patch_json(
                &format!("{}/api/vehicles/{}/contracts/insurance/{}", crate::config::API_BASE, vid, cid),
                &token,
                &serde_json::json!({ "auto_renew": val }),
            ).await;
            if result.is_ok() { on_upd.call(()); }
            result
        }
    });

    let renew_action = create_action(move |(vid, cid): &(Uuid, Uuid)| {
        let (vid, cid) = (*vid, *cid);
        let on_upd = on_updated.clone();
        async move {
            let token = get_token().unwrap_or_default();
            let result = post_json(
                &format!("{}/api/vehicles/{}/contracts/insurance/{}/renew", crate::config::API_BASE, vid, cid),
                &token,
                &serde_json::json!({}),
            ).await;
            if result.is_ok() { on_upd.call(()); }
            result
        }
    });

    let is_toggle_pending = create_memo(move |_| toggle_action.pending().get());
    let is_renew_pending  = create_memo(move |_| renew_action.pending().get());
    let delete_action = create_action(move |_: &()| async move {
        let token = get_token().unwrap_or_default();
        let url = format!(
            "{}/api/vehicles/{}/contracts/insurance/{}",
            crate::config::API_BASE, vehicle_id, contract_id
        );
        let mut opts = web_sys::RequestInit::new();
        opts.method("DELETE");
        let headers = web_sys::Headers::new().unwrap();
        headers.set("Authorization", &format!("Bearer {}", token)).ok();
        opts.headers(&headers);
        let req = web_sys::Request::new_with_str_and_init(&url, &opts).unwrap();
        if let Ok(resp_val) = wasm_bindgen_futures::JsFuture::from(
            leptos::window().fetch_with_request(&req)
        ).await {
            let resp: web_sys::Response = resp_val.dyn_into().unwrap();
            if resp.ok() || resp.status() == 204 {
                on_deleted.call(());
            }
        }
        set_show_confirm_delete.set(false);
    });
    let contract_for_pdf = contract.clone();
    let pct =
        ((contract.km_consumed as f64 / contract.km_annual_limit as f64) * 100.0).min(100.0) as u32;
    let (bar_color, badge_color, badge_label) = match contract.status.as_str() {
        "exceeded" => ("bg-red-500", "bg-red-100 text-red-700", "Dépassé"),
        "closed" => ("bg-gray-400", "bg-gray-100 text-gray-600", "Clôturé"),
        _ => {
            if contract.overage_risk {
                (
                    "bg-amber-400",
                    "bg-amber-100 text-amber-700",
                    "Risque dépassement",
                )
            } else {
                ("bg-indigo-500", "bg-green-100 text-green-700", "Active")
            }
        }
    };

    view! {
        <div class="bg-white rounded-xl border border-gray-100 p-5 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-sm font-bold text-gray-800">"Assurance"</span>
                    {contract.insurer.map(|ins| view! {
                        <span class="text-sm text-gray-400">{ins}</span>
                    })}
                    {move || auto_renew.get().then(|| view! {
                        <span class="text-xs px-1.5 py-0.5 rounded-full bg-indigo-100 text-indigo-600 font-medium">"↻ Auto"</span>
                    })}
                </div>
                <span class=format!("text-xs font-medium px-2.5 py-1 rounded-full {}", badge_color)>{badge_label}</span>
            </div>
            <div>
                <div class="flex justify-between text-xs text-gray-400 mb-1.5">
                    <span>{format_km(contract.km_consumed)}" consommés"</span>
                    <span>{format_km(contract.km_annual_limit)}" / an"</span>
                </div>
                <div class="w-full bg-gray-100 rounded-full h-2.5">
                    <div class=format!("h-2.5 rounded-full transition-all {}", bar_color) style=format!("width: {}%", pct) />
                </div>
                <p class="text-right text-xs text-gray-400 mt-1">{pct}"% utilisé"</p>
            </div>
            <div class="grid grid-cols-3 gap-3">
                <div class="bg-gray-50 rounded-lg p-3 text-center">
                    <p class="text-xs text-gray-400 mb-1">"Restant"</p>
                    <p class="text-sm font-bold text-gray-800">{format_km(contract.km_remaining)}</p>
                </div>
                <div class=format!("rounded-lg p-3 text-center {}", if contract.overage_risk { "bg-amber-50" } else { "bg-gray-50" })>
                    <p class="text-xs text-gray-400 mb-1">"Projection"</p>
                    <p class=format!("text-sm font-bold {}", if contract.overage_risk { "text-amber-600" } else { "text-gray-800" })>
                        {format_km(contract.forecast_km)}
                    </p>
                </div>
                <div class="bg-gray-50 rounded-lg p-3 text-center">
                    <p class="text-xs text-gray-400 mb-1">"Échéance"</p>
                    <p class="text-sm font-bold text-gray-800">{contract.days_remaining}" j"</p>
                </div>
            </div>
            {contract.estimated_limit_date.map(|d| view! {
                <p class=format!("text-xs {}", if contract.overage_risk { "text-amber-600" } else { "text-gray-400" })>
                    "📅 Limite estimée : "{d.to_string()}
                </p>
            })}
            <div class="flex items-center justify-between pt-1 border-t border-gray-50">
                <div class="flex text-xs text-gray-400 gap-2">
                    <span>"Du "{contract.start_date.to_string()}</span>
                    <span>"→"</span>
                    <span>{contract.end_date.to_string()}</span>
                </div>
                <div class="flex gap-1.5">
                    {
                        let c = contract_for_pdf;
                        view! {
                            <button
                                on:click=move |_| export_insurance_pdf(&c)
                                title="Exporter en PDF"
                                class="flex items-center gap-1 text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                            >
                                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                                </svg>
                                "PDF"
                            </button>
                        }
                    }
                    {
                        let vid        = contract.vehicle_id;
                        let km_start   = contract.km_start;
                        let km_total   = contract.km_annual_limit;
                        let start_date = contract.start_date;
                        let end_date   = contract.end_date;
                        let csv_action = create_action(move |_: &()| async move {
                            if let Some(token) = get_token() {
                                if let Ok(entries) = fetch_json::<Vec<MileageLog>>(
                                    &format!("{}/api/vehicles/{}/mileage", crate::config::API_BASE, vid),
                                    &token,
                                ).await {
                                    download_mileage_csv(&entries, km_start, km_total, start_date, end_date, &format!("releves-assurance-{}.csv", vid));
                                }
                            }
                        });
                        view! {
                            <button
                                on:click=move |_| { csv_action.dispatch(()); }
                                title="Exporter les relevés CSV"
                                class="flex items-center gap-1 text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 hover:text-green-600 transition duration-150"
                            >
                                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3" />
                                </svg>
                                "CSV"
                            </button>
                        }
                    }
                    // Bouton suppression (owner uniquement)
                    <Show when=move || can_manage fallback=|| ()>
                        <button
                            on:click=move |_| set_show_confirm_delete.set(true)
                            title="Supprimer ce contrat"
                            class="flex items-center gap-1 text-xs px-2 py-1 rounded border border-red-100 text-red-400 hover:bg-red-50 hover:text-red-600 transition duration-150"
                        >
                            <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                            </svg>
                        </button>
                    </Show>
                </div>
            </div>
            // Section renouvellement automatique (owner/editor)
            {if can_manage {
                view! {
                    <div class="pt-3 border-t border-gray-100 space-y-2">
                        <label class="flex items-center gap-2 cursor-pointer select-none">
                            <input
                                type="checkbox"
                                class="sr-only peer"
                                prop:checked=move || auto_renew.get()
                                disabled=move || is_toggle_pending.get()
                                on:change=move |_| {
                                    let new_val = !auto_renew.get();
                                    set_auto_renew.set(new_val);
                                    toggle_action.dispatch((vehicle_id, contract_id, new_val));
                                }
                            />
                            <div class="relative w-9 h-5 bg-gray-200 rounded-full peer-checked:bg-indigo-500 transition-colors after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:w-4 after:h-4 after:bg-white after:rounded-full after:transition-all peer-checked:after:translate-x-4" />
                            <span class=move || format!(
                                "text-sm {}",
                                if is_toggle_pending.get() { "text-gray-400 animate-pulse" } else { "text-gray-600" }
                            )>
                                "Renouvellement automatique (J-7)"
                            </span>
                        </label>
                        <button
                            type="button"
                            disabled=move || is_renew_pending.get()
                            on:click=move |_| renew_action.dispatch((vehicle_id, contract_id))
                            class="w-full text-sm py-2 px-4 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                        >
                            {move || if is_renew_pending.get() { "Renouvellement en cours..." } else { "Renouveler maintenant →" }}
                        </button>
                        {move || {
                            renew_action.value().get()
                                .and_then(|r| r.err())
                                .map(|e| view! { <p class="text-sm text-center text-red-600">{e}</p> })
                        }}
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
        </div>

        // Modal confirmation suppression Assurance
        <Show when=move || show_confirm_delete.get() fallback=|| ()>
            <ConfirmDeleteModal
                label="ce contrat d'assurance"
                on_cancel=Callback::new(move |_| set_show_confirm_delete.set(false))
                on_confirm=Callback::new(move |_| { delete_action.dispatch(()); })
                pending=delete_action.pending()
            />
        </Show>
    }
}

// ─── Modals ──────────────────────────────────────────────────────

#[component]
fn ConfirmDeleteModal(
    label: &'static str,
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
fn LoaModal(
    vehicle_id: ReadSignal<Option<Uuid>>,
    on_close: Callback<()>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (km_allowed, set_km_allowed)     = create_signal(String::new());
    let (km_start, set_km_start)         = create_signal(String::new());
    let (start_date, set_start_date)     = create_signal(String::new());
    let (end_date, set_end_date)         = create_signal(String::new());
    let (price_per_km, set_price_per_km) = create_signal(String::new());
    let (error, set_error)               = create_signal(String::new());

    let submit = create_action(
        move |(vid, km_a, km_s, sd, ed, price): &(Uuid, String, String, String, String, String)| {
            let (vid, km_a, km_s, sd, ed, price) =
                (*vid, km_a.clone(), km_s.clone(), sd.clone(), ed.clone(), price.clone());
            async move {
                let token = get_token().unwrap_or_default();
                let price_val = price.trim().parse::<f64>().ok();
                let body = serde_json::json!({
                    "km_allowed": km_a.parse::<i32>().unwrap_or(0),
                    "km_start": km_s.parse::<i32>().unwrap_or(0),
                    "start_date": sd, "end_date": ed,
                    "price_per_extra_km": price_val,
                });
                match post_json(
                    &format!("{}/api/vehicles/{}/contracts/loa", crate::config::API_BASE, vid),
                    &token,
                    &body,
                ).await {
                    Ok(_) => { on_created.call(()); on_close.call(()); }
                    Err(e) => set_error.set(e),
                }
            }
        },
    );

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(id) = vehicle_id.get() else { return };
        set_error.set(String::new());
        submit.dispatch((id, km_allowed.get(), km_start.get(), start_date.get(), end_date.get(), price_per_km.get()));
    };

    view! {
        <Modal title="Nouveau contrat LOA" on_close=on_close>
            <form on:submit=on_submit class="space-y-4">
                <Field label="Kilométrage autorisé">
                    <input type="number" min="1" required prop:value=km_allowed on:input=move |ev| set_km_allowed.set(event_target_value(&ev)) placeholder="ex: 45000" class=input_class() />
                </Field>
                <Field label="Kilométrage au départ">
                    <input type="number" min="0" required prop:value=km_start on:input=move |ev| set_km_start.set(event_target_value(&ev)) placeholder="ex: 12000" class=input_class() />
                </Field>
                <div class="grid grid-cols-2 gap-3">
                    <Field label="Date de début">
                        <input type="date" required prop:value=start_date on:input=move |ev| set_start_date.set(event_target_value(&ev)) class=input_class() />
                    </Field>
                    <Field label="Date de fin">
                        <input type="date" required prop:value=end_date on:input=move |ev| set_end_date.set(event_target_value(&ev)) class=input_class() />
                    </Field>
                </div>
                <Field label="Prix/km dépassement (optionnel, €)">
                    <input type="number" min="0" step="0.01" prop:value=price_per_km
                        on:input=move |ev| set_price_per_km.set(event_target_value(&ev))
                        placeholder="ex: 0.08" class=input_class() />
                </Field>
                <ModalActions pending=submit.pending() on_cancel=Callback::new(move |_| on_close.call(())) label_submit="Créer le contrat" error=error />
            </form>
        </Modal>
    }
}

#[component]
fn InsuranceModal(
    vehicle_id: ReadSignal<Option<Uuid>>,
    on_close: Callback<()>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (km_limit, set_km_limit)     = create_signal(String::new());
    let (km_start, set_km_start)     = create_signal(String::new());
    let (start_date, set_start_date) = create_signal(String::new());
    let (end_date, set_end_date)     = create_signal(String::new());
    let (insurer, set_insurer)       = create_signal(String::new());
    let (auto_renew, set_auto_renew) = create_signal(false);
    let (error, set_error)           = create_signal(String::new());

    let submit = create_action(
        move |(vid, km_l, km_s, sd, ed, ins, ar): &(Uuid, String, String, String, String, String, bool)| {
            let (vid, km_l, km_s, sd, ed, ins, ar) = (
                *vid, km_l.clone(), km_s.clone(), sd.clone(), ed.clone(), ins.clone(), *ar,
            );
            async move {
                let token = get_token().unwrap_or_default();
                let body = serde_json::json!({
                    "km_annual_limit": km_l.parse::<i32>().unwrap_or(0),
                    "km_start": km_s.parse::<i32>().unwrap_or(0),
                    "start_date": sd, "end_date": ed,
                    "insurer": if ins.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(ins) },
                    "auto_renew": ar,
                });
                match post_json(
                    &format!("{}/api/vehicles/{}/contracts/insurance", crate::config::API_BASE, vid),
                    &token,
                    &body,
                ).await {
                    Ok(_) => { on_created.call(()); on_close.call(()); }
                    Err(e) => set_error.set(e),
                }
            }
        },
    );

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(id) = vehicle_id.get() else { return };
        set_error.set(String::new());
        submit.dispatch((id, km_limit.get(), km_start.get(), start_date.get(), end_date.get(), insurer.get(), auto_renew.get()));
    };

    view! {
        <Modal title="Nouveau contrat Assurance" on_close=on_close>
            <form on:submit=on_submit class="space-y-4">
                <Field label="Limite kilométrique annuelle">
                    <input type="number" min="1" required prop:value=km_limit on:input=move |ev| set_km_limit.set(event_target_value(&ev)) placeholder="ex: 15000" class=input_class() />
                </Field>
                <Field label="Kilométrage au départ">
                    <input type="number" min="0" required prop:value=km_start on:input=move |ev| set_km_start.set(event_target_value(&ev)) placeholder="ex: 12000" class=input_class() />
                </Field>
                <Field label="Assureur (optionnel)">
                    <input type="text" prop:value=insurer on:input=move |ev| set_insurer.set(event_target_value(&ev)) placeholder="ex: Maif" class=input_class() />
                </Field>
                <div class="grid grid-cols-2 gap-3">
                    <Field label="Date de début">
                        <input type="date" required prop:value=start_date on:input=move |ev| set_start_date.set(event_target_value(&ev)) class=input_class() />
                    </Field>
                    <Field label="Date de fin">
                        <input type="date" required prop:value=end_date on:input=move |ev| set_end_date.set(event_target_value(&ev)) class=input_class() />
                    </Field>
                </div>
                <label class="flex items-center gap-3 cursor-pointer select-none py-1">
                    <input type="checkbox" class="sr-only peer"
                        prop:checked=move || auto_renew.get()
                        on:change=move |ev| set_auto_renew.set(event_target_checked(&ev)) />
                    <div class="relative w-9 h-5 bg-gray-200 rounded-full peer-checked:bg-indigo-500 transition-colors after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:w-4 after:h-4 after:bg-white after:rounded-full after:transition-all peer-checked:after:translate-x-4" />
                    <span class="text-sm text-gray-700">"Renouvellement automatique (J-7)"</span>
                </label>
                <ModalActions pending=submit.pending() on_cancel=Callback::new(move |_| on_close.call(())) label_submit="Créer le contrat" error=error />
            </form>
        </Modal>
    }
}

#[component]
fn Modal(title: &'static str, on_close: Callback<()>, children: Children) -> impl IntoView {
    view! {
        <button type="button" class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm w-full cursor-default" on:click=move |_| on_close.call(()) />
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
            <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md p-8 space-y-6">
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

async fn fetch_json<T: for<'de> serde::Deserialize<'de>>(
    url: &str,
    token: &str,
) -> Result<T, String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
    headers.set("Cache-Control", "no-cache").ok();
    opts.headers(&headers);
    let req =
        web_sys::Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
            .await
            .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;
    if !resp.ok() {
        return Err(format!("Erreur HTTP : {}", resp.status()));
    }
    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
        .await
        .map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(json).map_err(|e| format!("{:?}", e))
}

async fn post_json(url: &str, token: &str, body: &serde_json::Value) -> Result<(), String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers.set("Authorization", &format!("Bearer {}", token)).ok();
    headers.set("Content-Type", "application/json").ok();
    opts.headers(&headers);
    opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));
    let req =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
            .await
            .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;
    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error_response(resp).await)
    }
}

async fn patch_json(url: &str, token: &str, body: &serde_json::Value) -> Result<(), String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("PATCH");
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers.set("Authorization", &format!("Bearer {}", token)).ok();
    headers.set("Content-Type", "application/json").ok();
    opts.headers(&headers);
    opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));
    let req =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
            .await
            .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;
    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error_response(resp).await)
    }
}

async fn parse_error_response(resp: web_sys::Response) -> String {
    let status = resp.status();
    if let Ok(promise) = resp.text() {
        if let Ok(val) = wasm_bindgen_futures::JsFuture::from(promise).await {
            if let Some(text) = val.as_string() {
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(msg) = obj.get("error").and_then(|v| v.as_str()) {
                        return msg.to_string();
                    }
                }
            }
        }
    }
    match status {
        409 => "Un contrat existe déjà sur cette période.".to_string(),
        402 => "Accès en lecture seule — licence expirée.".to_string(),
        403 => "Action non autorisée.".to_string(),
        404 => "Ressource introuvable.".to_string(),
        429 => "Trop de requêtes, réessayez dans quelques secondes.".to_string(),
        _ => format!("Erreur inattendue (HTTP {}).", status),
    }
}

// ─── Export PDF ───────────────────────────────────────────────────

fn export_loa_pdf(c: &ContractLoa) {
    let pct = ((c.km_consumed as f64 / c.km_allowed as f64) * 100.0).min(100.0) as u32;
    let status_label = match c.status.as_str() {
        "exceeded" => "Dépassé",
        "closed"   => "Clôturé",
        _          => if c.overage_risk { "Risque dépassement" } else { "Actif" },
    };
    let limit_line = c.estimated_limit_date
        .map(|d| format!("<tr><td>Limite estimée</td><td>{}</td></tr>", d))
        .unwrap_or_default();

    let cost_line = c.price_per_extra_km.and_then(|price| {
        let extra_km = if c.km_consumed > c.km_allowed {
            c.km_consumed - c.km_allowed
        } else if c.forecast_km > c.km_allowed {
            c.forecast_km - c.km_allowed
        } else { return None; };
        let cost = extra_km as f64 * price;
        let label = if c.km_consumed > c.km_allowed {
            format!("Coût dépassement ({:.2} €/km)", price)
        } else { format!("Coût projeté ({:.2} €/km)", price) };
        Some(format!("<tr><td style='color:#dc2626;font-weight:700'>{}</td><td style='color:#dc2626;font-weight:700'>{:.2} €</td></tr>", label, cost))
    }).unwrap_or_default();

    let html = format!(r#"<!DOCTYPE html>
<html lang="fr"><head><meta charset="UTF-8"/>
<title>Contrat LOA — LimTrack</title>
<style>
body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;max-width:680px;margin:40px auto;color:#1e293b;font-size:14px}}
h1{{color:#4f46e5;font-size:22px;margin-bottom:4px}}
.sub{{color:#94a3b8;font-size:12px;margin-bottom:32px}}
h2{{font-size:13px;font-weight:600;text-transform:uppercase;letter-spacing:.05em;color:#94a3b8;margin:28px 0 12px}}
table{{width:100%;border-collapse:collapse}}
td{{padding:8px 12px;border-bottom:1px solid #f1f5f9}}
td:first-child{{color:#64748b;width:50%}}
td:last-child{{font-weight:600}}
.progress-wrap{{background:#e2e8f0;border-radius:6px;height:10px;margin:16px 0}}
.progress-bar{{background:#4f46e5;border-radius:6px;height:10px}}
.badge{{display:inline-block;padding:3px 10px;border-radius:99px;font-size:12px;font-weight:600;background:#e0e7ff;color:#4338ca}}
footer{{margin-top:40px;font-size:11px;color:#94a3b8;border-top:1px solid #f1f5f9;padding-top:12px}}
@media print{{@page{{margin:20mm}}button{{display:none}}}}
</style></head>
<body>
<script>window.addEventListener('load',function(){{setTimeout(function(){{window.print()}},400)}})</script>
<h1>Contrat LOA</h1>
<div class="sub">Généré le {} — LimTrack</div>
<span class="badge">{}</span>
<h2>Kilométrage</h2>
<div class="progress-wrap"><div class="progress-bar" style="width:{}%"></div></div>
<table>
<tr><td>Kilométrage autorisé</td><td>{} km</td></tr>
<tr><td>Kilomètres consommés</td><td>{} km ({}%)</td></tr>
<tr><td>Kilomètres restants</td><td>{} km</td></tr>
<tr><td>Projection à échéance</td><td>{} km</td></tr>
{}
{}
</table>
<h2>Période</h2>
<table>
<tr><td>Date de début</td><td>{}</td></tr>
<tr><td>Date de fin</td><td>{}</td></tr>
<tr><td>Jours restants</td><td>{}</td></tr>
</table>
<footer>LimTrack · limtrack.app · Rapport généré automatiquement</footer>
</body></html>"#,
        chrono::Local::now().format("%d/%m/%Y"),
        status_label, pct,
        format_km(c.km_allowed), format_km(c.km_consumed), pct,
        format_km(c.km_remaining), format_km(c.forecast_km),
        limit_line, cost_line,
        c.start_date, c.end_date, c.days_remaining,
    );
    open_print_window(&html);
}

fn export_insurance_pdf(c: &ContractInsurance) {
    let pct = ((c.km_consumed as f64 / c.km_annual_limit as f64) * 100.0).min(100.0) as u32;
    let status_label = match c.status.as_str() {
        "exceeded" => "Dépassé",
        "closed"   => "Clôturé",
        _          => if c.overage_risk { "Risque dépassement" } else { "Active" },
    };
    let insurer_line = c.insurer.as_deref()
        .map(|ins| format!("<tr><td>Assureur</td><td>{}</td></tr>", ins))
        .unwrap_or_default();
    let limit_line = c.estimated_limit_date
        .map(|d| format!("<tr><td>Limite estimée</td><td>{}</td></tr>", d))
        .unwrap_or_default();

    let html = format!(r#"<!DOCTYPE html>
<html lang="fr"><head><meta charset="UTF-8"/>
<title>Contrat Assurance — LimTrack</title>
<style>
body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;max-width:680px;margin:40px auto;color:#1e293b;font-size:14px}}
h1{{color:#4f46e5;font-size:22px;margin-bottom:4px}}
.sub{{color:#94a3b8;font-size:12px;margin-bottom:32px}}
h2{{font-size:13px;font-weight:600;text-transform:uppercase;letter-spacing:.05em;color:#94a3b8;margin:28px 0 12px}}
table{{width:100%;border-collapse:collapse}}
td{{padding:8px 12px;border-bottom:1px solid #f1f5f9}}
td:first-child{{color:#64748b;width:50%}}
td:last-child{{font-weight:600}}
.progress-wrap{{background:#e2e8f0;border-radius:6px;height:10px;margin:16px 0}}
.progress-bar{{background:#4f46e5;border-radius:6px;height:10px}}
.badge{{display:inline-block;padding:3px 10px;border-radius:99px;font-size:12px;font-weight:600;background:#e0e7ff;color:#4338ca}}
footer{{margin-top:40px;font-size:11px;color:#94a3b8;border-top:1px solid #f1f5f9;padding-top:12px}}
@media print{{@page{{margin:20mm}}button{{display:none}}}}
</style></head>
<body>
<script>window.addEventListener('load',function(){{setTimeout(function(){{window.print()}},400)}})</script>
<h1>Contrat Assurance</h1>
<div class="sub">Généré le {} — LimTrack</div>
<span class="badge">{}</span>
<h2>Kilométrage</h2>
<div class="progress-wrap"><div class="progress-bar" style="width:{}%"></div></div>
<table>
{}
<tr><td>Limite annuelle</td><td>{} km</td></tr>
<tr><td>Kilomètres consommés</td><td>{} km ({}%)</td></tr>
<tr><td>Kilomètres restants</td><td>{} km</td></tr>
<tr><td>Projection à échéance</td><td>{} km</td></tr>
{}
</table>
<h2>Période</h2>
<table>
<tr><td>Date de début</td><td>{}</td></tr>
<tr><td>Date de fin</td><td>{}</td></tr>
<tr><td>Jours restants</td><td>{}</td></tr>
</table>
<footer>LimTrack · limtrack.app · Rapport généré automatiquement</footer>
</body></html>"#,
        chrono::Local::now().format("%d/%m/%Y"),
        status_label, pct,
        insurer_line,
        format_km(c.km_annual_limit), format_km(c.km_consumed), pct,
        format_km(c.km_remaining), format_km(c.forecast_km),
        limit_line,
        c.start_date, c.end_date, c.days_remaining,
    );
    open_print_window(&html);
}

fn open_print_window(html: &str) {
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(html));
    let mut opts = web_sys::BlobPropertyBag::new();
    opts.type_("text/html;charset=utf-8");
    if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&array, &opts) {
        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
            leptos::window().open_with_url_and_target(&url, "_blank").ok();
        }
    }
}

// ─── Export CSV ───────────────────────────────────────────────────

fn download_mileage_csv(
    entries: &[MileageLog],
    km_start: i32,
    km_total: i32,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    filename: &str,
) {
    let total_days = (end_date - start_date).num_days().max(1);
    let mut csv = String::from("Date,Kilométrage (km),Écart relevé précédent (km),Trajectoire idéale (km),Écart vs idéale (km),Statut trajectoire,Source\n");

    for (i, entry) in entries.iter().enumerate() {
        let ecart_prev = if i + 1 < entries.len() {
            (entry.value - entries[i + 1].value).to_string()
        } else {
            String::new()
        };

        let days_elapsed = (entry.recorded_at - start_date).num_days().max(0);
        let ideal = km_start + (km_total as f64 * days_elapsed as f64 / total_days as f64) as i32;
        let ecart_ideal = entry.value - ideal;
        let statut = if ecart_ideal >= 0 { "En avance" } else { "En retard" };

        let source = match entry.source.as_str() {
            "manual" => "Manuelle",
            "import" => "Import",
            "api"    => "API",
            s        => s,
        };
        csv.push_str(&format!("{},{},{},{},{},{},{}\n",
            entry.recorded_at, entry.value, ecart_prev, ideal, ecart_ideal, statut, source));
    }
    trigger_download(&csv, filename, "text/csv;charset=utf-8");
}

fn trigger_download(content: &str, filename: &str, mime: &str) {
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(content));
    let mut opts = web_sys::BlobPropertyBag::new();
    opts.type_(mime);
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&array, &opts) else { return };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else { return };

    let document = leptos::document();
    let Ok(el) = document.create_element("a") else { return };
    let Ok(a) = el.dyn_into::<web_sys::HtmlAnchorElement>() else { return };
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    web_sys::Url::revoke_object_url(&url).ok();
}
