use crate::dashboard::backend::fetch_dashboard_data::fetch_todos_dashboard_tuples;
use crate::dashboard::frontend::dashboard_calendar::DashboardCalendar;
use crate::dashboard::frontend::dashboard_chat::DashboardChat;
use crate::dashboard::frontend::dashboard_todos::DashboardTodos;
use crate::utils::structs::CalendarEventLight;
use chrono::Local;
use dioxus::prelude::*;

#[component]
pub fn DashboardView() -> Element {
    //Todos der Woche holen in resource
    let todos_resource = use_resource(move || async move { fetch_todos_dashboard_tuples().await });
    //noch leerer Vec für Calender, Arno/Max dann
    let cal_evts: Vec<CalendarEventLight> = Vec::new();
    // Resource lesen
    let dashboard_todos = match &*todos_resource.read() {
        Some(Ok(data)) => data.clone(),
        _ => vec![], // Bei Fehler vom lesen der resource leeren Vec geben
    };

    //Für Loading Screen
    let is_loading = todos_resource.read().is_none();

    if is_loading {
        return rsx! {
            div {
                style: "display: flex; justify-content: center; align-items: center; height: 100%; width: 100%; background: #05060b; color: white;",
                div {
                    style: "display: flex; flex-direction: column; align-items: center; gap: 10px;",
                    div { style: "width: 30px; height: 30px; border: 3px solid #3A6BFF; border-top-color: transparent; border-radius: 50%; animation: spin 1s linear infinite;" }
                    "Loading Dashboard..."
                }
                style { "@keyframes spin {{ 0% {{ transform: rotate(0deg); }} 100% {{ transform: rotate(360deg); }} }}" }
            }
        };
    }

    rsx! {
        div {
            style: "width: 100%; height: 100%; background: #05060b; padding: 24px; display: flex; flex-direction: column; gap: 24px; font-family: sans-serif; box-sizing: border_box;",

            //Obere Reihe, Time/Date und Kalender
            div {
                style: "display: flex; gap: 24px; height: 35%;",

                DashboardDateTimeWidget {}

                // Kalender
            div {
                    style: "flex: 1; background: #171923; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); overflow: hidden;",
                    DashboardCalendar { evts: cal_evts }
                }
            }

            //Untere Reihe, Todos und Chat Platzhalter
            div {
                style: "display: flex; gap: 24px; flex: 1; overflow: hidden;",
                //Todos
                div {
                    style: "flex: 1; background: #171923; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); display: flex; flex-direction: column; overflow: hidden;",
                    DashboardTodos {
                        todos: dashboard_todos
                    }
                }

                // Chat
                div {
                    style: "flex: 1; background: #171923; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); display: flex; align-items: center; justify-content: center;",
                    DashboardChat {}
                }
            }
        }
    }
}

//Datum und Uhrzeit Widget
#[component]
fn DashboardDateTimeWidget() -> Element {
    let now = Local::now();
    let time_str = now.format("%H:%M").to_string();
    let date_str = now.format("%A, %d. %B").to_string();

    rsx! {
        div {
            style: "width: 250px; background: linear-gradient(145deg, #222531 0%, #171923 100%); border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); display: flex; flex-direction: column; justify-content: center; align-items: center; color: white; box-shadow: 0 10px 25px rgba(0,0,0,0.3);",
            div {
                style: "font-size: 48px; font-weight: 700; background: -webkit-linear-gradient(#fff, #9ca3af); -webkit-background-clip: text; -webkit-text-fill-color: transparent;",
                "{time_str}"
            }
            div {
                style: "font-size: 16px; color: #9ca3af; margin-top: 4px; text-transform: uppercase; letter-spacing: 1px;",
                "{date_str}"
            }
        }
    }
}
