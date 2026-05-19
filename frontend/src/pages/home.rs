use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <nav class="bg-blue-200 pt-3" data-pg-collapsed>
            <div class="gap-10 mt-15 text-right">
                <a class="bg-sky-500 font-bold hover:bg-sky-700 m-5 p-4 rounded-lg text-white" href="/login">Login</a>
                <a class="bg-blue-500 font-bold hover:bg-blue-700 m-5 p-4 rounded-lg text-white" href="/register">Inscription</a>
            </div>
        </nav>
        <header class="bg-blue-200 p-20" data-pg-collapsed>
            <h1 class="text-6xl font-black">ODO.IO</h1>
            <div>
                <p class="font-medium mt-8 text-2xl">
                    Définitions de odomètre <span class="font-normal italic text-xl"> (nom masculin)</span> </p>
                <p class="italic mt-8 text-xl">Appareil qui sert à mesurer un trajet parcouru par un véhicule ou un piéton</p>
            </div>
        </header>
        <section data-empty-placeholder class="bg-blue-200"></section>
        <footer class="bg-blue-200 p-5" data-pg-collapsed>
            <p class="text-base text-center">version : ## - (c) TSODev 2026</p>
        </footer>
    }
}
