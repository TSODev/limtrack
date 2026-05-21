use leptos::*;
use leptos_router::A; // Import spécifique
use leptos_router::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let navigate = use_navigate();
    // 1. Signaux pour capturer les entrées des champs
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());

    // Un signal pour afficher un message d'erreur ou de succès à l'utilisateur
    let (status_message, set_status_message) = create_signal(String::new());

    // 2. La fonction qui va appeler ton backend (asynchrone)
    let login_action = create_action(move |(user, pass): &(String, String)| {
        let user = user.clone();
        let pass = pass.clone();

        // On clone navigate ICI, au sein de l'action
        let navigate_submit = navigate.clone();

        async move {
            // Remplace par l'URL réelle de ton backend
            let url = "https://api.tsodev.fr/login";

            // Préparation du body en JSON
            let body = serde_json::json!({
                "username": user,
                "password": pass
            });

            // Appel Fetch via reqwest (courant en Leptos frontend)
            // Assure-je d'avoir la dépendance `reqwest = { version = "...", features = ["json"] }`
            let client = reqwest::Client::new();
            match client.post(url).json(&body).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        // On extrait le token (en supposant que ton backend renvoie {"token": "..."})
                        if let Ok(json) = response.json::<serde_json::Value>().await {
                            if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                                // leptos::window() renvoie directement la Window (il panique si elle n'existe pas,
                                // ce qui n'arrive jamais dans un navigateur)
                                if let Ok(Some(storage)) = leptos::window().local_storage() {
                                    let _ = storage.set_item("jwt_token", token);
                                }
                                set_status_message.set("Connexion réussie !".to_string());
                                // 2. REDIRECTION VERS LA PAGE PRINCIPALE
                                // On utilise NavigateOptions::default() pour une redirection standard
                                navigate_submit("/mainpage", NavigateOptions::default());
                            }
                        }
                    } else {
                        set_status_message.set(format!("Erreur : {}", response.status()));
                    }
                }
                Err(_) => {
                    set_status_message
                        .set("Impossible de contacter le serveur backend.".to_string());
                }
            }
        }
    });

    // 3. Gestion de la soumission du formulaire
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default(); // Empêche la page de se recharger
                              // On déclenche l'action en lui passant le username et le password actuels
        login_action.dispatch((username.get(), password.get()));
    };

    // 4. L'interface utilisateur (HTML / Vue)
    view! {
        // Conteneur principal : prend tout l'écran, centre le formulaire verticalement et horizontalement
        <div class="min-h-screen flex items-center justify-center bg-gray-50 px-4 sm:px-6 lg:px-8">

            // Carte du formulaire : fond blanc, surélevée avec une ombre, coins arrondis
            <div class="max-w-md w-full space-y-6 md:space-y-8 p-6 md:p-8 bg-white rounded-xl shadow-lg border border-gray-100">

                // En-tête (Titre)
                <div class="text-center">
                    <h2 class="text-2xl md:text-3xl font-extrabold text-gray-900 tracking-tight">
                        "Connexion"
                    </h2>
                    <p class="mt-2 text-sm text-gray-600">
                        "Accédez à votre espace de travail"
                    </p>
                </div>

                // Formulaire
                <form class="mt-8 space-y-6" on:submit=on_submit>

                    // Groupe : Identifiant / Mouton
                    <div class="space-y-2">
                        <label for="username" class="text-sm font-medium text-gray-700 block">
                            "Nom d'utilisateur :"
                        </label>
                        <input
                            type="text"
                            id="username"
                            prop:value=username
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            required
                            class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            placeholder="Ex: shaun_the_sheep"
                        />
                    </div>

                    // Groupe : Mot de passe
                    <div class="space-y-2">
                        <label for="password" class="text-sm font-medium text-gray-700 block">
                            "Mot de passe :"
                        </label>
                        <input
                            type="password"
                            id="password"
                            prop:value=password
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            required
                            class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            placeholder="••••••••"
                        />
                    </div>

                    // Bouton de soumission avec effet de survol (hover) et état désactivé (disabled)
                    <div>
                        <button
                            type="submit"
                            prop:disabled=login_action.pending()
                            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition duration-150"
                        >
                            {move || if login_action.pending().get() { "Connexion en cours..." } else { "Se connecter" }}
                        </button>
                    </div>
                </form>

                // Zone de message dynamique (Succès ou Erreur)
                <div class="min-h-[24px] text-center">
                    <p class=move || {
                        let msg = status_message.get();
                        if msg.contains("réussie") {
                            "text-sm font-medium text-green-600 animate-pulse"
                        } else if msg.is_empty() {
                            "hidden"
                        } else {
                            "text-sm font-medium text-red-600"
                        }
                    }>
                        {move || status_message.get()}
                    </p>
                </div>

                // Lien vers l'inscription
                  <p class="text-center text-sm text-gray-600">
                           "Pas encore de compte ? "
                       <A href="/register" class="font-medium text-indigo-600 hover:text-indigo-500 transition duration-150">
                          "S'inscrire"
                      </A>
                   </p>

            </div>
        </div>
    }
}
