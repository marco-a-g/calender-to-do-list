#![allow(non_snake_case)]

mod auth;
mod calendar;
mod chat;
mod dashboard;
mod database;
mod groups;
mod navbar;
mod tasks;
mod user;

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        "Hello World!"
    }
}
