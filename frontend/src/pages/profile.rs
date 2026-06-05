// src/pages/profile.rs
use crate::components::ui::{get_token, input_class};
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::JsCast;

// ─── Types ───────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
struct UserProfile {
    id: Uuid,
    username: String,
    email: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SharedUser {
    user_id: Uuid,
    username: String,
    role: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct OwnedVehicleAccesses {
    vehicle_id: Uuid,
    make: String,
    model: String,
    plate_number: String,
    accesses: Vec<SharedUser>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SharedVehicle {
    vehicle_id: Uuid,
    make: String,
    model: String,
    plate_number: String,
    role: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ProfileShares {
    owned: Vec<OwnedVehicleAccesses>,
    shared_with_me: Vec<SharedVehicle>,
}

#[derive(Clone, Serialize, Deserialize)]
struct UserPreferences {
    notif_days_before: i32,
    notif_km_percent: i32,
    updated_once: bool,
}

// ─── Page principale ─────────────────────────────────────────────

#[component]
pub fn ProfilePage() -> impl IntoView {
    let navigate = use_navigate();
    let (profile, set_profile) = create_signal(Option::<UserProfile>::None);
    let (shares, set_shares) = create_signal(Option::<ProfileShares>::None);
    let (preferences, set_preferences) = create_signal(Option::<UserPreferences>::None);
    let (loading, set_loading) = create_signal(true);

    create_effect(move |_| {
        let token = get_token();
        let Some(token) = token else {
            navigate("/", NavigateOptions::default());
            return;
        };

        spawn_local(async move {
            let p = fetch_json::<UserProfile>(
                &format!("{}/api/profile", crate::config::API_BASE),
                &token,
            )
            .await;
            let s = fetch_json::<ProfileShares>(
                &format!("{}/api/profile/shares", crate::config::API_BASE),
                &token,
            )
            .await;
            let pref = fetch_json::<UserPreferences>(
                &format!("{}/api/profile/preferences", crate::config::API_BASE),
                &token,
            )
            .await;

            if let Ok(p) = p {
                set_profile.set(Some(p));
            }
            if let Ok(s) = s {
                set_shares.set(Some(s));
            }
            if let Ok(pref) = pref {
                set_preferences.set(Some(pref));
            }
            set_loading.set(false);
        });
    });

    let reload_shares = move || {
        if let Some(token) = get_token() {
            spawn_local(async move {
                if let Ok(s) = fetch_json::<ProfileShares>(
                    &format!("{}/api/profile/shares", crate::config::API_BASE),
                    &token,
                )
                .await
                {
                    set_shares.set(Some(s));
                }
            });
        }
    };

    view! {
        <div class="min-h-screen bg-gray-100">
            <nav class="bg-white shadow-sm border-b border-gray-200">
                <div class="max-w-4xl mx-auto px-4 h-14 md:h-16 flex items-center justify-between">
                    <A href="/mainpage"
                        class="flex items-center gap-2 text-indigo-600 hover:text-indigo-700 font-medium text-sm transition duration-150"
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
                        </svg>
                        "Retour"
                    </A>
                    <span class="text-xl font-bold text-indigo-600">"LimTrack"</span>
                    <div class="w-20" />
                </div>
            </nav>

            <div class="max-w-4xl mx-auto px-4 py-4 md:py-8 space-y-4 md:space-y-8">
                <Show when=move || loading.get() fallback=|| ()>
                    <div class="flex justify-center py-12">
                        <p class="text-gray-400 animate-pulse">"Chargement..."</p>
                    </div>
                </Show>

                <Show when=move || !loading.get() fallback=|| ()>
                    {move || profile.get().map(|p| view! {
                        <ProfileInfoSection profile=p />
                    })}

                    <ChangePasswordSection />

                    {move || preferences.get().map(|pref| view! {
                        <PreferencesSection
                            preferences=pref
                            on_saved=Callback::new(move |updated| set_preferences.set(Some(updated)))
                        />
                    })}

                    {move || shares.get().map(|s| view! {
                        <SharesSection shares=s on_change=reload_shares />
                    })}

                    // Section Licence masquée sur iOS — accès inclus dans l'achat App Store
                    <Show when=move || !crate::config::is_tauri() fallback=|| ()>
                        <LicenseSection />
                    </Show>
                    // Section Flotte masquée sur iOS Personal
                    <Show when=move || !crate::config::is_tauri() fallback=|| ()>
                        <FleetSection />
                    </Show>
                    <DeleteAccountSection />
                </Show>
            </div>
        </div>
    }
}

// ─── Section Informations ─────────────────────────────────────────

#[component]
fn ProfileInfoSection(profile: UserProfile) -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-4">
            <h2 class="text-lg font-bold text-gray-900">"Mes informations"</h2>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="bg-gray-50 rounded-lg p-4">
                    <p class="text-xs text-gray-400 uppercase tracking-wide mb-1">"Nom d'utilisateur"</p>
                    <p class="text-sm font-semibold text-gray-800">{profile.username}</p>
                </div>
                <div class="bg-gray-50 rounded-lg p-4">
                    <p class="text-xs text-gray-400 uppercase tracking-wide mb-1">"Email"</p>
                    <p class="text-sm font-semibold text-gray-800">{profile.email}</p>
                </div>
            </div>
        </div>
    }
}

// ─── Section Mot de passe ─────────────────────────────────────────

#[component]
fn ChangePasswordSection() -> impl IntoView {
    let (current, set_current) = create_signal(String::new());
    let (new_pass, set_new_pass) = create_signal(String::new());
    let (confirm, set_confirm) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());
    let (success, set_success) = create_signal(false);

    let submit = create_action(
        move |(current, new_pass, confirm): &(String, String, String)| {
            let (current, new_pass, confirm) = (current.clone(), new_pass.clone(), confirm.clone());
            async move {
                set_error.set(String::new());
                set_success.set(false);

                if new_pass != confirm {
                    set_error.set("Les mots de passe ne correspondent pas.".to_string());
                    return;
                }
                let token = get_token().unwrap_or_default();
                let body = serde_json::json!({
                    "current_password": current,
                    "new_password":     new_pass,
                });

                match post_json(
                    &format!("{}/api/profile/password", crate::config::API_BASE),
                    &token,
                    &body,
                )
                .await
                {
                    Ok(_) => {
                        set_success.set(true);
                        set_current.set(String::new());
                        set_new_pass.set(String::new());
                        set_confirm.set(String::new());
                    }
                    Err(e) => set_error.set(e),
                }
            }
        },
    );

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        submit.dispatch((current.get(), new_pass.get(), confirm.get()));
    };

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-4">
            <h2 class="text-lg font-bold text-gray-900">"Modifier le mot de passe"</h2>
            <form on:submit=on_submit class="space-y-4">
                <div class="space-y-1">
                    <label class="text-sm font-medium text-gray-700 block">"Mot de passe actuel"</label>
                    <input type="password" required prop:value=current
                        on:input=move |ev| set_current.set(event_target_value(&ev))
                        class=input_class() />
                </div>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div class="space-y-1">
                        <label class="text-sm font-medium text-gray-700 block">"Nouveau mot de passe"</label>
                        <input type="password" required prop:value=new_pass
                            on:input=move |ev| set_new_pass.set(event_target_value(&ev))
                            placeholder="Choisissez un mot de passe robuste"
                            class=input_class() />
                    </div>
                    <div class="space-y-1">
                        <label class="text-sm font-medium text-gray-700 block">"Confirmer"</label>
                        <input type="password" required prop:value=confirm
                            on:input=move |ev| set_confirm.set(event_target_value(&ev))
                            class=input_class() />
                    </div>
                </div>
                <p class="text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded px-3 py-2 leading-relaxed">
                    "Le mot de passe doit être suffisamment complexe (score \u{2265}3/4). "
                    "Mélangez majuscules, chiffres et symboles. "
                    "Évitez les prénoms, dates et mots courants."
                </p>
                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-red-600">{move || error.get()}</p>
                </Show>
                <Show when=move || success.get() fallback=|| ()>
                    <p class="text-sm text-green-600 font-medium">"Mot de passe modifié avec succès !"</p>
                </Show>
                <button
                    type="submit"
                    prop:disabled=move || submit.pending().get()
                    class="px-6 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                >
                    {move || if submit.pending().get() { "Modification..." } else { "Modifier" }}
                </button>
            </form>
        </div>
    }
}

