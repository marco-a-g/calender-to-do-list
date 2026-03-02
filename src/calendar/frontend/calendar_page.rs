/*
Side Note Important! :  be aware that major parts of the css styling was made with LLM's (GroundLayer with ChatGpt & some details with Claude)
                        refactoring parts were consulted with LLM (Claude)
                        anything else is highlighted in the spot where it was used
*/

//! Calendar page — top-level view for the calendar feature.
//!
//! Composes `CalendarGrid` and `EventForm` into a single page layout:
//! - Left:  Calendar grid (month/week/day view with event chips)
//! - Right: Slide-in event form (create or edit, appears on day/event click)
//!
//! Data flow:
//! 1. `use_resource` fetches calendars, events, groups, and profiles from local SQLite
//! 2. Recurring events are expanded into individual instances via `expand_recurring_events`
//! 3. Events are filtered by `active_calendar_ids` (empty = show all)
//! 4. Calendar colors are resolved from group colors via `build_calendar_color_map`
//! 5. `CalendarLight` structs are converted to full `Calendar` structs for the grid
//!
//! Helper functions:
//! - `resolve_calendar_name`:    Resolves display name (group name or username)
//! - `light_to_calendar`:        Converts `CalendarLight` -> `Calendar` with parsed UUIDs/dates
//! - `build_calendar_color_map`: Maps calendar_id -> hex color from the owning group

use chrono::Utc;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::join;
use uuid::Uuid;

use crate::calendar::backend::handle_recurrence_cal_events::expand_recurring_events;
use crate::calendar::frontend::calendar_grid::{CalendarGrid, ViewMode};
use crate::calendar::frontend::event_form::{EventForm, EventFormMode};
use crate::database::local::init_fetch::init_fetch_local_db::{
    fetch_calendar_events_lokal_db, fetch_calendars_lokal_db, fetch_groups_lokal_db,
    fetch_profiles_lokal_db,
};
use crate::utils::functions::parse_calendar_event_light_to_calendar_event;
use crate::utils::structs::{
    Calendar, CalendarEvent, CalendarEventLight, CalendarLight, GroupLight, OwnedBy, OwnerType,
    ProfileLight,
};

