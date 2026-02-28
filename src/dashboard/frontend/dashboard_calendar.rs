use crate::utils::structs::CalendarEventLight;
use chrono::{DateTime, Datelike, Duration, Local};
use dioxus::prelude::*;

// Aus Datetime nur Uhrzeit holen, Hilfsfunktion bei Dashboard
/// Extracts the time (HH:MM) in UTC from a datetime string.
///
/// Falls back to returning an empty string if provided string is invalid or cannot be parsed.
///
/// ## Arguments
///
/// * `datetime` - A string slice containing the RFC 3339 formatted datetime.
fn extract_time_for_dashboard(datetime: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(datetime) {
        Ok(dt) => {
            let utc_time = dt.with_timezone(&chrono::Utc);
            utc_time.format("%H:%M").to_string()
        }
        Err(e) => {
            eprintln!(
                "Error on extracting time for dashboardevents '{}': {}",
                datetime, e
            );
            "".to_string()
        }
    }
}

/// UI-Element that renders a weekly calendar dashboard widget displaying events for the current week.
///
/// Dynamically calculates the current week and groups the provided calendar events into daily columns. Multi-day events are checked and  visually segregated "all-day" events from time-specific events.
///
/// ## Arguments
///
/// * `evts` - A vector of tuples containing the event and its associated group metadata:
///            `(CalendarEventLight, group_name, group_color)`.
#[component]
pub fn DashboardCalendar(evts: Vec<(CalendarEventLight, String, String)>) -> Element {
    //Wochengrenzen vorbereiten
    let today = Local::now();
    let current_weekday_num = today.weekday().num_days_from_monday();
    let start_of_week = today - Duration::days(current_weekday_num as i64);

    // Daten für Calenderansicht vorbereiten, für jeden Tag (day_name, day_num, is_today, all_day_events:vec, timed_events:vec) rausfiltern
    let days_of_week: Vec<_> = (0..7)
        .map(|i| {
            let date = start_of_week + Duration::days(i);
            let day_name = date.format("%a").to_string();
            let day_num = date.format("%d").to_string();
            let is_today = date.date_naive() == today.date_naive();

            // Events an dem jeweiligen Tag kriegen, checkt auch ob events über mehrere Tage sind
            let mut day_events: Vec<(CalendarEventLight, String, String)> = evts
                .iter()
                .filter(|(evt, _, _)| {
                    if let Ok(start_dt) = DateTime::parse_from_rfc3339(&evt.from_date_time) {
                        let start_date = start_dt.with_timezone(&chrono::Utc).date_naive();
                        let end_date = evt
                            .to_date_time
                            .as_deref()
                            .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
                            .map(|dt| dt.with_timezone(&chrono::Utc).date_naive())
                            .unwrap_or(start_date);
                        let current_date_to_check = date.date_naive();

                        //nehmen event an dem Tag rein, wenn Tag innerhalb des Zeitraums des Events liegt (wegen mehrtägiger evts)
                        current_date_to_check >= start_date && current_date_to_check <= end_date
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            // events des tages sortieren nach Start datetime
            day_events.sort_by(|(a, _, _), (b, _, _)| a.from_date_time.cmp(&b.from_date_time));

            // Events des Tages aufteilen in Ganztägig und Nichtganztägig bzw. Normal
            //Ganztägig
            let all_day_events: Vec<_> = day_events
                .iter()
                .filter(|(e, _, _)| e.is_all_day)
                .cloned()
                .collect();

            // Normale; Hier Uhrzeitspanne als String mitausgeben
            let timed_events: Vec<_> = day_events
                .into_iter()
                .filter(|(e, _, _)| !e.is_all_day)
                .map(|(evt, group_name, group_color)| {
                    let start_time = extract_time_for_dashboard(&evt.from_date_time);
                    let end_time = evt
                        .to_date_time
                        .as_deref()
                        .map(extract_time_for_dashboard)
                        .unwrap_or_default();
                    let time_string = if !end_time.is_empty() {
                        format!("{} - {}", start_time, end_time)
                    } else {
                        start_time
                    };
                    //normale events werden als Tupel mit Uhrzeitspanne, Gruppenname und Farbe ausgegeben für rsx block unten, keine Interaktion im Dashboard das reicht also
                    (evt, time_string, group_name, group_color)
                })
                .collect();

            (day_name, day_num, is_today, all_day_events, timed_events)
        })
        .collect();

    rsx! {
        div {
            style: "padding: 20px; height: 100%; display: flex; flex-direction: column;",

            div { style: "margin-bottom: 10px; color: #9ca3af; font-size: 14px; text-transform: uppercase; letter-spacing: 1px;", "This Week" }

            div {
                style: "display: grid; grid-template-columns: repeat(7, 1fr); height: 100%; gap: 1px; background: rgba(255,255,255,0.05); border-radius: 12px; overflow: hidden;",
                //Kalenderspalten generieren
                for (day_name, day_num, is_today, all_day_events, timed_events) in days_of_week {
                    div {
                        style: format!(
                            "background: {}; display: flex; flex-direction: column; align-items: center; padding: 12px 6px; border-right: 1px solid rgba(255,255,255,0.05); position: relative; overflow: hidden;",
                            if is_today { "rgba(58, 107, 255, 0.10)" } else { "#171923" }
                        ),

                        span { style: "font-size: 12px; color: #9ca3af; margin-bottom: 4px;", "{day_name}" }
                        span {
                            style: format!("font-size: 16px; font-weight: 600; color: {}; margin-bottom: 12px;", if is_today { "#3A6BFF" } else { "white" }),
                            "{day_num}"
                        }

                            // Ganztägige Events (abgetrennter Bereich)
                            div {
                                style: "display: flex; flex-direction: column; gap: 4px; width: 100%; height: 70px; flex-shrink: 0; overflow-y: auto; margin-bottom: 8px; padding-bottom: 8px; border-bottom: 1px solid rgba(255,255,255,0.1);",

                                for (evt, _group_name, group_color) in all_day_events {
                                    div {
                                        style: format!(
                                            "background: color-mix(in srgb, {}, transparent 90%); color: {}; padding: 4px 6px; border-radius: 6px; display: flex; justify-content: center; align-items: center; flex-shrink: 0;",
                                            group_color, group_color
                                        ),
                                        span {
                                            style: "font-weight: 600; font-size: 11px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; text-align: center; max-width: 100%;",
                                            "{evt.summary}"
                                        }
                                    }
                                }
                            }

                        // Nicht ganztägige bzw. normale Kalendereinträge
                        div {
                            style: "display: flex; flex-direction: column; gap: 6px; width: 100%; overflow-y: auto; flex: 1; padding-right: 2px;",
                            //über alle itterieren und rendern
                            for (evt, time_string, group_name, group_color) in timed_events {
                                div {
                                    style: "background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.05); border-radius: 8px; padding: 8px; display: flex; flex-direction: column; gap: 4px; transition: background 0.2s;",
                                    class: "hover:bg-white/5",

                                    span {
                                        style: "color: white; font-size: 12px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                        "{evt.summary}"
                                    }

                                    div {
                                        style: "display: flex; align-items: center; gap: 4px; color: #9ca3af; font-size: 10px;",
                                        span { "🕒" }
                                        "{time_string}"
                                    }

                                    // Gruppenbadge mit Farbe
                                    div {
                                        style: format!(
                                            "font-size: 9px; font-weight: 700; text-transform: uppercase; padding: 2px 6px; border-radius: 4px; color: {}; background: color-mix(in srgb, {}, transparent 85%); align-self: flex-start; margin-top: 2px;",
                                            group_color, group_color
                                        ),
                                        "{group_name}"
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
