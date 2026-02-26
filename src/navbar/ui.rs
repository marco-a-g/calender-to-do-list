use crate::Route;
use crate::auth::backend::{AuthStatus, logout};
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use dioxus::prelude::*;
use dioxus_router::{Link, Outlet, use_route};

#[component]
pub fn Navbar() -> Element {
    let mut auth_status = use_context::<Signal<AuthStatus>>();
    let mut sync_counter = use_context::<Signal<u32>>();

    let mut syncing = use_signal(|| false);
    let mut sync_feedback = use_signal(|| Option::<String>::None);
    rsx! {
        div {
            style: "
                width: 100vw;
                height: 100vh;
                display: flex;
                overflow: hidden;
                background: #050609;
            ",

            div {
                style: "
                    width: 112px;
                    background: linear-gradient(180deg, #11121b 0%, #05060b 100%);
                    border-right: 1px solid rgba(255,255,255,0.06);
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    padding: 18px 12px 22px 12px;
                    gap: 20px;
                ",

                div {
                    style: "
                        width: 42px;
                        height: 42px;
                        border-radius: 14px;
                        background: radial-gradient(
                            circle at 25% 0%,
                            #6b8bff 0%,
                            #24345e 40%,
                            #151725 100%
                        );
                    ",
                }


                div {
                    style: "
                        width: 80%;
                        flex: 1;
                        border-radius: 26px;
                        background: radial-gradient(circle at top, #181b24 0%, #080910 70%);
                        padding: 20px 14px;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 16px;
                    ",

                    NavButton {
                        to: Route::DashboardView ,
                        icon: "🏠",
                    }

                    NavButton {
                        to: Route::ToDoDashboard ,
                        icon: "📝",
                    }

                    NavButton {
                        to: Route::Calendar ,
                        icon: "📅",
                    }

                    NavButton {
                        to: Route::Groups ,
                        icon: "👥",
                    }

                }

                NavButton {
                    to: Route::ProfileView,
                    icon: "⚙️",
                }

                button {
                    style: "
                        width: 44px;
                        height: 44px;
                        border-radius: 14px;
                        background: rgba(107, 139, 255, 0.12);
                        border: 1px solid rgba(107,139,255,0.25);
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 18px;
                        cursor: pointer;
                        transition: background 0.2s;
                        opacity: 1;
                    ",
                    disabled: syncing(),
                    onclick: move |_| {
                        spawn(async move {
                            syncing.set(true);
                            sync_feedback.set(Some("syncing".to_string()));

                            let _ = sync_local_to_remote_db().await;

                            sync_counter += 1;
                            sync_feedback.set(Some("done".to_string()));
                            syncing.set(false);
                        });
                    },
                    if syncing() { "🔄" } else { "🔃" }
                }

                button {
                    style: "
                    width: 44px;
                    height: 44px;
                    border-radius: 14px;
                    background: rgba(239, 68, 68, 0.15);
                    border: 1px solid rgba(239,68,68,0.3);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 18px;
                    cursor: pointer;
                    transition: background 0.2s;
                ",
                onclick: move |_| {
                    spawn(async move{
                        if logout().await.is_ok() {
                            auth_status.set(AuthStatus::Unauthenticated);
                        }
                    });
                },
                "🚪"
                }


            }

            main {
                style: "
                    flex: 1;
                    min-height: 0;
                    height: 100%;
                    overflow-y: auto;    
                    overflow-x: hidden;
                    padding: 20px;
                    background: transparent;
                ",
                {
                    let key = sync_counter();
                    rsx! {
                        div {
                            key: "{key}",
                            style: "height: 100%; width: 100%;",
                            Outlet::<Route> {}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavButton(to: Route, icon: &'static str) -> Element {
    let current = use_route::<Route>();
    let is_active = current == to;

    rsx! {
        div {
            style: "
                position: relative;
                width: 100%;
                display: flex;
                justify-content: center;
            ",

            if is_active {
                div {
                    style: "
                        position: absolute;
                        left: -12px;
                        top: 6px;
                        width: 4px;
                        height: 32px;
                        border-radius: 2px;
                        background: linear-gradient(
                            to bottom,
                            #6b8bff,
                            #4c6fff
                        );
                        box-shadow:
                            0 0 12px rgba(107,139,255,0.6);
                    "
                }
            }

            Link {
                to: to,
                style: "
                    width: 44px;
                    height: 44px;
                    border-radius: 14px;
                    background: rgba(255,255,255,0.04);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 18px;
                    color: white;
                    text-decoration: none;
                    box-shadow:
                        inset 0 1px rgba(255,255,255,0.05),
                        inset 0 -1px rgba(0,0,0,0.35);
                ",
                "{icon}"
            }
        }
    }
}