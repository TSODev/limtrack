use crate::components::ui::get_token as get_jwt_token;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct AdminStats {
    total_users: i64,
    trial: i64,
    active: i64,
    expired: i64,
    total_vehicles: i64,
    total_license_requests: i64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct AdminUser {
    id: String,
    username: String,
    email: String,
    is_admin: bool,
    is_ios: bool,
    license_type: String,
    status: String,
    trial_ends_at: String,
    access_expires_at: Option<String>,
    created_at: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct GrowthWeek {
    week: String,
    count: i64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct GrowthData {
    users: Vec<GrowthWeek>,
    vehicles: Vec<GrowthWeek>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct LicenseRequest {
    email: String,
    requested_at: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct AdminCompanyMember {
    username: String,
    email: String,
    fleet_role: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct AdminCompanyVehicle {
    make: String,
    model: String,
    plate_number: String,
    org_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct AdminCompanyOrg {
    name: String,
    vehicle_count: i64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct AdminCompany {
    id: String,
    name: String,
    siret: Option<String>,
    members: Vec<AdminCompanyMember>,
    vehicles: Vec<AdminCompanyVehicle>,
    organizations: Vec<AdminCompanyOrg>,
}

// ─── HTTP helpers ─────────────────────────────────────────────

async fn api_get<T: for<'de> serde::Deserialize<'de>>(path: &str) -> Result<T, String> {
    let token = get_jwt_token().ok_or("Non connecté")?;
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
    let token = get_jwt_token().ok_or("Non connecté")?;
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

async fn api_patch(path: &str, body: &str) -> Result<(), String> {
    let token = get_jwt_token().ok_or("Non connecté")?;
    let url = format!("{}{}", crate::config::API_BASE, path);
    let mut opts = web_sys::RequestInit::new();
    opts.method("PATCH");
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
    if resp.ok() {
        return Ok(());
    }
    let text = wasm_bindgen_futures::JsFuture::from(resp.text().unwrap())
        .await
        .map_err(|_| "Erreur lecture")?
        .as_string()
        .unwrap_or_default();
    let msg = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
        .unwrap_or(text);
    Err(msg)
}

// ─── Composants badges ────────────────────────────────────────

#[component]
fn StatusBadge(status: String) -> impl IntoView {
    let (cls, label) = match status.as_str() {
        "active"  => ("bg-green-100 text-green-700",  "Actif"),
        "trial"   => ("bg-blue-100 text-blue-700",    "Essai"),
        _         => ("bg-red-100 text-red-700",      "Expiré"),
    };
    view! { <span class=format!("px-2 py-0.5 rounded-full text-xs font-semibold {}", cls)>{label}</span> }
}

#[component]
fn LicenseTypeBadge(lt: String) -> impl IntoView {
    let (cls, label) = if lt == "fleet" {
        ("bg-indigo-100 text-indigo-700", "Fleet")
    } else {
        ("bg-gray-100 text-gray-600", "Personal")
    };
    view! { <span class=format!("px-2 py-0.5 rounded-full text-xs font-medium {}", cls)>{label}</span> }
}

// ─── Merges growth data ───────────────────────────────────────

fn merge_growth(users: &[GrowthWeek], vehicles: &[GrowthWeek]) -> Vec<(String, i64, i64)> {
    let mut weeks: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for w in users    { weeks.insert(w.week.clone()); }
    for w in vehicles { weeks.insert(w.week.clone()); }
    let user_map: std::collections::HashMap<&str, i64> =
        users.iter().map(|w| (w.week.as_str(), w.count)).collect();
    let veh_map: std::collections::HashMap<&str, i64> =
        vehicles.iter().map(|w| (w.week.as_str(), w.count)).collect();
    weeks.into_iter().rev()
        .map(|w| {
            let uc = user_map.get(w.as_str()).copied().unwrap_or(0);
            let vc = veh_map.get(w.as_str()).copied().unwrap_or(0);
            (w, uc, vc)
        })
        .collect()
}

// ─── Page admin ───────────────────────────────────────────────

#[component]
pub fn AdminPage() -> impl IntoView {
    let (refresh, set_refresh) = create_signal(0u32);
    // "apercu" | "users" | "licences" | "flottes" | "generation"
    let (tab, set_tab) = create_signal("apercu".to_string());
    // Filtres
    let (user_text, set_user_text) = create_signal(String::new());
    let (user_status, set_user_status) = create_signal(String::new());
    let (license_filter, set_license_filter) = create_signal(String::new());
    let (fleet_filter, set_fleet_filter) = create_signal(String::new());
    // Édition inline utilisateur
    let (editing_id, set_editing_id) = create_signal(Option::<String>::None);
    let (edit_username, set_edit_username) = create_signal(String::new());
    let (edit_email, set_edit_email) = create_signal(String::new());
    let (edit_is_admin, set_edit_is_admin) = create_signal(false);
    let (edit_is_ios, set_edit_is_ios) = create_signal(false);
    let (edit_license_type, set_edit_license_type) = create_signal("personal".to_string());
    let (edit_expires, set_edit_expires) = create_signal(String::new());
    let (edit_error, set_edit_error) = create_signal(Option::<String>::None);
    let (edit_loading, set_edit_loading) = create_signal(false);
    // Génération token
    let (gen_email, set_gen_email)     = create_signal(String::new());
    let (gen_days, set_gen_days)       = create_signal(365i32);
    let (gen_type, set_gen_type)       = create_signal("personal".to_string());
    let (gen_result, set_gen_result)   = create_signal(Option::<Result<String, String>>::None);
    let (gen_loading, set_gen_loading) = create_signal(false);

    let stats     = create_resource(move || refresh.get(), |_| async { api_get::<AdminStats>("/api/admin/stats").await });
    let users     = create_resource(move || refresh.get(), |_| async { api_get::<Vec<AdminUser>>("/api/admin/users").await });
    let requests  = create_resource(move || refresh.get(), |_| async { api_get::<Vec<LicenseRequest>>("/api/admin/license-requests").await });
    let companies = create_resource(move || refresh.get(), |_| async { api_get::<Vec<AdminCompany>>("/api/admin/companies").await });
    let growth    = create_resource(move || refresh.get(), |_| async { api_get::<GrowthData>("/api/admin/growth").await });

    let save_edit = create_action(move |(id, u, e, ia, ii, lt, ex): &(String, String, String, bool, bool, String, String)| {
        let id = id.clone(); let u = u.clone(); let e = e.clone();
        let ia = *ia; let ii = *ii; let lt = lt.clone(); let ex = ex.clone();
        async move {
            set_edit_loading.set(true);
            set_edit_error.set(None);
            let mut payload = serde_json::json!({
                "username": u, "email": e,
                "is_admin": ia, "is_ios": ii, "license_type": lt,
            });
            if !ex.is_empty() {
                payload["access_expires_at"] = serde_json::Value::String(format!("{}T00:00:00Z", ex));
            }
            match api_patch(&format!("/api/admin/users/{}", id), &payload.to_string()).await {
                Ok(_) => { set_editing_id.set(None); set_refresh.update(|n| *n += 1); }
                Err(e) => set_edit_error.set(Some(e)),
            }
            set_edit_loading.set(false);
        }
    });

    let generate_action = create_action(move |(email, days, ltype): &(String, i32, String)| {
        let email = email.clone(); let days = *days; let ltype = ltype.clone();
        async move {
            set_gen_loading.set(true);
            set_gen_result.set(None);
            let payload = serde_json::json!({
                "email": if email.trim().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(email) },
                "days": days, "license_type": ltype,
            });
            let result = api_post("/api/admin/generate-token", &payload.to_string()).await;
            if result.is_ok() { set_refresh.update(|n| *n += 1); }
            set_gen_result.set(Some(result));
            set_gen_loading.set(false);
        }
    });

    let input_cls = "appearance-none block w-full px-2 py-1 border border-gray-300 rounded text-xs focus:outline-none focus:ring-indigo-500 focus:border-indigo-500";
    let tab_btn = move |id: &'static str, label: &'static str| {
        let tab2 = tab.clone();
        view! {
            <button
                on:click=move |_| set_tab.set(id.to_string())
                class=move || if tab2.get() == id {
                    "px-4 py-2 text-sm font-semibold text-indigo-600 border-b-2 border-indigo-600 -mb-px"
                } else {
                    "px-4 py-2 text-sm text-gray-500 hover:text-gray-700 border-b-2 border-transparent -mb-px"
                }
            >{label}</button>
        }
    };

    view! {
        <div class="min-h-screen bg-gray-100">

            // ─── Navbar + Barre d'onglets (sticky) ───────────
            <div class="sticky top-0 z-20">
                <nav class="bg-white shadow-sm border-b border-gray-200" style="padding-top: var(--nav-top)">
                    <div class="max-w-5xl mx-auto px-4 h-14 md:h-16 flex items-center justify-between">
                        <A href="/mainpage" class="flex items-center gap-2 text-indigo-600 hover:text-indigo-700 font-medium text-sm transition">
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
                            </svg>
                            "Retour"
                        </A>
                        <span class="text-xl font-bold text-indigo-600">"Dashboard Admin"</span>
                        <div class="w-20" />
                    </div>
                </nav>
                <div class="bg-white border-b border-gray-200">
                    <div class="max-w-5xl mx-auto px-4 flex gap-1 overflow-x-auto">
                        {tab_btn("apercu", "Aperçu")}
                        {tab_btn("users", "Utilisateurs")}
                        {tab_btn("licences", "Licences")}
                        {tab_btn("flottes", "Flottes")}
                        {tab_btn("generation", "Génération")}
                    </div>
                </div>
            </div>

            <div class="max-w-5xl mx-auto px-4 py-4 md:py-6 space-y-4 md:space-y-6">

                // ══════════════════════════════════════════════
                // Onglet : Aperçu
                // ══════════════════════════════════════════════
                <Show when=move || tab.get() == "apercu" fallback=|| ()>
                    // Cartes stats
                    <Suspense fallback=|| view! { <p class="text-sm text-gray-400">"Chargement..."</p> }>
                        {move || stats.get().map(|res| match res {
                            Err(e) => view! { <p class="text-sm text-red-600">{format!("Erreur : {}", e)}</p> }.into_view(),
                            Ok(s) => {
                                let cards: Vec<(&str, i64, &str, &str, &str)> = vec![
                                    ("Utilisateurs", s.total_users, "text-gray-900",  "users",    ""),
                                    ("En essai",     s.trial,       "text-blue-600",  "users",    "trial"),
                                    ("Actifs",       s.active,      "text-green-600", "users",    "active"),
                                    ("Expirés",      s.expired,     "text-red-600",   "users",    "expired"),
                                    ("Véhicules",    s.total_vehicles, "text-amber-600", "",        ""),
                                    ("Demandes",     s.total_license_requests, "text-indigo-600", "licences", ""),
                                ];
                                view! {
                                    <div class="grid grid-cols-3 md:grid-cols-6 gap-3">
                                        {cards.into_iter().map(|(label, val, cls, dest_tab, dest_status)| {
                                            let clickable = !dest_tab.is_empty();
                                            view! {
                                                <button
                                                    on:click=move |_| {
                                                        if !dest_tab.is_empty() {
                                                            set_tab.set(dest_tab.to_string());
                                                            if !dest_status.is_empty() {
                                                                set_user_status.set(dest_status.to_string());
                                                            }
                                                        }
                                                    }
                                                    class=move || format!(
                                                        "bg-white rounded-xl border border-gray-100 shadow-sm p-4 text-center {}",
                                                        if clickable { "hover:border-indigo-200 hover:shadow-md transition cursor-pointer" } else { "cursor-default" }
                                                    )
                                                >
                                                    <p class=format!("text-2xl font-bold {}", cls)>{val}</p>
                                                    <p class="text-xs text-gray-500 mt-1">{label}</p>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }
                        })}
                    </Suspense>

                    // Tableau de croissance
                    <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
                        <div class="p-4 border-b border-gray-100">
                            <h2 class="text-base font-bold text-gray-900">"Croissance (12 dernières semaines)"</h2>
                        </div>
                        <Suspense fallback=|| view! { <p class="p-4 text-sm text-gray-400">"Chargement..."</p> }>
                            {move || growth.get().map(|res| match res {
                                Err(e) => view! { <p class="p-4 text-sm text-red-600">{format!("Erreur : {}", e)}</p> }.into_view(),
                                Ok(g) => {
                                    let merged = merge_growth(&g.users, &g.vehicles);
                                    if merged.is_empty() {
                                        return view! { <p class="p-4 text-sm text-gray-400">"Aucune donnée disponible."</p> }.into_view();
                                    }
                                    let max_u = merged.iter().map(|(_, u, _)| *u).max().unwrap_or(1).max(1);
                                    let max_v = merged.iter().map(|(_, _, v)| *v).max().unwrap_or(1).max(1);
                                    view! {
                                        <div class="overflow-x-auto">
                                            <table class="w-full text-sm">
                                                <thead class="bg-gray-50 text-xs text-gray-500 uppercase">
                                                    <tr>
                                                        <th class="px-4 py-3 text-left">"Semaine"</th>
                                                        <th class="px-4 py-3 text-right">"Nouveaux utilisateurs"</th>
                                                        <th class="px-4 py-3 text-left w-24"></th>
                                                        <th class="px-4 py-3 text-right">"Nouveaux véhicules"</th>
                                                        <th class="px-4 py-3 text-left w-24"></th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-100">
                                                    {merged.into_iter().map(|(week, uc, vc)| {
                                                        let bar_u = ((uc as f64 / max_u as f64) * 64.0) as u32;
                                                        let bar_v = ((vc as f64 / max_v as f64) * 64.0) as u32;
                                                        view! {
                                                            <tr class="hover:bg-gray-50">
                                                                <td class="px-4 py-2 font-mono text-xs text-gray-500">{week}</td>
                                                                <td class="px-4 py-2 text-right font-semibold text-gray-800">{uc}</td>
                                                                <td class="px-4 py-2">
                                                                    <div class="h-3 bg-indigo-400 rounded-sm" style=format!("width: {}px", bar_u.max(2)) />
                                                                </td>
                                                                <td class="px-4 py-2 text-right font-semibold text-gray-800">{vc}</td>
                                                                <td class="px-4 py-2">
                                                                    <div class="h-3 bg-amber-400 rounded-sm" style=format!("width: {}px", bar_v.max(2)) />
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                        <div class="px-4 py-2 flex gap-4 text-xs text-gray-400 border-t border-gray-100">
                                            <span class="flex items-center gap-1"><span class="inline-block w-3 h-2 bg-indigo-400 rounded-sm" />"Utilisateurs"</span>
                                            <span class="flex items-center gap-1"><span class="inline-block w-3 h-2 bg-amber-400 rounded-sm" />"Véhicules"</span>
                                        </div>
                                    }.into_view()
                                }
                            })}
                        </Suspense>
                    </div>
                </Show>

                // ══════════════════════════════════════════════
                // Onglet : Utilisateurs
                // ══════════════════════════════════════════════
                <Show when=move || tab.get() == "users" fallback=|| ()>
                    // Barre de filtres
                    <div class="flex flex-col sm:flex-row gap-2">
                        <input
                            type="text"
                            placeholder="Filtrer par nom ou email..."
                            prop:value=user_text
                            on:input=move |ev| { set_user_text.set(event_target_value(&ev)); set_editing_id.set(None); }
                            class="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                        />
                        <div class="flex gap-1">
                            {[("", "Tous"), ("trial", "Essai"), ("active", "Actifs"), ("expired", "Expirés")].into_iter().map(|(v, l)| {
                                view! {
                                    <button
                                        on:click=move |_| { set_user_status.set(v.to_string()); set_editing_id.set(None); }
                                        class=move || format!(
                                            "px-3 py-2 rounded-md text-xs font-medium transition {}",
                                            if user_status.get() == v { "bg-indigo-600 text-white" } else { "bg-white border border-gray-300 text-gray-600 hover:bg-gray-50" }
                                        )
                                    >{l}</button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    // Message erreur édition
                    {move || edit_error.get().map(|e| view! {
                        <p class="text-sm text-red-600 p-3 rounded-lg bg-red-50 border border-red-200">{e}</p>
                    })}

                    <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
                        <Suspense fallback=|| view! { <p class="p-4 text-sm text-gray-400">"Chargement..."</p> }>
                            {move || users.get().map(|res| match res {
                                Err(e) => view! { <p class="p-4 text-sm text-red-600">{format!("Erreur : {}", e)}</p> }.into_view(),
                                Ok(list) => {
                                    let text_f = user_text.get().to_lowercase();
                                    let status_f = user_status.get();
                                    let filtered: Vec<AdminUser> = list.into_iter()
                                        .filter(|u| {
                                            (text_f.is_empty() || u.username.to_lowercase().contains(&text_f) || u.email.to_lowercase().contains(&text_f))
                                            && (status_f.is_empty() || u.status == status_f)
                                        })
                                        .collect();
                                    let count = filtered.len();
                                    view! {
                                        <div class="px-4 py-2 border-b border-gray-100 text-xs text-gray-400">{count}" résultat(s)"</div>
                                        <div class="overflow-x-auto">
                                            <table class="w-full text-sm">
                                                <thead class="bg-gray-50 text-xs text-gray-500 uppercase">
                                                    <tr>
                                                        <th class="px-3 py-3 text-left">"Utilisateur"</th>
                                                        <th class="px-3 py-3 text-left">"Email"</th>
                                                        <th class="px-3 py-3 text-left">"Statut"</th>
                                                        <th class="px-3 py-3 text-left">"Type"</th>
                                                        <th class="px-3 py-3 text-center">"iOS"</th>
                                                        <th class="px-3 py-3 text-center">"Admin"</th>
                                                        <th class="px-3 py-3 text-left">"Expiration"</th>
                                                        <th class="px-3 py-3 text-right">"Actions"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-100">
                                                    {filtered.into_iter().map(|u| {
                                                        let uid        = u.id.clone();
                                                        let username   = u.username.clone();
                                                        let email      = u.email.clone();
                                                        let status     = u.status.clone();
                                                        let lt         = u.license_type.clone();
                                                        let is_ios_u   = u.is_ios;
                                                        let is_admin_u = u.is_admin;
                                                        let expiry_display = u.access_expires_at.as_deref()
                                                            .or(Some(u.trial_ends_at.as_str()))
                                                            .map(|s| s.chars().take(10).collect::<String>())
                                                            .unwrap_or_default();
                                                        let expiry_for_input = u.access_expires_at.as_deref()
                                                            .map(|s| s.chars().take(10).collect::<String>())
                                                            .unwrap_or_default();
                                                        // u_btn : copie pour le handler "Modifier"
                                                        let u_btn = u.clone();
                                                        view! {
                                                            {move || {
                                                                let is_editing = editing_id.get().as_deref() == Some(uid.as_str());
                                                                if !is_editing {
                                                                    // ── Ligne normale ──────────────────
                                                                    let uid_c  = uid.clone();
                                                                    let u_c    = u_btn.clone();
                                                                    let efi    = expiry_for_input.clone();
                                                                    view! {
                                                                        <tr class="hover:bg-gray-50">
                                                                            <td class="px-3 py-2.5 font-medium text-gray-900">{username.clone()}</td>
                                                                            <td class="px-3 py-2.5 text-gray-500 text-xs">{email.clone()}</td>
                                                                            <td class="px-3 py-2.5"><StatusBadge status=status.clone() /></td>
                                                                            <td class="px-3 py-2.5"><LicenseTypeBadge lt=lt.clone() /></td>
                                                                            <td class="px-3 py-2.5 text-center">
                                                                                {if is_ios_u { view! { <span class="text-indigo-500 font-semibold text-xs">"iOS"</span> }.into_view() } else { view! { <span class="text-gray-300 text-xs">"—"</span> }.into_view() }}
                                                                            </td>
                                                                            <td class="px-3 py-2.5 text-center">
                                                                                {if is_admin_u { view! { <span class="text-indigo-600 font-bold text-xs">"✓"</span> }.into_view() } else { view! { <span></span> }.into_view() }}
                                                                            </td>
                                                                            <td class="px-3 py-2.5 text-gray-400 font-mono text-xs">{expiry_display.clone()}</td>
                                                                            <td class="px-3 py-2.5 text-right">
                                                                                <button
                                                                                    on:click=move |_| {
                                                                                        set_editing_id.set(Some(uid_c.clone()));
                                                                                        set_edit_username.set(u_c.username.clone());
                                                                                        set_edit_email.set(u_c.email.clone());
                                                                                        set_edit_is_admin.set(u_c.is_admin);
                                                                                        set_edit_is_ios.set(u_c.is_ios);
                                                                                        set_edit_license_type.set(u_c.license_type.clone());
                                                                                        set_edit_expires.set(efi.clone());
                                                                                        set_edit_error.set(None);
                                                                                    }
                                                                                    class="text-xs text-indigo-600 hover:text-indigo-800 font-medium"
                                                                                >"Modifier"</button>
                                                                            </td>
                                                                        </tr>
                                                                    }.into_view()
                                                                } else {
                                                                    // ── Ligne d'édition ─────────────────
                                                                    view! {
                                                                        <tr class="bg-indigo-50">
                                                                            <td class="px-3 py-2">
                                                                                <input type="text" prop:value=edit_username
                                                                                    on:input=move |ev| set_edit_username.set(event_target_value(&ev))
                                                                                    class=input_cls />
                                                                            </td>
                                                                            <td class="px-3 py-2">
                                                                                <input type="email" prop:value=edit_email
                                                                                    on:input=move |ev| set_edit_email.set(event_target_value(&ev))
                                                                                    class=input_cls />
                                                                            </td>
                                                                            <td class="px-3 py-2 text-xs text-gray-400">"—"</td>
                                                                            <td class="px-3 py-2">
                                                                                <select on:change=move |ev| set_edit_license_type.set(event_target_value(&ev))
                                                                                    class=input_cls>
                                                                                    <option value="personal" prop:selected=move || edit_license_type.get() == "personal">"Personal"</option>
                                                                                    <option value="fleet"    prop:selected=move || edit_license_type.get() == "fleet">"Fleet"</option>
                                                                                </select>
                                                                            </td>
                                                                            <td class="px-3 py-2 text-center">
                                                                                <input type="checkbox" prop:checked=edit_is_ios
                                                                                    on:change=move |ev| {
                                                                                        use wasm_bindgen::JsCast;
                                                                                        let checked = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().checked();
                                                                                        set_edit_is_ios.set(checked);
                                                                                    }
                                                                                    class="w-4 h-4 rounded border-gray-300 text-indigo-600" />
                                                                            </td>
                                                                            <td class="px-3 py-2 text-center">
                                                                                <input type="checkbox" prop:checked=edit_is_admin
                                                                                    on:change=move |ev| {
                                                                                        use wasm_bindgen::JsCast;
                                                                                        let checked = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().checked();
                                                                                        set_edit_is_admin.set(checked);
                                                                                    }
                                                                                    class="w-4 h-4 rounded border-gray-300 text-indigo-600" />
                                                                            </td>
                                                                            <td class="px-3 py-2">
                                                                                <input type="date" prop:value=edit_expires
                                                                                    on:input=move |ev| set_edit_expires.set(event_target_value(&ev))
                                                                                    class=input_cls />
                                                                            </td>
                                                                            <td class="px-3 py-2 text-right">
                                                                                <div class="flex items-center justify-end gap-2">
                                                                                    <button
                                                                                        on:click=move |_| save_edit.dispatch((
                                                                                            editing_id.get().unwrap_or_default(),
                                                                                            edit_username.get(), edit_email.get(),
                                                                                            edit_is_admin.get(), edit_is_ios.get(),
                                                                                            edit_license_type.get(), edit_expires.get(),
                                                                                        ))
                                                                                        prop:disabled=edit_loading
                                                                                        class="text-xs font-medium px-2 py-1 rounded bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-40"
                                                                                    >{move || if edit_loading.get() { "…" } else { "Sauvegarder" }}</button>
                                                                                    <button
                                                                                        on:click=move |_| { set_editing_id.set(None); set_edit_error.set(None); }
                                                                                        class="text-xs text-gray-500 hover:text-gray-700"
                                                                                    >"Annuler"</button>
                                                                                </div>
                                                                            </td>
                                                                        </tr>
                                                                    }.into_view()
                                                                }
                                                            }}
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_view()
                                }
                            })}
                        </Suspense>
                    </div>
                </Show>

                // ══════════════════════════════════════════════
                // Onglet : Licences
                // ══════════════════════════════════════════════
                <Show when=move || tab.get() == "licences" fallback=|| ()>
                    <input
                        type="text"
                        placeholder="Filtrer par email..."
                        prop:value=license_filter
                        on:input=move |ev| set_license_filter.set(event_target_value(&ev))
                        class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                    />
                    <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
                        <Suspense fallback=|| view! { <p class="p-4 text-sm text-gray-400">"Chargement..."</p> }>
                            {move || requests.get().map(|res| match res {
                                Err(e) => view! { <p class="p-4 text-sm text-red-600">{format!("Erreur : {}", e)}</p> }.into_view(),
                                Ok(list) => {
                                    let f = license_filter.get().to_lowercase();
                                    let filtered: Vec<LicenseRequest> = list.into_iter()
                                        .filter(|r| f.is_empty() || r.email.to_lowercase().contains(&f))
                                        .collect();
                                    if filtered.is_empty() {
                                        return view! { <p class="p-4 text-sm text-gray-400">"Aucune demande."</p> }.into_view();
                                    }
                                    view! {
                                        <div class="overflow-x-auto">
                                            <table class="w-full text-sm">
                                                <thead class="bg-gray-50 text-xs text-gray-500 uppercase">
                                                    <tr>
                                                        <th class="px-4 py-3 text-left">"Email"</th>
                                                        <th class="px-4 py-3 text-left">"Date"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-100">
                                                    {filtered.into_iter().map(|r| {
                                                        let date = r.requested_at.chars().take(10).collect::<String>();
                                                        view! {
                                                            <tr class="hover:bg-gray-50">
                                                                <td class="px-4 py-3 text-gray-700">{r.email}</td>
                                                                <td class="px-4 py-3 text-gray-400 font-mono text-xs">{date}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_view()
                                }
                            })}
                        </Suspense>
                    </div>
                </Show>

                // ══════════════════════════════════════════════
                // Onglet : Flottes
                // ══════════════════════════════════════════════
                <Show when=move || tab.get() == "flottes" fallback=|| ()>
                    <input
                        type="text"
                        placeholder="Filtrer par nom d'entreprise..."
                        prop:value=fleet_filter
                        on:input=move |ev| set_fleet_filter.set(event_target_value(&ev))
                        class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                    />
                    <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
                        <Suspense fallback=|| view! { <p class="p-4 text-sm text-gray-400">"Chargement..."</p> }>
                            {move || companies.get().map(|res| match res {
                                Err(e) => view! { <p class="p-4 text-sm text-red-600">{format!("Erreur : {}", e)}</p> }.into_view(),
                                Ok(list) => {
                                    let f = fleet_filter.get().to_lowercase();
                                    let filtered: Vec<AdminCompany> = list.into_iter()
                                        .filter(|c| f.is_empty() || c.name.to_lowercase().contains(&f))
                                        .collect();
                                    if filtered.is_empty() {
                                        return view! { <p class="p-4 text-sm text-gray-400">"Aucune flotte."</p> }.into_view();
                                    }
                                    view! {
                                        <div class="divide-y divide-gray-100">
                                            {filtered.into_iter().map(|company| {
                                                let (open, set_open) = create_signal(false);
                                                let nb_m = company.members.len();
                                                let nb_v = company.vehicles.len();
                                                let nb_o = company.organizations.len();
                                                let has_m = nb_m > 0;
                                                let has_v = nb_v > 0;
                                                let has_o = nb_o > 0;
                                                let (members,  _) = create_signal(company.members.clone());
                                                let (vehicles, _) = create_signal(company.vehicles.clone());
                                                let (orgs,     _) = create_signal(company.organizations.clone());
                                                view! {
                                                    <div>
                                                        <button
                                                            on:click=move |_| set_open.update(|v| *v = !*v)
                                                            class="w-full flex items-center justify-between px-4 py-3 hover:bg-gray-50 transition text-left"
                                                        >
                                                            <div class="flex items-center gap-3">
                                                                <svg class="w-5 h-5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 21h16.5M4.5 3h15M5.25 3v18m13.5-18v18M9 6.75h1.5m-1.5 3h1.5m-1.5 3h1.5m3-6H15m-1.5 3H15m-1.5 3H15M9 21v-3.375c0-.621.504-1.125 1.125-1.125h3.75c.621 0 1.125.504 1.125 1.125V21" />
                                                                </svg>
                                                                <div>
                                                                    <span class="font-semibold text-sm text-gray-900">{company.name}</span>
                                                                    {company.siret.as_ref().map(|s| view! {
                                                                        <span class="ml-2 text-xs text-gray-400">"SIRET : "{s.clone()}</span>
                                                                    })}
                                                                </div>
                                                            </div>
                                                            <div class="flex items-center gap-3 text-xs text-gray-500">
                                                                <span>{nb_m}" membres"</span>
                                                                <span>{nb_o}" orgs"</span>
                                                                <span>{nb_v}" véhicules"</span>
                                                                <svg class=move || if open.get() { "w-4 h-4 text-gray-400 rotate-180 transition-transform" } else { "w-4 h-4 text-gray-400 transition-transform" }
                                                                    fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
                                                                </svg>
                                                            </div>
                                                        </button>
                                                        <Show when=move || open.get() fallback=|| ()>
                                                            <div class="px-4 pb-4 space-y-3 bg-gray-50 border-t border-gray-100">
                                                                {if has_m { view! {
                                                                    <div class="pt-3">
                                                                        <p class="text-xs font-semibold text-gray-500 uppercase mb-2">"Membres"</p>
                                                                        <div class="space-y-1">
                                                                            {move || members.get().into_iter().map(|m| {
                                                                                let role_cls = match m.fleet_role.as_deref() {
                                                                                    Some("fleet_admin")  => Some(("Admin",  "bg-indigo-100 text-indigo-700")),
                                                                                    Some("fleet_viewer") => Some(("Viewer", "bg-gray-100 text-gray-600")),
                                                                                    _                    => None,
                                                                                };
                                                                                view! {
                                                                                    <div class="flex items-center justify-between text-sm py-0.5">
                                                                                        <div class="flex items-center gap-2">
                                                                                            <span class="font-medium text-gray-800 text-xs">{m.username}</span>
                                                                                            <span class="text-gray-400 text-xs">{m.email}</span>
                                                                                        </div>
                                                                                        {role_cls.map(|(label, cls)| view! {
                                                                                            <span class=format!("px-2 py-0.5 rounded-full text-xs font-semibold {}", cls)>{label}</span>
                                                                                        })}
                                                                                    </div>
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    </div>
                                                                }.into_view() } else { ().into_view() }}
                                                                {if has_o { view! {
                                                                    <div>
                                                                        <p class="text-xs font-semibold text-gray-500 uppercase mb-2">"Organisations"</p>
                                                                        <div class="flex flex-wrap gap-2">
                                                                            {move || orgs.get().into_iter().map(|o| view! {
                                                                                <span class="px-2 py-1 rounded-md bg-white border border-gray-200 text-xs text-gray-700">{o.name}</span>
                                                                            }).collect_view()}
                                                                        </div>
                                                                    </div>
                                                                }.into_view() } else { ().into_view() }}
                                                                {if has_v { view! {
                                                                    <div>
                                                                        <p class="text-xs font-semibold text-gray-500 uppercase mb-2">"Véhicules"</p>
                                                                        <div class="overflow-x-auto">
                                                                            <table class="w-full text-xs">
                                                                                <tbody class="divide-y divide-gray-100">
                                                                                    {move || vehicles.get().into_iter().map(|v| view! {
                                                                                        <tr>
                                                                                            <td class="py-1.5 pr-4 text-gray-800 font-medium">{format!("{} {}", v.make, v.model)}</td>
                                                                                            <td class="py-1.5 pr-4 font-mono text-gray-500">{v.plate_number}</td>
                                                                                            <td class="py-1.5 text-gray-400">{v.org_name.unwrap_or_default()}</td>
                                                                                        </tr>
                                                                                    }).collect_view()}
                                                                                </tbody>
                                                                            </table>
                                                                        </div>
                                                                    </div>
                                                                }.into_view() } else { ().into_view() }}
                                                            </div>
                                                        </Show>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_view()
                                }
                            })}
                        </Suspense>
                    </div>
                </Show>

                // ══════════════════════════════════════════════
                // Onglet : Génération
                // ══════════════════════════════════════════════
                <Show when=move || tab.get() == "generation" fallback=|| ()>
                    <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-4">
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
                                    on:change=move |ev| { if let Ok(d) = event_target_value(&ev).parse::<i32>() { set_gen_days.set(d); } }
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
                                let v = serde_json::from_str::<serde_json::Value>(&json).unwrap_or_default();
                                let token    = v["token"].as_str().unwrap_or("").to_string();
                                let assigned = v["assigned_to"].as_str().map(|s| s.to_string());
                                let label    = if assigned.is_some() { "Jeton généré et assigné :" } else { "Jeton généré (email introuvable) :" };
                                let badge    = if assigned.is_some() { "bg-green-50 border-green-200" } else { "bg-amber-50 border-amber-200" };
                                let tcls     = if assigned.is_some() { "text-green-800" } else { "text-amber-800" };
                                let lcls     = if assigned.is_some() { "text-green-600" } else { "text-amber-600" };
                                view! {
                                    <div class=format!("p-3 rounded-lg border {}", badge)>
                                        <p class=format!("text-xs font-medium mb-1 {}", lcls)>{label}</p>
                                        <p class=format!("font-mono text-sm font-bold tracking-widest {}", tcls)>{token}</p>
                                        {assigned.map(|email| view! {
                                            <p class="text-xs text-green-600 mt-1">"✓ Licence appliquée à "{email}</p>
                                        })}
                                    </div>
                                }.into_view()
                            }
                            Err(e) => view! {
                                <p class="text-sm text-red-600 p-3 rounded-lg bg-red-50 border border-red-200">{e}</p>
                            }.into_view(),
                        })}
                        <button
                            on:click=move |_| generate_action.dispatch((gen_email.get(), gen_days.get(), gen_type.get()))
                            prop:disabled=gen_loading
                            class="px-5 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-40 transition"
                        >
                            {move || if gen_loading.get() { "Génération..." } else { "Générer" }}
                        </button>
                    </div>
                </Show>

            </div>
        </div>
    }
}
