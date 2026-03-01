/*
Side Note Important! :  be aware that major parts of the css styling was made with LLM's (GroundLayer with ChatGpt & some details with Claude)
                        refactoring parts were consulted with LLM (Claude)
                        anything else is highlighted in the spot where it was used
*/

//! Calendar grid frontend module.
//!
//! Renders the main calendar view with three modes (Month, Week, Day).
//! The grid is composed of several nested components:
//!
//! - `CalendarGrid`:   Top-level wrapper; switches between Month/Week/Day based on signal.
//! - `GridToolbar`:    Navigation bar with prev/next buttons, "Today" shortcut,
//!                     view-mode toggle, and a calendar filter dropdown.
//! - `MonthGrid`:      7-column CSS grid; one cell per day, events rendered as chips.
//! - `DayCell`:        Single day in the month grid; shows day number and event chips.
//! - `EventChip`:      Compact event display with calendar color coding.
//! - `WeekGrid`/`DayGrid`: Placeholder stubs (not yet implemented).
//!
//! Helper functions:
//! - `days_in_month`:  Calculates the number of days in a given month/year.
//! - `month_name`:     Maps month number (1–12) to English name.

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::utils::structs::{Calendar, CalendarEventLight};

/// View mode enum — determines which grid layout is rendered.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Month,
    Week,
    Day,
}

/// Top-level calendar grid component.
///
/// Delegates to MonthGrid, WeekGrid, or DayGrid based on the current view_mode signal.
/// The calendar_color_by_id map is wrapped in Arc for cheap cloning across child components.
#[component]
pub fn CalendarGrid(
    events: Vec<CalendarEventLight>,
    calendars: Vec<Calendar>,
    displayed_date: Signal<DateTime<Utc>>,
    view_mode: Signal<ViewMode>,
    active_calendar_ids: Signal<Vec<String>>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
    calendar_color_by_id: Arc<HashMap<String, String>>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; flex: 1; overflow: hidden; min-height: 0vh;",
            GridToolbar {
                displayed_date,
                view_mode,
                calendars: calendars.clone(),
                active_calendar_ids,
            }
            div {
                // Flex container fills remaining space below toolbar
                style: "flex: 1; overflow: hidden; display: flex; flex-direction: column; min-height: 0;",
                if view_mode() == ViewMode::Month {
                    MonthGrid {
                        events: events.clone(),
                        calendars: calendars.clone(),
                        displayed_date: displayed_date(),
                        on_day_click: on_day_click.clone(),
                        on_event_click: on_event_click.clone(),
                        calendar_color_by_id: calendar_color_by_id.clone(),
                    }
                } else if view_mode() == ViewMode::Week {
                    WeekGrid {
                        events: events.clone(),
                        calendars: calendars.clone(),
                        displayed_date: displayed_date(),
                        on_day_click: on_day_click.clone(),
                        on_event_click: on_event_click.clone(),
                        calendar_color_by_id: calendar_color_by_id.clone(),
                    }
                } else {
                    DayGrid {
                        events,
                        calendars,
                        displayed_date: displayed_date(),
                        on_day_click,
                        on_event_click,
                        calendar_color_by_id: calendar_color_by_id.clone(),
                    }
                }
            }
        }
    }
}

