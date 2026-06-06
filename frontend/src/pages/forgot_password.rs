use leptos::*;
use leptos_router::A;

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (status, set_status) = create_signal(Option::<(bool, String)>::None);
    let (loading, set_loading) = create_signal(false);

    let submit = create_action(move |email: &String| {
        let email = email.clone();
        async move {
            set_loading.set(true);
            set_status.set(None);

            let url = format!("{}/api/user/forgot-password", crate::config::API_BASE);
            let body = serde_json::json!({ "email": email });

            let client = reqwest::Client::new();
            match client.post(&url).json(&body).send().await {
                Ok(_) => {
                    set_status.set(Some((
                        true,
                        "Si cette adresse est associée à un compte, vous recevrez un email dans quelques instants.".to_string(),
                    )));
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
        submit.dispatch(email.get());
    };

    let sent = move || status.get().map_or(false, |(ok, _)| ok);

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-6 md:space-y-8 p-6 md:p-8 bg-white rounded-xl shadow-lg border border-gray-100">

                <div class="text-center">
                    <h2 class="text-2xl md:text-3xl font-extrabold text-gray-900 tracking-tight">
                        "Mot de passe oublié"
                    </h2>
                    <p class="mt-2 text-sm text-gray-600">
                        "Entrez votre adresse email pour recevoir un lien de réinitialisation."
                    </p>
                </div>

                {move || if sent() {
                    view! {
                        <div class="rounded-lg bg-green-50 border border-green-200 p-4 text-center">
                            <p class="text-sm font-medium text-green-800">
                                {move || status.get().map(|(_, msg)| msg).unwrap_or_default()}
                            </p>
                            <p class="mt-2 text-xs text-green-600">
                                "Vérifiez aussi vos spams. Le lien expire dans 1 heure."
                            </p>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <form class="mt-8 space-y-6" on:submit=on_submit>
                            <div class="space-y-2">
                                <label for="email" class="text-sm font-medium text-gray-700 block">
                                    "Adresse email :"
                                </label>
                                <input
                                    type="email"
                                    id="email"
                                    prop:value=email
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    required
                                    autocomplete="email"
                                    class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                                    placeholder="vous@exemple.com"
                                />
                            </div>

                            {move || status.get().and_then(|(ok, msg)| if !ok { Some(msg) } else { None }).map(|msg| view! {
                                <p class="text-sm font-medium text-red-600 text-center">{msg}</p>
                            })}

                            <button
                                type="submit"
                                prop:disabled=loading
                                class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                            >
                                {move || if loading.get() { "Envoi en cours..." } else { "Envoyer le lien" }}
                            </button>
                        </form>
                    }.into_view()
                }}

                <p class="text-center text-sm text-gray-600">
                    <A href="/login" class="font-medium text-indigo-600 hover:text-indigo-500 transition duration-150">
                        "Retour à la connexion"
                    </A>
                </p>
            </div>
        </div>
    }
}
