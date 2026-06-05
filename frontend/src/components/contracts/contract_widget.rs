// src/components/contracts/contracts_widget.rs
use crate::components::ui::{format_km, get_token};
use common::{ContractInsurance, ContractLoa};
use leptos::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[derive(Clone)]
struct ContractsData {
    loa: Vec<ContractLoa>,
    insurance: Vec<ContractInsurance>,
}

#[component]
pub fn ContractsWidget(
    vehicle_id: ReadSignal<Option<Uuid>>,
    can_manage_contracts: Memo<bool>,
) -> impl IntoView {
    let (data, set_data) = create_signal(Option::<ContractsData>::None);
    let (loading, set_loading) = create_signal(false);
    let (show_loa_modal, set_show_loa_modal) = create_signal(false);
    let (show_insurance_modal, set_show_insurance_modal) = create_signal(false);

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_data.set(None);
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
        }
    });

    let on_created = move || {
        if let Some(id) = vehicle_id.get() {
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
        }
    };

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 flex flex-col gap-4">
            <div class="flex items-center justify-between">
                <h3 class="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                    "Contrats actifs"
                </h3>
                // Boutons visibles uniquement pour owner
                <Show when=move || can_manage_contracts.get() fallback=|| ()>
                    <div class="flex gap-2">
                        <button
                            on:click=move |_| set_show_loa_modal.set(true)
                            class="text-xs px-3 py-1.5 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 transition duration-150"
                        >
                            "+ LOA"
                        </button>
                        <button
                            on:click=move |_| set_show_insurance_modal.set(true)
                            class="text-xs px-3 py-1.5 rounded-lg border border-indigo-200 text-indigo-600 hover:bg-indigo-50 transition duration-150"
                        >
                            "+ Assurance"
                        </button>
                    </div>
                </Show>
            </div>

            <Show when=move || loading.get() fallback=|| ()>
                <p class="text-xs text-gray-400 animate-pulse">"Chargement..."</p>
            </Show>

            <Show when=move || !loading.get() && data.get().is_some() fallback=|| ()>
                {move || data.get().map(|d| {
                    let total = d.loa.len() + d.insurance.len();
                    if total == 0 {
                        return view! {
                            <p class="text-xs text-gray-400 italic">
                                "Aucun contrat — rendez-vous dans l'onglet Contrats."
                            </p>
                        }.into_view();
                    }
                    view! {
                        <div class="flex flex-col gap-4">
                            {d.loa.into_iter().map(|c| view! {
                                <ContractLoaSummary contract=c />
                            }).collect_view()}
                            {d.insurance.into_iter().map(|c| view! {
                                <ContractInsuranceSummary contract=c />
                            }).collect_view()}
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

struct StatusColors {
    bar: &'static str,
    badge_bg: &'static str,
    badge_text: &'static str,
    badge_label: &'static str,
    card_bg: &'static str,
    text: &'static str,
}

fn status_colors(status: &str, overage_risk: bool) -> StatusColors {
    match status {
        "exceeded" => StatusColors {
            bar: "bg-red-500",
            badge_bg: "bg-red-100",
            badge_text: "text-red-700",
            badge_label: "Dépassé",
            card_bg: "bg-red-50",
            text: "text-red-700",
        },
        "closed" => StatusColors {
            bar: "bg-gray-400",
            badge_bg: "bg-gray-100",
            badge_text: "text-gray-600",
            badge_label: "Clôturé",
            card_bg: "bg-gray-50",
            text: "text-gray-500",
        },
        _ if overage_risk => StatusColors {
            bar: "bg-amber-400",
            badge_bg: "bg-amber-100",
            badge_text: "text-amber-700",
            badge_label: "Risque",
            card_bg: "bg-amber-50",
            text: "text-amber-700",
        },
        _ => StatusColors {
            bar: "bg-indigo-500",
            badge_bg: "bg-green-100",
            badge_text: "text-green-700",
            badge_label: "Actif",
            card_bg: "bg-green-50",
            text: "text-green-700",
        },
    }
}

#[component]
fn ContractLoaSummary(contract: ContractLoa) -> impl IntoView {
    let pct =
        ((contract.km_consumed as f64 / contract.km_allowed as f64) * 100.0).min(100.0) as u32;
    let colors = status_colors(&contract.status, contract.overage_risk);
    let forecast_label = if contract.forecast_km > contract.km_allowed {
        format!("⚠ {} estimés à échéance", format_km(contract.forecast_km))
    } else {
        format!("{} estimés à échéance", format_km(contract.forecast_km))
    };
    let limit_date_label = contract.estimated_limit_date.map(|d| {
        if d <= contract.end_date {
            format!("Limite atteinte vers le {}", d)
        } else {
            format!("Limite atteinte après l'échéance ({})", d)
        }
    });

    view! {
        <div class=format!("rounded-xl border p-4 space-y-3 {}", colors.card_bg)>
            <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-gray-700">"LOA"</span>
                <span class=format!("text-xs font-medium px-2 py-0.5 rounded-full {} {}", colors.badge_bg, colors.badge_text)>
                    {colors.badge_label}
                </span>
            </div>
            <div>
                <div class="flex justify-between text-xs text-gray-400 mb-1">
                    <span>{format_km(contract.km_consumed)}" / "{format_km(contract.km_allowed)}</span>
                    <span>{pct}"%"</span>
                </div>
                <div class="w-full bg-white rounded-full h-1.5">
                    <div class=format!("h-1.5 rounded-full {}", colors.bar) style=format!("width: {}%", pct) />
                </div>
            </div>
            <div class="space-y-1.5">
                <div class=format!("flex items-center gap-1.5 text-xs font-medium {}", colors.text)>
                    <span>"📊"</span><span>{forecast_label}</span>
                </div>
                {limit_date_label.map(|label| view! {
                    <div class=format!("flex items-center gap-1.5 text-xs {}", colors.text)>
                        <span>"📅"</span><span>{label}</span>
                    </div>
                })}
                <div class="flex justify-between text-xs text-gray-500 pt-1">
                    <span>{format_km(contract.km_remaining)}" restants"</span>
                    <span>{contract.days_remaining}" j jusqu'à l'échéance"</span>
                </div>
                {contract.price_per_extra_km.and_then(|price| {
                    let extra_km = if contract.km_consumed > contract.km_allowed {
                        contract.km_consumed - contract.km_allowed
                    } else if contract.forecast_km > contract.km_allowed {
                        contract.forecast_km - contract.km_allowed
                    } else {
                        return None;
                    };
                    let cost = extra_km as f64 * price;
                    let label = if contract.km_consumed > contract.km_allowed {
                        format!("Coût dépassement : {:.2} €", cost)
                    } else {
                        format!("Coût projeté : {:.2} €", cost)
                    };
                    Some(view! {
                        <div class=format!("flex items-center gap-1.5 text-xs font-semibold {}", colors.text)>
                            <span>"💶"</span><span>{label}</span>
                        </div>
                    })
                })}
            </div>
        </div>
    }
}

#[component]
fn ContractInsuranceSummary(contract: ContractInsurance) -> impl IntoView {
    let pct =
        ((contract.km_consumed as f64 / contract.km_annual_limit as f64) * 100.0).min(100.0) as u32;
    let colors = status_colors(&contract.status, contract.overage_risk);
    let forecast_label = if contract.forecast_km > contract.km_annual_limit {
        format!("⚠ {} estimés à échéance", format_km(contract.forecast_km))
    } else {
        format!("{} estimés à échéance", format_km(contract.forecast_km))
    };
    let limit_date_label = contract.estimated_limit_date.map(|d| {
        if d <= contract.end_date {
            format!("Limite atteinte vers le {}", d)
        } else {
            format!("Limite atteinte après l'échéance ({})", d)
        }
    });

    view! {
        <div class=format!("rounded-xl border p-4 space-y-3 {}", colors.card_bg)>
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-1.5">
                    <span class="text-xs font-bold text-gray-700">"Assurance"</span>
                    {contract.insurer.map(|ins| view! {
                        <span class="text-xs text-gray-400">"("{ins}")"</span>
                    })}
                </div>
                <span class=format!("text-xs font-medium px-2 py-0.5 rounded-full {} {}", colors.badge_bg, colors.badge_text)>
                    {colors.badge_label}
                </span>
            </div>
            <div>
                <div class="flex justify-between text-xs text-gray-400 mb-1">
                    <span>{format_km(contract.km_consumed)}" / "{format_km(contract.km_annual_limit)}</span>
                    <span>{pct}"%"</span>
                </div>
                <div class="w-full bg-white rounded-full h-1.5">
                    <div class=format!("h-1.5 rounded-full {}", colors.bar) style=format!("width: {}%", pct) />
                </div>
            </div>
            <div class="space-y-1.5">
                <div class=format!("flex items-center gap-1.5 text-xs font-medium {}", colors.text)>
                    <span>"📊"</span><span>{forecast_label}</span>
                </div>
                {limit_date_label.map(|label| view! {
                    <div class=format!("flex items-center gap-1.5 text-xs {}", colors.text)>
                        <span>"📅"</span><span>{label}</span>
                    </div>
                })}
                <div class="flex justify-between text-xs text-gray-500 pt-1">
                    <span>{format_km(contract.km_remaining)}" restants"</span>
                    <span>{contract.days_remaining}" j jusqu'à l'échéance"</span>
                </div>
            </div>
        </div>
    }
}

// ─── Modals ──────────────────────────────────────────────────────

#[component]
fn LoaModal(
    vehicle_id: ReadSignal<Option<Uuid>>,
    on_close: Callback<()>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (km_allowed, set_km_allowed)         = create_signal(String::new());
    let (km_start, set_km_start)             = create_signal(String::new());
    let (start_date, set_start_date)         = create_signal(String::new());
    let (end_date, set_end_date)             = create_signal(String::new());
    let (price_per_km, set_price_per_km)     = create_signal(String::new());
    let (error, set_error)                   = create_signal(String::new());

    let submit = create_action(
        move |(vid, km_a, km_s, sd, ed, price): &(Uuid, String, String, String, String, String)| {
            let (vid, km_a, km_s, sd, ed, price) =
                (*vid, km_a.clone(), km_s.clone(), sd.clone(), ed.clone(), price.clone());
            async move {
                let token = get_token().unwrap_or_default();
                let price_val = price.trim().parse::<f64>().ok();
                let body = serde_json::json!({
                    "km_allowed": km_a.parse::<i32>().unwrap_or(0),
                    "km_start":   km_s.parse::<i32>().unwrap_or(0),
                    "start_date": sd, "end_date": ed,
                    "price_per_extra_km": price_val,
                });
                match post_json(
                    &format!("{}/api/vehicles/{}/contracts/loa", crate::config::API_BASE, vid),
                    &token,
                    &body,
                )
                .await
                {
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
                    <input type="number" min="1" required prop:value=km_allowed
                        on:input=move |ev| set_km_allowed.set(event_target_value(&ev))
                        placeholder="ex: 45000" class=input_class() />
                </Field>
                <Field label="Kilométrage au départ">
                    <input type="number" min="0" required prop:value=km_start
                        on:input=move |ev| set_km_start.set(event_target_value(&ev))
                        placeholder="ex: 12000" class=input_class() />
                </Field>
                <div class="grid grid-cols-2 gap-3">
                    <Field label="Date de début">
                        <input type="date" required prop:value=start_date
                            on:input=move |ev| set_start_date.set(event_target_value(&ev))
                            class=input_class() />
                    </Field>
                    <Field label="Date de fin">
                        <input type="date" required prop:value=end_date
                            on:input=move |ev| set_end_date.set(event_target_value(&ev))
                            class=input_class() />
                    </Field>
                </div>
                <Field label="Prix/km en cas de dépassement (optionnel, €)">
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
    let (km_limit, set_km_limit) = create_signal(String::new());
    let (km_start, set_km_start) = create_signal(String::new());
    let (start_date, set_start_date) = create_signal(String::new());
    let (end_date, set_end_date) = create_signal(String::new());
    let (insurer, set_insurer) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());

    let submit = create_action(
        move |(vid, km_l, km_s, sd, ed, ins): &(Uuid, String, String, String, String, String)| {
            let (vid, km_l, km_s, sd, ed, ins) = (
                *vid,
                km_l.clone(),
                km_s.clone(),
                sd.clone(),
                ed.clone(),
                ins.clone(),
            );
            async move {
                let token = get_token().unwrap_or_default();
                let body = serde_json::json!({
                    "km_annual_limit": km_l.parse::<i32>().unwrap_or(0),
                    "km_start": km_s.parse::<i32>().unwrap_or(0),
                    "start_date": sd, "end_date": ed,
                    "insurer": if ins.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(ins) },
                });
                match post_json(
                    &format!("{}/api/vehicles/{}/contracts/insurance", crate::config::API_BASE, vid),
                    &token,
                    &body,
                )
                .await
                {
                    Ok(_) => {
                        on_created.call(());
                        on_close.call(());
                    }
                    Err(e) => set_error.set(e),
                }
            }
        },
    );

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(id) = vehicle_id.get() else { return };
        set_error.set(String::new());
        submit.dispatch((
            id,
            km_limit.get(),
            km_start.get(),
            start_date.get(),
            end_date.get(),
            insurer.get(),
        ));
    };

    view! {
        <Modal title="Nouveau contrat Assurance" on_close=on_close>
            <form on:submit=on_submit class="space-y-4">
                <Field label="Limite kilométrique annuelle">
                    <input type="number" min="1" required prop:value=km_limit
                        on:input=move |ev| set_km_limit.set(event_target_value(&ev))
                        placeholder="ex: 15000" class=input_class() />
                </Field>
                <Field label="Kilométrage au départ">
                    <input type="number" min="0" required prop:value=km_start
                        on:input=move |ev| set_km_start.set(event_target_value(&ev))
                        placeholder="ex: 12000" class=input_class() />
                </Field>
                <Field label="Assureur (optionnel)">
                    <input type="text" prop:value=insurer
                        on:input=move |ev| set_insurer.set(event_target_value(&ev))
                        placeholder="ex: Maif" class=input_class() />
                </Field>
                <div class="grid grid-cols-2 gap-3">
                    <Field label="Date de début">
                        <input type="date" required prop:value=start_date
                            on:input=move |ev| set_start_date.set(event_target_value(&ev))
                            class=input_class() />
                    </Field>
                    <Field label="Date de fin">
                        <input type="date" required prop:value=end_date
                            on:input=move |ev| set_end_date.set(event_target_value(&ev))
                            class=input_class() />
                    </Field>
                </div>
                <ModalActions pending=submit.pending() on_cancel=Callback::new(move |_| on_close.call(())) label_submit="Créer le contrat" error=error />
            </form>
        </Modal>
    }
}

// ─── Utilitaires UI ──────────────────────────────────────────────

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
            <button type="button" on:click=move |_| on_cancel.call(())
                class="flex-1 py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150">
                "Annuler"
            </button>
            <button type="submit" prop:disabled=move || pending.get()
                class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150">
                {move || if pending.get() { "Envoi..." } else { label_submit }}
            </button>
        </div>
    }
}

fn input_class() -> &'static str {
    "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm \
     placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 \
     sm:text-sm transition duration-150"
}

// ─── Helpers réseau ──────────────────────────────────────────────

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
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
    headers.set("Content-Type", "application/json").ok();
    opts.headers(&headers);
    opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));
    let req =
        web_sys::Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
            .await
            .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;
    if resp.ok() || resp.status() == 201 {
        Ok(())
    } else {
        Err(format!("Erreur HTTP : {}", resp.status()))
    }
}
