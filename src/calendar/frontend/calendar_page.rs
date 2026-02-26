use chrono::Utc;
use dioxus::prelude::*;
use tokio::join;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;

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

#[component]
pub fn CalendarPage() -> Element {
    let displayed_date = use_signal(|| Utc::now());
    let view_mode = use_signal(|| ViewMode::Month);

    let mut selected_event: Signal<Option<CalendarEvent>> = use_signal(|| None);
    let mut show_form = use_signal(|| false);
    let mut prefilled_date = use_signal(|| None);

    let active_calendar_ids: Signal<Vec<String>> = use_signal(|| Vec::new());

    let mut db_resource = use_resource(move || async move {
        join!(
            fetch_calendars_lokal_db(),
            fetch_calendar_events_lokal_db(),
            fetch_groups_lokal_db(),
            fetch_profiles_lokal_db(),
        )
    });

    let (calendars_light, all_events_light, groups, profiles) = match &*db_resource.read() {
        Some((Ok(cals), Ok(evts), Ok(grps), Ok(profs))) => {
            (cals.clone(), evts.clone(), grps.clone(), profs.clone())
        }
        Some((Ok(cals), Ok(evts), _, _)) => (cals.clone(), evts.clone(), vec![], vec![]),
        Some((Ok(cals), _, _, _)) => (cals.clone(), vec![], vec![], vec![]),
        _ => (vec![], vec![], vec![], vec![]),
    };

    let calendar_color_by_id = Arc::new(build_calendar_color_map(&calendars_light, &groups));

    let calendars_full: Vec<Calendar> = calendars_light
        .iter()
        .map(|c| light_to_calendar(c, &groups, &profiles))
        .collect();

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

    let mut handle_refresh = move |_| {
        db_resource.restart();
    };

    let form_mode = use_memo(move || match selected_event() {
        Some(event) => Some(EventFormMode::Edit(event)),
        None if show_form() => Some(EventFormMode::Create),
        _ => None,
    });

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
            style: "width: 100%; height: 80vh; display: flex; overflow: hidden; color: white; background: #080910; padding: 20px; box-sizing: border-box; gap: 16px;",

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
                    on_day_click: move |date| {
                        prefilled_date.set(Some(date));
                        selected_event.set(None);
                        show_form.set(true);
                    },
                    on_event_click: move |light: CalendarEventLight| {
                        selected_event.set(parse_calendar_event_light_to_calendar_event(light).ok());
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
    format!("Calendar ({})", &c.id[..8])
}

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

fn build_calendar_color_map(
    calendars: &[CalendarLight],
    groups: &[GroupLight],
) -> HashMap<String, String> {
    calendars
        .iter()
        .map(|cal| {
            let fallback = "#9ca3af".to_string();

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

