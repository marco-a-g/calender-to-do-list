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
use crate::database::local::heartbeat::start_heartbeat;
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

    // initialize Supabase client
    // maybe wrap with use_effect
    spawn(async move {
        match init_client() {
            Ok(_) => initialized.clone().set(true),
            Err(_) => initialized.clone().set(false),
        }
    });

    use_future(|| async move {
        start_heartbeat().await;
    });
    rsx! {
        document::Stylesheet { href: CSS }

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
                }
            ),
            AuthStatus::Authenticated { .. } => rsx!(
                Router::<Route> {}
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