// ─── Section Préférences ──────────────────────────────────────────

#[component]
fn PreferencesSection(
    preferences: UserPreferences,
    on_saved: Callback<UserPreferences>,
) -> impl IntoView {
    let (days, set_days) = create_signal(preferences.notif_days_before.to_string());
    let (percent, set_percent) = create_signal(preferences.notif_km_percent.to_string());
    let (error, set_error) = create_signal(String::new());
    let (success, set_success) = create_signal(false);

    let submit = create_action(move |(days, percent): &(String, String)| {
        let (days, percent) = (days.clone(), percent.clone());
        async move {
            set_error.set(String::new());
            set_success.set(false);

            let days_val = days.parse::<i32>().unwrap_or(0);
            let percent_val = percent.parse::<i32>().unwrap_or(0);

            if days_val < 1 || days_val > 365 {
                set_error.set("Les jours doivent être entre 1 et 365.".to_string());
                return;
            }
            if percent_val < 1 || percent_val > 100 {
                set_error.set("Le pourcentage doit être entre 1 et 100.".to_string());
                return;
            }

            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({
                "notif_days_before": days_val,
                "notif_km_percent":  percent_val,
            });

            match put_json(
                &format!("{}/api/profile/preferences", crate::config::API_BASE),
                &token,
                &body,
            )
            .await
            {
                Ok(_) => {
                    set_success.set(true);
                    on_saved.call(UserPreferences {
                        notif_days_before: days_val,
                        notif_km_percent: percent_val,
                        updated_once: true,
                    });
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        submit.dispatch((days.get(), percent.get()));
    };

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-4 md:space-y-6">
            <div>
                <h2 class="text-lg font-bold text-gray-900">"Préférences de notification"</h2>
                <p class="text-sm text-gray-500 mt-1">
                    "Définissez les seuils à partir desquels vous souhaitez être alerté."
                </p>
            </div>

            <form on:submit=on_submit class="space-y-5">
                // Seuil jours
                <div class="space-y-2">
                    <label class="text-sm font-medium text-gray-700 block">
                        "Alerter quand l'échéance est dans moins de "
                        <span class="text-indigo-600 font-bold">{move || days.get()}</span>
                        " jours"
                    </label>
                    <div class="flex items-center gap-4">
                        <input
                            type="range" min="1" max="180" step="1"
                            prop:value=days
                            on:input=move |ev| set_days.set(event_target_value(&ev))
                            class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"
                        />
                        <input
                            type="number" min="1" max="180"
                            prop:value=days
                            on:input=move |ev| set_days.set(event_target_value(&ev))
                            class="w-20 px-2 py-1.5 border border-gray-300 rounded-md text-sm text-center focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                        />
                    </div>
                    <p class="text-xs text-gray-400">"Par défaut : 30 jours"</p>
                </div>

                // Seuil kilométrage
                <div class="space-y-2">
                    <label class="text-sm font-medium text-gray-700 block">
                        "Alerter quand "
                        <span class="text-indigo-600 font-bold">{move || percent.get()}</span>
                        "% du kilométrage autorisé est atteint"
                    </label>
                    <div class="flex items-center gap-4">
                        <input
                            type="range" min="1" max="100" step="1"
                            prop:value=percent
                            on:input=move |ev| set_percent.set(event_target_value(&ev))
                            class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"
                        />
                        <input
                            type="number" min="1" max="100"
                            prop:value=percent
                            on:input=move |ev| set_percent.set(event_target_value(&ev))
                            class="w-20 px-2 py-1.5 border border-gray-300 rounded-md text-sm text-center focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                        />
                    </div>
                    <p class="text-xs text-gray-400">"Par défaut : 80%"</p>
                </div>

                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-red-600">{move || error.get()}</p>
                </Show>
                <Show when=move || success.get() fallback=|| ()>
                    <p class="text-sm text-green-600 font-medium">"Préférences enregistrées !"</p>
                </Show>

                <button
                    type="submit"
                    prop:disabled=move || submit.pending().get()
                    class="px-6 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                >
                    {move || if submit.pending().get() { "Enregistrement..." } else { "Enregistrer" }}
                </button>
            </form>
        </div>
    }
}

// ─── Section Partages ─────────────────────────────────────────────

#[component]
fn SharesSection(shares: ProfileShares, on_change: impl Fn() + 'static + Copy) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-4">
                <h2 class="text-lg font-bold text-gray-900">"Mes véhicules partagés"</h2>
                {if shares.owned.is_empty() {
                    view! {
                        <p class="text-sm text-gray-400 italic">"Aucun véhicule partagé."</p>
                    }.into_view()
                } else {
                    shares.owned.into_iter().map(|v| view! {
                        <OwnedVehicleCard vehicle=v on_revoke=on_change />
                    }).collect_view()
                }}
            </div>

            <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-4">
                <h2 class="text-lg font-bold text-gray-900">"Véhicules partagés avec moi"</h2>
                {if shares.shared_with_me.is_empty() {
                    view! {
                        <p class="text-sm text-gray-400 italic">"Aucun véhicule partagé avec vous."</p>
                    }.into_view()
                } else {
                    shares.shared_with_me.into_iter().map(|v| view! {
                        <SharedVehicleCard
                            vehicle=v
                            on_leave=Callback::new(move |_| on_change())
                        />
                    }).collect_view()
                }}
            </div>
        </div>
    }
}

// ─── Carte véhicule possédé ───────────────────────────────────────

#[component]
fn OwnedVehicleCard(
    vehicle: OwnedVehicleAccesses,
    on_revoke: impl Fn() + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="border border-gray-100 rounded-xl p-4 space-y-3">
            <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-indigo-50 flex items-center justify-center shrink-0">
                    <svg class="w-4 h-4 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                    </svg>
                </div>
                <div>
                    <p class="text-sm font-bold text-gray-800">
                        {format!("{} {}", vehicle.make, vehicle.model)}
                    </p>
                    <p class="text-xs font-mono text-indigo-600">{vehicle.plate_number}</p>
                </div>
            </div>

            {if vehicle.accesses.is_empty() {
                view! {
                    <p class="text-xs text-gray-400 italic pl-11">"Aucun utilisateur partagé."</p>
                }.into_view()
            } else {
                let vehicle_id = vehicle.vehicle_id;
                vehicle.accesses.into_iter().map(move |user| {
                    let user_id = user.user_id;
                    let role_label = match user.role.as_str() {
                        "editor" => ("Éditeur", "bg-amber-100 text-amber-700"),
                        _        => ("Lecteur", "bg-gray-100 text-gray-600"),
                    };
                    view! {
                        <div class="flex items-center justify-between pl-11 py-1">
                            <div class="flex items-center gap-2">
                                <span class="text-sm text-gray-700">{user.username}</span>
                                <span class=format!(
                                    "text-xs px-2 py-0.5 rounded-full font-medium {}",
                                    role_label.1
                                )>
                                    {role_label.0}
                                </span>
                            </div>
                            <button
                                on:click=move |_| {
                                    spawn_local(async move {
                                        let token = get_token().unwrap_or_default();
                                        let url = format!("{}/api/vehicles/{}/access/{}", crate::config::API_BASE, vehicle_id, user_id);
                                        if delete_request(&url, &token).await.is_ok() {
                                            on_revoke();
                                        }
                                    });
                                }
                                class="text-xs px-3 py-1 rounded-lg border border-red-200 text-red-600 hover:bg-red-50 transition duration-150"
                            >
                                "Révoquer"
                            </button>
                        </div>
                    }
                }).collect_view()
            }}
        </div>
    }
}

// ─── Carte véhicule partagé avec moi ─────────────────────────────

#[component]
fn SharedVehicleCard(vehicle: SharedVehicle, on_leave: Callback<()>) -> impl IntoView {
    let vehicle_id = vehicle.vehicle_id;
    let role_label = match vehicle.role.as_str() {
        "editor" => ("Éditeur", "bg-amber-100 text-amber-700"),
        _ => ("Lecteur", "bg-gray-100 text-gray-600"),
    };

    view! {
        <div class="flex items-center justify-between border border-gray-100 rounded-xl p-4">
            <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-gray-50 flex items-center justify-center shrink-0">
                    <svg class="w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                    </svg>
                </div>
                <div>
                    <p class="text-sm font-bold text-gray-800">
                        {format!("{} {}", vehicle.make, vehicle.model)}
                    </p>
                    <div class="flex items-center gap-2 mt-0.5">
                        <p class="text-xs font-mono text-indigo-600">{vehicle.plate_number}</p>
                        <span class=format!(
                            "text-xs px-2 py-0.5 rounded-full font-medium {}",
                            role_label.1
                        )>
                            {role_label.0}
                        </span>
                    </div>
                </div>
            </div>
            <button
                on:click=move |_| {
                    spawn_local(async move {
                        let token = get_token().unwrap_or_default();
                        let url   = format!("{}/api/vehicles/{}/leave", crate::config::API_BASE, vehicle_id);
                        if delete_request(&url, &token).await.is_ok() {
                            on_leave.call(());
                        }
                    });
                }
                class="text-xs px-3 py-1.5 rounded-lg border border-red-200 text-red-600 hover:bg-red-50 transition duration-150"
            >
                "Quitter"
            </button>
        </div>
    }
}

// ─── Section Licence ─────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct LicenseStatus {
    status: String,
    trial_ends_at: String,
    access_expires_at: Option<String>,
    days_until_expiry: Option<i64>,
    license_type: String,
}