/// Main calendar page component.
///
/// State management:
/// - displayed_date:       Which month/week/day is currently shown
/// - view_mode:            Month, Week, or Day toggle
/// - selected_event:       Currently selected event for editing (None = no selection)
/// - show_form:            Whether the event form panel is visible
/// - prefilled_date:       Date to pre-fill when creating a new event from a day click
/// - active_calendar_ids:  Filter — which calendars are visible (empty = all)
#[component]
pub fn CalendarPage() -> Element {
    let displayed_date = use_signal(Utc::now);
    let view_mode = use_signal(|| ViewMode::Month);

    let mut selected_event: Signal<Option<CalendarEvent>> = use_signal(|| None);
    let mut show_form = use_signal(|| false);
    let mut prefilled_date = use_signal(|| None);

    let active_calendar_ids: Signal<Vec<String>> = use_signal(Vec::new);

    // Fetch all required data from local DB in parallel using tokio::join!
    let mut db_resource = use_resource(move || async move {
        join!(
            fetch_calendars_lokal_db(),
            fetch_calendar_events_lokal_db(),
            fetch_groups_lokal_db(),
            fetch_profiles_lokal_db(),
        )
    });

    // Destructure the parallel fetch results with graceful fallbacks.
    // Recurring events are expanded into individual instances here.
    // If groups or profiles fail, we continue with empty vecs (non-critical data).
    let (calendars_light, all_events_light, groups, profiles) = match &*db_resource.read() {
        // All four fetches succeeded
        Some((Ok(cals), Ok(evts), Ok(grps), Ok(profs))) => {
            let expanded = expand_recurring_events(evts.clone(), Some(Utc::now()))
                .unwrap_or_else(|_| (evts.clone(), evts.clone()))
                .0;
            (cals.clone(), expanded, grps.clone(), profs.clone())
        }
        // Groups or profiles failed still usable without them
        Some((Ok(cals), Ok(evts), _, _)) => {
            let expanded = expand_recurring_events(evts.clone(), Some(Utc::now()))
                .unwrap_or_else(|_| (evts.clone(), evts.clone()))
                .0;
            (cals.clone(), expanded, vec![], vec![])
        }
        // Events failed show empty calendar
        Some((Ok(cals), _, _, _)) => (cals.clone(), vec![], vec![], vec![]),
        // Everything failed or still loading
        _ => (vec![], vec![], vec![], vec![]),
    };

    // Build color lookup: calendar_id -> hex color (from owning group)
    // Wrapped in Arc for cheap cloning into child components
    let calendar_color_by_id = Arc::new(build_calendar_color_map(&calendars_light, &groups));

    // Convert Light structs to full Calendar structs (with parsed UUIDs and dates)
    let calendars_full: Vec<Calendar> = calendars_light
        .iter()
        .map(|c| light_to_calendar(c, &groups, &profiles))
        .collect();

    // Apply calendar filter: empty active_calendar_ids means show all
    let visible_events: Vec<CalendarEventLight> = {
        let ids = active_calendar_ids();
        if ids.is_empty() {
            all_events_light.clone()
        } else {
            all_events_light
                .iter()
                .filter(|e| ids.contains(&e.calendar_id))
                .cloned()
                .collect()
        }
    };

    // Callback to refresh data after event creation/edit/deletion
    let mut handle_refresh = move |_| {
        db_resource.restart();
    };

    // Derive form mode from state: Edit if event selected, Create if form open without event
    let form_mode = use_memo(move || match selected_event() {
        Some(event) => Some(EventFormMode::View(Box::new(event))),
        None if show_form() => Some(EventFormMode::Create),
        _ => None,
    });

    // Show loading spinner while initial data fetch is in progress
    if db_resource.read().is_none() {
        return rsx! {
            div {
                style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; gap: 12px; background: #080910; color: white;",
                div { class: "w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" }
                span { style: "font-size: 13px; color: rgba(255,255,255,0.4);", "Loading..." }
            }
        };
    }

    rsx! {
        div {
            style: "width: 100%; height: 90vh; display: flex; overflow: hidden; color: white; background: #080910; padding: 20px; box-sizing: border-box; gap: 16px;",

            // Left panel: calendar grid
            div {
                style: "
                    flex: 1;
                    display: flex;
                    flex-direction: column;
                    background: linear-gradient(145deg, #1a1d27 0%, #12141d 100%);
                    border-radius: 16px;
                    border: 1px solid rgba(255,255,255,0.07);
                    box-shadow: 0 8px 32px rgba(0,0,0,0.6), inset 0 1px 0 rgba(255,255,255,0.04);
                    overflow: hidden;
                    min-height: 0;
                ",
                CalendarGrid {
                    events: visible_events,
                    calendars: calendars_full.clone(),
                    calendar_color_by_id: calendar_color_by_id.clone(),
                    displayed_date,
                    view_mode,
                    active_calendar_ids,
                    // Day click -> open create form with pre-filled date
                    on_day_click: move |date| {
                        prefilled_date.set(Some(date));
                        selected_event.set(None);
                        show_form.set(true);
                    },
                    // Event click -> parse Light to full CalendarEvent, open edit form
                    on_event_click: move |light: CalendarEventLight| {
                        selected_event.set(parse_calendar_event_light_to_calendar_event(light).ok());
                        show_form.set(true);
                    },
                }
            }

            // Right panel: event form (slide-in, only visible when form_mode is Some)
            if let Some(mode) = form_mode() {
                EventForm {
                    mode,
                    calendars: calendars_full,
                    events: all_events_light.clone(),
                    prefilled_date: prefilled_date(),
                    on_close: move |_| {
                        show_form.set(false);
                        selected_event.set(None);
                        prefilled_date.set(None);
                    },
                    on_refresh: move |_| {
                        show_form.set(false);
                        selected_event.set(None);
                        handle_refresh(());
                    },
                }
            }
        }
    }
}

/// Resolves a human-readable name for a calendar.
///
/// - Group calendars: uses the group name
/// - Private calendars: uses the owner's username
/// - Fallback: "Calendar (first 8 chars of ID)"
fn resolve_calendar_name(
    c: &CalendarLight,
    groups: &[GroupLight],
    profiles: &[ProfileLight],
) -> String {
    if c.calendar_type == "group" {
        if let Some(gid) = &c.group_id {
            if let Some(g) = groups.iter().find(|g| &g.id == gid) {
                return g.name.clone();
            }
        }
    } else if let Some(oid) = &c.owner_id {
        if let Some(p) = profiles.iter().find(|p| &p.id == oid) {
            return p.username.clone();
        }
    }
    format!("Calendar ({})", &c.id[..8])
}

/// Converts a CalendarLight (flat strings from SQLite) to a full Calendar struct.
///
/// Parses UUIDs and timestamps, resolves the display name, and determines
/// the owner type (Group vs Private) from the calendar_type field.
/// Uses Uuid::nil() and Utc::now() as safe fallbacks for unparseable values.
fn light_to_calendar(
    c: &CalendarLight,
    groups: &[GroupLight],
    profiles: &[ProfileLight],
) -> Calendar {
    // Try owner_id first, fall back to group_id, then Uuid::nil()
    let owner_id = c
        .owner_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .or_else(|| c.group_id.as_deref().and_then(|s| Uuid::parse_str(s).ok()))
        .unwrap_or(Uuid::nil());

    let owner_type = if c.calendar_type == "group" {
        OwnerType::Group
    } else {
        OwnerType::Private
    };

    Calendar {
        id: Uuid::parse_str(&c.id).unwrap_or(Uuid::nil()),
        name: resolve_calendar_name(c, groups, profiles),
        description: c.description.clone(),
        owned_by: OwnedBy {
            owner_type,
            owner_id,
        },
        created_at: c.created_at.parse().unwrap_or_else(|_| Utc::now()),
        created_by: Uuid::parse_str(&c.created_by).unwrap_or(Uuid::nil()),
        last_mod: c.last_mod.parse().unwrap_or_else(|_| Utc::now()),
    }
}

/// Builds a map from calendar_id -> hex color string.
///
/// For group calendars, the color comes from the group's color field.
fn build_calendar_color_map(
    calendars: &[CalendarLight],
    groups: &[GroupLight],
) -> HashMap<String, String> {
    calendars
        .iter()
        .map(|cal| {
            let fallback = "#7a808a".to_string();

            // Look up the owning group's color; fallback if no group or no match
            let color = cal
                .group_id
                .as_ref()
                .and_then(|gid| groups.iter().find(|g| g.id == *gid))
                .map(|g| g.color.clone())
                .unwrap_or(fallback);

            (cal.id.clone(), color)
        })
        .collect()
}
