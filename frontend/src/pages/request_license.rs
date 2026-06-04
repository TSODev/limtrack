use leptos::*;
use leptos_router::*;

#[component]
pub fn RequestLicensePage() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (status, set_status) = create_signal(Option::<(bool, String)>::None);
    let (loading, set_loading) = create_signal(false);

    let submit_action = create_action(move |email: &String| {
        let email = email.clone();
        async move {
            set_loading.set(true);
            set_status.set(None);

            let url = format!("{}/api/license/request", crate::config::API_BASE);
            let mut opts = web_sys::RequestInit::new();
            opts.method("POST");

            let headers = web_sys::Headers::new().expect("Headers");
            headers.set("Content-Type", "application/json").expect("Content-Type");
            opts.headers(&headers);

            let body = serde_json::json!({ "email": email });
            opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));

            let request = web_sys::Request::new_with_str_and_init(&url, &opts)
                .expect("Request");

            let window = leptos::window();
            match wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await {
                Ok(resp_value) => {
                    use wasm_bindgen::JsCast;
                    let resp: web_sys::Response = resp_value.dyn_into().expect("Response");
                    let ok = resp.ok();
                    let text = wasm_bindgen_futures::JsFuture::from(resp.text().expect("text"))
                        .await
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();

                    if ok {
                        set_status.set(Some((
                            true,
                            format!("Jeton envoyé à {} — vérifiez votre boîte mail !", email),
                        )));
                    } else {
                        let msg = serde_json::from_str::<serde_json::Value>(&text)
                            .ok()
                            .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
                            .unwrap_or_else(|| "Une erreur est survenue".to_string());
                        set_status.set(Some((false, msg)));
                    }
                }
                Err(_) => {
                    set_status.set(Some((false, "Impossible de contacter le serveur".to_string())));
                }
            }
            set_loading.set(false);
        }
    });

    let success = move || status.get().map_or(false, |(ok, _)| ok);

    view! {
        <div class="min-h-screen bg-gray-100">

            // ─── Navbar ────────────────────────────────────────────
            <nav class="bg-white shadow-sm border-b border-gray-200">
                <div class="max-w-lg mx-auto px-4 h-14 md:h-16 flex items-center justify-between">
                    <A
                        href="/"
                        class="flex items-center gap-2 text-indigo-600 hover:text-indigo-700 font-medium text-sm transition duration-150"
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
                        </svg>
                        "Accueil"
                    </A>
                    <span class="text-xl font-bold text-indigo-600">"LimTrack"</span>
                    <div class="w-20" />
                </div>
            </nav>

            <div class="max-w-lg mx-auto px-4 py-8 space-y-6">

                // ─── Formulaire ────────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 space-y-6">
                    <div class="text-center space-y-2">
                        <div class="w-14 h-14 rounded-2xl bg-green-50 flex items-center justify-center mx-auto">
                            <svg class="w-8 h-8 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25a3 3 0 0 1 3 3m3 0a6 6 0 0 1-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 0 1 21.75 8.25Z" />
                            </svg>
                        </div>
                        <h1 class="text-2xl font-bold text-gray-900">"Obtenir une licence gratuite"</h1>
                        <p class="text-sm text-gray-500 leading-relaxed">
                            "LimTrack est open source et gratuit. Renseignez votre email "
                            "pour recevoir un jeton de licence de 365 jours."
                        </p>
                    </div>

                    <div class="bg-indigo-50 border border-indigo-100 rounded-lg p-4 space-y-1">
                        <p class="text-sm font-semibold text-indigo-700">"Licence gratuite — 365 jours"</p>
                        <p class="text-sm text-indigo-600">"Une seule licence par adresse email. Renouvelable avant expiration."</p>
                    </div>

                    <Show when=move || !success() fallback=|| ()>
                        <div class="space-y-4">
                            <div class="space-y-1">
                                <label class="text-sm font-medium text-gray-700 block">"Adresse email"</label>
                                <input
                                    type="email"
                                    prop:value=email
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    placeholder="votre@email.com"
                                    class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                                />
                            </div>

                            {move || status.get().map(|(ok, msg)| {
                                let cls = if ok {
                                    "text-sm p-3 rounded-lg bg-green-50 border border-green-200 text-green-700"
                                } else {
                                    "text-sm p-3 rounded-lg bg-red-50 border border-red-200 text-red-700"
                                };
                                view! { <p class=cls>{msg}</p> }
                            })}

                            <button
                                on:click=move |_| {
                                    let e = email.get();
                                    if !e.trim().is_empty() {
                                        submit_action.dispatch(e);
                                    }
                                }
                                prop:disabled=move || loading.get() || email.get().trim().is_empty()
                                class="w-full flex items-center justify-center gap-2 px-5 py-2.5 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-40 disabled:cursor-not-allowed transition duration-150"
                            >
                                {move || if loading.get() { "Envoi en cours…" } else { "Recevoir mon jeton par email" }}
                            </button>
                        </div>
                    </Show>

                    <Show when=success fallback=|| ()>
                        <div class="text-center space-y-3 py-2">
                            <div class="w-12 h-12 rounded-full bg-green-100 flex items-center justify-center mx-auto">
                                <svg class="w-6 h-6 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 12.75 6 6 9-13.5" />
                                </svg>
                            </div>
                            {move || status.get().map(|(_, msg)| view! {
                                <p class="text-sm font-medium text-green-700">{msg}</p>
                            })}
                            <p class="text-sm text-gray-500">
                                "Activez votre jeton dans "
                                <A href="/profile" class="text-indigo-600 hover:underline">"Profil → Licence"</A>
                                "."
                            </p>
                        </div>
                    </Show>
                </div>

                // ─── Don volontaire ────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 text-center space-y-3">
                    <p class="text-sm font-semibold text-gray-700">"Soutenir le projet"</p>
                    <p class="text-sm text-gray-500 leading-relaxed">
                        "LimTrack est développé et hébergé bénévolement. "
                        "Un don aide à couvrir les frais d'infrastructure (~5 €/mois)."
                    </p>
                    <div class="flex justify-center gap-3 flex-wrap">
                        <a
                            href="https://ko-fi.com/limtrack"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium text-white bg-amber-500 hover:bg-amber-600 transition duration-150"
                        >
                            "☕ Ko-fi"
                        </a>
                        <a
                            href="https://github.com/sponsors/TSODev"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium text-white bg-gray-800 hover:bg-gray-900 transition duration-150"
                        >
                            "♥ GitHub Sponsors"
                        </a>
                    </div>
                </div>

            </div>
        </div>
    }
}
