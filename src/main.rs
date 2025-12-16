#![allow(non_snake_case)]

mod auth;
mod calendar;
mod dashboard;
mod database;
mod groups;
mod navbar;
mod todos;
mod user;

use crate::navbar::ui::*;
use crate::todos::ui::*;
use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }

        Router::<Route> {}
    }
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    DashboardView,

    #[route("/todos")]
    ToDoView,
}

#[component]
fn DashboardView() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full text-white gap-4",
            h1 { class: "text-3xl font-bold", "Willkommen bei Plantify" }
            p { class: "text-gray-400", "Wähle links im Menü eine Funktion aus." }
        }
    }
}
