use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    // Ermittelt die aktuelle Route für das Highlighting
    let current_route = use_route::<Route>();

    rsx! {
        div {
            style: "display: flex; width: 100vw; height: 100vh; overflow: hidden; background-color: #05060b; color: white; font-family: sans-serif;",

            nav {
                style: "width: 112px; height: 100%; background: linear-gradient(180deg, #11121b 0%, #05060b 100%); border-right: 1px solid rgba(255,255,255,0.06); display: flex; flex-direction: column; align-items: center; padding: 18px 12px 22px 12px; gap: 20px; flex-shrink: 0;",

                // Logo
                div {
                    style: "width: 34px; height: 34px; border-radius: 14px; background: radial-gradient(circle at 25% 0%, #6b8bff 0%, #24345e 40%, #151725 100%); flex-shrink: 0;",
                }

                // Navigation Container
                div {
                    style: "width: 100%; flex: 1; border-radius: 26px; background: radial-gradient(circle at top, #181b24 0%, #080910 70%); padding: 20px 0px; display: flex; flex-direction: column; align-items: center; gap: 16px;",

                    // Link: Dashboard
                    NavbarLink {
                        to: Route::DashboardView,
                        label: "🪟",
                        is_active: current_route == Route::DashboardView
                    }

                    // Link: ToDos
                    NavbarLink {
                        to: Route::ToDoView,
                        label: "📝",
                        is_active: current_route == Route::ToDoView
                    }
                }
            }

            // Main Content Area
            main {
                id: "content",
                style: "flex: 1; height: 100%; overflow-y: auto; position: relative;",

                Outlet::<Route> {}
            }
        }
    }
}

// Hilfskomponente für sauberen Code und "Active State" (Blauer Hintergrund)
#[component]
fn NavbarLink(to: Route, label: &'static str, is_active: bool) -> Element {
    let bg_color = if is_active {
        "rgba(58, 107, 255, 0.2)"
    } else {
        "rgba(255,255,255,0.03)"
    };
    let border = if is_active {
        "1px solid #3A6BFF"
    } else {
        "none"
    };

    rsx! {
        Link {
            to: to,
            class: "hover:bg-gray-700 transition-colors",
            style: "
                text-decoration: none;
                color: rgba(255,255,255,0.7);
                font-size: 1.2em;
                padding: 10px;
                border-radius: 12px;
                width: 60px;
                height: 60px;
                display: flex;
                align-items: center;
                justify-content: center;
                background-color: {bg_color};
                border: {border};
            ",
            "{label}"
        }
    }
}
