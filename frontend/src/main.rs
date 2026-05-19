// On utilise le type partagé !
use leptos::*;
use leptos_router::*;

pub mod components;
pub mod models;

mod pages; // On importe notre nouveau module
use pages::home::HomePage;
use pages::login::LoginPage;
use pages::mainpage::MainPage;
use pages::register::RegisterPage;

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes>
                    <Route path="/" view=HomePage/>
                    <Route path="/login" view=LoginPage/>
                    <Route path="mainpage" view=MainPage/>
                    <Route path="/register" view=RegisterPage />
                </Routes>
            </main>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
