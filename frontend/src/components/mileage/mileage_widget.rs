// src/components/mileage/mileage_widget.rs
use crate::components::ui::{format_km, get_token};
use common::{ContractInsurance, ContractLoa, MileageLog};
use leptos::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[derive(Clone)]
struct WidgetData {
    entries: Vec<MileageLog>,
    loa: Vec<ContractLoa>,
    insurance: Vec<ContractInsurance>,
}

#[component]
pub fn MileageWidget(vehicle_id: ReadSignal<Option<Uuid>>) -> impl IntoView {
    let (data, set_data) = create_signal(Option::<WidgetData>::None);
    let (loading, set_loading) = create_signal(false);

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_data.set(None);
            set_loading.set(true);
            spawn_local(async move {
                let Some(token) = get_token() else { return };

                let entries =
                    fetch_json::<Vec<MileageLog>>(&format!("/api/vehicles/{}/mileage", id), &token)
                        .await
                        .unwrap_or_default();

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

                set_data.set(Some(WidgetData {
                    entries,
                    loa,
                    insurance,
                }));
                set_loading.set(false);
            });
        }
    });

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 flex flex-col gap-3 md:gap-4">
            <h3 class="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                "Kilométrage"
            </h3>

            <Show when=move || loading.get() fallback=|| ()>
                <p class="text-xs text-gray-400 animate-pulse">"Chargement..."</p>
            </Show>

            <Show when=move || !loading.get() fallback=|| ()>
                {move || {
                    let Some(d) = data.get() else { return view! { <div /> }.into_view(); };

                    if d.entries.is_empty() {
                        return view! {
                            <p class="text-xs text-gray-400 italic">
                                "Aucun relevé — rendez-vous dans l'onglet Kilométrage."
                            </p>
                        }.into_view();
                    }

                    let last    = d.entries.first().unwrap().clone();
                    let entries = d.entries.clone();

                    let active_contract: Option<(i32, i32, chrono::NaiveDate, chrono::NaiveDate)> =
                        d.loa.iter()
                            .find(|c| c.status == "active")
                            .map(|c| (c.km_start, c.km_allowed, c.start_date, c.end_date))
                            .or_else(|| {
                                d.insurance.iter()
                                    .find(|c| c.status == "active")
                                    .map(|c| (c.km_start, c.km_start + c.km_annual_limit, c.start_date, c.end_date))
                            });

                    let recent: Vec<MileageLog> = entries.iter()
                        .take(8).cloned().collect::<Vec<_>>()
                        .into_iter().rev().collect();

                    let km_values: Vec<i32> = recent.iter().map(|e| e.value).collect();

                    let (km_min, km_max) = {
                        let mut all_vals = km_values.clone();
                        if let Some((km_start, km_allowed, _, _)) = active_contract {
                            all_vals.push(km_start);
                            all_vals.push(km_allowed);
                        }
                        let mn = *all_vals.iter().min().unwrap_or(&0) as f64;
                        let mx = *all_vals.iter().max().unwrap_or(&1) as f64;
                        (mn, (mx - mn).max(1.0))
                    };

                    let first_date = recent.first().map(|e| e.recorded_at);
                    let last_date  = recent.last().map(|e| e.recorded_at);

                    let svg_w = 300.0_f64;
                    let svg_h = 60.0_f64;

                    let today = chrono::Local::now().date_naive();

                    // Plage de dates — inclut la plage du contrat pour que la trajectoire idéale soit visible
                    let date_range = {
                        let fd = first_date.unwrap_or(today);
                        let contract_start = active_contract.map(|(_, _, sd, _)| sd);
                        let range_start = contract_start.map(|cs| cs.min(fd)).unwrap_or(fd);
                        let days = (today - range_start).num_days().max(1) as f64;
                        days
                    };

                    // Recalcule first_date en tenant compte du contrat
                    let effective_start = {
                        let fd = first_date.unwrap_or(today);
                        let contract_start = active_contract.map(|(_, _, sd, _)| sd);
                        contract_start.map(|cs| cs.min(fd)).unwrap_or(fd)
                    };

                    let real_points: Vec<(f64, f64)> = recent.iter().map(|e| {
                        let x = (e.recorded_at - effective_start).num_days() as f64 / date_range * svg_w;
                        let y = svg_h - ((e.value as f64 - km_min) / km_max * (svg_h - 10.0)) - 5.0;
                        (x, y)
                    }).collect();

                    let real_polyline = if real_points.len() > 1 {
                        real_points.iter()
                            .map(|(x, y)| format!("{:.1},{:.1}", x, y))
                            .collect::<Vec<_>>().join(" ")
                    } else {
                        String::new()
                    };

                    let ideal_polyline: Option<String> = active_contract.map(|(km_start, km_allowed, start_date, end_date)| {
                        let total_days = (end_date - start_date).num_days().max(1) as f64;
                        let elapsed    = (today - start_date).num_days().max(0) as f64;
                        let km_today   = km_start as f64 + (km_allowed - km_start) as f64 * (elapsed / total_days);

                        let x_start = ((start_date - effective_start).num_days() as f64 / date_range * svg_w).clamp(0.0, svg_w);
                        let x_end   = ((today - effective_start).num_days() as f64 / date_range * svg_w).clamp(0.0, svg_w);
                        let y_start = svg_h - ((km_start as f64 - km_min) / km_max * (svg_h - 10.0)) - 5.0;
                        let y_end   = svg_h - ((km_today - km_min) / km_max * (svg_h - 10.0)) - 5.0;

                        format!("{:.1},{:.1} {:.1},{:.1}", x_start, y_start, x_end, y_end)
                    });

                    let is_over_ideal = active_contract.map(|(km_start, km_allowed, start_date, end_date)| {
                        let total_days = (end_date - start_date).num_days().max(1) as f64;
                        let elapsed    = (last.recorded_at - start_date).num_days().max(0) as f64;
                        let ideal_km   = km_start as f64 + (km_allowed - km_start) as f64 * (elapsed / total_days);
                        last.value as f64 > ideal_km
                    }).unwrap_or(false);

                    let line_color = if is_over_ideal { "#f59e0b" } else { "#6366f1" };

                    // ── SVG généré comme string pour compatibilité Android ──
                    let show_sparkline = real_points.len() > 1 || (real_points.len() == 1 && active_contract.is_some());

                    let ideal_svg = ideal_polyline.as_ref().map(|pts| format!(
                        "<polyline points='{}' fill='none' stroke='#d1d5db' stroke-width='1.5' stroke-dasharray='4 3' stroke-linejoin='round' stroke-linecap='round'/>",
                        pts
                    )).unwrap_or_default();

                    let last_point_svg = real_points.last().map(|(x, y)| format!(
                        "<circle cx='{:.1}' cy='{:.1}' r='3' fill='{}'/>",
                        x, y, line_color
                    )).unwrap_or_default();

                    let svg_html = format!(
                        "<svg viewBox='0 0 {vw} {vh}' width='100%' height='60' preserveAspectRatio='none' xmlns='http://www.w3.org/2000/svg' style='display:block'>{ideal}<polyline points='{pts}' fill='none' stroke='{color}' stroke-width='2' stroke-linejoin='round' stroke-linecap='round'/>{dot}</svg>",
                        vw    = svg_w as i32,
                        vh    = svg_h as i32,
                        ideal = ideal_svg,
                        pts   = real_polyline,
                        color = line_color,
                        dot   = last_point_svg
                    );

                    let has_contract = active_contract.is_some();

                    view! {
                        <div class="space-y-3 md:space-y-4">
                            // Valeur principale
                            <div>
                                <p class="text-2xl md:text-3xl font-extrabold text-gray-900 tracking-tight">
                                    {format_km(last.value)}
                                </p>
                                <p class="text-xs text-gray-400 mt-1">
                                    "Relevé du "{last.recorded_at.to_string()}
                                </p>
                            </div>

                            // Sparkline via innerHTML — compatibilité Android
                            <Show when=move || show_sparkline fallback=|| ()>
                                <div class="space-y-1">
                                    <div inner_html=svg_html.clone() />

                                    // Légende
                                    <Show when=move || has_contract fallback=|| ()>
                                        <div class="flex items-center gap-4 text-xs text-gray-400">
                                            <div class="flex items-center gap-1.5">
                                                <svg width="16" height="8" xmlns="http://www.w3.org/2000/svg">
                                                    <line x1="0" y1="4" x2="16" y2="4"
                                                        stroke=line_color stroke-width="2"
                                                        stroke-linecap="round"/>
                                                </svg>
                                                "Réel"
                                            </div>
                                            <div class="flex items-center gap-1.5">
                                                <svg width="16" height="8" xmlns="http://www.w3.org/2000/svg">
                                                    <line x1="0" y1="4" x2="16" y2="4"
                                                        stroke="#d1d5db" stroke-width="1.5"
                                                        stroke-dasharray="4 3"/>
                                                </svg>
                                                "Trajectoire idéale"
                                            </div>
                                        </div>
                                    </Show>
                                </div>
                            </Show>

                            // Indicateur position vs trajectoire
                            <Show when=move || has_contract fallback=|| ()>
                                <div class=move || format!(
                                    "flex items-center gap-1.5 text-xs font-medium px-2.5 py-1.5 rounded-lg {}",
                                    if is_over_ideal { "bg-amber-50 text-amber-700" }
                                    else { "bg-green-50 text-green-700" }
                                )>
                                    {if is_over_ideal {
                                        "⚠ Au-dessus de la trajectoire idéale"
                                    } else {
                                        "✓ En dessous de la trajectoire idéale"
                                    }}
                                </div>
                            </Show>
                        </div>
                    }.into_view()
                }}
            </Show>
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
