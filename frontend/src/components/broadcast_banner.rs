// src/components/broadcast_banner.rs
use leptos::*;

#[component]
pub fn BroadcastBanner(message: String, on_dismiss: Callback<()>) -> impl IntoView {
    // Auto-dismiss après 10 secondes
    let on_dismiss_auto = on_dismiss.clone();
    spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(10_000).await;
        on_dismiss_auto.call(());
    });

    view! {
        <div class="fixed bottom-6 left-0 right-0 z-50 flex justify-center px-4 pointer-events-none">
            <div class="w-full max-w-md pointer-events-auto bg-white rounded-2xl shadow-2xl border border-indigo-100 p-4 flex items-start gap-3">
                // Icône info
                <div class="shrink-0 w-8 h-8 rounded-full bg-indigo-50 flex items-center justify-center mt-0.5">
                    <svg class="w-4 h-4 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" />
                    </svg>
                </div>

                // Texte
                <div class="flex-1 min-w-0">
                    <p class="text-xs font-semibold text-indigo-600 uppercase tracking-wide mb-0.5">
                        "Message de LimTrack"
                    </p>
                    <p class="text-sm text-gray-700 leading-snug">{message}</p>
                </div>

                // Bouton fermer
                <button
                    on:click=move |_| on_dismiss.call(())
                    class="shrink-0 text-gray-300 hover:text-gray-500 transition duration-150 mt-0.5"
                    aria-label="Fermer"
                >
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
        </div>
    }
}
