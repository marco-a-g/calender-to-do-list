mod auth;
mod calendar;
mod dashboard;
mod database;
mod groups;
mod navbar;
mod todos;
mod user;
mod utils;

use crate::auth::backend::{AuthStatus, AuthView, init_client};
use crate::auth::ui::{CreateProfileView, LoginView, RegisterView};
use crate::database::local::heartbeat::start_heartbeat;
use crate::database::local::init_fetch::init_fetch_local_db::init_database;
use crate::todos::frontend::todo_view::*;
use dioxus::prelude::*;
use dioxus_router::{Routable, Router};

static CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(navbar::ui::Navbar)]
    #[route("/")]
    DashboardView,

    #[route("/todos")]
    ToDoView,

    #[route("/Calendar")]
    Calendar,

    #[route("/Groups")]
    Groups,
}

#[component]
fn App() -> Element {
    let auth_status = use_signal(|| AuthStatus::Unauthenticated);
    let auth_view = use_signal(|| AuthView::Login);
    let initialized = use_signal(|| false); // use later to enable offline mode/view, maybe enum ClientState {Ready, Offline, Error(AuthError)}
    let mut db_is_ready = use_signal(|| false);

    // initialize Supabase client
    // maybe wrap with use_effect
    spawn(async move {
        match init_client() {
            Ok(_) => initialized.clone().set(true),
            Err(_) => initialized.clone().set(false),
        }
    });

    use_effect(move || {
        // DB init & Heartbeat startet erst, wenn user auth ist
        if let AuthStatus::Authenticated { .. } = auth_status() {
            spawn(async move {
                println!("Login erfolgreich. Starte Local-DB-Initialisierung...");
                match init_database().await {
                    Ok(_) => {
                        println!("DB Init erfolgreich.");
                        db_is_ready.set(true);
                        start_heartbeat().await;
                    }
                    Err(e) => {
                        eprintln!("Datenbank konnte nicht initialisiert werden: {}", e);
                    }
                }
            });
        }
    });

    rsx! {
        document::Stylesheet { href: CSS }

        // nach signup (bei aktivierter Email Verification auch erst danach) ist man schon authenticated, heißt CreateProfile müsste vielleicht in AuthStatus::Authenticated, aber das kann Probleme geben, weil App vllt davon ausgeht, dass Profil schon existiert
        match auth_status() {
            AuthStatus::Unauthenticated => rsx!(
                match auth_view() {
                    AuthView::Login => rsx!(
                        LoginView {
                            auth_status,
                            auth_view,
                        }
                    ),
                    AuthView::Register => rsx!(
                        RegisterView {
                            auth_view,
                        }
                    ),
                    AuthView::CreateProfile => rsx!(
                        CreateProfileView {
                            auth_status,
                            auth_view,
                        }
                    ),
                }
            ),
            AuthStatus::Authenticated { .. } => rsx!(
                if db_is_ready() {
                    Router::<Route> {}
                } else {
                    // Ladebildschirm während init_database() läuft
                    div {
                        class: "h-screen w-screen flex flex-col items-center justify-center bg-[#080910] text-white gap-4",
                        div { class: "animate-spin rounded-full h-10 w-10 border-t-2 border-b-2 border-blue-500" }
                        div { "Loading DB..." }
                    }
                }
            ),
        }
    }
}

#[component]
fn DashboardView() -> Element {
    rsx! {
        div {
            "Dashboard"
        }
    }
}

#[component]
fn Calendar() -> Element {
    rsx! {
        div {
            "Calendar"
        }
    }
}

#[component]
fn Groups() -> Element {
    rsx! {
        div {
            "Groups"
        }
    }
}
