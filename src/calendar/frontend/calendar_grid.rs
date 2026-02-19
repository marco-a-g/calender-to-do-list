use chrono::{DateTime, Datelike, Utc};
use dioxus::prelude::*;

use crate::utils::structs::{Calendar, CalendarEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Month,
    Week,
    Day,
}

#[component]
pub fn CalendarGrid(
    /// Pre-filtered events (only from active calendars)
    events: Vec<CalendarEvent>,
    /// Calendar metadata used for color lookup per event
    calendars: Vec<Calendar>,
    displayed_date: Signal<DateTime<Utc>>,
    view_mode: Signal<ViewMode>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEvent>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col flex-1 h-full overflow-hidden",

            GridToolbar { displayed_date, view_mode }

            div {
                class: "flex-1 overflow-auto",
                match view_mode() {
                    ViewMode::Month => rsx! {
                        MonthGrid {
                            events,
                            calendars,
                            displayed_date: displayed_date(),
                            on_day_click,
                            on_event_click,
                        }
                    },
                    ViewMode::Week => rsx! {
                        WeekGrid {
                            events,
                            calendars,
                            displayed_date: displayed_date(),
                            on_day_click,
                            on_event_click,
                        }
                    },
                    ViewMode::Day => rsx! {
                        DayGrid {
                            events,
                            calendars,
                            displayed_date: displayed_date(),
                            on_day_click,
                            on_event_click,
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn GridToolbar(
    displayed_date: Signal<DateTime<Utc>>,
    view_mode: Signal<ViewMode>,
) -> Element {
    let title = use_memo(move || {
        let d = displayed_date();
        match view_mode() {
            ViewMode::Month => format!("{} {}", month_name(d.month()), d.year()),
            ViewMode::Week => format!("KW {} – {}", d.iso_week().week(), d.year()),
            ViewMode::Day => format!("{}.{}.{}", d.day(), d.month(), d.year()),
        }
    });

    rsx! {
        div {
            class: "flex items-center justify-between px-6 py-4 border-b border-white/10",

            div {
                class: "flex items-center gap-3",

                button {
                    class: "px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 text-sm transition",
                    onclick: move |_| {
                        // TODO: Subtract 1 month/week/day from displayed_date based on view_mode
                    },
                    "<"
                }

                span {
                    class: "text-white font-medium text-base min-w-[160px] text-center",
                    "{title}"
                }

                button {
                    class: "px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 text-sm transition",
                    onclick: move |_| {
                        // TODO: Add 1 month/week/day to displayed_date based on view_mode
                    },
                    ">"
                }

                button {
                    class: "px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 text-xs text-white/60 transition",
                    onclick: move |_| displayed_date.set(Utc::now()),
                    "Heute"
                }
            }

            div {
                class: "flex gap-1 bg-white/5 rounded-xl p-1",
                for (label, mode) in [("Monat", ViewMode::Month), ("Woche", ViewMode::Week), ("Tag", ViewMode::Day)] {
                    button {
                        class: if view_mode() == mode {
                            "px-3 py-1 rounded-lg bg-white/15 text-white text-sm font-medium transition"
                        } else {
                            "px-3 py-1 rounded-lg text-white/50 hover:text-white text-sm transition"
                        },
                        onclick: move |_| view_mode.set(mode.clone()),
                        "{label}"
                    }
                }
            }
        }
    }
}

#[component]
fn MonthGrid(
    events: Vec<CalendarEvent>,
    calendars: Vec<Calendar>,
    displayed_date: DateTime<Utc>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEvent>,
) -> Element {
    // TODO: Compute actual days of the month including leading/trailing overflow cells
    // Each cell should receive only the events whose date matches that day
    rsx! {
        div {
            class: "grid grid-cols-7 gap-px bg-white/5 flex-1",

            for day in ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"] {
                div {
                    class: "py-2 text-center text-xs text-white/40 bg-[#070B18]",
                    "{day}"
                }
            }

            for _ in 0..35 {
                DayCell {
                    date: displayed_date,
                    events: vec![],
                    calendars: calendars.clone(),
                    is_today: false,
                    is_current_month: true,
                    on_day_click: on_day_click.clone(),
                    on_event_click: on_event_click.clone(),
                }
            }
        }
    }
}

#[component]
fn DayCell(
    date: DateTime<Utc>,
    /// Events filtered to this specific day
    events: Vec<CalendarEvent>,
    calendars: Vec<Calendar>,
    is_today: bool,
    is_current_month: bool,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEvent>,
) -> Element {
    rsx! {
        div {
            class: if is_today {
                "min-h-[100px] p-1.5 bg-[#070B18] border border-blue-500/40 cursor-pointer hover:bg-white/5 transition"
            } else {
                "min-h-[100px] p-1.5 bg-[#070B18] cursor-pointer hover:bg-white/5 transition"
            },
            onclick: move |_| on_day_click.call(date),

            span {
                class: if is_today {
                    "text-xs font-bold text-blue-400"
                } else if is_current_month {
                    "text-xs text-white/70"
                } else {
                    "text-xs text-white/20"
                },
                "{date.day()}"
            }

            div {
                class: "flex flex-col gap-0.5 mt-1",
                for event in events {
                    EventChip {
                        event: event.clone(),
                        calendars: calendars.clone(),
                        on_click: on_event_click.clone(),
                    }
                }
            }
        }
    }
}

/// Compact event pill shown inside a day cell
#[component]
fn EventChip(
    event: CalendarEvent,
    calendars: Vec<Calendar>,
    on_click: EventHandler<CalendarEvent>,
) -> Element {
    // TODO: Derive color from the matching Calendar in `calendars` by calendar_id
    let color = "#3A6BFF";

    rsx! {
        div {
            class: "text-[10px] px-1.5 py-0.5 rounded text-white truncate cursor-pointer hover:opacity-80 transition",
            style: "background: {color}44; border-left: 2px solid {color};",
            onclick: move |e| {
                e.stop_propagation();
                on_click.call(event.clone());
            },
            "{event.summary}"
        }
    }
}

#[component]
fn WeekGrid(
    events: Vec<CalendarEvent>,
    calendars: Vec<Calendar>,
    displayed_date: DateTime<Utc>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEvent>,
) -> Element {
    // TODO: Render 7 columns (Mon–Sun) with 24 hour rows, events as positioned blocks
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full text-white/40 text-sm",
            "Week view – not yet implemented"
        }
    }
}

#[component]
fn DayGrid(
    events: Vec<CalendarEvent>,
    calendars: Vec<Calendar>,
    displayed_date: DateTime<Utc>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEvent>,
) -> Element {
    // TODO: Render 24 hour rows, events as blocks positioned by start/end time
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full text-white/40 text-sm",
            "Day view – not yet implemented"
        }
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "Januar", 2 => "Februar", 3 => "März", 4 => "April",
        5 => "Mai", 6 => "Juni", 7 => "Juli", 8 => "August",
        9 => "September", 10 => "Oktober", 11 => "November", 12 => "Dezember",
        _ => "",
    }
}