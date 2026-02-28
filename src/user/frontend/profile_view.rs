use crate::user::backend::{get_own_username, is_username_available, update_username};
use dioxus::{core::Element, prelude::*};
use std::time::Duration;
use tokio::time::sleep;

fn input_style_enabled() -> &'static str {
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

fn input_style_disabled() -> &'static str {
    "
    width: 100%;
    height: 44px;
    padding: 0 14px;
    border-radius: 12px;
    background: rgba(255,255,255,0.04);
    border: 1px solid rgba(255,255,255,0.08);
    color: #a1a4a7;
    font-size: 14px;
    outline: none;
    box-shadow:
        inset 0 1px rgba(255,255,255,0.04),
        inset 0 -1px rgba(0,0,0,0.4);
    transition: all 0.18s ease;
    "
}

#[component]
pub fn ProfileView() -> Element {
    let mut username = use_signal(String::new); // dynamic signal for input field
    let mut own_username = use_signal(String::new); // holds current username as it is in db
    let mut username_fetch = use_resource(get_own_username);
    let mut editing = use_signal(|| false);
    let mut info = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut error_hovered = use_signal(|| false);
    let mut status = use_signal(|| None::<bool>); // if username is available
    let mut checking = use_signal(|| false); // if currently checking for username availability

    // fetch username
    use_effect(move || {
        if let Some(Ok(name)) = &*username_fetch.read() {
            username.set(name.into());
            own_username.set(name.into());
        }
    });

    // konzeptionelle Hilfe von KI
    use_effect(move || {
        let name = username();

        // check only on input
        if name.is_empty() {
            checking.set(false);
            status.set(None);
            return;
        }

        // check not if input is own username
        if name == own_username() {
            checking.set(false);
            status.set(None);
            return;
        }

        info.set(None);
        error.set(None);
        checking.set(true);
        status.set(None); // makes loading icon appear when typing

        spawn(async move {
            sleep(Duration::from_millis(500)).await; // check only after typing pause

            // prevents race condition with checking only most recent input
            if username() != name {
                return;
            }

            let available = is_username_available(&name).await;
            info.set(None);
            error.set(None);
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
            right: 10em;
            height: 1em;
            width: 1em;
            "
        }),
        Some(true) => rsx!(span {style: "
            position: absolute;
            right: 10em;
            top: 50%;
            transform: translateY(-50%);
            pointer-events: none;
        ", "✅"}),
        Some(false) => rsx!(span {style: "
            position: absolute;
            right: 10em;
            top: 50%;
            transform: translateY(-50%);
            pointer-events: none;
        ", "❌"}),
        _ => rsx!(),
    };

    rsx! {
        div {
            style:
            "flex: 1;
             padding: 24px; 
             display: flex; 
             flex-direction: column; 
             background: #080910;",
            div {
                style:
                "background: linear-gradient(145deg, #1f222c 0%, #14161f 100%);
                 border-radius: 18px; 
                 padding: 24px; 
                 box-shadow: 0 18px 40px rgba(0,0,0,0.85); 
                 border: 1px solid rgba(255,255,255,0.06); 
                 flex: 1; 
                 display: flex; 
                 flex-direction: column; 
                 gap: 16px; 
                 overflow: hidden;
                 max-width: 35em;",
                div {
                    style:
                    "border-bottom: 1px solid rgba(255,255,255,0.06);
                     padding-bottom: 16px; 
                     margin-bottom: 8px;",
                    h1 { style:
                        "margin: 0;
                         font-size: 24px; 
                         font-weight: 600; 
                         color: #f9fafb;", 
                         "Profil" }
                }

                div { // div for all rows
                    div { class: "", // flex-1 overflow-y-auto pr-2 flex flex-col gap-3 // username
                        style: "
                            position: relative;
                            display: flex;
                            flex-direction: row;
                            gap: 1em;
                        ",

                        div { style:
                            "font-size: 16px;
                            color: #bbbfc7; 
                            font-weight: 600; 
                            margin-top: 8px; 
                            margin-bottom: 4px; 
                            text-transform: uppercase; 
                            letter-spacing: 0.05em;", 
                            "Username"
                        }

                        match editing() {
                            false => rsx!(
                                input { value: "{username}", style: input_style_disabled(), disabled: "true"}

                                button {style: "
                                    //position: absolute;
                                    //right: 1em;
                                    //top: 50%;
                                    //transform: translateY(-50%);
                                    cursor: pointer;

                                    display: flex;
                                    gap: 10px;
                                    flex: 1;
                                    padding: .5em .5em .5em .5em;
                                    border-radius: 8px; 
                                    background: #3A6BFF; 
                                    color: white;
                                    border: none; 
                                    font-weight: 600;
                                    font-size: 14px;
                                ", onclick: move |_| {
                                    editing.set(true);
                                },
                                "Edit"}
                            ),
                            true => rsx!(
                                input { value: "{username}", oninput: move |e| username.set(e.value()), style: input_style_enabled() }

                                {username_check}

                                div {style:
                                    "display: flex;
                                    gap: 10px; 
                                    //margin-top: 10px;",
                                    span {style: "
                                        //position: absolute;
                                        //right: 3em;
                                        //top: 50%;
                                        //transform: translateY(-50%);
                                        cursor: pointer;
                                        
                                        flex: 1;
                                        padding: .5em .5em .5em .5em;
                                        border-radius: 8px; 
                                        background: #3A6BFF; 
                                        color: white;
                                        border: none; 
                                        font-weight: 600;
                                        font-size: 14px; 
                                    ", onclick: move |_| {
                                        spawn(async move {
                                            match update_username(&username()).await {
                                                Ok(_) => {
                                                    editing.set(false);
                                                    username_fetch.restart();
                                                    checking.set(false);
                                                    status.set(None);
                                                    info.set(Some("Username changed!".to_string()));
                                                    error.set(None);
                                                },
                                                Err(msg) => {
                                                    checking.set(false);
                                                    status.set(None);
                                                    error.set(Some(msg.to_string()))
                                                },
                                            }
                                        });
                                    },
                                    "Change"}

                                    span {style: "
                                        //position: absolute;
                                        //right: 1em;
                                        //top: 50%;
                                        //transform: translateY(-50%);
                                        cursor: pointer;

                                        flex: 1;
                                        padding: .5em; 
                                        border-radius: 8px; 
                                        border: 1px solid rgba(255,255,255,0.1);
                                        color: #9ca3af; 
                                        background: transparent;
                                        font-size: 14px;
                                    ", onclick: move |_| {
                                        editing.set(false);
                                        username_fetch.restart();
                                        info.set(None);
                                        error.set(None);
                                    },
                                    "Cancel"}
                                }
                            ),
                        }

                        if let Some(msg) = info() {
                                    div { style: "
                                            position: absolute;
                                            right: 10em;
                                            top: 50%;
                                            transform: translateY(-50%);
                                    "//,  onmouseover: move |_| error_hovered.set(true)
                                    //,   onmouseout: move |_| error_hovered.set(false)
                                    ,   span {style: "
                                            position: relative;
                                            font-size: 0.8em;
                                            cursor: default;
                                        ",
                                        "ℹ️"}

                                        //if error_hovered() {
                                            span {style: "
                                                position: absolute;
                                                right: 125%;
                                                top: 50%;
                                                transform: translateY(-50%);
                                                font-size: 0.75em;
                                                color: #f9fafb;
                                                background-color: #1f222c;
                                                padding: 0.5em 0.75em;
                                                border-radius: 8px;
                                                box-shadow: 0 4px 10px rgba(0,0,0,0.5);
                                                white-space: nowrap;
                                                z-index: 10;
                                                ", "{msg}"
                                            }
                                        //}
                                    }
                                }

                                if let Some(msg) = error() {
                                    div { style: "
                                            position: absolute;
                                            right: 10em;
                                            top: 50%;
                                            transform: translateY(-50%);
                                    ",  onmouseover: move |_| error_hovered.set(true)
                                    ,   onmouseout: move |_| error_hovered.set(false)
                                    ,   span {style: "
                                            position: relative;
                                            font-size: 0.8em;
                                            cursor: default;
                                        ",
                                        "❗"}

                                        if error_hovered() {
                                            span {style: "
                                                position: absolute;
                                                bottom: 125%;
                                                left: 50%;
                                                transform: translateX(-50%);
                                                font-size: 0.75em;
                                                color: #f9fafb;
                                                background-color: #1f222c;
                                                padding: 0.5em 0.75em;
                                                border-radius: 8px;
                                                box-shadow: 0 4px 10px rgba(0,0,0,0.5);
                                                white-space: nowrap;
                                                z-index: 10;
                                                ", "{msg}"
                                            }
                                        }
                                    }
                                }
                    }
                }
            }
        }
    }
}
