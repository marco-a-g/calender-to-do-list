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
use crate::auth::ui::{LoginView, RegisterView};
use crate::calendar::frontend::calendar_page::CalendarPage;
use crate::dashboard::frontend::dashboard::DashboardView;
use crate::database::local::heartbeat::start_heartbeat;
use crate::database::local::init_fetch::init_fetch_local_db::init_database;
use crate::database::local::sync_local_db::SyncIndicator;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::groups::frontend::GroupsPage;
use crate::groups::frontend::group_detail::GroupDetailPage;
use crate::todos::frontend::todo_dashboard::ToDoDashboard;
use crate::user::frontend::{create_profile::CreateProfileView, profile_view::ProfileView};
use axum::extract::DefaultBodyLimit;
use dioxus::prelude::*;
use dioxus_router::{Routable, Router};
static CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    //Launch/Windowbuilder nutzen um Kontextmenü oben in App weg zu bekommen und um "immer im Vordergrund" zu deaktivieren
    #[cfg(not(feature = "server"))]
    {
        let mut builder = LaunchBuilder::new();

        #[cfg(feature = "desktop")]
        {
            let window = dioxus::desktop::WindowBuilder::new()
                .with_title("Planify")
                .with_always_on_top(false);
            let cfg = dioxus::desktop::Config::new()
                .with_window(window)
                .with_menu(None)
                .with_disable_context_menu(true);

            builder = builder.with_cfg(cfg);
        }
        builder.launch(App);
    }
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(navbar::ui::Navbar)]
    #[route("/")]
    DashboardView,
    #[route("/todos")]
    ToDoDashboard,
    #[route("/Calendar")]
    Calendar,
    #[route("/Groups")]
    Groups,
    #[route("/Profile")]
    ProfileView,
    #[route("/groups/:id")]
    GroupDetail { id: String },
}

#[component]
fn App() -> Element {
    let auth_status = use_signal(|| AuthStatus::Unauthenticated);
    use_context_provider(|| auth_status);
    let sync_counter = use_signal(|| 0u32);
    use_context_provider(|| sync_counter);
    let auth_view = use_signal(|| AuthView::Login);
    let mut initialized = use_signal(|| false); // use later to enable offline mode/view, maybe enum ClientState {Ready, Offline, Error(AuthError)}
    let mut db_is_ready = use_signal(|| false);
    let mut is_syncing = use_signal(|| false);
    use_context_provider(|| is_syncing);
    // initialize Supabase client
    use_effect(move || {
        spawn(async move {
            match init_client() {
                Ok(_) => initialized.set(true),
                Err(_) => initialized.set(false),
            }
        });
    });

    use_effect(move || {
        // DB init & Heartbeat startet erst, wenn user auth ist
        if let AuthStatus::Authenticated { .. } = auth_status() {
            spawn(async move {
                println!("Login erfolgreich. Starte Local-DB-Initialisierung...");
                match init_database().await {
                    Ok(_) => {
                        println!("DB Init erfolgreich.");
                        if let Err(e) = sync_local_to_remote_db().await {
                            eprintln!("Initial sync failed: {}", e);
                        }
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
        SyncIndicator {}
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
                        div { "Initialization of Database ..." }
                    }
                }
            ),
        }
    }
}

#[component]
fn Calendar() -> Element {
    rsx! {
        CalendarPage {}
    }
}

#[component]
fn Groups() -> Element {
    let auth_status = use_context::<Signal<AuthStatus>>();

    rsx! { GroupsPage { auth_status } }
}

#[component]
fn GroupDetail(id: String) -> Element {
    let auth_status = use_context::<Signal<AuthStatus>>();

    rsx!(GroupDetailPage { id, auth_status })
}
