// src/pages/fleet.rs — Gestion de flotte

use crate::components::ui::{format_km, get_token, input_class};
use common::{FleetReportContract, FleetReportVehicle};
use js_sys;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::JsCast;

// ─── Types locaux ─────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct Company {
    id: Uuid,
    name: String,
    siret: Option<String>,
    my_role: Option<String>,
    member_count: i64,
    vehicle_count: i64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct Organization {
    id: Uuid,
    parent_org_id: Option<Uuid>,
    name: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct CompanyMember {
    user_id: Uuid,
    username: String,
    email: String,
    fleet_role: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct FleetVehicle {
    id: Uuid,
    make: String,
    model: String,
    plate_number: String,
    year: Option<i16>,
    org_id: Option<Uuid>,
    org_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct FleetRoleEntry {
    id: Uuid,
    user_id: Uuid,
    org_id: Option<Uuid>,
    role: String,
    username: String,
    email: String,
    org_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct PersonalVehicle {
    id: Uuid,
    make: String,
    model: String,
    plate_number: String,
    role: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum AdminTab {
    Members,
    Orgs,
    Roles,
    Vehicles,
}

// ─── Page principale ──────────────────────────────────────────────

#[component]
pub fn FleetPage() -> impl IntoView {
    let navigate = use_navigate();
    let (companies, set_companies) = create_signal(Vec::<Company>::new());
    let (loading, set_loading) = create_signal(true);
    let (selected_id, set_selected_id) = create_signal(Option::<Uuid>::None);

    create_effect(move |_| {
        let Some(token) = get_token() else {
            navigate("/", NavigateOptions::default());
            return;
        };
        spawn_local(async move {
            match fetch_json::<Vec<Company>>(
                &format!("{}/api/companies", crate::config::API_BASE),
                &token,
            )
            .await
            {
                Ok(list) => {
                    if let Some(first) = list.first() {
                        set_selected_id.set(Some(first.id));
                    }
                    set_companies.set(list);
                }
                Err(e) => leptos::logging::error!("fetch companies: {e}"),
            }
            set_loading.set(false);
        });
    });

    let reload = move || {
        if let Some(token) = get_token() {
            spawn_local(async move {
                if let Ok(list) = fetch_json::<Vec<Company>>(
                    &format!("{}/api/companies", crate::config::API_BASE),
                    &token,
                )
                .await
                {
                    if let Some(first) = list.first() {
                        set_selected_id.set(Some(first.id));
                    }
                    set_companies.set(list);
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
                    <span class="text-xl font-bold text-indigo-600">"Flotte"</span>
                    <div class="w-20" />
                </div>
            </nav>

            <div class="max-w-4xl mx-auto px-4 py-4 md:py-8 space-y-4 md:space-y-8">
                <Show when=move || loading.get() fallback=|| ()>
                    <div class="flex justify-center py-20">
                        <p class="text-gray-400 animate-pulse">"Chargement..."</p>
                    </div>
                </Show>
                <Show when=move || !loading.get() fallback=|| ()>
                    <Show when=move || companies.get().is_empty() fallback=|| ()>
                        <EmptyFleetState on_created=reload />
                    </Show>
                    <Show when=move || !companies.get().is_empty() fallback=|| ()>
                        <FleetView
                            companies=companies
                            selected_id=selected_id
                            set_selected_id=set_selected_id
                            on_change=reload
                        />
                    </Show>
                </Show>
            </div>
        </div>
    }
}

// ─── État vide ────────────────────────────────────────────────────

#[component]
fn EmptyFleetState(on_created: impl Fn() + 'static + Copy) -> impl IntoView {
    view! {
        <div class="max-w-lg mx-auto pt-12 space-y-8">
            <div class="text-center">
                <div class="w-16 h-16 bg-indigo-50 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <svg class="w-8 h-8 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M3.75 21h16.5M4.5 3h15M5.25 3v18m13.5-18v18M9 6.75h1.5m-1.5 3h1.5m-1.5 3h1.5m3-6H15m-1.5 3H15m-1.5 3H15M9 21v-3.375c0-.621.504-1.125 1.125-1.125h3.75c.621 0 1.125.504 1.125 1.125V21" />
                    </svg>
                </div>
                <h1 class="text-2xl font-bold text-gray-900">"Gestion de flotte"</h1>
                <p class="text-sm text-gray-500 mt-2">
                    "Vous n'appartenez à aucune entreprise. Créez la vôtre ou demandez à un administrateur de vous inviter."
                </p>
            </div>
            <CreateCompanyCard on_created=on_created />
        </div>
    }
}

// ─── Carte création entreprise ────────────────────────────────────

#[component]
fn CreateCompanyCard(on_created: impl Fn() + 'static + Copy) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (siret, set_siret) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());

    let submit = create_action(move |(name, siret): &(String, String)| {
        let (name, siret) = (name.clone(), siret.clone());
        async move {
            set_error.set(String::new());
            let token = get_token().unwrap_or_default();
            let siret_val = if siret.trim().is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(siret.trim().to_string())
            };
            let body = serde_json::json!({ "name": name.trim(), "siret": siret_val });
            match post_json(
                &format!("{}/api/companies", crate::config::API_BASE),
                &token,
                &body,
            )
            .await
            {
                Ok(_) => {
                    set_name.set(String::new());
                    set_siret.set(String::new());
                    on_created();
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 space-y-4">
            <h2 class="text-lg font-bold text-gray-900">"Créer une entreprise"</h2>
            <div class="space-y-3">
                <div class="space-y-1">
                    <label class="text-sm font-medium text-gray-700 block">"Nom"</label>
                    <input type="text" required placeholder="Mon Entreprise SAS"
                        prop:value=name
                        on:input=move |ev| set_name.set(event_target_value(&ev))
                        class=input_class() />
                </div>
                <div class="space-y-1">
                    <label class="text-sm font-medium text-gray-700 block">
                        "SIRET " <span class="text-gray-400 font-normal">"(optionnel)"</span>
                    </label>
                    <input type="text" placeholder="12345678901234"
                        prop:value=siret
                        on:input=move |ev| set_siret.set(event_target_value(&ev))
                        class=input_class() />
                </div>
                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-red-600">{move || error.get()}</p>
                </Show>
                <button
                    on:click=move |_| {
                        let n = name.get();
                        if !n.trim().is_empty() { submit.dispatch((n, siret.get())); }
                    }
                    prop:disabled=move || submit.pending().get()
                    class="w-full py-2.5 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 transition duration-150"
                >
                    {move || if submit.pending().get() { "Création..." } else { "Créer" }}
                </button>
            </div>
        </div>
    }
}

// ─── Vue flotte principale ────────────────────────────────────────

#[component]
fn FleetView(
    companies: ReadSignal<Vec<Company>>,
    selected_id: ReadSignal<Option<Uuid>>,
    set_selected_id: WriteSignal<Option<Uuid>>,
    on_change: impl Fn() + 'static + Copy,
) -> impl IntoView {
    let (fleet_refresh, set_fleet_refresh) = create_signal(0u32);
    let selected = create_memo(move |_| {
        let id = selected_id.get()?;
        companies.get().into_iter().find(|c| c.id == id)
    });

    view! {
        <div class="space-y-6">
            // Sélecteur entreprises + bouton créer
            <div class="flex items-center justify-between flex-wrap gap-3">
                <div class="flex items-center gap-2 flex-wrap">
                    <For
                        each=move || companies.get()
                        key=|c| c.id
                        children=move |c| {
                            let id = c.id;
                            let name = c.name.clone();
                            let active = create_memo(move |_| selected_id.get() == Some(id));
                            view! {
                                <button
                                    on:click=move |_| set_selected_id.set(Some(id))
                                    class=move || if active.get() {
                                        "px-4 py-2 rounded-lg text-sm font-semibold bg-indigo-600 text-white shadow-sm"
                                    } else {
                                        "px-4 py-2 rounded-lg text-sm font-medium bg-white border border-gray-200 text-gray-700 hover:border-indigo-300 hover:text-indigo-600 transition duration-150"
                                    }
                                >
                                    {name}
                                </button>
                            }
                        }
                    />
                </div>
                <NewCompanyInline on_created=on_change />
            </div>

            // Contenu entreprise sélectionnée
            {move || selected.get().map(|company| {
                let company_id = company.id;
                let is_admin = company.my_role.as_deref() == Some("fleet_admin");
                let role_label = company.my_role.clone().unwrap_or_else(|| "Membre".to_string());
                view! {
                    <div class="space-y-6">
                        // Stats
                        <div class="grid grid-cols-3 gap-3">
                            <StatCard value=company.vehicle_count.to_string() label="Véhicules" />
                            <StatCard value=company.member_count.to_string() label="Membres" />
                            <StatCard value=role_label label="Mon rôle" />
                        </div>

                        // Véhicules flotte
                        <FleetVehiclesSection
                            company_id=company_id
                            company_name=company.name.clone()
                            company_siret=company.siret.clone()
                            refresh=fleet_refresh
                        />

                        // Panel admin
                        <Show when=move || is_admin fallback=|| ()>
                            <AdminPanel company_id=company_id on_vehicles_changed=Callback::new(move |_| set_fleet_refresh.update(|n| *n += 1)) />
                        </Show>
                    </div>
                }
            })}
        </div>
    }
}

#[component]
fn StatCard(value: String, label: &'static str) -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 text-center">
            <p class="text-xl font-bold text-indigo-600">{value}</p>
            <p class="text-xs text-gray-500 mt-1">{label}</p>
        </div>
    }
}

// ─── Bouton inline créer entreprise ──────────────────────────────

#[component]
fn NewCompanyInline(on_created: impl Fn() + 'static + Copy) -> impl IntoView {
    let (show, set_show) = create_signal(false);
    let (name, set_name) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());

    let submit = create_action(move |name: &String| {
        let name = name.clone();
        async move {
            set_error.set(String::new());
            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({ "name": name.trim() });
            match post_json(
                &format!("{}/api/companies", crate::config::API_BASE),
                &token,
                &body,
            )
            .await
            {
                Ok(_) => {
                    set_show.set(false);
                    set_name.set(String::new());
                    on_created();
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    view! {
        <Show
            when=move || show.get()
            fallback=move || view! {
                <button
                    on:click=move |_| set_show.set(true)
                    class="flex items-center gap-1.5 px-3 py-2 rounded-lg text-sm text-gray-500 border border-dashed border-gray-300 hover:border-indigo-300 hover:text-indigo-600 transition duration-150"
                >
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                    </svg>
                    <span class="hidden sm:inline">"Nouvelle entreprise"</span>
                </button>
            }
        >
            <div class="flex items-center gap-2 bg-white border border-gray-200 rounded-xl px-3 py-2 shadow-sm">
                <input
                    type="text" placeholder="Nom de l'entreprise" autofocus
                    prop:value=name
                    on:input=move |ev| set_name.set(event_target_value(&ev))
                    class="flex-1 text-sm outline-none"
                />
                <button
                    on:click=move |_| { let n = name.get(); if !n.trim().is_empty() { submit.dispatch(n); } }
                    prop:disabled=move || submit.pending().get()
                    class="px-3 py-1 rounded text-xs font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 transition"
                >"OK"</button>
                <button on:click=move |_| { set_show.set(false); set_error.set(String::new()); }
                    class="text-gray-400 hover:text-gray-600">
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
            <Show when=move || !error.get().is_empty() fallback=|| ()>
                <p class="text-xs text-red-600 mt-1">{move || error.get()}</p>
            </Show>
        </Show>
    }
}

// ─── Section véhicules flotte ─────────────────────────────────────

#[component]
fn FleetVehiclesSection(
    company_id: Uuid,
    company_name: String,
    company_siret: Option<String>,
    refresh: ReadSignal<u32>,
) -> impl IntoView {
    let (vehicles, set_vehicles) = create_signal(Vec::<FleetVehicle>::new());
    let (loading, set_loading) = create_signal(true);
    let (fetch_error, set_fetch_error) = create_signal(String::new());

    create_effect(move |_| {
        let _ = refresh.get();
        let Some(token) = get_token() else { return };
        spawn_local(async move {
            match fetch_json::<Vec<FleetVehicle>>(
                &format!("{}/api/companies/{}/vehicles", crate::config::API_BASE, company_id),
                &token,
            )
            .await
            {
                Ok(list) => { set_fetch_error.set(String::new()); set_vehicles.set(list); }
                Err(e) => { leptos::logging::error!("fetch fleet vehicles: {e}"); set_fetch_error.set(e); }
            }
            set_loading.set(false);
        });
    });

    // Stocké dans un signal pour être accessible par multiple closures
    let (company_name_sig, _) = create_signal(company_name.clone());
    // Action PDF : fetche membres + rapport flotte (contrats inclus) en 2 appels
    let cn = company_name.clone();
    let cs = company_siret.clone();
    let pdf_action = create_action(move |_: &()| {
        let cn = cn.clone();
        let cs = cs.clone();
        async move {
            if let Some(token) = get_token() {
                let url_members = format!("{}/api/companies/{}/members", crate::config::API_BASE, company_id);
                let url_report  = format!("{}/api/companies/{}/fleet-report", crate::config::API_BASE, company_id);
                let (members, report) = futures::join!(
                    fetch_json::<Vec<CompanyMember>>(&url_members, &token),
                    fetch_json::<Vec<FleetReportVehicle>>(&url_report, &token)
                );
                export_fleet_pdf(&cn, cs.as_deref(), &members.unwrap_or_default(), &report.unwrap_or_default());
            }
        }
    });

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
            <div class="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
                <h2 class="text-sm font-semibold text-gray-800">"Véhicules de la flotte"</h2>
                <div class="flex items-center gap-3">
                    <span class="text-xs text-gray-400">
                        {move || {
                            let n = vehicles.get().len();
                            format!("{} véhicule{}", n, if n > 1 { "s" } else { "" })
                        }}
                    </span>
                    <Show when=move || !vehicles.get().is_empty() fallback=|| ()>
                        <div class="flex gap-1.5">
                            <button
                                on:click=move |_| {
                                    export_fleet_csv(&vehicles.get_untracked(), &company_name_sig.get_untracked());
                                }
                                title="Exporter en CSV"
                                class="flex items-center gap-1 text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 hover:text-green-600 transition duration-150"
                            >
                                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3" />
                                </svg>
                                "CSV"
                            </button>
                            <button
                                on:click=move |_| { pdf_action.dispatch(()); }
                                title="Exporter en PDF"
                                class="flex items-center gap-1 text-xs px-2 py-1 rounded border border-gray-200 text-gray-500 hover:bg-gray-50 hover:text-indigo-600 transition duration-150"
                            >
                                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                                </svg>
                                "PDF"
                            </button>
                        </div>
                    </Show>
                </div>
            </div>
            <Show when=move || loading.get() fallback=|| ()>
                <p class="text-sm text-gray-400 animate-pulse text-center p-6">"Chargement..."</p>
            </Show>
            <Show when=move || !fetch_error.get().is_empty() fallback=|| ()>
                <p class="text-xs text-red-500 text-center p-4">{move || format!("Erreur : {}", fetch_error.get())}</p>
            </Show>
            <Show when=move || !loading.get() && fetch_error.get().is_empty() && vehicles.get().is_empty() fallback=|| ()>
                <div class="p-6 text-center">
                    <p class="text-sm text-gray-400 italic">"Aucun véhicule dans cette flotte."</p>
                    <p class="text-xs text-gray-300 mt-1">"Assignez des véhicules depuis le panel d'administration."</p>
                </div>
            </Show>
            <Show when=move || !loading.get() && fetch_error.get().is_empty() && !vehicles.get().is_empty() fallback=|| ()>
                <div class="divide-y divide-gray-50">
                    <For
                        each=move || vehicles.get()
                        key=|v| v.id
                        children=move |v| {
                            let year_str = v.year.map(|y| format!(" ({})", y)).unwrap_or_default();
                            let org = v.org_name.clone();
                            view! {
                                <div class="flex items-center gap-4 px-4 py-3 hover:bg-gray-50 transition duration-100">
                                    <div class="w-8 h-8 rounded-lg bg-indigo-50 flex items-center justify-center shrink-0">
                                        <svg class="w-4 h-4 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                            <path stroke-linecap="round" stroke-linejoin="round"
                                                d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                                        </svg>
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <p class="text-sm font-semibold text-gray-800">
                                            {format!("{} {}{}", v.make, v.model, year_str)}
                                        </p>
                                        <p class="text-xs font-mono text-indigo-600">{v.plate_number}</p>
                                    </div>
                                    {org.map(|o| view! {
                                        <span class="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-600 shrink-0">{o}</span>
                                    })}
                                </div>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}

// ─── Panel administration ─────────────────────────────────────────

#[component]
fn AdminPanel(company_id: Uuid, on_vehicles_changed: Callback<()>) -> impl IntoView {
    let (tab, set_tab) = create_signal(AdminTab::Members);

    let tab_cls = move |t: AdminTab| {
        if tab.get() == t {
            "px-4 py-2.5 text-sm font-semibold text-indigo-600 border-b-2 border-indigo-600 whitespace-nowrap"
        } else {
            "px-4 py-2.5 text-sm font-medium text-gray-500 hover:text-gray-700 border-b-2 border-transparent transition duration-150 whitespace-nowrap"
        }
    };

    view! {
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden">
            <div class="px-4 pt-3 border-b border-gray-100">
                <h2 class="text-sm font-semibold text-gray-800 mb-3">"Administration"</h2>
                <div class="flex overflow-x-auto -mb-px gap-1">
                    <button on:click=move |_| set_tab.set(AdminTab::Members)
                        class=move || tab_cls(AdminTab::Members)>"Membres"</button>
                    <button on:click=move |_| set_tab.set(AdminTab::Orgs)
                        class=move || tab_cls(AdminTab::Orgs)>"Organisations"</button>
                    <button on:click=move |_| set_tab.set(AdminTab::Roles)
                        class=move || tab_cls(AdminTab::Roles)>"Rôles"</button>
                    <button on:click=move |_| set_tab.set(AdminTab::Vehicles)
                        class=move || tab_cls(AdminTab::Vehicles)>"Véhicules"</button>
                </div>
            </div>
            <div class="p-4">
                {move || match tab.get() {
                    AdminTab::Members  => view! { <MembersSection  company_id=company_id /> }.into_view(),
                    AdminTab::Orgs     => view! { <OrgsSection     company_id=company_id /> }.into_view(),
                    AdminTab::Roles    => view! { <RolesSection    company_id=company_id /> }.into_view(),
                    AdminTab::Vehicles => view! { <VehiclesSection company_id=company_id on_changed=on_vehicles_changed /> }.into_view(),
                }}
            </div>
        </div>
    }
}

// ─── Membres ──────────────────────────────────────────────────────

#[component]
fn MembersSection(company_id: Uuid) -> impl IntoView {
    let (members, set_members) = create_signal(Vec::<CompanyMember>::new());
    let (email, set_email) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());

    let reload = move || {
        if let Some(token) = get_token() {
            spawn_local(async move {
                if let Ok(list) = fetch_json::<Vec<CompanyMember>>(
                    &format!("{}/api/companies/{}/members", crate::config::API_BASE, company_id),
                    &token,
                )
                .await
                {
                    set_members.set(list);
                }
            });
        }
    };

    create_effect(move |_| reload());

    let add = create_action(move |email: &String| {
        let email = email.clone();
        async move {
            set_error.set(String::new());
            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({ "email": email.trim() });
            match post_json(
                &format!("{}/api/companies/{}/members", crate::config::API_BASE, company_id),
                &token,
                &body,
            )
            .await
            {
                Ok(_) => {
                    set_email.set(String::new());
                    reload();
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    view! {
        <div class="space-y-4">
            <div class="flex gap-2">
                <input type="email" placeholder="Email de l'utilisateur"
                    prop:value=email
                    on:input=move |ev| set_email.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            let e = email.get();
                            if !e.trim().is_empty() { add.dispatch(e); }
                        }
                    }
                    class="flex-1 px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                />
                <button
                    on:click=move |_| { let e = email.get(); if !e.trim().is_empty() { add.dispatch(e); } }
                    prop:disabled=move || add.pending().get()
                    class="px-4 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 transition"
                >"Ajouter"</button>
            </div>
            <Show when=move || !error.get().is_empty() fallback=|| ()>
                <p class="text-sm text-red-600">{move || error.get()}</p>
            </Show>
            <div class="space-y-2">
                <Show when=move || members.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-gray-400 italic text-center py-4">"Aucun membre."</p>
                </Show>
                <For
                    each=move || members.get()
                    key=|m| m.user_id
                    children=move |m| {
                        let uid = m.user_id;
                        let initial = m.username.chars().next().unwrap_or('?').to_uppercase().to_string();
                        let badge = m.fleet_role.clone().map(|r| {
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
                            <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                <div class="flex items-center gap-3">
                                    <div class="w-7 h-7 rounded-full bg-indigo-100 flex items-center justify-center text-xs font-bold text-indigo-600">
                                        {initial}
                                    </div>
                                    <div>
                                        <div class="flex items-center gap-2">
                                            <p class="text-sm font-medium text-gray-800">{m.username}</p>
                                            {badge}
                                        </div>
                                        <p class="text-xs text-gray-400">{m.email}</p>
                                    </div>
                                </div>
                                <button
                                    on:click=move |_| {
                                        spawn_local(async move {
                                            let token = get_token().unwrap_or_default();
                                            let url = format!("{}/api/companies/{}/members/{}", crate::config::API_BASE, company_id, uid);
                                            if delete_request(&url, &token).await.is_ok() { reload(); }
                                        });
                                    }
                                    class="text-xs px-2 py-1 rounded border border-red-200 text-red-500 hover:bg-red-50 transition"
                                >"Retirer"</button>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

// ─── Organisations ────────────────────────────────────────────────

#[component]
fn OrgsSection(company_id: Uuid) -> impl IntoView {
    let (orgs, set_orgs) = create_signal(Vec::<Organization>::new());
    let (name, set_name) = create_signal(String::new());
    let (parent_id, set_parent_id) = create_signal(Option::<Uuid>::None);
    let (error, set_error) = create_signal(String::new());

    let reload = move || {
        if let Some(token) = get_token() {
            spawn_local(async move {
                if let Ok(list) = fetch_json::<Vec<Organization>>(
                    &format!("{}/api/companies/{}/organizations", crate::config::API_BASE, company_id),
                    &token,
                )
                .await
                {
                    set_orgs.set(list);
                }
            });
        }
    };

    create_effect(move |_| reload());

    let create_org = create_action(move |(n, pid): &(String, Option<Uuid>)| {
        let (n, pid) = (n.clone(), *pid);
        async move {
            set_error.set(String::new());
            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({ "name": n.trim(), "parent_org_id": pid });
            match post_json(
                &format!("{}/api/companies/{}/organizations", crate::config::API_BASE, company_id),
                &token,
                &body,
            )
            .await
            {
                Ok(_) => {
                    set_name.set(String::new());
                    set_parent_id.set(None);
                    reload();
                }
                Err(e) => set_error.set(e),
            }
        }
    });

    let top_orgs = create_memo(move |_| {
        orgs.get().into_iter().filter(|o| o.parent_org_id.is_none()).collect::<Vec<_>>()
    });

    view! {
        <div class="space-y-4">
            <div class="space-y-2">
                <div class="flex gap-2">
                    <input type="text" placeholder="Nom de l'organisation"
                        prop:value=name
                        on:input=move |ev| set_name.set(event_target_value(&ev))
                        class="flex-1 px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                    />
                    <button
                        on:click=move |_| { let n = name.get(); if !n.trim().is_empty() { create_org.dispatch((n, parent_id.get())); } }
                        prop:disabled=move || create_org.pending().get()
                        class="px-4 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 transition"
                    >"Ajouter"</button>
                </div>
                <Show when=move || !top_orgs.get().is_empty() fallback=|| ()>
                    <select
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            set_parent_id.set(if val.is_empty() { None } else { Uuid::parse_str(&val).ok() });
                        }
                        class="w-full px-3 py-2 border border-gray-200 rounded-md text-sm text-gray-600 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                    >
                        <option value="">"— Niveau 1 (département) —"</option>
                        <For
                            each=move || top_orgs.get()
                            key=|o| o.id
                            children=move |o| {
                                let id = o.id.to_string();
                                let n = o.name.clone();
                                view! { <option value=id>{format!("Sous {}", n)}</option> }
                            }
                        />
                    </select>
                </Show>
                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-red-600">{move || error.get()}</p>
                </Show>
            </div>

            // Arbre d'organisations
            <div class="space-y-2">
                <Show when=move || top_orgs.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-gray-400 italic text-center py-4">"Aucune organisation."</p>
                </Show>
                <For
                    each=move || top_orgs.get()
                    key=|o| o.id
                    children=move |org| {
                        let oid = org.id;
                        let oname = org.name.clone();
                        let children = create_memo(move |_| {
                            orgs.get().into_iter().filter(|o| o.parent_org_id == Some(oid)).collect::<Vec<_>>()
                        });
                        view! {
                            <div class="border border-gray-100 rounded-lg overflow-hidden">
                                <div class="flex items-center justify-between px-4 py-3 bg-gray-50">
                                    <span class="text-sm font-semibold text-gray-800">{oname}</span>
                                    <button
                                        on:click=move |_| {
                                            spawn_local(async move {
                                                let token = get_token().unwrap_or_default();
                                                let url = format!("{}/api/companies/{}/organizations/{}", crate::config::API_BASE, company_id, oid);
                                                if delete_request(&url, &token).await.is_ok() { reload(); }
                                            });
                                        }
                                        class="text-xs px-2 py-1 rounded border border-red-200 text-red-500 hover:bg-red-50 transition"
                                    >"Supprimer"</button>
                                </div>
                                <Show when=move || !children.get().is_empty() fallback=|| ()>
                                    <div class="divide-y divide-gray-50">
                                        <For
                                            each=move || children.get()
                                            key=|c| c.id
                                            children=move |child| {
                                                let cid = child.id;
                                                let cname = child.name.clone();
                                                view! {
                                                    <div class="flex items-center justify-between px-4 py-2.5 pl-10">
                                                        <div class="flex items-center gap-2">
                                                            <span class="w-3 h-px bg-gray-300 inline-block" />
                                                            <span class="text-sm text-gray-600">{cname}</span>
                                                        </div>
                                                        <button
                                                            on:click=move |_| {
                                                                spawn_local(async move {
                                                                    let token = get_token().unwrap_or_default();
                                                                    let url = format!("{}/api/companies/{}/organizations/{}", crate::config::API_BASE, company_id, cid);
                                                                    if delete_request(&url, &token).await.is_ok() { reload(); }
                                                                });
                                                            }
                                                            class="text-xs px-2 py-1 rounded border border-red-200 text-red-500 hover:bg-red-50 transition"
                                                        >"Supprimer"</button>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                </Show>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

// ─── Rôles fleet ──────────────────────────────────────────────────

#[component]
fn RolesSection(company_id: Uuid) -> impl IntoView {
    let (roles, set_roles) = create_signal(Vec::<FleetRoleEntry>::new());
    let (members, set_members) = create_signal(Vec::<CompanyMember>::new());
    let (orgs, set_orgs) = create_signal(Vec::<Organization>::new());
    let (sel_uid, set_sel_uid) = create_signal(String::new());
    let (sel_org, set_sel_org) = create_signal(String::new());
    let (sel_role, set_sel_role) = create_signal("fleet_viewer".to_string());
    let (error, set_error) = create_signal(String::new());

    let reload = move || {
        if let Some(token) = get_token() {
            let t2 = token.clone();
            let t3 = token.clone();
            spawn_local(async move {
                if let Ok(list) = fetch_json::<Vec<FleetRoleEntry>>(
                    &format!("{}/api/companies/{}/fleet-roles", crate::config::API_BASE, company_id),
                    &token,
                ).await { set_roles.set(list); }
            });
            spawn_local(async move {
                if let Ok(list) = fetch_json::<Vec<CompanyMember>>(
                    &format!("{}/api/companies/{}/members", crate::config::API_BASE, company_id),
                    &t2,
                ).await { set_members.set(list); }
            });
            spawn_local(async move {
                if let Ok(list) = fetch_json::<Vec<Organization>>(
                    &format!("{}/api/companies/{}/organizations", crate::config::API_BASE, company_id),
                    &t3,
                ).await { set_orgs.set(list); }
            });
        }
    };

    create_effect(move |_| reload());

    let assign = create_action(move |(uid, org, role): &(String, String, String)| {
        let (uid, org, role) = (uid.clone(), org.clone(), role.clone());
        async move {
            set_error.set(String::new());
            let Ok(uid) = Uuid::parse_str(&uid) else {
                set_error.set("Sélectionnez un utilisateur".to_string());
                return;
            };
            let org_id_val = if org.is_empty() {
                serde_json::Value::Null
            } else {
                Uuid::parse_str(&org).map(|u| serde_json::Value::String(u.to_string()))
                    .unwrap_or(serde_json::Value::Null)
            };
            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({ "user_id": uid, "org_id": org_id_val, "role": role });
            match post_json(
                &format!("{}/api/companies/{}/fleet-roles", crate::config::API_BASE, company_id),
                &token,
                &body,
            ).await {
                Ok(_) => { set_sel_uid.set(String::new()); reload(); }
                Err(e) => set_error.set(e),
            }
        }
    });

    view! {
        <div class="space-y-4">
            <div class="bg-gray-50 rounded-lg p-3 space-y-3">
                <p class="text-xs font-semibold text-gray-500 uppercase tracking-wide">"Attribuer un rôle"</p>
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
                    <select on:change=move |ev| set_sel_uid.set(event_target_value(&ev))
                        class="px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500">
                        <option value="">"— Utilisateur —"</option>
                        <For each=move || members.get() key=|m| m.user_id children=move |m| {
                            let id = m.user_id.to_string();
                            let name = m.username.clone();
                            view! { <option value=id>{name}</option> }
                        } />
                    </select>
                    <select on:change=move |ev| set_sel_org.set(event_target_value(&ev))
                        class="px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500">
                        <option value="">"— Global (toute l'entreprise) —"</option>
                        <For each=move || orgs.get() key=|o| o.id children=move |o| {
                            let id = o.id.to_string();
                            let name = o.name.clone();
                            view! { <option value=id>{name}</option> }
                        } />
                    </select>
                    <select on:change=move |ev| set_sel_role.set(event_target_value(&ev))
                        class="px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500">
                        <option value="fleet_viewer">"Lecteur flotte"</option>
                        <option value="fleet_admin">"Admin flotte"</option>
                    </select>
                </div>
                <button
                    on:click=move |_| assign.dispatch((sel_uid.get(), sel_org.get(), sel_role.get()))
                    prop:disabled=move || assign.pending().get() || sel_uid.get().is_empty()
                    class="px-4 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 transition"
                >"Attribuer"</button>
                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-red-600">{move || error.get()}</p>
                </Show>
            </div>

            <div class="space-y-2">
                <Show when=move || roles.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-gray-400 italic text-center py-4">"Aucun rôle fleet attribué."</p>
                </Show>
                <For each=move || roles.get() key=|r| r.id children=move |entry| {
                    let rid = entry.id;
                    let (lbl, cls) = if entry.role == "fleet_admin" {
                        ("Admin", "bg-indigo-100 text-indigo-700")
                    } else {
                        ("Lecteur", "bg-gray-100 text-gray-600")
                    };
                    let scope = entry.org_name.clone()
                        .map(|n| format!("org : {}", n))
                        .unwrap_or_else(|| "global".to_string());
                    view! {
                        <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                            <div>
                                <div class="flex items-center gap-2">
                                    <p class="text-sm font-medium text-gray-800">{entry.username}</p>
                                    <span class=format!("text-xs px-2 py-0.5 rounded-full font-medium {}", cls)>{lbl}</span>
                                </div>
                                <p class="text-xs text-gray-400">{scope}</p>
                            </div>
                            <button
                                on:click=move |_| {
                                    spawn_local(async move {
                                        let token = get_token().unwrap_or_default();
                                        let url = format!("{}/api/companies/{}/fleet-roles/{}", crate::config::API_BASE, company_id, rid);
                                        if delete_request(&url, &token).await.is_ok() { reload(); }
                                    });
                                }
                                class="text-xs px-2 py-1 rounded border border-red-200 text-red-500 hover:bg-red-50 transition"
                            >"Révoquer"</button>
                        </div>
                    }
                } />
            </div>
        </div>
    }
}

// ─── Véhicules (assignation) ──────────────────────────────────────

#[component]
fn VehiclesSection(company_id: Uuid, on_changed: Callback<()>) -> impl IntoView {
    let (personal, set_personal) = create_signal(Vec::<PersonalVehicle>::new());
    let (fleet_veh, set_fleet_veh) = create_signal(Vec::<FleetVehicle>::new());
    let (orgs, set_orgs) = create_signal(Vec::<Organization>::new());
    let (sel_vid, set_sel_vid) = create_signal(String::new());
    let (sel_org, set_sel_org) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());

    let reload = move || {
        if let Some(token) = get_token() {
            let t2 = token.clone();
            let t3 = token.clone();
            spawn_local(async move {
                if let Ok(list) = fetch_json::<Vec<PersonalVehicle>>(
                    &format!("{}/api/vehicles", crate::config::API_BASE), &token,
                ).await { set_personal.set(list); }
            });
            spawn_local(async move {
                if let Ok(list) = fetch_json::<Vec<FleetVehicle>>(
                    &format!("{}/api/companies/{}/vehicles", crate::config::API_BASE, company_id), &t2,
                ).await { set_fleet_veh.set(list); }
            });
            spawn_local(async move {
                if let Ok(list) = fetch_json::<Vec<Organization>>(
                    &format!("{}/api/companies/{}/organizations", crate::config::API_BASE, company_id), &t3,
                ).await { set_orgs.set(list); }
            });
        }
    };

    create_effect(move |_| reload());

    let fleet_ids = create_memo(move |_| {
        fleet_veh.get().iter().map(|v| v.id).collect::<std::collections::HashSet<_>>()
    });
    let unassigned = create_memo(move |_| {
        let ids = fleet_ids.get();
        personal.get().into_iter()
            .filter(|v| !ids.contains(&v.id) && v.role.as_deref() == Some("owner"))
            .collect::<Vec<_>>()
    });

    let assign = create_action(move |(vid, org): &(String, String)| {
        let (vid, org) = (vid.clone(), org.clone());
        async move {
            set_error.set(String::new());
            let Ok(vid) = Uuid::parse_str(&vid) else {
                set_error.set("Sélectionnez un véhicule".to_string());
                return;
            };
            let org_id_val = if org.is_empty() {
                serde_json::Value::Null
            } else {
                Uuid::parse_str(&org).map(|u| serde_json::Value::String(u.to_string()))
                    .unwrap_or(serde_json::Value::Null)
            };
            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({ "company_id": company_id, "org_id": org_id_val });
            match post_json(
                &format!("{}/api/vehicles/{}/fleet", crate::config::API_BASE, vid),
                &token,
                &body,
            ).await {
                Ok(_) => { set_sel_vid.set(String::new()); reload(); on_changed.call(()); }
                Err(e) => set_error.set(e),
            }
        }
    });

    view! {
        <div class="space-y-4">
            <div class="bg-gray-50 rounded-lg p-3 space-y-3">
                <p class="text-xs font-semibold text-gray-500 uppercase tracking-wide">"Ajouter un véhicule à la flotte"</p>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                    <select on:change=move |ev| set_sel_vid.set(event_target_value(&ev))
                        class="px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500">
                        <option value="">"— Mon véhicule —"</option>
                        <For each=move || unassigned.get() key=|v| v.id children=move |v| {
                            let id = v.id.to_string();
                            let label = format!("{} {} · {}", v.make, v.model, v.plate_number);
                            view! { <option value=id>{label}</option> }
                        } />
                    </select>
                    <select on:change=move |ev| set_sel_org.set(event_target_value(&ev))
                        class="px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500">
                        <option value="">"— Organisation (optionnel) —"</option>
                        <For each=move || orgs.get() key=|o| o.id children=move |o| {
                            let id = o.id.to_string();
                            let name = o.name.clone();
                            view! { <option value=id>{name}</option> }
                        } />
                    </select>
                </div>
                <button
                    on:click=move |_| assign.dispatch((sel_vid.get(), sel_org.get()))
                    prop:disabled=move || assign.pending().get() || sel_vid.get().is_empty()
                    class="px-4 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 transition"
                >"Assigner à la flotte"</button>
                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-red-600">{move || error.get()}</p>
                </Show>
                <Show when=move || unassigned.get().is_empty() fallback=|| ()>
                    <p class="text-xs text-gray-400">"Tous vos véhicules (propriétaire) sont déjà dans cette flotte."</p>
                </Show>
            </div>

            <div class="space-y-2">
                <Show when=move || fleet_veh.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-gray-400 italic text-center py-4">"Aucun véhicule dans la flotte."</p>
                </Show>
                <For each=move || fleet_veh.get() key=|v| v.id children=move |v| {
                    let vid = v.id;
                    let org = v.org_name.clone();
                    view! {
                        <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                            <div>
                                <p class="text-sm font-medium text-gray-800">{format!("{} {}", v.make, v.model)}</p>
                                <div class="flex items-center gap-2 mt-0.5">
                                    <p class="text-xs font-mono text-indigo-600">{v.plate_number}</p>
                                    {org.map(|o| view! {
                                        <span class="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-600">{o}</span>
                                    })}
                                </div>
                            </div>
                            <button
                                on:click=move |_| {
                                    spawn_local(async move {
                                        let token = get_token().unwrap_or_default();
                                        let url = format!("{}/api/vehicles/{}/fleet", crate::config::API_BASE, vid);
                                        if delete_request(&url, &token).await.is_ok() { reload(); on_changed.call(()); }
                                    });
                                }
                                class="text-xs px-2 py-1 rounded border border-red-200 text-red-500 hover:bg-red-50 transition"
                            >"Retirer"</button>
                        </div>
                    }
                } />
            </div>
        </div>
    }
}

// ─── Helpers réseau ───────────────────────────────────────────────

pub async fn fetch_companies_count(token: &str) -> usize {
    fetch_json::<Vec<serde_json::Value>>(
        &format!("{}/api/companies", crate::config::API_BASE),
        token,
    )
    .await
    .map(|v| v.len())
    .unwrap_or(0)
}

async fn fetch_json<T: for<'de> serde::Deserialize<'de>>(url: &str, token: &str) -> Result<T, String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers.set("Authorization", &format!("Bearer {token}")).ok();
    headers.set("Cache-Control", "no-cache").ok();
    opts.headers(&headers);
    let req = web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
        .await.map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    if !resp.ok() { return Err(format!("HTTP {}", resp.status())); }
    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|e| format!("{e:?}"))?)
        .await.map_err(|e| format!("{e:?}"))?;
    serde_wasm_bindgen::from_value(json).map_err(|e| format!("{e:?}"))
}

async fn post_json(url: &str, token: &str, body: &serde_json::Value) -> Result<(), String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers.set("Authorization", &format!("Bearer {token}")).ok();
    headers.set("Content-Type", "application/json").ok();
    opts.headers(&headers);
    opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));
    let req = web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
        .await.map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    if resp.ok() || resp.status() == 201 || resp.status() == 204 {
        return Ok(());
    }
    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|e| format!("{e:?}"))?)
        .await.ok();
    let msg = json
        .and_then(|j| serde_wasm_bindgen::from_value::<serde_json::Value>(j).ok())
        .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| format!("HTTP {}", resp.status()));
    Err(msg)
}

async fn delete_request(url: &str, token: &str) -> Result<(), String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("DELETE");
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers.set("Authorization", &format!("Bearer {token}")).ok();
    opts.headers(&headers);
    let req = web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(leptos::window().fetch_with_request(&req))
        .await.map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    if resp.ok() || resp.status() == 204 { Ok(()) } else { Err(format!("HTTP {}", resp.status())) }
}

// ─── Export CSV flotte ─────────────────────────────────────────────

fn export_fleet_csv(vehicles: &[FleetVehicle], company_name: &str) {
    let mut csv = String::from("Marque,Modèle,Immatriculation,Année,Organisation\n");
    for v in vehicles {
        let year = v.year.map(|y| y.to_string()).unwrap_or_default();
        let org  = v.org_name.as_deref().unwrap_or("");
        csv.push_str(&format!("{},{},{},{},{}\n",
            v.make, v.model, v.plate_number, year, org));
    }
    let filename = format!("flotte-{}.csv",
        company_name.to_lowercase().replace(' ', "-"));
    fleet_download(&csv, &filename, "text/csv;charset=utf-8");
}

// ─── Export PDF flotte ─────────────────────────────────────────────

fn export_fleet_pdf(
    company_name: &str,
    siret: Option<&str>,
    members: &[CompanyMember],
    report: &[FleetReportVehicle],
) {
    let siret_line = siret
        .map(|s| format!("<tr><td>SIRET</td><td>{}</td></tr>", s))
        .unwrap_or_default();

    let members_rows = members.iter().map(|m| {
        let role = match m.fleet_role.as_deref() {
            Some("fleet_admin")  => "Administrateur",
            Some("fleet_viewer") => "Observateur",
            _                    => "Membre",
        };
        format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>", m.username, m.email, role)
    }).collect::<String>();

    // Véhicules avec leurs contrats
    let vehicles_section = report.iter().map(|v| {
        let year = v.year.map(|y| y.to_string()).unwrap_or_else(|| "—".to_string());
        let org  = v.org_name.as_deref().unwrap_or("—");

        let contracts_html = if v.contracts.is_empty() {
            "<p style='color:#94a3b8;font-size:12px;margin:4px 0 0 0'>Aucun contrat actif</p>".to_string()
        } else {
            v.contracts.iter().map(|c| contract_row_html(c)).collect::<String>()
        };

        format!(r#"<div style="margin-bottom:20px;border:1px solid #e2e8f0;border-radius:8px;overflow:hidden">
<div style="background:#f8fafc;padding:10px 14px;display:flex;justify-content:space-between;align-items:center">
  <span style="font-weight:700;color:#1e293b">{make} {model}</span>
  <span style="font-family:monospace;color:#4f46e5;font-size:13px">{plate}</span>
  <span style="font-size:12px;color:#64748b">{year} · {org}</span>
</div>
<div style="padding:10px 14px">{contracts}</div>
</div>"#,
            make=v.make, model=v.model, plate=v.plate_number,
            year=year, org=org, contracts=contracts_html)
    }).collect::<String>();

    let html = format!(r#"<!DOCTYPE html>
<html lang="fr"><head><meta charset="UTF-8"/>
<title>Rapport Flotte — {company_name}</title>
<style>
body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;max-width:740px;margin:40px auto;color:#1e293b;font-size:13px}}
h1{{color:#4f46e5;font-size:22px;margin-bottom:4px}}
.sub{{color:#94a3b8;font-size:12px;margin-bottom:32px}}
h2{{font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:.05em;color:#94a3b8;margin:24px 0 10px}}
table{{width:100%;border-collapse:collapse}}
th{{text-align:left;padding:7px 10px;background:#f8fafc;font-size:11px;color:#64748b;border-bottom:1px solid #e2e8f0}}
td{{padding:7px 10px;border-bottom:1px solid #f1f5f9}}
.info-table td:first-child{{color:#64748b;width:40%}}
.info-table td:last-child{{font-weight:600}}
footer{{margin-top:32px;font-size:11px;color:#94a3b8;border-top:1px solid #f1f5f9;padding-top:10px}}
@media print{{@page{{margin:15mm}}}}
</style></head>
<body>
<script>window.addEventListener('load',function(){{setTimeout(function(){{window.print()}},400)}})</script>
<h1>Rapport de Flotte</h1>
<div class="sub">Généré le {date} — LimTrack</div>
<h2>Entreprise</h2>
<table class="info-table">
<tr><td>Nom</td><td>{company_name}</td></tr>
{siret_line}
<tr><td>Véhicules</td><td>{nb_vehicles}</td></tr>
<tr><td>Membres</td><td>{nb_members}</td></tr>
</table>
<h2>Membres</h2>
<table>
<thead><tr><th>Utilisateur</th><th>Email</th><th>Rôle</th></tr></thead>
<tbody>{members_rows}</tbody>
</table>
<h2>Véhicules et contrats actifs</h2>
{vehicles_section}
<footer>LimTrack · limtrack.app · Rapport généré automatiquement</footer>
</body></html>"#,
        company_name   = company_name,
        date           = chrono::Local::now().format("%d/%m/%Y"),
        siret_line     = siret_line,
        nb_vehicles    = report.len(),
        nb_members     = members.len(),
        members_rows   = members_rows,
        vehicles_section = vehicles_section,
    );

    fleet_open_print(&html);
}

fn contract_row_html(c: &FleetReportContract) -> String {
    let type_label = if c.contract_type == "loa" { "LOA" } else { "Assurance" };
    let pct = ((c.km_consumed as f64 / c.km_authorized as f64) * 100.0).min(100.0) as u32;
    let status_color = match c.status.as_str() {
        "exceeded" => "#dc2626",
        _ if c.overage_risk => "#d97706",
        _ => "#4f46e5",
    };
    let status_label = match c.status.as_str() {
        "exceeded" => "Dépassé",
        _ if c.overage_risk => "Risque",
        _ => "Actif",
    };

    let insurer_line = c.insurer.as_deref()
        .map(|ins| format!(" · {}", ins))
        .unwrap_or_default();

    let cost_line = c.price_per_extra_km.and_then(|price| {
        let extra_km = if c.km_consumed > c.km_authorized {
            c.km_consumed - c.km_authorized
        } else if c.forecast_km > c.km_authorized {
            c.forecast_km - c.km_authorized
        } else { return None; };
        let cost = extra_km as f64 * price;
        let label = if c.km_consumed > c.km_authorized { "Coût dépassement" } else { "Coût projeté" };
        Some(format!("<span style='color:#dc2626;font-weight:700'> · {} : {:.2} €</span>", label, cost))
    }).unwrap_or_default();

    format!(r#"<div style="margin:6px 0;padding:8px 10px;background:#f8fafc;border-radius:6px;border-left:3px solid {color}">
  <div style="display:flex;justify-content:space-between;margin-bottom:4px">
    <span style="font-weight:600;font-size:12px">{type_label}{insurer}</span>
    <span style="font-size:11px;color:{color};font-weight:700">{status}</span>
  </div>
  <div style="font-size:11px;color:#64748b">
    {consumed} / {authorized} km ({pct}%) · Restant : {remaining} km · Projection : {forecast} km{cost}
  </div>
  <div style="font-size:11px;color:#94a3b8">Du {start} au {end} · J-{days}</div>
</div>"#,
        color=status_color, type_label=type_label, insurer=insurer_line,
        status=status_label, pct=pct,
        consumed=format_km(c.km_consumed), authorized=format_km(c.km_authorized),
        remaining=format_km(c.km_remaining), forecast=format_km(c.forecast_km),
        cost=cost_line,
        start=c.start_date, end=c.end_date, days=c.days_remaining,
    )
}

fn fleet_open_print(html: &str) {
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

fn fleet_download(content: &str, filename: &str, mime: &str) {
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(content));
    let mut opts = web_sys::BlobPropertyBag::new();
    opts.type_(mime);
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&array, &opts) else { return };
    let Ok(url)  = web_sys::Url::create_object_url_with_blob(&blob) else { return };
    let document = leptos::document();
    let Ok(el)   = document.create_element("a") else { return };
    let Ok(a)    = el.dyn_into::<web_sys::HtmlAnchorElement>() else { return };
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    web_sys::Url::revoke_object_url(&url).ok();
}
