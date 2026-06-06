use leptos::*;
use leptos_router::*;

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.with(|q| q.get("token").cloned().unwrap_or_default());

    let (new_password, set_new_password) = create_signal(String::new());
    let (confirm_password, set_confirm_password) = create_signal(String::new());
    let (status, set_status) = create_signal(Option::<(bool, String)>::None);
    let (loading, set_loading) = create_signal(false);
    let (done, set_done) = create_signal(false);

    let navigate = use_navigate();

    let submit = create_action(move |(tok, new_pw, confirm_pw): &(String, String, String)| {
        let tok = tok.clone();
        let new_pw = new_pw.clone();
        let confirm_pw = confirm_pw.clone();
        let navigate = navigate.clone();
        async move {
            if tok.is_empty() {
                set_status.set(Some((false, "Lien invalide — token manquant.".to_string())));
                return;
            }
            if new_pw != confirm_pw {
                set_status.set(Some((false, "Les mots de passe ne correspondent pas.".to_string())));
                return;
            }
            if new_pw.len() < 8 {
                set_status.set(Some((false, "Le mot de passe doit contenir au moins 8 caractères.".to_string())));
                return;
            }

            set_loading.set(true);
            set_status.set(None);

            let url = format!("{}/api/user/reset-password", crate::config::API_BASE);
            let body = serde_json::json!({ "token": tok, "new_password": new_pw });

            let client = reqwest::Client::new();
            match client.post(&url).json(&body).send().await {
                Ok(resp) if resp.status().is_success() => {
                    set_done.set(true);
                    set_status.set(Some((true, "Mot de passe mis à jour ! Redirection...".to_string())));
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                    navigate("/login", NavigateOptions::default());
                }
                Ok(resp) => {
                    let msg = resp.json::<serde_json::Value>().await
                        .ok()
                        .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "Une erreur est survenue.".to_string());
                    set_status.set(Some((false, msg)));
                }
                Err(_) => {
                    set_status.set(Some((false, "Impossible de contacter le serveur.".to_string())));
                }
            }
            set_loading.set(false);
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        submit.dispatch((token(), new_password.get(), confirm_password.get()));
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-6 md:space-y-8 p-6 md:p-8 bg-white rounded-xl shadow-lg border border-gray-100">

                <div class="text-center">
                    <h2 class="text-2xl md:text-3xl font-extrabold text-gray-900 tracking-tight">
                        "Nouveau mot de passe"
                    </h2>
                    <p class="mt-2 text-sm text-gray-600">
                        "Choisissez un mot de passe fort pour sécuriser votre compte."
                    </p>
                </div>

                {move || if token().is_empty() {
                    view! {
                        <div class="rounded-lg bg-red-50 border border-red-200 p-4 text-center">
                            <p class="text-sm font-medium text-red-800">
                                "Lien invalide. Veuillez demander un nouveau lien de réinitialisation."
                            </p>
                        </div>
                    }.into_view()
                } else if done.get() {
                    view! {
                        <div class="rounded-lg bg-green-50 border border-green-200 p-4 text-center">
                            <p class="text-sm font-medium text-green-800">
                                "Mot de passe mis à jour avec succès !"
                            </p>
                            <p class="mt-1 text-xs text-green-600">"Redirection vers la connexion..."</p>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <form class="mt-8 space-y-6" on:submit=on_submit>
                            <div class="space-y-4">
                                <div class="space-y-2">
                                    <label for="new_password" class="text-sm font-medium text-gray-700 block">
                                        "Nouveau mot de passe :"
                                    </label>
                                    <input
                                        type="password"
                                        id="new_password"
                                        prop:value=new_password
                                        on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                        required
                                        autocomplete="new-password"
                                        class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                                        placeholder="••••••••"
                                    />
                                </div>

                                <div class="space-y-2">
                                    <label for="confirm_password" class="text-sm font-medium text-gray-700 block">
                                        "Confirmer le mot de passe :"
                                    </label>
                                    <input
                                        type="password"
                                        id="confirm_password"
                                        prop:value=confirm_password
                                        on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                        required
                                        autocomplete="new-password"
                                        class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                                        placeholder="••••••••"
                                    />
                                </div>
                            </div>

                            <p class="text-xs text-gray-500">
                                "Score minimum requis : 3/4 (zxcvbn). Évitez les mots de passe courants."
                            </p>

                            {move || status.get().and_then(|(ok, msg)| if !ok { Some(msg) } else { None }).map(|msg| view! {
                                <p class="text-sm font-medium text-red-600 text-center">{msg}</p>
                            })}

                            <button
                                type="submit"
                                prop:disabled=loading
                                class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                            >
                                {move || if loading.get() { "Mise à jour..." } else { "Changer le mot de passe" }}
                            </button>
                        </form>
                    }.into_view()
                }}

                <p class="text-center text-sm text-gray-600">
                    <A href="/forgot-password" class="font-medium text-indigo-600 hover:text-indigo-500 transition duration-150">
                        "Demander un nouveau lien"
                    </A>
                </p>
            </div>
        </div>
    }
}
