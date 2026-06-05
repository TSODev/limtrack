use leptos::*;
use leptos_router::*;

#[component]
pub fn PrivacyPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-100">

            // ─── Navbar ──────────────────────────────────────────────
            <nav class="bg-white shadow-sm border-b border-gray-200">
                <div class="max-w-4xl mx-auto px-4 h-14 md:h-16 flex items-center justify-between">
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

            <div class="max-w-4xl mx-auto px-4 py-4 md:py-8 space-y-4 md:space-y-6">

                // ─── En-tête ─────────────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6">
                    <h1 class="text-2xl font-bold text-gray-900 mb-2">"Politique de confidentialité"</h1>
                    <p class="text-sm text-gray-500">"Dernière mise à jour : juin 2026"</p>
                </div>

                // ─── Sections ────────────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-6 text-sm text-gray-700 leading-relaxed">

                    <section class="space-y-2">
                        <h2 class="text-base font-bold text-gray-900">"1. Présentation"</h2>
                        <p>
                            "LimTrack est une application de gestion de flotte kilométrique développée par Thierry Soulie (TSODev). "
                            "Cette politique décrit les données collectées, leur utilisation et vos droits."
                        </p>
                        <p>
                            "Contact : "
                            <a href="mailto:thierry.soulie@tsodev.fr" class="text-indigo-600 hover:underline">
                                "thierry.soulie@tsodev.fr"
                            </a>
                        </p>
                    </section>

                    <section class="space-y-2">
                        <h2 class="text-base font-bold text-gray-900">"2. Données collectées"</h2>
                        <p>"LimTrack collecte uniquement les données nécessaires au fonctionnement du service :"</p>
                        <ul class="list-disc pl-5 space-y-1">
                            <li>"Nom d'utilisateur et adresse email (à l'inscription)"</li>
                            <li>"Mot de passe (stocké sous forme de hash bcrypt — jamais en clair)"</li>
                            <li>"Données de véhicules : marque, modèle, immatriculation, année"</li>
                            <li>"Données de contrats : dates, kilométrages, assureur"</li>
                            <li>"Relevés kilométriques : valeur et date"</li>
                            <li>"Données de flotte : entreprise, organisation, membres et rôles"</li>
                        </ul>
                        <p>"Aucune donnée de localisation, de paiement ou de navigation n'est collectée."</p>
                    </section>

                    <section class="space-y-2">
                        <h2 class="text-base font-bold text-gray-900">"3. Utilisation des données"</h2>
                        <p>"Vos données sont utilisées exclusivement pour :"</p>
                        <ul class="list-disc pl-5 space-y-1">
                            <li>"Vous authentifier et sécuriser votre compte"</li>
                            <li>"Afficher et gérer vos véhicules, contrats et relevés"</li>
                            <li>"Envoyer des notifications d'expiration de licence par email (Resend)"</li>
                            <li>"Gérer les accès partagés à vos véhicules"</li>
                        </ul>
                        <p>"Vos données ne sont jamais vendues, louées ou partagées avec des tiers à des fins commerciales."</p>
                    </section>

                    <section class="space-y-2">
                        <h2 class="text-base font-bold text-gray-900">"4. Hébergement et sous-traitants"</h2>
                        <ul class="list-disc pl-5 space-y-1">
                            <li>
                                <strong>"Base de données"</strong>
                                " : NeonDB (PostgreSQL) — hébergé en Europe"
                            </li>
                            <li>
                                <strong>"Backend"</strong>
                                " : Railway — hébergé aux États-Unis (données chiffrées en transit)"
                            </li>
                            <li>
                                <strong>"Frontend"</strong>
                                " : Cloudflare Pages — réseau CDN mondial"
                            </li>
                            <li>
                                <strong>"Emails"</strong>
                                " : Resend — uniquement pour les notifications transactionnelles"
                            </li>
                        </ul>
                    </section>

                    <section class="space-y-2">
                        <h2 class="text-base font-bold text-gray-900">"5. Durée de conservation"</h2>
                        <p>
                            "Vos données sont conservées tant que votre compte est actif. "
                            "Vous pouvez supprimer votre compte à tout moment depuis la page Profil — "
                            "toutes vos données sont alors supprimées définitivement (CASCADE)."
                        </p>
                    </section>

                    <section class="space-y-2">
                        <h2 class="text-base font-bold text-gray-900">"6. Vos droits (RGPD)"</h2>
                        <p>"Conformément au RGPD, vous disposez des droits suivants :"</p>
                        <ul class="list-disc pl-5 space-y-1">
                            <li>"Droit d'accès à vos données"</li>
                            <li>"Droit de rectification"</li>
                            <li>"Droit à l'effacement (suppression de compte)"</li>
                            <li>"Droit à la portabilité (export CSV/PDF disponible)"</li>
                            <li>"Droit d'opposition au traitement"</li>
                        </ul>
                        <p>
                            "Pour exercer ces droits, contactez : "
                            <a href="mailto:thierry.soulie@tsodev.fr" class="text-indigo-600 hover:underline">
                                "thierry.soulie@tsodev.fr"
                            </a>
                        </p>
                    </section>

                    <section class="space-y-2">
                        <h2 class="text-base font-bold text-gray-900">"7. Sécurité"</h2>
                        <ul class="list-disc pl-5 space-y-1">
                            <li>"Mots de passe hashés avec bcrypt"</li>
                            <li>"Authentification par tokens JWT"</li>
                            <li>"Secrets gérés via Infisical (chiffrement E2EE)"</li>
                            <li>"Communications chiffrées en TLS/HTTPS"</li>
                            <li>"Validation de la robustesse des mots de passe (zxcvbn, score ≥ 3/4)"</li>
                        </ul>
                    </section>

                    <section class="space-y-2">
                        <h2 class="text-base font-bold text-gray-900">"8. Cookies et tracking"</h2>
                        <p>
                            "LimTrack n'utilise pas de cookies de tracking, de publicité ou d'analyse comportementale. "
                            "Le token d'authentification est stocké dans le localStorage de votre navigateur uniquement pour maintenir votre session."
                        </p>
                    </section>

                    <section class="space-y-2">
                        <h2 class="text-base font-bold text-gray-900">"9. Modifications"</h2>
                        <p>
                            "Cette politique peut être mise à jour. La date de dernière modification est indiquée en haut de page. "
                            "Pour toute question : "
                            <a href="mailto:thierry.soulie@tsodev.fr" class="text-indigo-600 hover:underline">
                                "thierry.soulie@tsodev.fr"
                            </a>
                        </p>
                    </section>

                </div>
            </div>
        </div>
    }
}