/// Toolbar above the calendar grid.
///
/// Contains:
/// - Previous/Next month navigation (handles year boundary rollover)
/// - "Today" shortcut button
/// - Calendar filter dropdown (multi-select toggle per calendar)
/// - View mode toggle (Month / Week / Day segmented control)
///
/// Month navigation resets to day 1 before changing month to avoid
#[component]
fn GridToolbar(
    displayed_date: Signal<DateTime<Utc>>,
    view_mode: Signal<ViewMode>,
    calendars: Vec<Calendar>,
    active_calendar_ids: Signal<Vec<String>>,
) -> Element {
    let mut show_filter = use_signal(|| false);

    // Reactive title: updates whenever displayed_date or view_mode changes
    let title = use_memo(move || {
        let d = displayed_date();
        match view_mode() {
            ViewMode::Month => format!("{} {}", month_name(d.month()), d.year()),
            ViewMode::Week => format!("CW {} – {}", d.iso_week().week(), d.year()),
            ViewMode::Day => format!("{}.{}.{}", d.day(), d.month(), d.year()),
        }
    });

    // Label for the filter button: "All Calendars" or "N selected"
    let filter_label = use_memo(move || {
        let ids = active_calendar_ids();
        if ids.is_empty() {
            "All Calendars".to_string()
        } else {
            format!("{} selected", ids.len())
        }
    });

    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; padding: 12px 16px; border-bottom: 1px solid rgba(255,255,255,0.06); flex-wrap: wrap; gap: 8px;",

            // Left side: navigation + filter
            div {
                style: "display: flex; align-items: center; gap: 6px; flex-wrap: wrap;",

                // Previous month rolls back year at January
                button {
                    style: "padding: 4px 10px; border-radius: 8px; background: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.8); cursor: pointer; font-size: 13px;",
                    onclick: move |_| {
                        let d = displayed_date();
                        let (prev_year, prev_month) = if d.month() == 1 {
                            (d.year() - 1, 12)
                        } else {
                            (d.year(), d.month() - 1)
                        };
                        // Reset to day 1 first to avoid invalid dates
                        displayed_date.set(
                            d.with_day(1).unwrap()
                                .with_year(prev_year).unwrap()
                                .with_month(prev_month).unwrap()
                        );
                    },
                    "<"
                }
                span {
                    style: "color: white; font-weight: 700; font-size: 15px; min-width: 140px; text-align: center;",
                    "{title}"
                }
                // Next month rolls forward year at December
                button {
                    style: "padding: 4px 10px; border-radius: 8px; background: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.8); cursor: pointer; font-size: 13px;",
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
                // "Today" shortcut jumps back to current date
                button {
                    style: "padding: 4px 10px; border-radius: 8px; background: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.45); cursor: pointer; font-size: 11px;",
                    onclick: move |_| displayed_date.set(Utc::now()),
                    "Today"
                }

                // Calendar filter dropdown
                div {
                    style: "position: relative;",
                    // Toggle button visually highlights when filters are active (blue tint)
                    button {
                        style: if active_calendar_ids().is_empty() {
                            "display: flex; align-items: center; gap: 5px; padding: 4px 10px; border-radius: 8px; background: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.45); cursor: pointer; font-size: 11px;"
                        } else {
                            "display: flex; align-items: center; gap: 5px; padding: 4px 10px; border-radius: 8px; background: rgba(59,130,246,0.15); border: 1px solid rgba(59,130,246,0.35); color: #60a5fa; cursor: pointer; font-size: 11px;"
                        },
                        onclick: move |_| show_filter.set(!show_filter()),
                        span { "▾" }
                        span { "{filter_label}" }
                    }

                    if show_filter() {
                        // Invisible fullscreen backdrop closes dropdown on click-outside
                        div {
                            style: "position: fixed; inset: 0; z-index: 10;",
                            onclick: move |_| show_filter.set(false),
                        }
                        // Dropdown panel
                        div {
                            style: "position: absolute; top: 100%; left: 0; margin-top: 4px; z-index: 20; min-width: 190px; background: linear-gradient(145deg, #1f222c 0%, #14161f 100%); border: 1px solid rgba(255,255,255,0.08); border-radius: 12px; box-shadow: 0 12px 32px rgba(0,0,0,0.7); overflow: hidden;",

                            // "All Calendars" option clears filter (empty vec = show all)
                            div {
                                style: "display: flex; align-items: center; gap: 8px; padding: 9px 12px; cursor: pointer;",
                                onclick: move |_| {
                                    active_calendar_ids.set(vec![]);
                                    show_filter.set(false);
                                },
                                div {
                                    style: if active_calendar_ids().is_empty() {
                                        "width: 13px; height: 13px; border-radius: 3px; border: 1px solid #3b82f6; background: #3b82f6; display: flex; align-items: center; justify-content: center;"
                                    } else {
                                        "width: 13px; height: 13px; border-radius: 3px; border: 1px solid rgba(255,255,255,0.2);"
                                    },
                                    if active_calendar_ids().is_empty() {
                                        span { style: "color: white; font-size: 8px;", "✓" }
                                    }
                                }
                                span { style: "font-size: 12px; color: rgba(255,255,255,0.7);", "All Calendars" }
                            }

                            div { style: "height: 1px; background: rgba(255,255,255,0.06);" }

                            // Per-calendar toggle rows
                            for cal in &calendars {
                                {
                                    let cal_id = cal.id.to_string();
                                    let cal_name = cal.name.clone();
                                    let is_active = active_calendar_ids().contains(&cal_id);

                                    rsx! {
                                        div {
                                            key: "{cal_id}",
                                            style: "display: flex; align-items: center; gap: 8px; padding: 9px 12px; cursor: pointer;",
                                            // Toggle: add to or remove from active_calendar_ids
                                            onclick: move |_| {
                                                let mut ids = active_calendar_ids();
                                                if ids.contains(&cal_id) {
                                                    ids.retain(|id| id != &cal_id);
                                                } else {
                                                    ids.push(cal_id.clone());
                                                }
                                                active_calendar_ids.set(ids);
                                            },
                                            div {
                                                style: if is_active {
                                                    "width: 13px; height: 13px; border-radius: 3px; border: 1px solid #3b82f6; background: #3b82f6; display: flex; align-items: center; justify-content: center;"
                                                } else {
                                                    "width: 13px; height: 13px; border-radius: 3px; border: 1px solid rgba(255,255,255,0.2);"
                                                },
                                                if is_active {
                                                    span { style: "color: white; font-size: 8px;", "✓" }
                                                }
                                            }
                                            span { style: "font-size: 12px; color: rgba(255,255,255,0.8);", "{cal_name}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Right side: view mode segmented control (Month / Week / Day)
            div {
                style: "display: flex; gap: 2px; background: rgba(255,255,255,0.05); border-radius: 10px; padding: 3px; border: 1px solid rgba(255,255,255,0.08);",
                button {
                    style: if view_mode() == ViewMode::Month {
                        "padding: 4px 12px; border-radius: 7px; background: rgba(255,255,255,0.14); color: white; font-size: 12px; font-weight: 600; cursor: pointer; border: none;"
                    } else {
                        "padding: 4px 12px; border-radius: 7px; background: transparent; color: rgba(255,255,255,0.35); font-size: 12px; cursor: pointer; border: none;"
                    },
                    onclick: move |_| view_mode.set(ViewMode::Month),
                    "Month"
                }
                button {
                    style: if view_mode() == ViewMode::Week {
                        "padding: 4px 12px; border-radius: 7px; background: rgba(255,255,255,0.14); color: white; font-size: 12px; font-weight: 600; cursor: pointer; border: none;"
                    } else {
                        "padding: 4px 12px; border-radius: 7px; background: transparent; color: rgba(255,255,255,0.35); font-size: 12px; cursor: pointer; border: none;"
                    },
                    onclick: move |_| view_mode.set(ViewMode::Week),
                    "Week"
                }
                button {
                    style: if view_mode() == ViewMode::Day {
                        "padding: 4px 12px; border-radius: 7px; background: rgba(255,255,255,0.14); color: white; font-size: 12px; font-weight: 600; cursor: pointer; border: none;"
                    } else {
                        "padding: 4px 12px; border-radius: 7px; background: transparent; color: rgba(255,255,255,0.35); font-size: 12px; cursor: pointer; border: none;"
                    },
                    onclick: move |_| view_mode.set(ViewMode::Day),
                    "Day"
                }
            }
        }
    }
}

/// Month grid: 7-column CSS grid with weekday headers and one DayCell per day.
///
/// Layout logic:
/// 1. Find which weekday the 1st falls on (offset from Monday)
/// 2. Render empty cells for the padding days before the 1st
/// 3. Render one DayCell per day of the month
///
/// Per-day event filtering: parses from_date_time as DateTime, compares date_naive,
/// excludes skipped recurrence exceptions. Events are sorted: all-day first, then by time.
// LLM-consulted: event sorting logic (all-day first, then chronological) (Claude)
#[component]
fn MonthGrid(
    events: Vec<CalendarEventLight>,
    calendars: Vec<Calendar>,
    displayed_date: DateTime<Utc>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
    calendar_color_by_id: Arc<HashMap<String, String>>,
) -> Element {
    let first_day = displayed_date.with_day(1).unwrap();
    // Monday=0, Sunday=6 — determines how many empty cells before day 1
    let offset = first_day.weekday().num_days_from_monday() as usize;
    let days = days_in_month(displayed_date.year(), displayed_date.month());
    let today = Utc::now().date_naive();

    rsx! {
        div {
            // Responsive row height: clamp between 80px and 168px, scaling with viewport
            style: "display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); \
             grid-template-rows: auto repeat(6, minmax(clamp(80px, 12vh, 168px), 1fr)); \
            gap: 1px; background: rgba(255,255,255,0.04); flex: 1; min-height: 0; overflow: auto;",

            // Weekday column headers (Mon–Sun)
            div { style: "padding: 6px 8px; font-size: 10px; font-weight: 600; color: rgba(255,255,255,0.25); background: #14161f; letter-spacing: 0.08em; text-transform: uppercase;", "Mon" }
            div { style: "padding: 6px 8px; font-size: 10px; font-weight: 600; color: rgba(255,255,255,0.25); background: #14161f; letter-spacing: 0.08em; text-transform: uppercase;", "Tue" }
            div { style: "padding: 6px 8px; font-size: 10px; font-weight: 600; color: rgba(255,255,255,0.25); background: #14161f; letter-spacing: 0.08em; text-transform: uppercase;", "Wed" }
            div { style: "padding: 6px 8px; font-size: 10px; font-weight: 600; color: rgba(255,255,255,0.25); background: #14161f; letter-spacing: 0.08em; text-transform: uppercase;", "Thu" }
            div { style: "padding: 6px 8px; font-size: 10px; font-weight: 600; color: rgba(255,255,255,0.25); background: #14161f; letter-spacing: 0.08em; text-transform: uppercase;", "Fri" }
            div { style: "padding: 6px 8px; font-size: 10px; font-weight: 600; color: rgba(255,255,255,0.25); background: #14161f; letter-spacing: 0.08em; text-transform: uppercase;", "Sat" }
            div { style: "padding: 6px 8px; font-size: 10px; font-weight: 600; color: rgba(255,255,255,0.25); background: #14161f; letter-spacing: 0.08em; text-transform: uppercase;", "Sun" }

            // Empty padding cells before the 1st
            for _ in 0..offset {
                div { style: "background: rgba(255,255,255,0.015);" }
            }

            for day in 1..=days {
                {
                    let cell_date = displayed_date.with_day(day).unwrap();
                    let cell_naive = cell_date.date_naive();
                    let is_today = cell_naive == today;

                    // Filter events for this specific day
                    let mut day_events: Vec<CalendarEventLight> = events
                        .iter()
                        .filter(|e| {
                            e.from_date_time
                                .parse::<DateTime<Utc>>()
                                .map(|dt| dt.date_naive() == cell_naive)
                                .unwrap_or(false)
                        })
                        .filter(|e| !e.skipped) // exclude skipped recurrence exceptions
                        .cloned()
                        .collect();

                    // Sort: all-day events first, then by start time, fallback to summary name
                    day_events.sort_by(|a, b| {
                        match (a.is_all_day, b.is_all_day) {
                            (true, false) => return std::cmp::Ordering::Less,
                            (false, true) => return std::cmp::Ordering::Greater,
                            _ => {}
                        }

                        let adt = a.from_date_time.parse::<DateTime<Utc>>().ok();
                        let bdt = b.from_date_time.parse::<DateTime<Utc>>().ok();

                        match (adt, bdt) {
                            (Some(adt), Some(bdt)) => adt.cmp(&bdt),
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => a.summary.cmp(&b.summary),
                        }
                    });

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
                            calendar_color_by_id: calendar_color_by_id.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Single day cell in the month grid.
///
/// Visual behavior:
/// - Today: bluetinted background with blue border
/// - Hover: subtle white highlight
/// - Day number: blue pill badge for today, plain text otherwise
///
/// Clicking the cell background fires on_day_click (opens create-event form).
/// Event chips inside the cell have their own click handler.
#[component]
fn DayCell(
    date: DateTime<Utc>,
    events: Vec<CalendarEventLight>,
    calendars: Vec<Calendar>,
    is_today: bool,
    is_current_month: bool,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
    calendar_color_by_id: Arc<HashMap<String, String>>,
) -> Element {
    let mut hovered = use_signal(|| false);

    let cell_style = if is_today {
        if hovered() {
            "padding: 6px 8px; background: rgba(59,130,246,0.14); border: 1px solid rgba(59,130,246,0.35); cursor: pointer; overflow-y: auto;"
        } else {
            "padding: 6px 8px; background: rgba(59,130,246,0.08); border: 1px solid rgba(59,130,246,0.25); cursor: pointer; overflow-y: auto;"
        }
    } else if hovered() {
        "padding: 6px 8px; background: rgba(255,255,255,0.05); cursor: pointer; overflow-y: auto;"
    } else {
        "padding: 6px 8px; background: rgba(255,255,255,0.015); cursor: pointer; overflow-y: auto;"
    };

    let number_style = if is_today {
        "font-size: 11px; font-weight: 700; color: #60a5fa; background: rgba(59,130,246,0.2); padding: 1px 5px; border-radius: 5px; display: inline-block;"
    } else if is_current_month {
        "font-size: 11px; font-weight: 500; color: rgba(255,255,255,0.5); display: inline-block;"
    } else {
        "font-size: 11px; color: rgba(255,255,255,0.1); display: inline-block;"
    };

    rsx! {
        div {
            style: "{cell_style}",
            onclick: move |_| on_day_click.call(date),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),

            span { style: "{number_style}", "{date.day()}" }

            div {
                style: "display: flex; flex-direction: column; gap: 2px; margin-top: 4px;",
                for event in events {
                    EventChip {
                        event: event.clone(),
                        on_click: on_event_click.clone(),
                        calendar_color_by_id: calendar_color_by_id.clone(),
                    }
                }
            }
        }
    }
}

/// Compact event chip inside a DayCell.
///
/// Two visual modes based on is_all_day:
/// - All-day: solid colored background, bold text, "All Day" label
///
/// Uses stop_propagation on click to prevent the parent DayCell from also firing.
#[component]
fn EventChip(
    event: CalendarEventLight,
    on_click: EventHandler<CalendarEventLight>,
    calendar_color_by_id: Arc<HashMap<String, String>>,
) -> Element {
    let color = calendar_color_by_id
        .get(&event.calendar_id)
        .map(|s| s.as_str())
        .unwrap_or("#9ca3af");

    // Format start time (empty for all-day)
    let time_str = if event.is_all_day {
        String::new()
    } else {
        event
            .from_date_time
            .parse::<DateTime<Utc>>()
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_default()
    };

    // Format end time with " – " prefix
    let to_str = if event.is_all_day {
        String::new()
    } else {
        event
            .to_date_time
            .as_deref()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .map(|dt| format!(" – {}", dt.format("%H:%M")))
            .unwrap_or_default()
    };

    // Display prefix: "10:00 – 11:00 " for timed, empty for all-day
    let prefix = if event.is_all_day {
        String::new()
    } else if time_str.is_empty() {
        String::new()
    } else {
        format!("{time_str}{to_str} ")
    };

    rsx! {
        div {
            style: if event.is_all_day {
                format!(
                    "font-size: 10px; padding: 3px 6px; border-radius: 6px; color: rgba(255,255,255,0.92); \
                     overflow: hidden; text-overflow: ellipsis; white-space: nowrap; cursor: pointer; \
                     background: {color}; font-weight: 650; display: flex; align-items: center; gap: 6px;"
                )
            } else {
                format!(
                    "font-size: 10px; padding: 2px 5px; border-radius: 4px; color: rgba(255,255,255,0.85); \
                     overflow: hidden; text-overflow: ellipsis; white-space: nowrap; cursor: pointer; \
                     background: {color}2a; border-left: 2px solid {color}; font-weight: 500;"
                )
            },
            onclick: move |e| {
                e.stop_propagation();
                on_click.call(event.clone());
            },

            if event.is_all_day {
                span {
                    style: "font-size: 9px; letter-spacing: 0.06em; text-transform: uppercase; opacity: 0.95;",
                    "All Day"
                }
            }

            span {
                style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                "{prefix}{event.summary}"
            }
        }
    }
}

/// Week grid placeholder not yet implemented.
#[component]
fn WeekGrid(
    events: Vec<CalendarEventLight>,
    calendars: Vec<Calendar>,
    displayed_date: DateTime<Utc>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
    calendar_color_by_id: Arc<HashMap<String, String>>,
) -> Element {
    // TODO: Render 7 columns (Mon–Sun) with 24 hour rows, events as positioned blocks
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: rgba(255,255,255,0.3); font-size: 14px;",
            "Week view – not yet implemented"
        }
    }
}

/// Day grid placeholder not yet implemented.
#[component]
fn DayGrid(
    events: Vec<CalendarEventLight>,
    calendars: Vec<Calendar>,
    displayed_date: DateTime<Utc>,
    on_day_click: EventHandler<DateTime<Utc>>,
    on_event_click: EventHandler<CalendarEventLight>,
    calendar_color_by_id: Arc<HashMap<String, String>>,
) -> Element {
    // TODO: Render 24 hour rows, events as blocks positioned by start/end time
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: rgba(255,255,255,0.3); font-size: 14px;",
            "Day view – not yet implemented"
        }
    }
}

/// Calculates the number of days in a given month/year.
/// Approach: first day of next month, then pred_opt() to get the last day of this month.
fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

/// Maps month number (1–12) to English name.
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