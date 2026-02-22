use chrono::Utc;
use dioxus::prelude::*;
use tokio::join;
use uuid::Uuid;

use crate::calendar::frontend::calendar_grid::{CalendarGrid, ViewMode};
use crate::calendar::frontend::event_form::{EventForm, EventFormMode};
use crate::database::local::init_fetch::init_fetch_local_db::{
    fetch_calendar_events_lokal_db, fetch_calendars_lokal_db, fetch_groups_lokal_db,
    fetch_profiles_lokal_db,
};
use crate::utils::structs::{
    Calendar, CalendarEvent, CalendarEventLight, CalendarLight, GroupLight, OwnedBy, OwnerType,
    ProfileLight,
};

#[component]
pub fn CalendarPage() -> Element {
    let displayed_date = use_signal(|| Utc::now());
    let view_mode = use_signal(|| ViewMode::Month);

    // selected_event stores the full CalendarEvent for EventForm
    let mut selected_event: Signal<Option<CalendarEvent>> = use_signal(|| None);
    let mut show_form = use_signal(|| false);
    let mut prefilled_date = use_signal(|| None);

    let active_calendar_ids: Signal<Vec<String>> = use_signal(|| Vec::new());

    // Fetch all data needed to render the page in one concurrent batch
    // Load calendars, events, groups and profiles from local DB on mount.
    // Groups and profiles are needed to resolve calendar display names.
    let mut db_resource = use_resource(move || async move {
        join!(
            fetch_calendars_lokal_db(),
            fetch_calendar_events_lokal_db(),
            fetch_groups_lokal_db(),
            fetch_profiles_lokal_db(),
        )
    });

    // Gracefully degrade if some queries fail (empty fallback instead of crash)
    let (calendars_light, all_events_light, groups, profiles) = match &*db_resource.read() {
        Some((Ok(cals), Ok(evts), Ok(grps), Ok(profs))) => {
            (cals.clone(), evts.clone(), grps.clone(), profs.clone())
        }
        Some((Ok(cals), Ok(evts), _, _)) => (cals.clone(), evts.clone(), vec![], vec![]),
        Some((Ok(cals), _, _, _)) => (cals.clone(), vec![], vec![], vec![]),
        _ => (vec![], vec![], vec![], vec![]),
    };

    // Convert CalendarLight to Calendar with resolved display name from groups/profiles.
    let calendars_full: Vec<Calendar> = calendars_light
        .iter()
        .map(|c| light_to_calendar(c, &groups, &profiles))
        .collect();

    // If no calendar is explicitly selected, show all events
    // Filter events by active calendar IDs before rendering
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

    // Trigger full reload after create/update/delete in EventForm
    // Reload DB data after a save or delete
    let mut handle_refresh = move |_| {
        db_resource.restart();
    };

    // Derive form mode from current selection + visibility state
    // translate signal state into a typed form mode for EventForm
    let form_mode = use_memo(move || match selected_event() {
        Some(event) => Some(EventFormMode::Edit(event)),
        None if show_form() => Some(EventFormMode::Create),
        _ => None,
    });

    if db_resource.read().is_none() {
        return rsx! {
            div {
                class: "w-full h-full flex items-center justify-center gap-4",
                style: "background: linear-gradient(to bottom, #070B18, #050914, black); color: white;",
                div { class: "w-7 h-7 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" }
                span { class: "text-sm text-white/50", "Loading calendar..." }
            }
        };
    }

    rsx! {
        div {
            style: "width: 100%; height: 80vh; display: flex; overflow: hidden; color: white; background: #080910; padding: 20px; box-sizing: border-box; gap: 16px;",

            div {
                class: "flex flex-col flex-1 overflow-hidden",
                CalendarGrid {
                    events: visible_events,
                    calendars: calendars_light,
                    displayed_date,
                    view_mode,
                    on_day_click: move |date| {
                        // Open form in create mode for this date
                        prefilled_date.set(Some(date));
                        selected_event.set(None);
                        show_form.set(true);
                    },
                    // Convert Light → full CalendarEvent before opening EventForm
                    on_event_click: move |light: CalendarEventLight| {
                        // Open form in edit mode for the clicked event
                        selected_event.set(Some(light_to_calendar_event(&light)));
                        show_form.set(true);
                    },
                }
            }

            if let Some(mode) = form_mode() {
                EventForm {
                    mode,
                    calendars: calendars_full,
                    prefilled_date: prefilled_date(),
                    on_close: move |_| {
                        show_form.set(false);
                        selected_event.set(None);
                        prefilled_date.set(None);
                    },
                    on_saved: move |_| {
                        show_form.set(false);
                        selected_event.set(None);
                        handle_refresh(());
                    },
                    on_deleted: move |_| {
                        show_form.set(false);
                        selected_event.set(None);
                        handle_refresh(());
                    },
                }
            }
        }
    }
}

// Resolve display label for a calendar based on ownership
/// Resolves the display name for a calendar:
/// - group calendars   → group name from GroupLight
/// - private calendars → username from ProfileLight
/// Falls back to the calendar id if neither is found.
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
    } else {
        if let Some(oid) = &c.owner_id {
            if let Some(p) = profiles.iter().find(|p| &p.id == oid) {
                return p.username.clone();
            }
        }
    }
    // Last resort: fallback to shortened id
    // Fallback: use a shortened ID so the dropdown is at least usable
    format!("Calendar ({})", &c.id[..8])
}

// Map lightweight DB representation to full domain model
fn light_to_calendar(
    c: &CalendarLight,
    groups: &[GroupLight],
    profiles: &[ProfileLight],
) -> Calendar {
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

// Convert DB event into domain event used by EventForm
fn light_to_calendar_event(e: &CalendarEventLight) -> CalendarEvent {
    CalendarEvent {
        id: Uuid::parse_str(&e.id).unwrap_or(Uuid::nil()),
        summary: e.summary.clone(),
        description: e.description.clone(),
        calendar_id: Uuid::parse_str(&e.calendar_id).unwrap_or(Uuid::nil()),
        created_at: e.created_at.parse().unwrap_or_else(|_| Utc::now()),
        created_by: Uuid::parse_str(&e.created_by).unwrap_or(Uuid::nil()),
        from_date_time: e.from_date_time.parse().unwrap_or_else(|_| Utc::now()),
        to_date_time: e.to_date_time.as_deref().and_then(|s| s.parse().ok()),
        attachment: e.attachment.clone(),
        recurrence: None,
        recurrence_exception: None,
        location: e.location.clone(),
        categories: e.category.as_ref().map(|s| vec![s.clone()]),
        is_all_day: e.is_all_day,
        last_mod: e.last_mod.parse().unwrap_or_else(|_| Utc::now()),
    }
}
