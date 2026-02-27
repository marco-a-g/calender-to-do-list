use crate::{
    auth::backend::{AuthStatus, AuthView},
    user::backend::{create_profile, is_username_available},
};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

fn input_style() -> &'static str {
    "
    width: 100%;
    height: 44px;
    padding: 0 14px;
    border-radius: 12px;
    background: rgba(255,255,255,0.04);
    border: 1px solid rgba(255,255,255,0.08);
    color: #f9fafb;
    font-size: 14px;
    outline: none;
    box-shadow:
        inset 0 1px rgba(255,255,255,0.04),
        inset 0 -1px rgba(0,0,0,0.4);
    transition: all 0.18s ease;
    "
}

fn card_style(width: &str) -> String {
    format!(
        "
        width: {};
        padding: 32px;
        border-radius: 20px;
        background: linear-gradient(145deg, #1f222c, #14161f);
        border: 1px solid rgba(255,255,255,0.06);
        box-shadow: 0 40px 90px rgba(0,0,0,0.9);
        display: flex;
        flex-direction: column;
        gap: 16px;
        ",
        width
    )
}

#[component]
pub fn CreateProfileView(auth_status: Signal<AuthStatus>, auth_view: Signal<AuthView>) -> Element {
    let mut username = use_signal(String::new);
    let mut info = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut status = use_signal(|| None::<bool>);
    let mut checking = use_signal(|| false);

    // konzeptionelle Hilfe von KI
    use_effect(move || {
        let name = username();

        // check only on input
        if name.len() == 0 {
            checking.set(false);
            status.set(None);
            return;
        }

        checking.set(true);
        status.set(None); // makes loading icon appear when typing

        spawn(async move {
            sleep(Duration::from_millis(500)).await;

            // prevents race condition with checking only most recent input
            if username() != name {
                return;
            }

            let available = is_username_available(&name).await;
            status.set(Some(available));
            checking.set(false);
        });
    });

    // konzeptionelle Hilfe von KI
    let username_check = match status() {
        None if checking() => rsx!(div {
            class: "animate-spin rounded-full h-10 w-10 border-t-2 border-b-2 border-blue-500",
            style: "
            position: absolute;
            top: 1em;
            right: 1em;
            height: 1em;
            width: 1em;
            "
        }),
        Some(true) => rsx!(span {style: "
            position: absolute;
            right: 1em;
            top: 50%;
            transform: translateY(-50%);
            pointer-events: none;
        ", "✅"}),
        Some(false) => rsx!(span {style: "
            position: absolute;
            right: 1em;
            top: 50%;
            transform: translateY(-50%);
            pointer-events: none;
        ", "❌"}),
        _ => rsx!(),
    };

    rsx! {
        div {
            style: "
                width: 100vw;
                height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
                background: radial-gradient(circle at top, #11121b, #050609);
            ",

            div {
                style: "{card_style(\"420px\")}",

                h1 {
                    style: "
                        font-size: 20px;
                        font-weight: 600;
                        color: #f9fafb;
                        text-align: center;
                        margin-bottom: 6px;
                    ",
                    "Choose Username"
                }

                div { style: "position: relative;",
                    input { placeholder: "Username", value: "{username}", oninput: move |e| username.set(e.value()), style: input_style() }
                    {username_check}
                }

                if let Some(msg) = error() {
                    div {
                        style: "color: #f87171; font-size: 13px;",
                        "{msg}"
                    }
                }

                if let Some(msg) = info() {
                    div {
                        style: "color: #78dd35ff; font-size: 13px;",
                        "{msg}"
                    }
                }

                button {
                    style: "
                        height: 44px;
                        border-radius: 14px;
                        background: linear-gradient(180deg, #6b8bff, #4c6fff);
                        color: white;
                        font-weight: 600;
                        cursor: pointer;
                        box-shadow: 0 14px 30px rgba(107,139,255,0.45);
                        border: none;
                        margin-top: 4px;
                    ",
                    onclick: move |_| {
                        spawn(async move {
                            match create_profile(&username()).await {
                                    Ok(status) => {
                                        info.set(Some("Profile created".to_string()));
                                        error.set(None);
                                        auth_status.set(status);
                                    },
                                    Err(msg) => {
                                        error.set(Some(msg.to_string()));
                                        info.set(None);
                                    },
                                }
                            });
                    },
                    "Create Profile"
                }

                button {
                    style: "
                        background: none;
                        border: none;
                        color: #9ca3af;
                        font-size: 13px;
                        cursor: pointer;
                    ",
                    onclick: move |_| auth_view.set(AuthView::Login),
                    "Back to login"
                }
            }
        }
    }
}
