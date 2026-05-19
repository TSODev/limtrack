use leptos::*;
use leptos_router::*;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let navigate = use_navigate();

    let (username, set_username) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (confirm_password, set_confirm_password) = create_signal(String::new());
    let (status_message, set_status_message) = create_signal(String::new());
    let (is_success, set_is_success) = create_signal(false);

    let register_action = create_action(
        move |(user, mail, pass, confirm): &(String, String, String, String)| {
            let user = user.clone();
            let mail = mail.clone();
            let pass = pass.clone();
            let confirm = confirm.clone();
            let navigate_submit = navigate.clone();

            async move {
                // Validation côté client
                if pass != confirm {
                    set_status_message.set("Les mots de passe ne correspondent pas.".to_string());
                    set_is_success.set(false);
                    return;
                }

                if pass.len() < 8 {
                    set_status_message
                        .set("Le mot de passe doit contenir au moins 8 caractères.".to_string());
                    set_is_success.set(false);
                    return;
                }

                let url = "/api/user/register";

                let mut opts = web_sys::RequestInit::new();
                opts.method("POST");

                let headers = web_sys::Headers::new().expect("Impossible de créer les headers");
                headers
                    .set("Content-Type", "application/json")
                    .expect("Impossible de définir Content-Type");
                opts.headers(&headers);

                let body = serde_json::json!({
                    "username": user,
                    "email": mail,
                    "password": pass,
                });
                opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));

                let request = web_sys::Request::new_with_str_and_init(url, &opts)
                    .expect("Impossible de créer la requête");

                let window = leptos::window();
                match wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
                    .await
                {
                    Ok(resp_value) => {
                        use wasm_bindgen::JsCast;
                        let resp: web_sys::Response =
                            resp_value.dyn_into().expect("Réponse invalide");

                        if resp.ok() || resp.status() == 201 {
                            set_is_success.set(true);
                            set_status_message
                                .set("Compte créé avec succès ! Redirection...".to_string());
                            // Redirection vers le login après 2 secondes
                            let navigate_delayed = navigate_submit.clone();
                            gloo_timers::future::TimeoutFuture::new(2_000).await;
                            navigate_delayed("/", NavigateOptions::default());
                        } else {
                            set_is_success.set(false);
                            set_status_message.set(format!(
                                "Erreur : {} — identifiant ou email déjà utilisé.",
                                resp.status()
                            ));
                        }
                    }
                    Err(_) => {
                        set_is_success.set(false);
                        set_status_message.set("Impossible de contacter le serveur.".to_string());
                    }
                }
            }
        },
    );

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        register_action.dispatch((
            username.get(),
            email.get(),
            password.get(),
            confirm_password.get(),
        ));
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8 p-8 bg-white rounded-xl shadow-lg border border-gray-100">

                // En-tête
                <div class="text-center">
                    <span class="text-2xl font-bold text-indigo-600">"odo.io"</span>
                    <h2 class="mt-4 text-3xl font-extrabold text-gray-900 tracking-tight">
                        "Créer un compte"
                    </h2>
                    <p class="mt-2 text-sm text-gray-600">
                        "Rejoignez votre espace de gestion de flotte"
                    </p>
                </div>

                // Formulaire
                <form class="mt-8 space-y-5" on:submit=on_submit>

                    // Nom d'utilisateur
                    <div class="space-y-2">
                        <label for="username" class="text-sm font-medium text-gray-700 block">
                            "Nom d'utilisateur"
                        </label>
                        <input
                            type="text"
                            id="username"
                            prop:value=username
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            required
                            minlength="3"
                            class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            placeholder="ex: jean_dupont"
                        />
                    </div>

                    // Email
                    <div class="space-y-2">
                        <label for="email" class="text-sm font-medium text-gray-700 block">
                            "Adresse e-mail"
                        </label>
                        <input
                            type="email"
                            id="email"
                            prop:value=email
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            required
                            class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            placeholder="jean@exemple.fr"
                        />
                    </div>

                    // Mot de passe
                    <div class="space-y-2">
                        <label for="password" class="text-sm font-medium text-gray-700 block">
                            "Mot de passe"
                        </label>
                        <input
                            type="password"
                            id="password"
                            prop:value=password
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            required
                            minlength="8"
                            class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            placeholder="8 caractères minimum"
                        />
                    </div>

                    // Confirmation mot de passe
                    <div class="space-y-2">
                        <label for="confirm_password" class="text-sm font-medium text-gray-700 block">
                            "Confirmer le mot de passe"
                        </label>
                        <input
                            type="password"
                            id="confirm_password"
                            prop:value=confirm_password
                            on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            required
                            class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            placeholder="••••••••"
                        />
                    </div>

                    // Bouton de soumission
                    <div>
                        <button
                            type="submit"
                            prop:disabled=register_action.pending()
                            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                        >
                            {move || {
                                if register_action.pending().get() {
                                    "Création en cours..."
                                } else {
                                    "Créer mon compte"
                                }
                            }}
                        </button>
                    </div>
                </form>

                // Message de statut
                <div class="min-h-[24px] text-center">
                    <p class=move || {
                        if status_message.get().is_empty() {
                            "hidden"
                        } else if is_success.get() {
                            "text-sm font-medium text-green-600 animate-pulse"
                        } else {
                            "text-sm font-medium text-red-600"
                        }
                    }>
                        {move || status_message.get()}
                    </p>
                </div>

                // Lien vers login
                <p class="text-center text-sm text-gray-600">
                    "Déjà un compte ? "
                    <a href="/login" class="font-medium text-indigo-600 hover:text-indigo-500 transition duration-150">
                        "Se connecter"
                    </a>
                </p>

            </div>
        </div>
    }
}