#[component]
fn LicenseSection() -> impl IntoView {
    let (license, set_license) = create_signal(Option::<LicenseStatus>::None);
    let (token_input, set_token_input) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());
    let (success, set_success) = create_signal(String::new());

    let reload_license = move || {
        if let Some(jwt) = get_token() {
            spawn_local(async move {
                if let Ok(l) = fetch_json::<LicenseStatus>(
                    &format!("{}/api/profile/license", crate::config::API_BASE),
                    &jwt,
                )
                .await
                {
                    set_license.set(Some(l));
                }
            });
        }
    };

    create_effect(move |_| reload_license());

    let redeem = create_action(move |token: &String| {
        let token = token.clone();
        async move {
            set_error.set(String::new());
            set_success.set(String::new());

            if token.trim().is_empty() {
                set_error.set("Veuillez saisir un jeton.".to_string());
                return;
            }

            let jwt = get_token().unwrap_or_default();
            let body = serde_json::json!({ "token": token });

            match post_json(
                &format!("{}/api/profile/redeem", crate::config::API_BASE),
                &jwt,
                &body,
            )
            .await
            {
                Ok(_) => {
                    set_token_input.set(String::new());
                    set_success.set("Jeton activé avec succès !".to_string());
                    reload_license();
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        redeem.dispatch(token_input.get());
    };

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-4">
            <div>
                <h2 class="text-lg font-bold text-gray-900">"Licence"</h2>
                <p class="text-sm text-gray-500 mt-1">
                    "Période d'essai gratuite de 3 mois, puis activation par jeton."
                </p>
            </div>

            // Statut licence
            {move || license.get().map(|l| {
                let (badge_cls, badge_lbl, date_label) = match l.status.as_str() {
                    "active" => (
                        "bg-green-100 text-green-700",
                        "Active",
                        format!("Expire le {}", fmt_date(&l.access_expires_at.unwrap_or_default())),
                    ),
                    "trial" => (
                        "bg-amber-100 text-amber-700",
                        "Période d'essai",
                        format!("Essai jusqu'au {}", fmt_date(&l.trial_ends_at)),
                    ),
                    _ => (
                        "bg-red-100 text-red-700",
                        "Expirée",
                        "Activez un jeton pour continuer à utiliser LimTrack.".to_string(),
                    ),
                };
                view! {
                    <div class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                        <span class=format!("text-xs font-semibold px-2.5 py-1 rounded-full {}", badge_cls)>
                            {badge_lbl}
                        </span>
                        <span class="text-sm text-gray-600">{date_label}</span>
                    </div>
                }
            })}

            // Formulaire jeton
            <form on:submit=on_submit class="space-y-3">
                <div class="space-y-1">
                    <label class="text-sm font-medium text-gray-700 block">"Activer un jeton"</label>
                    <div class="flex gap-2">
                        <input
                            type="text"
                            prop:value=token_input
                            on:input=move |ev| set_token_input.set(event_target_value(&ev))
                            placeholder="XXXX-XXXX-XXXX-XXXX"
                            class="flex-1 font-mono uppercase appearance-none px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                        />
                        <button
                            type="submit"
                            prop:disabled=move || redeem.pending().get()
                            class="px-4 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150 shrink-0"
                        >
                            {move || if redeem.pending().get() { "Activation..." } else { "Activer" }}
                        </button>
                    </div>
                </div>
                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-red-600">{move || error.get()}</p>
                </Show>
                <Show when=move || !success.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-green-600 font-medium">{move || success.get()}</p>
                </Show>
            </form>
        </div>
    }
}

fn fmt_date(iso: &str) -> String {
    // ISO 8601 → "JJ/MM/AAAA"
    iso.get(..10)
        .map(|d| {
            let parts: Vec<&str> = d.split('-').collect();
            if parts.len() == 3 {
                format!("{}/{}/{}", parts[2], parts[1], parts[0])
            } else {
                d.to_string()
            }
        })
        .unwrap_or_else(|| iso.to_string())
}

// ─── Section Suppression compte ──────────────────────────────────

#[component]
fn DeleteAccountSection() -> impl IntoView {
    let navigate = use_navigate();
    let (password, set_password) = create_signal(String::new());
    let (confirm_text, set_confirm_text) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());
    let (show_modal, set_show_modal) = create_signal(false);

    let confirm_ok = create_memo(move |_| confirm_text.get().trim() == "SUPPRIMER");

    let submit = create_action(move |password: &String| {
        let password = password.clone();
        let navigate = navigate.clone();
        async move {
            set_error.set(String::new());

            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({
                "current_password": password,
                "new_password": password, // champ requis par le struct, ignoré côté serveur
            });

            match delete_json(
                &format!("{}/api/profile", crate::config::API_BASE),
                &token,
                &body,
            )
            .await
            {
                Ok(_) => {
                    // Supprimer le token et rediriger
                    if let Ok(Some(storage)) = leptos::window().local_storage() {
                        let _ = storage.remove_item("jwt_token");
                    }
                    navigate("/", NavigateOptions::default());
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        submit.dispatch(password.get());
    };

    view! {
        <div class="bg-white rounded-xl border border-red-200 shadow-sm p-4 md:p-6 space-y-4">
            <div>
                <h2 class="text-lg font-bold text-red-600">"Zone dangereuse"</h2>
                <p class="text-sm text-gray-500 mt-1">
                    "La suppression de votre compte est irréversible. Tous vos véhicules, contrats et relevés seront définitivement supprimés."
                </p>
            </div>

            <button
                on:click=move |_| set_show_modal.set(true)
                class="px-4 py-2 rounded-md text-sm font-medium text-white bg-red-600 hover:bg-red-700 transition duration-150"
            >
                "Supprimer mon compte"
            </button>

            <Show when=move || show_modal.get() fallback=|| ()>
                // Overlay
                <button
                    type="button"
                    class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm w-full cursor-default"
                    on:click=move |_| {
                        set_show_modal.set(false);
                        set_error.set(String::new());
                        set_password.set(String::new());
                        set_confirm_text.set(String::new());
                    }
                />

                // Modal
                <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
                    <div class="bg-white rounded-2xl shadow-2xl border border-red-200 w-full max-w-md p-8 space-y-6">

                        <div class="flex items-center justify-between">
                            <div>
                                <h2 class="text-xl font-bold text-gray-900">"Supprimer le compte"</h2>
                                <p class="text-sm text-gray-500 mt-1">"Cette action est irréversible"</p>
                            </div>
                            <button
                                on:click=move |_| {
                                    set_show_modal.set(false);
                                    set_error.set(String::new());
                                    set_password.set(String::new());
                                    set_confirm_text.set(String::new());
                                }
                                class="text-gray-400 hover:text-gray-600 text-xl font-light"
                            >"✕"</button>
                        </div>

                        <div class="bg-red-50 border border-red-200 rounded-xl p-4 space-y-2">
                            <p class="text-sm font-semibold text-red-700">"⚠ Action irréversible"</p>
                            <p class="text-xs text-red-600">
                                "Tous vos véhicules, contrats LOA, contrats d'assurance, relevés kilométriques et accès partagés seront définitivement supprimés."
                            </p>
                        </div>

                        <form on:submit=on_submit class="space-y-4">
                            <div class="space-y-1">
                                <label class="text-sm font-medium text-gray-700 block">
                                    "Mot de passe actuel"
                                </label>
                                <input
                                    type="password"
                                    required
                                    prop:value=password
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-red-500 focus:border-red-500 sm:text-sm transition duration-150"
                                />
                            </div>

                            <div class="space-y-1">
                                <label class="text-sm font-medium text-gray-700 block">
                                    "Tapez " <span class="font-mono font-bold text-red-600">"SUPPRIMER"</span> " pour confirmer"
                                </label>
                                <input
                                    type="text"
                                    required
                                    prop:value=confirm_text
                                    on:input=move |ev| set_confirm_text.set(event_target_value(&ev))
                                    placeholder="SUPPRIMER"
                                    class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-red-500 focus:border-red-500 sm:text-sm font-mono transition duration-150"
                                />
                            </div>

                            <Show when=move || !error.get().is_empty() fallback=|| ()>
                                <p class="text-sm text-center text-red-600">{move || error.get()}</p>
                            </Show>

                            <div class="flex gap-3 pt-2">
                                <button
                                    type="button"
                                    on:click=move |_| {
                                        set_show_modal.set(false);
                                        set_error.set(String::new());
                                        set_password.set(String::new());
                                        set_confirm_text.set(String::new());
                                    }
                                    class="flex-1 py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150"
                                >
                                    "Annuler"
                                </button>
                                <button
                                    type="submit"
                                    prop:disabled=move || !confirm_ok.get() || submit.pending().get()
                                    class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-red-600 hover:bg-red-700 disabled:opacity-40 disabled:cursor-not-allowed transition duration-150"
                                >
                                    {move || if submit.pending().get() { "Suppression..." } else { "Supprimer définitivement" }}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
        </div>
    }
}

// ─── Section Flotte (point d'entrée discret) ──────────────────────

#[derive(Clone, Serialize, Deserialize)]
struct CompanyBrief {
    id: Uuid,
    name: String,
    my_role: Option<String>,
}

#[component]
fn FleetSection() -> impl IntoView {
    let (companies, set_companies) = create_signal(Vec::<CompanyBrief>::new());
    let (loaded, set_loaded) = create_signal(false);
    let (has_fleet_license, set_has_fleet_license) = create_signal(false);

    create_effect(move |_| {
        if let Some(token) = get_token() {
            let token_lic = token.clone();
            spawn_local(async move {
                // Vérifier le type de licence avant d'afficher la section
                if let Ok(lic) = fetch_json::<LicenseStatus>(
                    &format!("{}/api/profile/license", crate::config::API_BASE),
                    &token_lic,
                )
                .await
                {
                    let is_fleet = lic.license_type == "fleet"
                        && (lic.status == "active" || lic.status == "trial");
                    set_has_fleet_license.set(is_fleet);

                    if is_fleet {
                        if let Ok(list) = fetch_json::<Vec<CompanyBrief>>(
                            &format!("{}/api/companies", crate::config::API_BASE),
                            &token_lic,
                        )
                        .await
                        {
                            set_companies.set(list);
                        }
                    }
                }
                set_loaded.set(true);
            });
        }
    });

    view! {
        <Show when=move || loaded.get() && has_fleet_license.get() fallback=|| ()>
            <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-3">
                <div class="flex items-center justify-between">
                    <h2 class="text-lg font-bold text-gray-900">"Gestion de flotte"</h2>
                    <A href="/fleet"
                        class="text-sm font-medium text-indigo-600 hover:text-indigo-700 flex items-center gap-1 transition duration-150"
                    >
                        {move || if companies.get().is_empty() { "Créer une entreprise" } else { "Accéder à la flotte" }}
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5 21 12m0 0-7.5 7.5M21 12H3" />
                        </svg>
                    </A>
                </div>

                <Show when=move || companies.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-gray-400 italic">
                        "Aucune entreprise. Créez-en une pour gérer une flotte de véhicules."
                    </p>
                </Show>

                <Show when=move || !companies.get().is_empty() fallback=|| ()>
                    <div class="space-y-2">
                        <For
                            each=move || companies.get()
                            key=|c| c.id
                            children=move |c| {
                                let role_badge = c.my_role.as_deref().map(|r| {
                                    let (lbl, cls) = if r == "fleet_admin" {
                                        ("Admin", "bg-indigo-100 text-indigo-700")
                                    } else {
                                        ("Lecteur", "bg-gray-100 text-gray-600")
                                    };
                                    view! {
                                        <span class=format!("text-xs px-2 py-0.5 rounded-full font-medium {}", cls)>{lbl}</span>
                                    }.into_view()
                                });
                                view! {
                                    <div class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                                        <svg class="w-4 h-4 text-indigo-400 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                            <path stroke-linecap="round" stroke-linejoin="round"
                                                d="M3.75 21h16.5M4.5 3h15M5.25 3v18m13.5-18v18M9 6.75h1.5m-1.5 3h1.5m-1.5 3h1.5m3-6H15m-1.5 3H15m-1.5 3H15M9 21v-3.375c0-.621.504-1.125 1.125-1.125h3.75c.621 0 1.125.504 1.125 1.125V21" />
                                        </svg>
                                        <span class="text-sm font-medium text-gray-800 flex-1">{c.name}</span>
                                        {role_badge}
                                    </div>
                                }
                            }
                        />
                    </div>
                </Show>
            </div>
        </Show>
    }
}

// ─── Helpers réseau ───────────────────────────────────────────────

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
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
            .await
            .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;
    if resp.ok() || resp.status() == 200 {
        Ok(())
    } else {
        let json =
            wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
                .await
                .ok();
        let msg = json
            .and_then(|j| serde_wasm_bindgen::from_value::<serde_json::Value>(j).ok())
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| e.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| format!("Erreur HTTP : {}", resp.status()));
        Err(msg)
    }
}

async fn put_json(url: &str, token: &str, body: &serde_json::Value) -> Result<(), String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("PUT");
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
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
    if resp.ok() || resp.status() == 200 {
        Ok(())
    } else {
        let json =
            wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
                .await
                .ok();
        let msg = json
            .and_then(|j| serde_wasm_bindgen::from_value::<serde_json::Value>(j).ok())
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| e.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| format!("Erreur HTTP : {}", resp.status()));
        Err(msg)
    }
}

async fn delete_request(url: &str, token: &str) -> Result<(), String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("DELETE");
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
    opts.headers(&headers);
    let req =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
            .await
            .map_err(|e| format!("{:?}", e))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{:?}", e))?;
    if resp.ok() || resp.status() == 204 {
        Ok(())
    } else {
        Err(format!("Erreur HTTP : {}", resp.status()))
    }
}

async fn delete_json(url: &str, token: &str, body: &serde_json::Value) -> Result<(), String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("DELETE");
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
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
    if resp.ok() || resp.status() == 204 {
        Ok(())
    } else {
        let json =
            wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
                .await
                .ok();
        let msg = json
            .and_then(|j| serde_wasm_bindgen::from_value::<serde_json::Value>(j).ok())
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| e.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| format!("Erreur HTTP : {}", resp.status()));
        Err(msg)
    }
}
