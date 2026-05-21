// src/components/mileage/mileage_widget.rs
// Version RÉSUMÉ — affiché dans l'onglet Tableau de bord

use crate::components::ui::{format_km, get_token};
use common::MileageLog;
use leptos::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[component]
pub fn MileageWidget(vehicle_id: ReadSignal<Option<Uuid>>) -> impl IntoView {
    let (entries, set_entries) = create_signal(Vec::<MileageLog>::new());
    let (loading, set_loading) = create_signal(false);

    create_effect(move |_| {
        if let Some(id) = vehicle_id.get() {
            set_entries.set(vec![]);
            set_loading.set(true);
            spawn_local(async move {
                let Some(token) = get_token() else { return };
                let data =
                    fetch_json::<Vec<MileageLog>>(&format!("/api/vehicles/{}/mileage", id), &token)
                        .await
                        .unwrap_or_default();
                set_entries.set(data);
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
                    let data = entries.get();
                    if data.is_empty() {
                        return view! {
                            <p class="text-xs text-gray-400 italic">
                                "Aucun relevé — rendez-vous dans l'onglet Kilométrage."
                            </p>
                        }.into_view();
                    }

                    let last = data.first().unwrap().clone();

                    // 5 derniers pour le mini sparkline (du plus ancien au plus récent)
                    let recent: Vec<i32> = data.iter().take(5).map(|e| e.value).collect();
                    let min_val = *recent.iter().min().unwrap_or(&0) as f64;
                    let max_val = *recent.iter().max().unwrap_or(&1) as f64;
                    let range = (max_val - min_val).max(1.0);

                    let points: Vec<(f64, f64)> = recent.iter().rev().enumerate().map(|(i, &v)| {
                        let x = i as f64 / (recent.len() - 1).max(1) as f64 * 200.0;
                        let y = 40.0 - ((v as f64 - min_val) / range * 35.0);
                        (x, y)
                    }).collect();

                    let polyline = points.iter()
                        .map(|(x, y)| format!("{:.1},{:.1}", x, y))
                        .collect::<Vec<_>>()
                        .join(" ");

                    view! {
                        <div class="space-y-4">
                            <div>
                                <p class="text-2xl md:text-3xl font-extrabold text-gray-900 tracking-tight">
                                    {format_km(last.value)}
                                </p>
                                <p class="text-xs text-gray-400 mt-1">
                                    "Relevé du "{last.recorded_at.to_string()}
                                </p>
                            </div>

                            {if points.len() > 1 { view! {
                                <svg viewBox="0 0 200 45" class="w-full h-10 overflow-visible">
                                    <polyline
                                        points=polyline
                                        fill="none"
                                        stroke="#6366f1"
                                        stroke-width="2"
                                        stroke-linejoin="round"
                                        stroke-linecap="round"
                                    />
                                    {points.last().map(|(x, y)| view! {
                                        <circle
                                            cx=x.to_string()
                                            cy=y.to_string()
                                            r="3"
                                            fill="#6366f1"
                                        />
                                    })}
                                </svg>
                            }.into_view() } else { view! { <div /> }.into_view() }}
                        </div>
                    }.into_view()
                }}
            </Show>
        </div>
    }
}

// ─── Helpers ─────────────────────────────────────────────────────

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
