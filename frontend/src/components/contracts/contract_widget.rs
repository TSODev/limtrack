// src/components/contracts/contracts_widget.rs
// Version RÉSUMÉ — affiché dans l'onglet Tableau de bord

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
pub fn ContractsWidget(vehicle_id: ReadSignal<Option<Uuid>>) -> impl IntoView {
    let (data, set_data) = create_signal(Option::<ContractsData>::None);
    let (loading, set_loading) = create_signal(false);

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_data.set(None);
            set_loading.set(true);
            spawn_local(async move {
                let Some(token) = get_token() else { return };
                let loa = fetch_json::<Vec<ContractLoa>>(
                    &format!("/api/vehicles/{}/contracts/loa", id),
                    &token,
                )
                .await
                .unwrap_or_default();
                let insurance = fetch_json::<Vec<ContractInsurance>>(
                    &format!("/api/vehicles/{}/contracts/insurance", id),
                    &token,
                )
                .await
                .unwrap_or_default();
                set_data.set(Some(ContractsData { loa, insurance }));
                set_loading.set(false);
            });
        }
    });

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 flex flex-col gap-4">
            <h3 class="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                "Contrats actifs"
            </h3>

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
    }
}

// ─── Couleurs selon état ──────────────────────────────────────────

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

// ─── Résumé LOA ──────────────────────────────────────────────────

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
            // En-tête
            <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-gray-700">"LOA"</span>
                <span class=format!(
                    "text-xs font-medium px-2 py-0.5 rounded-full {} {}",
                    colors.badge_bg, colors.badge_text
                )>
                    {colors.badge_label}
                </span>
            </div>

            // Barre de progression
            <div>
                <div class="flex justify-between text-xs text-gray-400 mb-1">
                    <span>{format_km(contract.km_consumed)}" / "{format_km(contract.km_allowed)}</span>
                    <span>{pct}"%"</span>
                </div>
                <div class="w-full bg-white rounded-full h-1.5">
                    <div
                        class=format!("h-1.5 rounded-full {}", colors.bar)
                        style=format!("width: {}%", pct)
                    />
                </div>
            </div>

            // Infos clés
            <div class="space-y-1.5">
                // Kilométrage estimé à échéance
                <div class=format!("flex items-center gap-1.5 text-xs font-medium {}", colors.text)>
                    <span>"📊"</span>
                    <span>{forecast_label}</span>
                </div>

                // Date estimée d'atteinte de la limite
                {limit_date_label.map(|label| view! {
                    <div class=format!("flex items-center gap-1.5 text-xs {}", colors.text)>
                        <span>"📅"</span>
                        <span>{label}</span>
                    </div>
                })}

                // Jours et km restants
                <div class="flex justify-between text-xs text-gray-500 pt-1">
                    <span>{format_km(contract.km_remaining)}" restants"</span>
                    <span>{contract.days_remaining}" j jusqu'à l'échéance"</span>
                </div>
            </div>
        </div>
    }
}

// ─── Résumé Assurance ────────────────────────────────────────────

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
            // En-tête
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-1.5">
                    <span class="text-xs font-bold text-gray-700">"Assurance"</span>
                    {contract.insurer.map(|ins| view! {
                        <span class="text-xs text-gray-400">"("{ins}")"</span>
                    })}
                </div>
                <span class=format!(
                    "text-xs font-medium px-2 py-0.5 rounded-full {} {}",
                    colors.badge_bg, colors.badge_text
                )>
                    {colors.badge_label}
                </span>
            </div>

            // Barre de progression
            <div>
                <div class="flex justify-between text-xs text-gray-400 mb-1">
                    <span>{format_km(contract.km_consumed)}" / "{format_km(contract.km_annual_limit)}</span>
                    <span>{pct}"%"</span>
                </div>
                <div class="w-full bg-white rounded-full h-1.5">
                    <div
                        class=format!("h-1.5 rounded-full {}", colors.bar)
                        style=format!("width: {}%", pct)
                    />
                </div>
            </div>

            // Infos clés
            <div class="space-y-1.5">
                <div class=format!("flex items-center gap-1.5 text-xs font-medium {}", colors.text)>
                    <span>"📊"</span>
                    <span>{forecast_label}</span>
                </div>

                {limit_date_label.map(|label| view! {
                    <div class=format!("flex items-center gap-1.5 text-xs {}", colors.text)>
                        <span>"📅"</span>
                        <span>{label}</span>
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
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;
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
