use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="relative min-h-screen overflow-hidden bg-gray-950">

            // ─── Image de fond ───────────────────────────────────────
            // Unsplash photo libre de droits : route sinueuse au coucher de soleil
            <div
                class="absolute inset-0 bg-cover bg-center bg-no-repeat"
                style="background-image: url('https://images.unsplash.com/photo-1469854523086-cc02fe5d8800?w=1920&q=80');"
            />
            // Dégradé sombre par-dessus pour la lisibilité
            <div class="absolute inset-0 bg-gradient-to-b from-gray-950/70 via-gray-950/50 to-gray-950/90" />

            // ─── Contenu ─────────────────────────────────────────────
            <div class="relative z-10 flex flex-col min-h-screen">

                // Navbar
                <nav class="flex items-center justify-between px-6 md:px-12 py-5">
                    <div class="flex items-center gap-2">
                        // Logo — icône compteur stylisée
                        <svg class="w-7 h-7 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round"
                                d="M12 3v1m0 16v1M4.22 4.22l.707.707m12.02 12.02.708.708M1 12h1m20 0h1M4.22 19.78l.707-.707M18.95 5.05l.708-.707M12 7a5 5 0 1 0 0 10A5 5 0 0 0 12 7Z" />
                        </svg>
                        <span class="text-white font-bold text-xl tracking-tight">"odo.io"</span>
                    </div>
                    <div class="flex items-center gap-3">
                        <a
                            href="/login"
                            class="text-sm font-medium text-white/80 hover:text-white transition duration-150 px-4 py-2"
                        >
                            "Se connecter"
                        </a>
                        <a
                            href="/register"
                            class="text-sm font-semibold text-gray-950 bg-white hover:bg-indigo-50 px-4 py-2 rounded-full transition duration-150"
                        >
                            "S'inscrire"
                        </a>
                    </div>
                </nav>

                // Hero — centré verticalement
                <div class="flex-1 flex flex-col items-center justify-center text-center px-6 py-16 md:py-24">

                    // Badge
                    <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-sm border border-white/20 text-white/80 text-xs font-medium px-4 py-1.5 rounded-full mb-8">
                        <span class="w-1.5 h-1.5 bg-indigo-400 rounded-full animate-pulse" />
                        "Gestion de flotte intelligente"
                    </div>

                    // Titre principal
                    <h1 class="text-5xl md:text-7xl font-black text-white tracking-tight leading-tight max-w-3xl">
                        "Votre flotte,"
                        <br />
                        <span class="text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-sky-400">
                            "sous contrôle."
                        </span>
                    </h1>

                    // Sous-titre
                    <p class="mt-6 text-base md:text-lg text-white/60 max-w-xl leading-relaxed">
                        "Suivez vos contrats LOA et assurance, surveillez vos kilométrages et recevez des alertes avant de dépasser vos limites."
                    </p>

                    // CTA
                    <div class="mt-10 flex flex-col sm:flex-row items-center gap-4">
                        <a
                            href="/register"
                            class="w-full sm:w-auto flex items-center justify-center gap-2 bg-indigo-600 hover:bg-indigo-500 text-white font-semibold px-8 py-3.5 rounded-full text-sm transition duration-150 shadow-lg shadow-indigo-500/30"
                        >
                            "Commencer gratuitement"
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5 21 12m0 0-7.5 7.5M21 12H3" />
                            </svg>
                        </a>
                        <a
                            href="/login"
                            class="w-full sm:w-auto flex items-center justify-center gap-2 bg-white/10 hover:bg-white/20 backdrop-blur-sm border border-white/20 text-white font-medium px-8 py-3.5 rounded-full text-sm transition duration-150"
                        >
                            "J'ai déjà un compte"
                        </a>
                    </div>

                    // Stats
                    <div class="mt-16 md:mt-20 grid grid-cols-3 gap-8 md:gap-16">
                        <div class="text-center">
                            <p class="text-2xl md:text-3xl font-black text-white">"LOA"</p>
                            <p class="text-xs md:text-sm text-white/50 mt-1">"Contrats suivis"</p>
                        </div>
                        <div class="text-center border-x border-white/10">
                            <p class="text-2xl md:text-3xl font-black text-white">"24h"</p>
                            <p class="text-xs md:text-sm text-white/50 mt-1">"Alertes en temps réel"</p>
                        </div>
                        <div class="text-center">
                            <p class="text-2xl md:text-3xl font-black text-white">"∞"</p>
                            <p class="text-xs md:text-sm text-white/50 mt-1">"Véhicules partagés"</p>
                        </div>
                    </div>
                </div>

                // Footer
                <footer class="text-center py-5 text-white/30 text-xs">
                    "© 2026 TSODev · odo.io"
                </footer>
            </div>
        </div>
    }
}
