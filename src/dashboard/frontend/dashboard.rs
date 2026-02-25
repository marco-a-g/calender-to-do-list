use crate::dashboard::backend::fetch_dashboard_calendar_data::fetch_calendar_dashboard_tuples;
use crate::dashboard::backend::fetch_dashboard_todo_data::fetch_todos_dashboard_tuples;
use crate::dashboard::frontend::dashboard_calendar::DashboardCalendar;
use crate::dashboard::frontend::dashboard_chat::DashboardChat;
use crate::dashboard::frontend::dashboard_news::DashboardNewsWidget;
use crate::dashboard::frontend::dashboard_todos::DashboardTodos;
use chrono::Local;
use dioxus::prelude::*;
use tokio::time::{Duration, sleep};

/// UI-Element that renders the primary dashboard view
///
/// Root container for the dashboard. Fetches the user's to-do items and calendar events and passes them on to the corresponding Subcomponents.
/// Contains a spinner-animatino until the data is ready.
//  Subcomponents are: DashboardDateTimeWidget, DashboardNewsWidget, DashboardCalendar, DashboardChat
#[component]
pub fn DashboardView() -> Element {
    //Daten holen
    let todos_resource = use_resource(move || async move { fetch_todos_dashboard_tuples().await });
    let cal_resource = use_resource(move || async move { fetch_calendar_dashboard_tuples().await });

    //TodoDaten lesen, falls leer oder fehler leeren Vec geben
    let dashboard_todos = match &*todos_resource.read() {
        Some(Ok(data)) => data.clone(),
        _ => vec![],
    };
    //Calenderevents lesen, falls leer oder fehler leeren Vec geben
    let dashboard_evts = match &*cal_resource.read() {
        Some(Ok(data)) => data.clone(),
        _ => vec![],
    };

    let is_loading = todos_resource.read().is_none();

    //loading Anzeige
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
            style: "width: 100%; height: 100%; background: #05060b; padding: 24px; display: flex; flex-direction: column; gap: 24px; font-family: sans-serif; box-sizing: border-box;", // Tippfehler-Fix: border-box statt border_box
            // Obere Reihe: Datumsanzeige, Newsfeed, Kalender
            div {
                style: "display: flex; gap: 24px; height: 50%;",
                // Datumswidget und Newsfeed
                div {
                    style: "display: flex; flex-direction: column; gap: 24px; width: 300px; flex-shrink: 0;",
                    // datum und Uhrzeit
                    DashboardDateTimeWidget {}
                    // dev.to Rust News Feed
                    DashboardNewsWidget {}
                }
                // Kalendergrid
                div {
                    style: "flex: 1; background: #171923; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); overflow: hidden;",
                    DashboardCalendar { evts: dashboard_evts }
                }
            }
            // Untere Reihe, Todos und Chat
            div {
                style: "display: flex; gap: 24px; flex: 1; overflow: hidden;",
                // Todos einbinden
                div {
                    style: "flex: 1; background: #171923; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); display: flex; flex-direction: column; overflow: hidden;",
                    DashboardTodos {
                        todos: dashboard_todos
                    }
                }
                // Chat eininden
                div {
                    style: "flex: 1; background: #171923; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); display: flex; align-items: center; justify-content: center;",
                    DashboardChat {}
                }
            }
        }
    }
}

//Widget für Datum und Uhrzeit
#[component]
pub fn DashboardDateTimeWidget() -> Element {
    let mut current_time = use_signal(Local::now);
    //Uhrzeit aktualisieren im Hintergrund
    use_future(move || async move {
        loop {
            sleep(Duration::from_secs(10)).await;
            current_time.set(Local::now());
        }
    });

    let time_str = current_time().format("%H:%M").to_string();
    let date_str = current_time().format("%a, %d. %b").to_string();

    rsx! {
        div {
            style: "background: #171923; border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; padding: 20px; height: 140px; flex-shrink: 0; color: white; display: flex; flex-direction: column; align-items: center; justify-content: center;",
            span {
                style: "font-size: 3rem; font-weight: bold;",
                "{time_str}"
            }
            span {
                style: "color: #9ca3af; font-size: 1rem; margin-top: 8px;",
                "{date_str}"
            }
        }
    }
}
