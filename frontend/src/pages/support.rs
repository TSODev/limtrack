use leptos::*;

#[component]
pub fn SupportPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-100">

            // ─── Navbar ───────────────────────────────────────────────
            <nav class="bg-white shadow-sm border-b border-gray-200" style="padding-top: var(--nav-top)">
                <div class="max-w-4xl mx-auto px-4 h-14 md:h-16 flex items-center justify-between">
                    <a
                        href="https://limtrack.app"
                        class="flex items-center gap-2 text-indigo-600 hover:text-indigo-700 font-medium text-sm transition duration-150"
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
                        </svg>
                        "Accueil"
                    </a>
                    <span class="text-xl font-bold text-indigo-600">"LimTrack"</span>
                    <div class="w-20" />
                </div>
            </nav>

            <div class="max-w-4xl mx-auto px-4 py-6 md:py-10 space-y-6">

                // ─── Hero ─────────────────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 text-center space-y-3">
                    <div class="flex items-center justify-center">
                        <div class="w-14 h-14 rounded-2xl bg-indigo-50 flex items-center justify-center">
                            <svg class="w-8 h-8 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M9.879 7.519c1.171-1.025 3.071-1.025 4.242 0 1.172 1.025 1.172 2.687 0 3.712-.203.179-.43.326-.67.442-.745.361-1.45.999-1.45 1.827v.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 5.25h.008v.008H12v-.008Z" />
                            </svg>
                        </div>
                    </div>
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900">"Support LimTrack"</h1>
                        <p class="text-sm text-gray-500 mt-1">"Comment pouvons-nous vous aider ?"</p>
                    </div>
                </div>

                // ─── Contact direct ───────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 space-y-3">
                    <h2 class="text-base font-bold text-gray-900">"Contacter le support"</h2>
                    <p class="text-sm text-gray-600 leading-relaxed">
                        "Pour toute question, problème technique ou demande d'assistance, "
                        "envoyez-nous un email. Nous répondons généralement sous 48 heures."
                    </p>
                    <a
                        href=format!("mailto:{}", crate::config::CONTACT_EMAIL)
                        class="inline-flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 transition duration-150"
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M21.75 6.75v10.5a2.25 2.25 0 0 1-2.25 2.25h-15a2.25 2.25 0 0 1-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0 0 19.5 4.5h-15a2.25 2.25 0 0 0-2.25 2.25m19.5 0v.243a2.25 2.25 0 0 1-1.07 1.916l-7.5 4.615a2.25 2.25 0 0 1-2.36 0L3.32 8.91a2.25 2.25 0 0 1-1.07-1.916V6.75" />
                        </svg>
                        {crate::config::CONTACT_EMAIL}
                    </a>
                </div>

                // ─── Signaler un bug ──────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 space-y-3">
                    <h2 class="text-base font-bold text-gray-900">"Signaler un problème"</h2>
                    <p class="text-sm text-gray-600 leading-relaxed">
                        "LimTrack est open source. Vous pouvez signaler un bug ou suggérer "
                        "une amélioration directement sur GitHub."
                    </p>
                    <a
                        href="https://github.com/TSODev/limtrack/issues"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="inline-flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium text-white transition duration-150"
                        style="background-color:#1f2937"
                    >
                        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                            <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0 1 12 6.844a9.59 9.59 0 0 1 2.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.02 10.02 0 0 0 22 12.017C22 6.484 17.522 2 12 2Z" />
                        </svg>
                        "Ouvrir une issue GitHub"
                    </a>
                </div>

                // ─── FAQ ──────────────────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 space-y-5">
                    <h2 class="text-base font-bold text-gray-900">"Questions fréquentes"</h2>

                    <div class="space-y-4 divide-y divide-gray-100">

                        <div class="space-y-1">
                            <h3 class="text-sm font-semibold text-gray-800">
                                "Comment créer un compte ?"
                            </h3>
                            <p class="text-sm text-gray-600 leading-relaxed">
                                "Rendez-vous sur "
                                <a href="https://limtrack.app/register" class="text-indigo-600 hover:underline">"limtrack.app/register"</a>
                                " et renseignez votre nom d'utilisateur, email et mot de passe. "
                                "Une période d'essai gratuite de 3 mois est incluse à l'inscription."
                            </p>
                        </div>

                        <div class="pt-4 space-y-1">
                            <h3 class="text-sm font-semibold text-gray-800">
                                "J'ai oublié mon mot de passe, comment le réinitialiser ?"
                            </h3>
                            <p class="text-sm text-gray-600 leading-relaxed">
                                "Sur la page de connexion, cliquez sur \"Mot de passe oublié\" et "
                                "renseignez votre adresse email. Vous recevrez un lien de "
                                "réinitialisation valable 1 heure."
                            </p>
                        </div>

                        <div class="pt-4 space-y-1">
                            <h3 class="text-sm font-semibold text-gray-800">
                                "Comment ajouter un véhicule ?"
                            </h3>
                            <p class="text-sm text-gray-600 leading-relaxed">
                                "Depuis la page principale, appuyez sur le bouton \"+ Ajouter un véhicule\", "
                                "renseignez la marque, le modèle et l'immatriculation. "
                                "Vous pouvez ensuite ajouter vos contrats LOA et assurance."
                            </p>
                        </div>

                        <div class="pt-4 space-y-1">
                            <h3 class="text-sm font-semibold text-gray-800">
                                "Comment partager un véhicule avec un autre utilisateur ?"
                            </h3>
                            <p class="text-sm text-gray-600 leading-relaxed">
                                "Depuis la fiche d'un véhicule, utilisez le bouton \"Partager\" pour "
                                "générer un code de partage au format XXX-XXX-XXX. "
                                "L'autre utilisateur peut rejoindre le véhicule depuis son compte "
                                "avec le bouton \"Rejoindre\"."
                            </p>
                        </div>

                        <div class="pt-4 space-y-1">
                            <h3 class="text-sm font-semibold text-gray-800">
                                "L'application iOS est-elle différente de la version web ?"
                            </h3>
                            <p class="text-sm text-gray-600 leading-relaxed">
                                "La version iOS disponible sur l'App Store est un achat unique qui "
                                "donne accès à toutes les fonctionnalités personnelles de suivi "
                                "kilométrique (véhicules, contrats LOA, assurance, relevés). "
                                "La version web inclut en plus la gestion de flotte pour les entreprises."
                            </p>
                        </div>

                        <div class="pt-4 space-y-1">
                            <h3 class="text-sm font-semibold text-gray-800">
                                "Mes données sont-elles sécurisées ?"
                            </h3>
                            <p class="text-sm text-gray-600 leading-relaxed">
                                "Vos données sont hébergées sur un serveur en France (OVH, Roubaix). "
                                "Les communications sont chiffrées via HTTPS/TLS. "
                                "Les mots de passe sont stockés sous forme de hash bcrypt. "
                                "Consultez notre "
                                <a href="/privacy" class="text-indigo-600 hover:underline">"politique de confidentialité"</a>
                                " pour plus de détails."
                            </p>
                        </div>

                    </div>
                </div>

            </div>
        </div>
    }
}
