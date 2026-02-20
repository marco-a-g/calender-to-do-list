use crate::utils::structs::CalendarEventLight;
use chrono::{Datelike, Duration, Local};
use dioxus::prelude::*;

//Bis jetzt Platzhalter pur LLM; dann Arno/Max
#[component]
pub fn DashboardCalendar(evts: Vec<CalendarEventLight>) -> Element {
    // Logik um die aktuelle Woche (Mo-So) zu berechnen
    let today = Local::now();
    let current_weekday_idx = today.weekday().num_days_from_monday(); // 0 = Montag, 6 = Sonntag
    // Montag der aktuellen Woche finden
    let start_of_week = today - Duration::days(current_weekday_idx as i64);

    let days_of_week = (0..7).map(|i| {
        let date = start_of_week + Duration::days(i);
        let day_name = date.format("%a").to_string(); // Mon, Tue...
        let day_num = date.format("%d").to_string(); // 01, 02...
        let is_today = date.format("%Y-%m-%d").to_string() == today.format("%Y-%m-%d").to_string();
        (day_name, day_num, is_today)
    });

    rsx! {
        div {
            style: "padding: 20px; height: 100%; display: flex; flex-direction: column;",

            // Header
            div { style: "margin-bottom: 10px; color: #9ca3af; font-size: 14px; text-transform: uppercase; letter-spacing: 1px;", "This Week" }

            // Kalender Grid (7 Spalten)
            div {
                style: "display: grid; grid-template-columns: repeat(7, 1fr); height: 100%; gap: 1px; background: rgba(255,255,255,0.05); border-radius: 12px; overflow: hidden;",

                for (day_name, day_num, is_today) in days_of_week {
                    div {
                        style: format!(
                            "background: {}; display: flex; flex-direction: column; align-items: center; padding: 12px 4px; border-right: 1px solid rgba(255,255,255,0.05); position: relative;",
                            if is_today { "rgba(58, 107, 255, 0.15)" } else { "#171923" }
                        ),

                        // Wochentag Header
                        span { style: "font-size: 12px; color: #9ca3af; margin-bottom: 4px;", "{day_name}" }
                        span {
                            style: format!("font-size: 16px; font-weight: 600; color: {};", if is_today { "#3A6BFF" } else { "white" }),
                            "{day_num}"
                        }

                        // Platzhalter für Events (Punkte)
                        div {
                            style: "margin-top: 10px; display: flex; gap: 4px; flex-wrap: wrap; justify-content: center;",
                            // Hier könnte man echte Events filtern und als Punkte anzeigen
                            // Beispiel statischer Punkt:
                            div { style: "width: 6px; height: 6px; background: #3A6BFF; border-radius: 50%; opacity: 0.5;" }
                        }
                    }
                }
            }
        }
    }
}
