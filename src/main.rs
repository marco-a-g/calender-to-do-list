mod auth;
mod calendar;
mod dashboard;
mod database;
mod groups;
mod navbar;
mod todos;
mod user;

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
    rsx! {
        document::Stylesheet { href: CSS }
        Router::<Route> {}
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
fn ToDoView() -> Element {
    rsx! {
        div {
            "ToDos"
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
