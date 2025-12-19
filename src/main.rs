mod auth;
mod calendar;
mod dashboard;
mod database;
mod groups;
mod navbar;
mod todos;
mod user;

use crate::todos::frontend::todo_view::*;
use crate::auth::backend::{AuthStatus, AuthView};
use crate::auth::ui::{LoginView, RegisterView};
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
