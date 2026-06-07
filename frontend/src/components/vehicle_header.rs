// src/components/vehicle_header.rs
use common::{AccessRole, ShareCode, VehicleWithAccess};
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn VehicleHeader(
    vehicle: ReadSignal<Option<VehicleWithAccess>>,
    on_deleted: Callback<uuid::Uuid>,
    on_archived: Callback<(uuid::Uuid, bool)>,
) -> impl IntoView {
    let (show_share_modal, set_show_share_modal) = create_signal(false);
    let (show_delete_modal, set_show_delete_modal) = create_signal(false);
    let (archive_error, set_archive_error) = create_signal(String::new());

    let is_owner = create_memo(move |_| {
        vehicle
            .get()
            .map(|v| matches!(v.my_role, AccessRole::Owner))
            .unwrap_or(false)
    });

    view! {
        <Show when=move || vehicle.get().is_some() fallback=|| ()>
            {move || vehicle.get().map(|v| {
                let vehicle_name       = format!("{} {}", v.make, v.model);
                let vehicle_name_share = vehicle_name.clone();
                let vehicle_name_del   = vehicle_name.clone();
                let plate              = v.plate_number.clone();
                let plate_display      = v.plate_number.clone();
                let is_archived        = v.archived_at.is_some();
                let vid                = v.id;

                let role_label = match v.my_role {
                    AccessRole::Owner  => ("Propriétaire", "bg-indigo-100 text-indigo-700"),
                    AccessRole::Editor => ("Éditeur",      "bg-amber-100 text-amber-700"),
                    AccessRole::Viewer => ("Lecteur",      "bg-gray-100 text-gray-600"),
                };

                view! {
                    <div class="flex items-center justify-between px-6 py-4 bg-white border-b border-gray-100">
                        <div class="flex items-center gap-4">
                            // Icône véhicule
                            <div class="w-12 h-12 rounded-xl bg-indigo-50 flex items-center justify-center shrink-0">
                                <svg class="w-7 h-7 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                                </svg>
                            </div>

                            // Infos
                            <div>
                                <div class="flex items-center gap-2">
                                    <h2 class="text-lg font-bold text-gray-900">
                                        {vehicle_name.clone()}
                                    </h2>
                                    <span class=format!(
                                        "text-xs font-medium px-2 py-0.5 rounded-full {}",
                                        role_label.1
                                    )>
                                        {role_label.0}
                                    </span>
                                </div>
                                <p class="text-sm font-mono font-semibold text-indigo-600 tracking-widest mt-0.5">
                                    {plate_display}
                                </p>
                            </div>
                        </div>

                        // Boutons — owner uniquement
                        <Show when=move || is_owner.get() fallback=|| ()>
                            <div class="flex flex-col items-end gap-1">
                                <div class="flex items-center gap-2">
                                    // Bouton partage (masqué si archivé)
                                    <Show when=move || !is_archived fallback=|| ()>
                                        <button
                                            on:click=move |_| set_show_share_modal.set(true)
                                            class="flex items-center gap-2 p-2 md:px-4 md:py-2 rounded-lg border border-gray-200 text-sm font-medium text-gray-600 hover:bg-gray-50 hover:border-indigo-300 hover:text-indigo-600 transition duration-150"
                                        >
                                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                                <path stroke-linecap="round" stroke-linejoin="round"
                                                    d="M7.217 10.907a2.25 2.25 0 1 0 0 2.186m0-2.186c.18.324.283.696.283 1.093s-.103.77-.283 1.093m0-2.186 9.566-5.314m-9.566 7.5 9.566 5.314m0 0a2.25 2.25 0 1 0 3.935 2.186 2.25 2.25 0 0 0-3.935-2.186zm0-12.814a2.25 2.25 0 1 0 3.933-2.185 2.25 2.25 0 0 0-3.933 2.185z" />
                                            </svg>
                                            <span class="hidden md:inline">"Partager"</span>
                                        </button>
                                    </Show>

                                    // Bouton archiver / désarchiver
                                    {if is_archived {
                                        view! {
                                            <button
                                                on:click=move |_| {
                                                    set_archive_error.set(String::new());
                                                    let token = get_token().unwrap_or_default();
                                                    spawn_local(async move {
                                                        let url = format!("{}/api/vehicles/{}/unarchive", crate::config::API_BASE, vid);
                                                        match patch_json(&url, &token).await {
                                                            Ok(_) => on_archived.call((vid, false)),
                                                            Err(e) => set_archive_error.set(e),
                                                        }
                                                    });
                                                }
                                                class="flex items-center gap-2 p-2 md:px-4 md:py-2 rounded-lg border border-green-200 text-sm font-medium text-green-700 hover:bg-green-50 transition duration-150"
                                            >
                                                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                                    <path stroke-linecap="round" stroke-linejoin="round"
                                                        d="M9 15 3 9m0 0 6-6M3 9h12a6 6 0 0 1 0 12h-3" />
                                                </svg>
                                                <span class="hidden md:inline">"Désarchiver"</span>
                                            </button>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <button
                                                on:click=move |_| {
                                                    set_archive_error.set(String::new());
                                                    let token = get_token().unwrap_or_default();
                                                    spawn_local(async move {
                                                        let url = format!("{}/api/vehicles/{}/archive", crate::config::API_BASE, vid);
                                                        match patch_json(&url, &token).await {
                                                            Ok(_) => on_archived.call((vid, true)),
                                                            Err(e) => set_archive_error.set(e),
                                                        }
                                                    });
                                                }
                                                class="flex items-center gap-2 p-2 md:px-4 md:py-2 rounded-lg border border-amber-200 text-sm font-medium text-amber-700 hover:bg-amber-50 transition duration-150"
                                            >
                                                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                                    <path stroke-linecap="round" stroke-linejoin="round"
                                                        d="m20.25 7.5-.625 10.632a2.25 2.25 0 0 1-2.247 2.118H6.622a2.25 2.25 0 0 1-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125Z" />
                                                </svg>
                                                <span class="hidden md:inline">"Archiver"</span>
                                            </button>
                                        }.into_view()
                                    }}

                                    // Bouton supprimer
                                    <button
                                        on:click=move |_| set_show_delete_modal.set(true)
                                        class="flex items-center gap-2 p-2 md:px-4 md:py-2 rounded-lg border border-red-200 text-sm font-medium text-red-600 hover:bg-red-50 transition duration-150"
                                    >
                                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                            <path stroke-linecap="round" stroke-linejoin="round"
                                                d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                                        </svg>
                                        <span class="hidden md:inline">"Supprimer"</span>
                                    </button>
                                </div>
                                <Show when=move || !archive_error.get().is_empty() fallback=|| ()>
                                    <p class="text-xs text-red-500">{move || archive_error.get()}</p>
                                </Show>
                            </div>
                        </Show>
                    </div>

                    // Modal de partage
                    <Show when=move || show_share_modal.get() fallback=|| ()>
                        <ShareModal
                            vehicle_id=v.id
                            vehicle_name=vehicle_name_share.clone()
                            on_close=Callback::new(move |_| set_show_share_modal.set(false))
                        />
                    </Show>

                    // Modal de suppression
                    <Show when=move || show_delete_modal.get() fallback=|| ()>
                        <DeleteModal
                            vehicle_id=v.id
                            vehicle_name=vehicle_name_del.clone()
                            plate_number=plate.clone()
                            on_close=Callback::new(move |_| set_show_delete_modal.set(false))
                            on_deleted=Callback::new(move |id| {
                                set_show_delete_modal.set(false);
                                on_deleted.call(id);
                            })
                        />
                    </Show>
                }
            })}
        </Show>
    }
}

