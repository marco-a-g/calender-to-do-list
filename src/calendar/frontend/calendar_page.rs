use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::calendar::frontend::{
    calendar_grid::{CalendarGrid, ViewMode},
    event_form::{EventForm, EventFormMode},
};
use crate::utils::structs::{Calendar, CalendarEvent};

#[component]
pub fn CalendarPage() -> Element {
    // TODO: Load calendars from backend on mount (fetch user's own + group calendars)
    let calendars = use_signal(|| Vec::<Calendar>::new());

    // IDs of calendars currently toggled visible — managed by CalendarSidebar (Person C)
    let active_calendar_ids = use_signal(|| Vec::<Uuid>::new());

    // TODO: Load events from backend when `displayed_date` or `view_mode` changes
    let events = use_signal(|| Vec::<CalendarEvent>::new());

    let displayed_date = use_signal(|| Utc::now());
    let view_mode = use_signal(|| ViewMode::Month);

    let mut selected_event: Signal<Option<CalendarEvent>> = use_signal(|| None);
    let mut show_form = use_signal(|| false);
    // Pre-fills the "from" date when opening the form via a day click
    let mut prefilled_date: Signal<Option<DateTime<Utc>>> = use_signal(|| None);

    // Derived: only render events whose calendar is currently active
    let visible_events = use_memo(move || {
        let ids = active_calendar_ids();
        events()
            .into_iter()
            .filter(|e| ids.contains(&e.calendar_id))
            .collect::<Vec<_>>()
    });

    // Derived: translate signal state into a typed form mode for EventForm
    let form_mode = use_memo(move || match selected_event() {
        Some(event) => Some(EventFormMode::Edit(event)),
        None if show_form() => Some(EventFormMode::Create),
        _ => None,
    });

    rsx! {
        div {
            class: "relative w-full min-h-screen flex overflow-hidden text-white",
            style: "background: linear-gradient(to bottom, #070B18, #050914, black);",

            div {
                class: "flex flex-col flex-1 overflow-hidden",

                CalendarGrid {
                    events: visible_events(),
                    calendars: calendars(),
                    displayed_date,
                    view_mode,
                    on_day_click: move |date: DateTime<Utc>| {
                        prefilled_date.set(Some(date));
                        selected_event.set(None);
                        show_form.set(true);
                    },
                    on_event_click: move |event: CalendarEvent| {
                        selected_event.set(Some(event));
                        show_form.set(true);
                    },
                }
            }

            // EventForm renders as a slide-in panel; only mounted when a mode is active
            if let Some(mode) = form_mode() {
                EventForm {
                    mode,
                    calendars: calendars(),
                    prefilled_date: prefilled_date(),
                    on_close: move |_| {
                        show_form.set(false);
                        selected_event.set(None);
                        prefilled_date.set(None);
                    },
                    on_saved: move |_| {
                        // TODO: Trigger event reload after successful save
                        show_form.set(false);
                        selected_event.set(None);
                    },
                    on_deleted: move |_| {
                        // TODO: Trigger event reload after successful delete
                        show_form.set(false);
                        selected_event.set(None);
                    },
                }
            }
        }
    }
}
