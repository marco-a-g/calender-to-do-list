//! Auth related UI elements
use crate::auth::backend::*;
use dioxus::prelude::*;

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

/// UI Login Window
#[component]
pub fn LoginView(auth_status: Signal<AuthStatus>, auth_view: Signal<AuthView>) -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut remember_me = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

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
                style: "{card_style(\"380px\")}",

                h1 {
                    style: "
                        font-size: 22px;
                        font-weight: 600;
                        color: #f9fafb;
                        text-align: center;
                        margin-bottom: 8px;
                    ",
                    "PLANIFY"
                }

                input {
                    r#type: "text",
                    placeholder: "E-Mail",
                    value: "{email}",
                    oninput: move |e| email.set(e.value()),
                    style: input_style(),
                }

                input {
                    r#type: "password",
                    placeholder: "Password",
                    value: "{password}",
                    oninput: move |e| password.set(e.value()),
                    style: input_style(),
                }

                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 10px;
                        font-size: 13px;
                        color: #9ca3af;
                    ",

                    input {
                        r#type: "checkbox",
                        checked: "{remember_me}",
                        onchange: move |_| remember_me.set(!remember_me()),
                    }

                    "Angemeldet bleiben"
                }

                if let Some(msg) = error() {
                    div {
                        style: "color: #f87171; font-size: 13px;",
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
                    ",
                    onclick: move |_| {
                        spawn(async move {
                            match login(&email(), &password()).await {
                                    Ok(status) => auth_status.set(status),
                                    Err(msg) => error.set(Some(msg.to_string())),
                                }
                            });
                    },
                    "Login"
                }

                button {
                    style: "
                        background: none;
                        border: none;
                        color: #9ca3af;
                        font-size: 13px;
                        cursor: pointer;
                        margin-top: 4px;
                    ",
                    onclick: move |_| auth_view.set(AuthView::Register),
                    "Register"
                }
            }
        }
    }
}

/// UI Register Window
#[component]
pub fn RegisterView(auth_view: Signal<AuthView>) -> Element {
    let mut firstname = use_signal(String::new);
    let mut lastname = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut info = use_signal(|| None::<String>);

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
                    "Register"
                }

                input { placeholder: "First name", value: "{firstname}", oninput: move |e| firstname.set(e.value()), style: input_style() }
                input { placeholder: "Last name", value: "{lastname}", oninput: move |e| lastname.set(e.value()), style: input_style() }
                input { placeholder: "Phone", value: "{phone}", oninput: move |e| phone.set(e.value()), style: input_style() }
                input { placeholder: "Email", value: "{email}", oninput: move |e| email.set(e.value()), style: input_style() }
                input { r#type: "password", placeholder: "Password", value: "{password}", oninput: move |e| password.set(e.value()), style: input_style() }

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
                            match signup(&email(), &password()).await {
                                    Ok(_) => {
                                        info.set(Some("Signup successful".to_string()));
                                        error.set(None);
                                        auth_view.set(AuthView::CreateProfile);
                                    },
                                    Err(msg) => {
                                        info.set(None);
                                        error.set(Some(msg.to_string()));
                                    },
                                }
                            });
                    },
                    "Create account"
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