// ─── Modal de suppression ────────────────────────────────────────

#[component]
fn DeleteModal(
    vehicle_id: uuid::Uuid,
    vehicle_name: String,
    plate_number: String,
    on_close: Callback<()>,
    on_deleted: Callback<uuid::Uuid>,
) -> impl IntoView {
    let (input_plate, set_input_plate) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());

    let plate_matches =
        create_memo(move |_| input_plate.get().trim().to_uppercase() == plate_number);

    let submit = create_action(move |_: &()| async move {
        set_error.set(String::new());
        let token = get_token().unwrap_or_default();
        let body = serde_json::json!({
            "plate_number": input_plate.get().trim().to_uppercase()
        });

        match delete_json(
            &format!("{}/api/vehicles/{}", crate::config::API_BASE, vehicle_id),
            &token,
            &body,
        )
        .await
        {
            Ok(_) => on_deleted.call(vehicle_id),
            Err(e) => set_error.set(e),
        }
    });

    view! {
        <button
            type="button"
            class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm w-full cursor-default"
            on:click=move |_| on_close.call(())
        />
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
            <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md p-8 space-y-6">

                // En-tête
                <div class="flex items-center justify-between">
                    <div>
                        <h2 class="text-xl font-bold text-gray-900">"Supprimer le véhicule"</h2>
                        <p class="text-sm text-gray-500 mt-1">{vehicle_name}</p>
                    </div>
                    <button
                        on:click=move |_| on_close.call(())
                        class="text-gray-400 hover:text-gray-600 text-xl font-light"
                    >"✕"</button>
                </div>

                // Avertissement
                <div class="bg-red-50 border border-red-200 rounded-xl p-4 space-y-2">
                    <p class="text-sm font-semibold text-red-700">"⚠ Action irréversible"</p>
                    <p class="text-xs text-red-600">
                        "La suppression entraînera la perte définitive de tous les contrats, relevés kilométriques et accès associés à ce véhicule."
                    </p>
                </div>

                // Champ de confirmation
                <div class="space-y-2">
                    <label class="text-sm font-medium text-gray-700 block">
                        "Saisissez la plaque d'immatriculation pour confirmer :"
                    </label>
                    <input
                        type="text"
                        prop:value=input_plate
                        on:input=move |ev| set_input_plate.set(event_target_value(&ev))
                        placeholder="ex: AB-123-CD"
                        class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-red-500 focus:border-red-500 sm:text-sm font-mono tracking-widest text-center transition duration-150"
                    />
                </div>

                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-center text-red-600">{move || error.get()}</p>
                </Show>

                // Actions
                <div class="flex gap-3">
                    <button
                        type="button"
                        on:click=move |_| on_close.call(())
                        class="flex-1 py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150"
                    >
                        "Annuler"
                    </button>
                    <button
                        type="button"
                        on:click=move |_| submit.dispatch(())
                        prop:disabled=move || !plate_matches.get() || submit.pending().get()
                        class="flex-1 py-2 px-4 rounded-md text-sm font-medium text-white bg-red-600 hover:bg-red-700 disabled:opacity-40 disabled:cursor-not-allowed transition duration-150"
                    >
                        {move || if submit.pending().get() { "Suppression..." } else { "Supprimer définitivement" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ─── Modal de partage ────────────────────────────────────────────

#[component]
fn ShareModal(
    vehicle_id: uuid::Uuid,
    vehicle_name: String,
    on_close: Callback<()>,
) -> impl IntoView {
    let (role, set_role) = create_signal("viewer".to_string());
    let (share_code, set_share_code) = create_signal(Option::<ShareCode>::None);
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(String::new());
    let (copied, set_copied) = create_signal(false);

    let generate = create_action(move |role: &String| {
        let role = role.clone();
        async move {
            set_loading.set(true);
            set_error.set(String::new());
            set_share_code.set(None);

            let token = get_token().unwrap_or_default();
            let body = serde_json::json!({ "role": role });

            match post_json(
                &format!(
                    "{}/api/vehicles/{}/share",
                    crate::config::API_BASE,
                    vehicle_id
                ),
                &token,
                &body,
            )
            .await
            {
                Ok(code) => set_share_code.set(Some(code)),
                Err(e) => set_error.set(e),
            }
            set_loading.set(false);
        }
    });

    let copy_to_clipboard = move |_| {
        if let Some(code) = share_code.get() {
            let clipboard = leptos::window().navigator().clipboard();
            let _ = clipboard.write_text(&code.code);
            set_copied.set(true);
            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(2_000).await;
                set_copied.set(false);
            });
        }
    };

    view! {
        <button
            type="button"
            class="fixed inset-0 z-40 bg-black bg-opacity-40 backdrop-blur-sm w-full cursor-default"
            on:click=move |_| on_close.call(())
        />
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
            <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md p-8 space-y-6">

                <div class="flex items-center justify-between">
                    <div>
                        <h2 class="text-xl font-bold text-gray-900">"Partager le véhicule"</h2>
                        <p class="text-sm text-gray-500 mt-1">{vehicle_name}</p>
                    </div>
                    <button
                        on:click=move |_| on_close.call(())
                        class="text-gray-400 hover:text-gray-600 text-xl font-light"
                    >"✕"</button>
                </div>

                <div class="space-y-3">
                    <p class="text-sm font-medium text-gray-700">"Rôle accordé :"</p>
                    <div class="grid grid-cols-2 gap-3">
                        <button
                            on:click=move |_| {
                                set_role.set("editor".to_string());
                                set_share_code.set(None);
                            }
                            class=move || format!(
                                "flex flex-col items-start p-4 rounded-xl border-2 transition duration-150 {}",
                                if role.get() == "editor" { "border-indigo-500 bg-indigo-50" }
                                else { "border-gray-200 hover:border-gray-300" }
                            )
                        >
                            <span class="text-sm font-semibold text-gray-800">"Éditeur"</span>
                            <span class="text-xs text-gray-500 mt-1">"Peut saisir des relevés kilométriques"</span>
                        </button>
                        <button
                            on:click=move |_| {
                                set_role.set("viewer".to_string());
                                set_share_code.set(None);
                            }
                            class=move || format!(
                                "flex flex-col items-start p-4 rounded-xl border-2 transition duration-150 {}",
                                if role.get() == "viewer" { "border-indigo-500 bg-indigo-50" }
                                else { "border-gray-200 hover:border-gray-300" }
                            )
                        >
                            <span class="text-sm font-semibold text-gray-800">"Lecteur"</span>
                            <span class="text-xs text-gray-500 mt-1">"Consultation uniquement"</span>
                        </button>
                    </div>
                </div>

                <Show when=move || share_code.get().is_none() fallback=|| ()>
                    <button
                        on:click=move |_| generate.dispatch(role.get())
                        prop:disabled=move || loading.get()
                        class="w-full py-2 px-4 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                    >
                        {move || if loading.get() { "Génération..." } else { "Générer un code" }}
                    </button>
                </Show>

                <Show when=move || !error.get().is_empty() fallback=|| ()>
                    <p class="text-sm text-center text-red-600">{move || error.get()}</p>
                </Show>

                <Show when=move || share_code.get().is_some() fallback=|| ()>
                    {move || share_code.get().map(|sc| view! {
                        <div class="space-y-3">
                            <p class="text-sm font-medium text-gray-700">"Code de partage :"</p>
                            <div class="flex items-center gap-2">
                                <div class="flex-1 bg-gray-50 border border-gray-200 rounded-lg px-4 py-3 text-center">
                                    <p class="text-2xl font-mono font-bold text-gray-900 tracking-widest">
                                        {sc.code.clone()}
                                    </p>
                                    <p class=format!(
                                        "text-xs mt-1 {}",
                                        if sc.role == "editor" { "text-amber-600" } else { "text-gray-400" }
                                    )>
                                        {if sc.role == "editor" { "Rôle : Éditeur" } else { "Rôle : Lecteur" }}
                                    </p>
                                </div>
                                <button
                                    on:click=copy_to_clipboard
                                    class=move || format!(
                                        "shrink-0 px-4 py-3 rounded-lg text-sm font-medium transition duration-150 {}",
                                        if copied.get() { "bg-green-100 text-green-700" }
                                        else { "bg-indigo-600 text-white hover:bg-indigo-700" }
                                    )
                                >
                                    {move || if copied.get() { "Copié ✓" } else { "Copier" }}
                                </button>
                            </div>
                            <p class="text-xs text-gray-400 text-center">
                                "Valable 24h · Usage unique — partagez ce code au destinataire."
                            </p>
                        </div>
                    })}
                </Show>

                <button
                    on:click=move |_| on_close.call(())
                    class="w-full py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 transition duration-150"
                >
                    "Fermer"
                </button>
            </div>
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

async fn post_json(url: &str, token: &str, body: &serde_json::Value) -> Result<ShareCode, String> {
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
        let json =
            wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
                .await
                .map_err(|e| format!("{:?}", e))?;
        serde_wasm_bindgen::from_value(json).map_err(|e| format!("{:?}", e))
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

async fn patch_json(url: &str, token: &str) -> Result<(), String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("PATCH");
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
    headers.set("Content-Type", "application/json").ok();
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
        web_sys::Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;
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
