use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────

#[derive(Clone, Deserialize, PartialEq)]
struct AdminStats {
    total_users: i64,
    trial: i64,
    active: i64,
    expired: i64,
    total_license_requests: i64,
}

#[derive(Clone, Deserialize, PartialEq)]
struct AdminUser {
    id: String,
    username: String,
    email: String,
    is_admin: bool,
    status: String,
    trial_ends_at: String,
    access_expires_at: Option<String>,
}

#[derive(Clone, Deserialize, PartialEq)]
struct LicenseRequest {
    email: String,
    requested_at: String,
}

#[derive(Clone, Serialize)]
struct GenerateTokenPayload {
    email: Option<String>,
    days: i32,
    license_type: String,
}

// ─── Helpers fetch ────────────────────────────────────────────

fn get_token() -> Option<String> {
    leptos::window()
        .local_storage()
        .ok()??
        .get_item("token")
        .ok()?
}

async fn api_get<T: for<'de> serde::Deserialize<'de>>(path: &str) -> Result<T, String> {
    let token = get_token().ok_or("Non connecté")?;
    let url = format!("{}{}", crate::config::API_BASE, path);
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    let headers = web_sys::Headers::new().unwrap();
    headers.set("Authorization", &format!("Bearer {}", token)).unwrap();
    opts.headers(&headers);
    let req = web_sys::Request::new_with_str_and_init(&url, &opts).map_err(|_| "Erreur requête")?;
    let resp = wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
        .await
        .map_err(|_| "Erreur réseau")?;
    use wasm_bindgen::JsCast;
    let resp: web_sys::Response = resp.dyn_into().unwrap();
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let text = wasm_bindgen_futures::JsFuture::from(resp.text().unwrap())
        .await
        .map_err(|_| "Erreur lecture")?
        .as_string()
        .unwrap_or_default();
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

async fn api_post(path: &str, body: &str) -> Result<String, String> {
    let token = get_token().ok_or("Non connecté")?;
    let url = format!("{}{}", crate::config::API_BASE, path);
    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    let headers = web_sys::Headers::new().unwrap();
    headers.set("Authorization", &format!("Bearer {}", token)).unwrap();
    headers.set("Content-Type", "application/json").unwrap();
    opts.headers(&headers);
    opts.body(Some(&wasm_bindgen::JsValue::from_str(body)));
    let req = web_sys::Request::new_with_str_and_init(&url, &opts).map_err(|_| "Erreur requête")?;
    let resp = wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
        .await
        .map_err(|_| "Erreur réseau")?;
    use wasm_bindgen::JsCast;
    let resp: web_sys::Response = resp.dyn_into().unwrap();
    let text = wasm_bindgen_futures::JsFuture::from(resp.text().unwrap())
        .await
        .map_err(|_| "Erreur lecture")?
        .as_string()
        .unwrap_or_default();
    if !resp.ok() {
        let msg = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
            .unwrap_or(text);
        return Err(msg);
    }
    Ok(text)
}

// ─── Composant badge statut ───────────────────────────────────

#[component]
fn StatusBadge(status: String) -> impl IntoView {
    let (cls, label) = match status.as_str() {
        "active"  => ("bg-green-100 text-green-700",  "Actif"),
        "trial"   => ("bg-blue-100 text-blue-700",    "Essai"),
        _         => ("bg-red-100 text-red-700",      "Expiré"),
    };
    view! { <span class=format!("px-2 py-0.5 rounded-full text-xs font-semibold {}", cls)>{label}</span> }
}

// ─── Page admin ───────────────────────────────────────────────

#[component]
pub fn AdminPage() -> impl IntoView {
    let stats    = create_resource(|| (), |_| async { api_get::<AdminStats>("/api/admin/stats").await });
    let users    = create_resource(|| (), |_| async { api_get::<Vec<AdminUser>>("/api/admin/users").await });
    let requests = create_resource(|| (), |_| async { api_get::<Vec<LicenseRequest>>("/api/admin/license-requests").await });

    // Formulaire génération token
    let (gen_email, set_gen_email)     = create_signal(String::new());
    let (gen_days, set_gen_days)       = create_signal(365i32);
    let (gen_type, set_gen_type)       = create_signal("personal".to_string());
    let (gen_result, set_gen_result)   = create_signal(Option::<Result<String, String>>::None);
    let (gen_loading, set_gen_loading) = create_signal(false);

    let generate_action = create_action(move |(email, days, ltype): &(String, i32, String)| {
        let email = email.clone();
        let days = *days;
        let ltype = ltype.clone();
        async move {
            set_gen_loading.set(true);
            set_gen_result.set(None);
            let payload = serde_json::json!({
                "email": if email.trim().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(email) },
                "days": days,
                "license_type": ltype,
            });
            let result = api_post("/api/admin/generate-token", &payload.to_string()).await;
            set_gen_result.set(Some(result));
            set_gen_loading.set(false);
        }
    });

    view! {
        <div class="min-h-screen bg-gray-100">

            // ─── Navbar ──────────────────────────────────────
            <nav class="bg-white shadow-sm border-b border-gray-200">
                <div class="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between">
                    <A href="/mainpage" class="flex items-center gap-2 text-indigo-600 hover:text-indigo-700 font-medium text-sm">
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
                        </svg>
                        "Retour"
                    </A>
                    <span class="text-xl font-bold text-indigo-600">"Dashboard Admin"</span>
                    <div class="w-20" />
                </div>
            </nav>

            <div class="max-w-6xl mx-auto px-4 py-6 space-y-6">

                // ─── Stats ───────────────────────────────────
                <Suspense fallback=|| view! { <p class="text-sm text-gray-400">"Chargement..."</p> }>
                    {move || stats.get().map(|res| match res {
                        Err(e) => view! { <p class="text-sm text-red-600">{format!("Erreur : {}", e)}</p> }.into_view(),
                        Ok(s) => view! {
                            <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
                                {[
                                    ("Utilisateurs", s.total_users, "text-gray-900"),
                                    ("En essai", s.trial, "text-blue-600"),
                                    ("Actifs", s.active, "text-green-600"),
                                    ("Expirés", s.expired, "text-red-600"),
                                    ("Demandes", s.total_license_requests, "text-indigo-600"),
                                ].into_iter().map(|(label, val, cls)| view! {
                                    <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 text-center">
                                        <p class=format!("text-3xl font-bold {}", cls)>{val}</p>
                                        <p class="text-xs text-gray-500 mt-1">{label}</p>
                                    </div>
                                }).collect_view()}
                            </div>
                        }.into_view(),
                    })}
                </Suspense>

                // ─── Générer un token ─────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 space-y-4">
                    <h2 class="text-base font-bold text-gray-900">"Générer un jeton"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
                        <div class="md:col-span-2 space-y-1">
                            <label class="text-xs font-medium text-gray-600 block">"Email (optionnel — assigne directement)"</label>
                            <input
                                type="email"
                                prop:value=gen_email
                                on:input=move |ev| set_gen_email.set(event_target_value(&ev))
                                placeholder="user@example.com"
                                class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                            />
                        </div>
                        <div class="space-y-1">
                            <label class="text-xs font-medium text-gray-600 block">"Durée"</label>
                            <select
                                on:change=move |ev| {
                                    if let Ok(d) = event_target_value(&ev).parse::<i32>() {
                                        set_gen_days.set(d);
                                    }
                                }
                                class="block w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                            >
                                <option value="30">"30 jours"</option>
                                <option value="90">"90 jours"</option>
                                <option value="180">"180 jours"</option>
                                <option value="365" selected>"365 jours"</option>
                                <option value="36500">"Lifetime"</option>
                            </select>
                        </div>
                        <div class="space-y-1">
                            <label class="text-xs font-medium text-gray-600 block">"Type"</label>
                            <select
                                on:change=move |ev| set_gen_type.set(event_target_value(&ev))
                                class="block w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                            >
                                <option value="personal" selected>"Personal"</option>
                                <option value="fleet">"Fleet"</option>
                            </select>
                        </div>
                    </div>

                    {move || gen_result.get().map(|res| match res {
                        Ok(json) => {
                            let token = serde_json::from_str::<serde_json::Value>(&json)
                                .ok()
                                .and_then(|v| v["token"].as_str().map(|s| s.to_string()))
                                .unwrap_or(json);
                            let assigned = serde_json::from_str::<serde_json::Value>(
                                &serde_json::to_string(&token).unwrap_or_default()
                            ).ok().and_then(|v| v["assigned_to"].as_str().map(|s| s.to_string()));
                            let _ = assigned;
                            view! {
                                <div class="p-3 rounded-lg bg-green-50 border border-green-200">
                                    <p class="text-xs text-green-600 font-medium mb-1">"Jeton généré :"</p>
                                    <p class="font-mono text-sm font-bold text-green-800 tracking-widest">{token}</p>
                                </div>
                            }.into_view()
                        },
                        Err(e) => view! {
                            <p class="text-sm text-red-600 p-3 rounded-lg bg-red-50 border border-red-200">{e}</p>
                        }.into_view(),
                    })}

                    <button
                        on:click=move |_| generate_action.dispatch((gen_email.get(), gen_days.get(), gen_type.get()))
                        prop:disabled=gen_loading
                        class="px-5 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-40 transition duration-150"
                    >
                        {move || if gen_loading.get() { "Génération..." } else { "Générer" }}
                    </button>
                </div>

                // ─── Liste utilisateurs ───────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
                    <div class="p-4 border-b border-gray-100">
                        <h2 class="text-base font-bold text-gray-900">"Utilisateurs"</h2>
                    </div>
                    <Suspense fallback=|| view! { <p class="p-4 text-sm text-gray-400">"Chargement..."</p> }>
                        {move || users.get().map(|res| match res {
                            Err(e) => view! { <p class="p-4 text-sm text-red-600">{format!("Erreur : {}", e)}</p> }.into_view(),
                            Ok(list) => view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-sm">
                                        <thead class="bg-gray-50 text-xs text-gray-500 uppercase">
                                            <tr>
                                                <th class="px-4 py-3 text-left">"Utilisateur"</th>
                                                <th class="px-4 py-3 text-left">"Email"</th>
                                                <th class="px-4 py-3 text-left">"Statut"</th>
                                                <th class="px-4 py-3 text-left">"Expiration"</th>
                                                <th class="px-4 py-3 text-left">"Admin"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-100">
                                            {list.into_iter().map(|u| {
                                                let expiry = u.access_expires_at
                                                    .as_deref()
                                                    .unwrap_or(&u.trial_ends_at)
                                                    .chars().take(10).collect::<String>();
                                                view! {
                                                    <tr class="hover:bg-gray-50">
                                                        <td class="px-4 py-3 font-medium text-gray-900">{u.username}</td>
                                                        <td class="px-4 py-3 text-gray-500">{u.email}</td>
                                                        <td class="px-4 py-3"><StatusBadge status=u.status /></td>
                                                        <td class="px-4 py-3 text-gray-500 font-mono text-xs">{expiry}</td>
                                                        <td class="px-4 py-3">
                                                            {if u.is_admin {
                                                                view! { <span class="text-indigo-600 font-semibold text-xs">"✓"</span> }.into_view()
                                                            } else {
                                                                view! { <span></span> }.into_view()
                                                            }}
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_view(),
                        })}
                    </Suspense>
                </div>

                // ─── Demandes de licence ──────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
                    <div class="p-4 border-b border-gray-100">
                        <h2 class="text-base font-bold text-gray-900">"Demandes de licence"</h2>
                    </div>
                    <Suspense fallback=|| view! { <p class="p-4 text-sm text-gray-400">"Chargement..."</p> }>
                        {move || requests.get().map(|res| match res {
                            Err(e) => view! { <p class="p-4 text-sm text-red-600">{format!("Erreur : {}", e)}</p> }.into_view(),
                            Ok(list) if list.is_empty() => view! {
                                <p class="p-4 text-sm text-gray-400">"Aucune demande pour le moment."</p>
                            }.into_view(),
                            Ok(list) => view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-sm">
                                        <thead class="bg-gray-50 text-xs text-gray-500 uppercase">
                                            <tr>
                                                <th class="px-4 py-3 text-left">"Email"</th>
                                                <th class="px-4 py-3 text-left">"Date"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-100">
                                            {list.into_iter().map(|r| {
                                                let date = r.requested_at.chars().take(10).collect::<String>();
                                                view! {
                                                    <tr class="hover:bg-gray-50">
                                                        <td class="px-4 py-3 text-gray-700">{r.email}</td>
                                                        <td class="px-4 py-3 text-gray-500 font-mono text-xs">{date}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_view(),
                        })}
                    </Suspense>
                </div>

            </div>
        </div>
    }
}
