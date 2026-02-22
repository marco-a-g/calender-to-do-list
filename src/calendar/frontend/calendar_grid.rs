use chrono::{DateTime, Datelike, NaiveDate, Utc};
use dioxus::prelude::*;

use crate::utils::structs::{CalendarEventLight, CalendarLight};

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Month,
    Week,
    Day,
}

#[component]
pub fn CalendarGrid(
    /// Pre-filtered events (only from active calendars)
    events: Vec<CalendarEventLight>,
    /// Calendar metadata used for color lookup per event
    calendars: Vec<CalendarLight>,
    displayed_date: Signal<DateTime<Utc>>,
    view_mode: Signal<ViewMode>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col flex-1 h-full overflow-hidden",
            GridToolbar { displayed_date, view_mode }
            div {
                class: "flex-1 overflow-auto",
                if view_mode() == ViewMode::Month {
                    MonthGrid {
                        events: events.clone(),
                        calendars: calendars.clone(),
                        displayed_date: displayed_date(),
                        on_day_click: on_day_click.clone(),
                        on_event_click: on_event_click.clone(),
                    }
                } else if view_mode() == ViewMode::Week {
                    WeekGrid {
                        events: events.clone(),
                        calendars: calendars.clone(),
                        displayed_date: displayed_date(),
                        on_day_click: on_day_click.clone(),
                        on_event_click: on_event_click.clone(),
                    }
                } else {
                    DayGrid {
                        events,
                        calendars,
                        displayed_date: displayed_date(),
                        on_day_click,
                        on_event_click,
                    }
                }
            }
        }
    }
}

#[component]
fn GridToolbar(displayed_date: Signal<DateTime<Utc>>, view_mode: Signal<ViewMode>) -> Element {
    let title = use_memo(move || {
        let d = displayed_date();
        match view_mode() {
            ViewMode::Month => format!("{} {}", month_name(d.month()), d.year()),
            ViewMode::Week => format!("CW {} – {}", d.iso_week().week(), d.year()),
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
                        let d = displayed_date();
                        let (prev_year, prev_month) = if d.month() == 1 {
                            (d.year() - 1, 12)
                        } else {
                            (d.year(), d.month() - 1)
                        };
                        displayed_date.set(
                            d.with_day(1).unwrap()
                                .with_year(prev_year).unwrap()
                                .with_month(prev_month).unwrap()
                        );
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
                        let d = displayed_date();
                        let (next_year, next_month) = if d.month() == 12 {
                            (d.year() + 1, 1)
                        } else {
                            (d.year(), d.month() + 1)
                        };
                        displayed_date.set(
                            d.with_day(1).unwrap()
                                .with_year(next_year).unwrap()
                                .with_month(next_month).unwrap()
                        );
                    },
                    ">"
                }
                button {
                    class: "px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 text-xs text-white/60 transition",
                    onclick: move |_| displayed_date.set(Utc::now()),
                    "Today"
                }
            }

            div {
                class: "flex gap-1 bg-white/5 rounded-xl p-1",
                button {
                    class: if view_mode() == ViewMode::Month {
                        "px-3 py-1 rounded-lg bg-white/15 text-white text-sm font-medium transition"
                    } else {
                        "px-3 py-1 rounded-lg text-white/50 hover:text-white text-sm transition"
                    },
                    onclick: move |_| view_mode.set(ViewMode::Month),
                    "Month"
                }
                button {
                    class: if view_mode() == ViewMode::Week {
                        "px-3 py-1 rounded-lg bg-white/15 text-white text-sm font-medium transition"
                    } else {
                        "px-3 py-1 rounded-lg text-white/50 hover:text-white text-sm transition"
                    },
                    onclick: move |_| view_mode.set(ViewMode::Week),
                    "Week"
                }
                button {
                    class: if view_mode() == ViewMode::Day {
                        "px-3 py-1 rounded-lg bg-white/15 text-white text-sm font-medium transition"
                    } else {
                        "px-3 py-1 rounded-lg text-white/50 hover:text-white text-sm transition"
                    },
                    onclick: move |_| view_mode.set(ViewMode::Day),
                    "Day"
                }
            }
        }
    }
}

