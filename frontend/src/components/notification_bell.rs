// src/components/notification_bell.rs
use common::{ContractInsurance, ContractLoa, LicenseStatus};
use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

// ─── Types ───────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
struct UserPreferences {
    notif_days_before: i32,
    notif_km_percent: i32,
    updated_once: bool,
}

#[derive(Clone)]
struct Alert {
    vehicle_name: String,
    message: String,
    level: AlertLevel,
}

#[derive(Clone, PartialEq)]
enum AlertLevel {
    Warning, // Risque de dépassement ou proche échéance
    Danger,  // Dépassé
}

// ─── Composant ───────────────────────────────────────────────────

#[component]
pub fn NotificationBell(vehicles: ReadSignal<Vec<common::Vehicle>>) -> impl IntoView {
    let (alerts, set_alerts) = create_signal(Vec::<Alert>::new());
    let (open, set_open) = create_signal(false);
    let (prefs, set_prefs) = create_signal(UserPreferences {
        notif_days_before: 30,
        notif_km_percent: 80,
        updated_once: false,
    });

    // Charge les préférences au montage
    create_effect(move |_| {
        spawn_local(async move {
            let Some(token) = get_token() else { return };
            if let Ok(p) = fetch_json::<UserPreferences>(
                &format!("{}/api/profile/preferences", crate::config::API_BASE),
                &token,
            )
            .await
            {
                set_prefs.set(p);
            }
        });
    });

    // Recalcule les alertes quand les véhicules ou les préférences changent
    create_effect(move |_| {
        let vehicle_list = vehicles.get();
        let p = prefs.get();

        if vehicle_list.is_empty() {
            return;
        }

        spawn_local(async move {
            let Some(token) = get_token() else { return };
            let mut new_alerts: Vec<Alert> = Vec::new();

            // Alerte expiration de licence
            if let Ok(license) = fetch_json::<LicenseStatus>(
                &format!("{}/api/profile/license", crate::config::API_BASE),
                &token,
            )
            .await
            {
                if let Some(days) = license.days_until_expiry {
                    let msg = if license.status == "trial" {
                        format!(
                            "Période d'essai : expire dans {} jour{}",
                            days,
                            if days > 1 { "s" } else { "" }
                        )
                    } else {
                        format!(
                            "Licence : expire dans {} jour{}",
                            days,
                            if days > 1 { "s" } else { "" }
                        )
                    };
                    new_alerts.push(Alert {
                        vehicle_name: "Licence LimTrack".to_string(),
                        message: msg,
                        level: if days <= 3 { AlertLevel::Danger } else { AlertLevel::Warning },
                    });
                }
            }

            for vehicle in &vehicle_list {
                let vehicle_name = format!("{} {}", vehicle.make, vehicle.model);
                let id = vehicle.id;

                // Fetch contrats LOA
                if let Ok(loas) = fetch_json::<Vec<ContractLoa>>(
                    &format!(
                        "{}/api/vehicles/{}/contracts/loa",
                        crate::config::API_BASE,
                        id
                    ),
                    &token,
                )
                .await
                {
                    for loa in loas {
                        if loa.status == "closed" {
                            continue;
                        }

                        // Alerte dépassement
                        if loa.status == "exceeded" {
                            new_alerts.push(Alert {
                                vehicle_name: vehicle_name.clone(),
                                message: format!(
                                    "LOA : kilométrage dépassé ({} / {})",
                                    format_km(loa.km_current),
                                    format_km(loa.km_allowed)
                                ),
                                level: AlertLevel::Danger,
                            });
                        } else {
                            // Alerte % kilométrage
                            let pct =
                                (loa.km_consumed as f64 / loa.km_allowed as f64 * 100.0) as i32;
                            if pct >= p.notif_km_percent {
                                new_alerts.push(Alert {
                                    vehicle_name: vehicle_name.clone(),
                                    message: format!(
                                        "LOA : {}% du kilométrage consommé ({} restants)",
                                        pct,
                                        format_km(loa.km_remaining)
                                    ),
                                    level: if loa.overage_risk {
                                        AlertLevel::Danger
                                    } else {
                                        AlertLevel::Warning
                                    },
                                });
                            }

                            // Alerte échéance
                            if loa.days_remaining <= p.notif_days_before as i64 {
                                new_alerts.push(Alert {
                                    vehicle_name: vehicle_name.clone(),
                                    message: format!(
                                        "LOA : échéance dans {} jour{}",
                                        loa.days_remaining,
                                        if loa.days_remaining > 1 { "s" } else { "" }
                                    ),
                                    level: if loa.days_remaining <= 7 {
                                        AlertLevel::Danger
                                    } else {
                                        AlertLevel::Warning
                                    },
                                });
                            }
                        }
                    }
                }

                // Fetch contrats Assurance
                if let Ok(insurances) = fetch_json::<Vec<ContractInsurance>>(
                    &format!(
                        "{}/api/vehicles/{}/contracts/insurance",
                        crate::config::API_BASE,
                        id
                    ),
                    &token,
                )
                .await
                {
                    for ins in insurances {
                        if ins.status == "closed" {
                            continue;
                        }

                        if ins.status == "exceeded" {
                            new_alerts.push(Alert {
                                vehicle_name: vehicle_name.clone(),
                                message: format!(
                                    "Assurance : kilométrage dépassé ({} / {})",
                                    format_km(ins.km_current),
                                    format_km(ins.km_annual_limit)
                                ),
                                level: AlertLevel::Danger,
                            });
                        } else {
                            let pct = (ins.km_consumed as f64 / ins.km_annual_limit as f64 * 100.0)
                                as i32;
                            if pct >= p.notif_km_percent {
                                new_alerts.push(Alert {
                                    vehicle_name: vehicle_name.clone(),
                                    message: format!(
                                        "Assurance : {}% du kilométrage consommé ({} restants)",
                                        pct,
                                        format_km(ins.km_remaining)
                                    ),
                                    level: if ins.overage_risk {
                                        AlertLevel::Danger
                                    } else {
                                        AlertLevel::Warning
                                    },
                                });
                            }

                            if ins.days_remaining <= p.notif_days_before as i64 {
                                new_alerts.push(Alert {
                                    vehicle_name: vehicle_name.clone(),
                                    message: format!(
                                        "Assurance : échéance dans {} jour{}",
                                        ins.days_remaining,
                                        if ins.days_remaining > 1 { "s" } else { "" }
                                    ),
                                    level: if ins.days_remaining <= 7 {
                                        AlertLevel::Danger
                                    } else {
                                        AlertLevel::Warning
                                    },
                                });
                            }
                        }
                    }
                }
            }

            set_alerts.set(new_alerts);
        });
    });

    let alert_count = create_memo(move |_| alerts.get().len());
    let has_danger =
        create_memo(move |_| alerts.get().iter().any(|a| a.level == AlertLevel::Danger));

    view! {
        <div class="relative">
            // Bouton cloche
            <button
                on:click=move |_| set_open.update(|v| *v = !*v)
                class="relative p-2 rounded-lg text-gray-500 hover:bg-gray-50 hover:text-gray-700 transition duration-150"
            >
                // Icône cloche
                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                    <path stroke-linecap="round" stroke-linejoin="round"
                        d="M14.857 17.082a23.848 23.848 0 0 0 5.454-1.31A8.967 8.967 0 0 1 18 9.75V9A6 6 0 0 0 6 9v.75a8.967 8.967 0 0 1-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 0 1-5.714 0m5.714 0a3 3 0 1 1-5.714 0" />
                </svg>

                // Badge
                    <Show when=move || !alerts.get().is_empty() fallback=|| ()>
                    <span class=move || format!(
                        "absolute -top-1 -right-1 flex items-center justify-center w-4 h-4 text-xs font-bold text-white rounded-full {}",
                        if has_danger.get() { "bg-red-500" } else { "bg-amber-400" }
                    )>
                        {move || {
                            let count = alert_count.get();
                            if count > 9 { "9+".to_string() } else { count.to_string() }
                        }}
                    </span>
                </Show>
            </button>

            // Dropdown
            <Show when=move || open.get() fallback=|| ()>
                // Overlay transparent pour fermer — button pour iOS Safari
                <button
                    type="button"
                    class="fixed inset-0 z-30 w-full cursor-default"
                    on:click=move |_| set_open.set(false)
                />

                // Panneau — top dynamique pour tenir compte du padding safe area
                <div
                    class="fixed sm:absolute left-2 right-2 sm:left-auto sm:right-0 sm:top-auto sm:mt-2 sm:w-96 bg-white rounded-xl shadow-lg border border-gray-100 z-40 overflow-hidden"
                    style="top: calc(var(--nav-top, 0px) + 3.5rem)"
                >
                    <div class="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
                        <h3 class="text-sm font-semibold text-gray-800">"Notifications"</h3>
                        <div class="flex items-center gap-3">
                            <Show when=move || !alerts.get().is_empty() fallback=|| ()>
                                <span class="text-xs text-gray-400">
                                    {move || format!("{} alerte{}", alert_count.get(), if alert_count.get() > 1 { "s" } else { "" })}
                                </span>
                            </Show>
                            <button
                                on:click=move |_| set_open.set(false)
                                class="text-gray-400 hover:text-gray-600 p-1 transition duration-150"
                            >
                                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>
                    </div>

                    <div class="max-h-80 overflow-y-auto">
                        <Show
                            when=move || alert_count.get() == 0
                            fallback=|| ()
                        >
                            <div class="flex flex-col items-center justify-center py-8 text-center px-4">
                                <svg class="w-8 h-8 text-gray-300 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M14.857 17.082a23.848 23.848 0 0 0 5.454-1.31A8.967 8.967 0 0 1 18 9.75V9A6 6 0 0 0 6 9v.75a8.967 8.967 0 0 1-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 0 1-5.714 0m5.714 0a3 3 0 1 1-5.714 0" />
                                </svg>
                                <p class="text-sm text-gray-400">"Aucune alerte"</p>
                                <p class="text-xs text-gray-300 mt-1">"Tous vos contrats sont dans les limites."</p>
                            </div>
                        </Show>

                        {move || alerts.get().into_iter().map(|alert| {
                            let (bg, border, icon_color, icon) = match alert.level {
                                AlertLevel::Danger => (
                                    "bg-red-50",
                                    "border-red-100",
                                    "text-red-500",
                                    "M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z",
                                ),
                                AlertLevel::Warning => (
                                    "bg-amber-50",
                                    "border-amber-100",
                                    "text-amber-500",
                                    "M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z",
                                ),
                            };

                            view! {
                                <div class=format!("flex gap-3 px-4 py-3 border-b {} {}", border, bg)>
                                    <div class="shrink-0 mt-0.5">
                                        <svg class=format!("w-4 h-4 {}", icon_color) fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                            <path stroke-linecap="round" stroke-linejoin="round" d=icon />
                                        </svg>
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <p class="text-xs font-semibold text-gray-700 truncate">
                                            {alert.vehicle_name}
                                        </p>
                                        <p class="text-xs text-gray-600 mt-0.5">
                                            {alert.message}
                                        </p>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </Show>
        </div>
    }
}

// ─── Helpers ─────────────────────────────────────────────────────

fn get_token() -> Option<String> {
    leptos::window()
        .local_storage()
        .ok()?
        .and_then(|s| s.get_item("jwt_token").ok()?)
}

fn format_km(km: i32) -> String {
    let s = km.to_string();
    let chars: Vec<char> = s.chars().collect();
    let formatted = chars
        .rchunks(3)
        .rev()
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\u{202F}");
    format!("{} km", formatted)
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
