use leptos::*;
use leptos_router::*;

const APP_VERSION: &str = env!("APP_VERSION");

fn percent_encode(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            ' ' => "%20".chars().collect::<Vec<_>>(),
            '\n' => "%0A".chars().collect(),
            '\r' => vec![],
            '&' => "%26".chars().collect(),
            '?' => "%3F".chars().collect(),
            '#' => "%23".chars().collect(),
            '%' => "%25".chars().collect(),
            '"' => "%22".chars().collect(),
            '+' => "%2B".chars().collect(),
            _ => vec![c],
        })
        .collect()
}

#[component]
pub fn AboutPage() -> impl IntoView {
    let (subject, set_subject) = create_signal(String::new());
    let (message, set_message) = create_signal(String::new());
    let (sent, set_sent) = create_signal(false);

    let on_send = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        let subj = percent_encode(&subject.get());
        let body = percent_encode(&message.get());
        let mailto = format!(
            "mailto:thierry.soulie@tsodev.fr?subject={}&body={}",
            subj, body
        );
        let _ = leptos::window().location().set_href(&mailto);
        set_sent.set(true);
    };

    view! {
        <div class="min-h-screen bg-gray-100">

            // ─── Navbar ──────────────────────────────────────────────
            <nav class="bg-white shadow-sm border-b border-gray-200">
                <div class="max-w-4xl mx-auto px-4 h-14 md:h-16 flex items-center justify-between">
                    <A
                        href="/mainpage"
                        class="flex items-center gap-2 text-indigo-600 hover:text-indigo-700 font-medium text-sm transition duration-150"
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
                        </svg>
                        "Retour"
                    </A>
                    <span class="text-xl font-bold text-indigo-600">"odo.io"</span>
                    <div class="w-20" />
                </div>
            </nav>

            <div class="max-w-4xl mx-auto px-4 py-4 md:py-8 space-y-4 md:space-y-8">

                // ─── Hero ─────────────────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 text-center space-y-4">
                    <div class="flex items-center justify-center">
                        <div class="w-16 h-16 rounded-2xl bg-indigo-50 flex items-center justify-center">
                            <svg class="w-9 h-9 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                <path stroke-linecap="round" stroke-linejoin="round"
                                    d="M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.125c.621 0 1.129-.504 1.09-1.124a17.902 17.902 0 0 0-3.213-9.193 2.056 2.056 0 0 0-1.58-.86H14.25M16.5 18.75h-2.25m0-11.177v-.958c0-.568-.422-1.048-.987-1.106a48.554 48.554 0 0 0-10.026 0 1.106 1.106 0 0 0-.987 1.106v7.635m12-6.677v6.677m0 4.5v-4.5m0 0h-12" />
                            </svg>
                        </div>
                    </div>
                    <div>
                        <h1 class="text-3xl font-bold text-gray-900">"odo.io"</h1>
                        <span class="inline-block mt-2 px-3 py-1 rounded-full text-xs font-mono font-semibold bg-indigo-100 text-indigo-700">
                            {APP_VERSION}
                        </span>
                    </div>
                </div>

                // ─── Description ──────────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-3">
                    <h2 class="text-base font-bold text-gray-900">"À propos"</h2>
                    <p class="text-sm text-gray-600 leading-relaxed">
                        "odo.io est une application web de gestion de flotte kilométrique. "
                        "Elle permet de suivre vos contrats LOA et d'assurance, d'enregistrer "
                        "vos relevés kilométriques et de recevoir des alertes personnalisées "
                        "avant d'atteindre les seuils contractuels."
                    </p>
                </div>

                // ─── Contact ──────────────────────────────────────────
                <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-4 md:p-6 space-y-4">
                    <div>
                        <h2 class="text-base font-bold text-gray-900">"Contact"</h2>
                        <a
                            href="mailto:thierry.soulie@tsodev.fr"
                            class="text-sm text-indigo-600 hover:text-indigo-700 transition duration-150"
                        >
                            "thierry.soulie@tsodev.fr"
                        </a>
                    </div>

                    <div class="space-y-3">
                        <div class="space-y-1">
                            <label class="text-sm font-medium text-gray-700 block">"Sujet"</label>
                            <input
                                type="text"
                                prop:value=subject
                                on:input=move |ev| {
                                    set_sent.set(false);
                                    set_subject.set(event_target_value(&ev));
                                }
                                placeholder="Votre sujet"
                                class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm transition duration-150"
                            />
                        </div>
                        <div class="space-y-1">
                            <label class="text-sm font-medium text-gray-700 block">"Message"</label>
                            <textarea
                                prop:value=message
                                on:input=move |ev| {
                                    set_sent.set(false);
                                    set_message.set(event_target_value(&ev));
                                }
                                placeholder="Votre message..."
                                rows="4"
                                class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm resize-none transition duration-150"
                            />
                        </div>

                        <Show when=move || sent.get() fallback=|| ()>
                            <p class="text-sm text-green-600 font-medium">
                                "Votre client mail a été ouvert avec le message pré-rempli."
                            </p>
                        </Show>

                        <button
                            on:click=on_send
                            prop:disabled=move || message.get().trim().is_empty()
                            class="flex items-center gap-2 px-5 py-2 rounded-md text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-40 disabled:cursor-not-allowed transition duration-150"
                        >
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 12 3.269 3.125A59.769 59.769 0 0 1 21.485 12 59.768 59.768 0 0 1 3.27 20.875L5.999 12Zm0 0h7.5" />
                            </svg>
                            "Envoyer un message"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