#[component]
fn MonthGrid(
    events: Vec<CalendarEventLight>,
    calendars: Vec<CalendarLight>,
    displayed_date: DateTime<Utc>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
) -> Element {
    let first_day = displayed_date.with_day(1).unwrap();
    // Monday = 0, Sunday = 6
    let offset = first_day.weekday().num_days_from_monday() as usize;
    let days = days_in_month(displayed_date.year(), displayed_date.month());
    let today = Utc::now().date_naive();

    rsx! {
        div {
            class: "grid grid-cols-7 gap-px bg-white/5 flex-1",

            // Weekday headers
            div { class: "py-2 text-center text-xs text-white/40 bg-[#070B18]", "Mon" }
            div { class: "py-2 text-center text-xs text-white/40 bg-[#070B18]", "Tue" }
            div { class: "py-2 text-center text-xs text-white/40 bg-[#070B18]", "Wed" }
            div { class: "py-2 text-center text-xs text-white/40 bg-[#070B18]", "Thu" }
            div { class: "py-2 text-center text-xs text-white/40 bg-[#070B18]", "Fri" }
            div { class: "py-2 text-center text-xs text-white/40 bg-[#070B18]", "Sat" }
            div { class: "py-2 text-center text-xs text-white/40 bg-[#070B18]", "Sun" }

            // Empty cells before the 1st of the month
            for _ in 0..offset {
                div { class: "min-h-[100px] bg-[#070B18]" }
            }

            // One cell per day, events filtered per day inside the loop
            for day in 1..=days {
                {
                    let cell_date = displayed_date.with_day(day).unwrap();
                    let cell_naive = cell_date.date_naive();
                    let is_today = cell_naive == today;

                    // Filter events whose from_date_time falls on this day.
                    // from_date_time is stored as an ISO string in CalendarEventLight.
                    let day_events: Vec<CalendarEventLight> = events
                        .iter()
                        .filter(|e| {
                            e.from_date_time
                                .parse::<DateTime<Utc>>()
                                .map(|dt| dt.date_naive() == cell_naive)
                                .unwrap_or(false)
                        })
                        .filter(|e| !e.skipped)
                        .cloned()
                        .collect();

                    rsx! {
                        DayCell {
                            key: "{day}",
                            date: cell_date,
                            events: day_events,
                            calendars: calendars.clone(),
                            is_today,
                            is_current_month: true,
                            on_day_click: on_day_click.clone(),
                            on_event_click: on_event_click.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DayCell(
    date: DateTime<Utc>,
    /// Events filtered to this specific day
    events: Vec<CalendarEventLight>,
    calendars: Vec<CalendarLight>,
    is_today: bool,
    is_current_month: bool,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
) -> Element {
    let cell_class = if is_today {
        "min-h-[100px] p-1.5 bg-[#070B18] border border-blue-500/40 cursor-pointer hover:bg-white/5 transition"
    } else {
        "min-h-[100px] p-1.5 bg-[#070B18] cursor-pointer hover:bg-white/5 transition"
    };

    let number_class = if is_today {
        "text-xs font-bold text-blue-400"
    } else if is_current_month {
        "text-xs text-white/70"
    } else {
        "text-xs text-white/20"
    };

    rsx! {
        div {
            class: "{cell_class}",
            onclick: move |_| on_day_click.call(date),

            span { class: "{number_class}", "{date.day()}" }

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

/// Compact event pill rendered inside a day cell.
#[component]
fn EventChip(
    event: CalendarEventLight,
    calendars: Vec<CalendarLight>,
    on_click: EventHandler<CalendarEventLight>,
) -> Element {
    // TODO: add a color field to CalendarLight and derive it here per calendar
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
    events: Vec<CalendarEventLight>,
    calendars: Vec<CalendarLight>,
    displayed_date: DateTime<Utc>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
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
    events: Vec<CalendarEventLight>,
    calendars: Vec<CalendarLight>,
    displayed_date: DateTime<Utc>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
) -> Element {
    // TODO: Render 24 hour rows, events as blocks positioned by start/end time
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full text-white/40 text-sm",
            "Day view – not yet implemented"
        }
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}