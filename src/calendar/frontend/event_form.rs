/*
Side note:  be aware that major parts of the css styling were made with LLMs
            anything else is highlighted in the spot where it was used
*/

//! Event form and affiliated
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Timelike, Utc};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::{
    calendar::backend::{
        change_calendar_event::{
            edit_calendar_event, edit_instance_of_recurrent_event, edit_single_calendar_event,
        },
        create_calendar_event::create_calendar_event,
        delete_calendar_event::{
            delete_calendar_event_with_all_instances, delete_instance_of_recurrent_event,
            delete_single_calendar_event,
        },
    },
    utils::{
        functions::parse_calendar_event_light_to_calendar_event,
        structs::{Calendar, CalendarEvent, CalendarEventLight, Recurrent, Rrule},
    },
};

/// Current event mode
#[derive(Debug, Clone, PartialEq)]
pub enum EventFormMode {
    Create,
    Edit(Box<CalendarEvent>),
    View(Box<CalendarEvent>),
}

/// Whether a recurring event edit applies to one instance or the entire series
#[derive(Debug, Clone, PartialEq)]
pub enum RecurrentEditScope {
    OnlyThis,
    All,
}

/// Event form for editing and creating calendar events
///
/// Opens with click on event or day cell
///
/// Contains
/// * RecurrencePicker
/// * DeleteButton
/// * RecurrentScopeDialog
/// * FormField
#[component]
pub fn EventForm(
    mode: EventFormMode,
    /// All user calendars — used to populate the calendar selector dropdown
    calendars: Vec<Calendar>,
    /// All visible events
    events: Vec<CalendarEventLight>,
    /// Pre-filled start date when form is opened via a day click
    prefilled_date: Option<DateTime<Utc>>,
    on_close: EventHandler<()>,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut action_mode = use_signal(|| mode);

    let initial_event = match action_mode() {
        EventFormMode::Edit(e) | EventFormMode::View(e) => *e.clone(),
        EventFormMode::Create => CalendarEvent {
            calendar_id: calendars.first().map(|c| c.id).unwrap_or(Uuid::nil()),
            summary: String::new(),
            description: None,
            from_date_time: prefilled_date
                .unwrap_or_else(Utc::now)
                .with_nanosecond(0) // cut nanoseconds off, else input field can't display it
                .unwrap(), // with_nanosecond(0) is unfailable
            to_date_time: Some(prefilled_date.unwrap_or_else(Utc::now) + Duration::hours(1)),
            attachment: None,
            location: None,
            categories: None,
            is_all_day: false,
            recurrence: None,
            recurrence_exception: None,
            // Ignore: following fields are not needed for this function and/or should only be handled by supabase, but are there to complete this struct
            id: Uuid::nil(),
            created_by: Uuid::nil(),
            created_at: Utc::now(),
            last_mod: Utc::now(),
        },
    };

    // needed event signals
    let mut summary = use_signal(|| initial_event.summary);
    let mut description = use_signal(|| initial_event.description);
    let mut selected_calendar_id = use_signal(|| initial_event.calendar_id);
    let mut from_date = use_signal(|| initial_event.from_date_time);
    let mut to_date = use_signal(|| initial_event.to_date_time);
    let mut attachment = use_signal(|| initial_event.attachment);
    let mut location = use_signal(|| initial_event.location);
    let mut categories = use_signal(|| initial_event.categories);
    let mut is_all_day = use_signal(|| initial_event.is_all_day);
    // only needed for information display later
    let mut id = use_signal(|| initial_event.id);
    let mut created_at = use_signal(|| initial_event.created_at);
    let mut created_by = use_signal(|| initial_event.created_by);
    let mut last_mod = use_signal(|| initial_event.last_mod);
    // Recurrence signals
    let mut recurrence = use_signal(|| initial_event.recurrence);
    let mut recurrence_exception = use_signal(|| initial_event.recurrence_exception);
    let is_recurrent = recurrence().is_some();
    let is_recurrence_exception = recurrence_exception().is_some();
    // other signals
    let mut show_recurrent_scope_dialog = use_signal(|| false);
    let mut recurrent_scope: Signal<Option<RecurrentEditScope>> = use_signal(|| None);

    let is_edit = use_memo(move || matches!(action_mode(), EventFormMode::Edit(_)));
    let is_view = use_memo(move || matches!(action_mode(), EventFormMode::View(_)));
    let is_loading = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);

    // effect created by AI due to time problems
    // for automatic switching values
    let default_calendar_id = calendars.first().map(|c| c.id).unwrap_or(Uuid::nil());
    use_effect(move || {
        let next = match action_mode() {
            EventFormMode::Edit(e) | EventFormMode::View(e) => *e.clone(),
            EventFormMode::Create => CalendarEvent {
                calendar_id: default_calendar_id,
                summary: String::new(),
                description: None,
                from_date_time: prefilled_date
                    .unwrap_or_else(Utc::now)
                    .with_nanosecond(0) // cut nanoseconds off, else input field can't display it
                    .unwrap(), // with_nanosecond(0) is unfailable
                to_date_time: Some(prefilled_date.unwrap_or_else(Utc::now) + Duration::hours(1)),
                attachment: None,
                location: None,
                categories: None,
                is_all_day: false,
                recurrence: None,
                recurrence_exception: None,
                // Ignore: following fields are not needed for this function and/or should only be handled by supabase, but are there to complete this struct
                id: Uuid::nil(),
                created_by: Uuid::nil(),
                created_at: Utc::now(),
                last_mod: Utc::now(),
            },
        };

        summary.set(next.summary);
        description.set(next.description);
        selected_calendar_id.set(next.calendar_id);
        from_date.set(next.from_date_time);
        to_date.set(next.to_date_time);
        attachment.set(next.attachment);
        location.set(next.location);
        categories.set(next.categories);
        is_all_day.set(next.is_all_day);
        id.set(next.id);
        created_at.set(next.created_at);
        created_by.set(next.created_by);
        last_mod.set(next.last_mod);
        recurrence.set(next.recurrence);
        recurrence_exception.set(next.recurrence_exception);
    });

    // for checking if this is a parent
    let initial_is_recurrent = is_recurrent;
    let initial_is_recurrence_exception = is_recurrence_exception;

    // memo created by Github Copilot (GPT)
    let from_date_formatted = use_memo(move || {
        if is_all_day() {
            from_date().date_naive().to_string()
        } else {
            from_date().naive_utc().format("%Y-%m-%dT%H:%M").to_string()
        }
    });

    // memo created by Github Copilot (GPT)
    let to_date_formatted = use_memo(move || {
        if is_all_day() {
            to_date()
                .map(|d| d.date_naive().to_string())
                .unwrap_or_else(|| "".to_string())
        } else {
            to_date()
                .map(|d| d.naive_utc().format("%Y-%m-%dT%H:%M").to_string())
                .unwrap_or_else(|| "".to_string())
        }
    });

    rsx! {
        // Dimmed backdrop — clicking it closes the form
        div {
            class: "fixed inset-0 bg-black/50 z-40",
            onclick: move |_| on_close.call(()),
        }

        div {
            class: "
                fixed right-0 top-0 h-full w-[420px] z-50
                bg-[#0E1120] border-l border-white/10
                shadow-2xl flex flex-col overflow-y-auto
            ",

            div {
                class: "flex items-center justify-between px-6 py-5 border-b border-white/10",
                h2 {
                    class: "text-white font-semibold text-base",
                    if is_edit() { "Edit Event" } else { "New Event" }
                }
                button {
                    class: "text-white/40 hover:text-white transition text-xl",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            div {
                class: "flex flex-col gap-5 px-6 py-6 flex-1",

                FormField {
                    label: "Title",
                    required: true,
                    input {
                        class: if !is_view() {field_input_class()} else {field_input_class_disabled()},
                        placeholder: "Event-Title (max. 25 characters)",
                        maxlength: 25,
                        value: "{summary}",
                        onchange: move |e| summary.set(e.value()),
                        disabled: is_view,
                    }
                }

                FormField {
                    label: "Calender",
                    required: true,
                    select {
                        class: if !is_view() {field_input_class()} else {field_input_class_disabled()},
                        onchange: move |e| {
                            selected_calendar_id.set(Uuid::try_parse(&e.value()).unwrap_or_else(|_| selected_calendar_id()));
                        },
                        disabled: is_view,
                        for cal in &calendars {
                            option {
                                value: "{cal.id}",
                                style: "background: #1A1D2B",
                                selected: cal.id == selected_calendar_id(),
                                "{cal.name}"
                            }
                        }
                    }
                }

                div {
                    class: "flex items-center gap-3",
                    input {
                        r#type: "checkbox",
                        class: "w-4 h-4 accent-blue-500",
                        checked: is_all_day(),
                        onchange: move |_| is_all_day.set(!is_all_day()),
                        disabled: is_view,
                    }
                    label { class: "text-sm text-white/70", "All Day" }
                }

                FormField {
                    label: "from",
                    required: true,
                    input {
                        class: if !is_view() {field_input_class()} else {field_input_class_disabled()},
                        r#type: if is_all_day() { "date" } else { "datetime-local" },
                        value: "{from_date_formatted}",
                        onchange: move |e| {
                            if is_all_day() {
                                from_date.set(
                                    // help with parsing by Github Copilot (GPT)
                                    NaiveDate::parse_from_str(&e.value(), "%Y-%m-%d")
                                    .unwrap_or_else(|_| from_date().date_naive())
                                    .and_hms_opt(0, 0, 0)
                                    .unwrap() // safe because 0,0,0 is always some
                                    .and_utc()
                                );
                            } else {
                                from_date.set(
                                    // help with parsing by Github Copilot (GPT)
                                    NaiveDateTime::parse_from_str(&e.value(), "%Y-%m-%dT%H:%M")
                                    .map(|d| d.and_utc())
                                    .unwrap_or_else(|_| from_date())
                                );
                            }
                        },
                        disabled: is_view,
                    }
                }

                FormField {
                    label: "to (optional)",
                    input {
                        class: if !is_view() {field_input_class()} else {field_input_class_disabled()},
                        r#type: if is_all_day() { "date" } else { "datetime-local" },
                        value: "{to_date_formatted}",
                        onchange: move |e| {
                            let value = e.value();
                            if value.is_empty() {
                                to_date.set(None);
                            } else if is_all_day() {
                                to_date.set(Some(
                                    // help with parsing by Github Copilot (GPT)
                                    NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                                    .unwrap_or_else(|_| from_date().date_naive())
                                    .and_hms_opt(0, 0, 0)
                                    .unwrap() // and_hms_opt(0, 0, 0) is unfailable
                                    .and_utc()
                                ));
                            } else {
                                to_date.set(Some(
                                    // help with parsing by Github Copilot (GPT)
                                    NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M")
                                    .map(|d| d.and_utc())
                                    .unwrap_or_else(|_| from_date())
                                ));
                            }
                        },
                        disabled: is_view,
                    }
                }

                FormField {
                    label: "Categories (optional)",
                    input {
                        class: if !is_view() {field_input_class()} else {field_input_class_disabled()},
                        placeholder: "Categories (separated by comma)",
                        value: "{categories().unwrap_or_default().join(\", \")}",
                        onchange: move |e| categories.set(Some(e.value().split(",").map(|s| s.trim().to_string()).collect())),
                        disabled: is_view,
                    }
                }

                FormField {
                    label: "Location (optional)",
                    input {
                        class: if !is_view() {field_input_class()} else {field_input_class_disabled()},
                        placeholder: "Location or Link",
                        value: "{location().unwrap_or_default()}",
                        onchange: move |e| location.set(Some(e.value())),
                        disabled: is_view,
                    }
                }

                FormField {
                    label: "Description (optional)",
                    textarea {
                        class: "
                            w-full px-3 py-2.5 rounded-xl
                            bg-white/5 border border-white/10
                            text-white text-sm placeholder:text-white/30
                            outline-none resize-none h-20
                        ",
                        placeholder: "Description…",
                        value: "{description().unwrap_or_default()}",
                        onchange: move |e| description.set(Some(e.value())),
                        disabled: is_view,
                    }
                }

                RecurrencePicker { recurrence,  from_date, is_view: is_view() }

                if let Some(msg) = error_msg() {
                    div {
                        class: "text-red-400 text-sm bg-red-400/10 px-3 py-2 rounded-xl",
                        "{msg}"
                    }
                }
            }

            div {
                class: "px-6 py-5 border-t border-white/10 flex flex-col gap-3",

                button {
                    class: "
                        w-full py-3 rounded-xl font-semibold text-sm
                        bg-gradient-to-b from-blue-500 to-blue-600
                        text-white shadow-lg shadow-blue-500/30
                        hover:opacity-90 transition
                        disabled:opacity-40 disabled:cursor-not-allowed
                    ",
                    disabled: is_loading(),
                    onclick: move |_| {
                        match action_mode() {
                            EventFormMode::Create => {
                                spawn(async move {
                                    match create_calendar_event(summary(), description(), selected_calendar_id(), from_date(), to_date(), attachment(), recurrence(), recurrence_exception(), location(), categories(), is_all_day()).await {
                                    Ok(()) => {
                                        on_refresh.call(());
                                    },
                                    Err(err) => {
                                        error_msg.set(Some(err.to_string()));
                                    },
                                }
                                });
                            },
                            EventFormMode::Edit(_) => {
                                // if recurrence or recurrence exception:
                                if initial_is_recurrent || initial_is_recurrence_exception {
                                    match recurrent_scope() {
                                        // if recurrent_scope() == RecurrentEditScope::All
                                        Some(RecurrentEditScope::All) => {
                                            spawn(async move {
                                                match edit_calendar_event(CalendarEvent{
                                                    id: id(),
                                                    summary: summary(),
                                                    description: description(),
                                                    calendar_id: selected_calendar_id(),
                                                    created_at: created_at(),
                                                    created_by: created_by(),
                                                    from_date_time: from_date(),
                                                    to_date_time: to_date(),
                                                    attachment: attachment(),
                                                    recurrence: recurrence(),
                                                    recurrence_exception: recurrence_exception(),
                                                    location: location(),
                                                    categories: categories(),
                                                    is_all_day: is_all_day(),
                                                    last_mod: last_mod(),
                                                },
                                                None,
                                                None).await {
                                                Ok(()) => {
                                                    on_refresh.call(());
                                                },
                                                Err(err) => {
                                                    error_msg.set(Some(err.to_string()));
                                                },
                                                }
                                            });
                                        },
                                        // if recurrent_scope() == RecurrentEditScope::OnlyThis
                                        Some(RecurrentEditScope::OnlyThis) => {
                                            spawn(async move {
                                                match edit_instance_of_recurrent_event(CalendarEvent{
                                                    id: id(),
                                                    summary: summary(),
                                                    description: description(),
                                                    calendar_id: selected_calendar_id(),
                                                    created_at: created_at(),
                                                    created_by: created_by(),
                                                    from_date_time: from_date(),
                                                    to_date_time: to_date(),
                                                    attachment: attachment(),
                                                    recurrence: recurrence(),
                                                    recurrence_exception: recurrence_exception(),
                                                    location: location(),
                                                    categories: categories(),
                                                    is_all_day: is_all_day(),
                                                    last_mod: last_mod(),
                                                }).await {
                                                Ok(()) => {
                                                    on_refresh.call(());
                                                },
                                                Err(err) => {
                                                    error_msg.set(Some(err.to_string()));
                                                },
                                                }
                                            });
                                        },
                                        None => {},
                                    }
                                // if normal event:
                                } else {
                                    spawn(async move {
                                        match edit_single_calendar_event(CalendarEvent{
                                            id: id(),
                                            summary: summary(),
                                            description: description(),
                                            calendar_id: selected_calendar_id(),
                                            created_at: created_at(),
                                            created_by: created_by(),
                                            from_date_time: from_date(),
                                            to_date_time: to_date(),
                                            attachment: attachment(),
                                            recurrence: recurrence(),
                                            recurrence_exception: recurrence_exception(),
                                            location: location(),
                                            categories: categories(),
                                            is_all_day: is_all_day(),
                                            last_mod: last_mod(),
                                        }).await {
                                        Ok(()) => {
                                            on_refresh.call(());
                                        },
                                        Err(err) => {
                                            error_msg.set(Some(err.to_string()));
                                        },
                                    }
                                    });
                                }
                            },
                            EventFormMode::View(e) => {
                                // if recurrence or exception:
                                if is_recurrent || is_recurrence_exception {
                                    //  first show_recurrent_scope_dialog()
                                    println!("Ist recurrent oder exception");
                                    show_recurrent_scope_dialog.set(true);
                                } else { // if not recurrent:
                                    //  simply edit mode
                                    action_mode.set(EventFormMode::Edit(e));
                                }
                            },
                        }


                    },
                    if is_view() {
                        "Edit"
                    } else if is_loading() {
                        "Saving…"
                    } else {
                        "Save"
                    }
                }

                if is_view() {
                    DeleteButton {
                        is_recurrent,
                        is_recurrence_exception,
                        is_loading,
                        on_delete_instance: move |_| {
                            spawn(async move {
                                match delete_instance_of_recurrent_event(recurrence_exception().map(|e| e.recurrence_id).unwrap_or(id()), from_date(), None, Some(true)).await {
                                    Ok(()) => {
                                        on_refresh.call(());
                                    },
                                    Err(err) => {
                                        error_msg.set(Some(err.to_string()));
                                    },
                                }
                            });
                        },
                        on_delete_all: move |_| {
                            spawn(async move {
                                match delete_calendar_event_with_all_instances(id()).await {
                                    Ok(()) => {
                                        on_refresh.call(());
                                    },
                                    Err(err) => {
                                        error_msg.set(Some(err.to_string()));
                                    },
                                }
                            });
                        },
                        on_delete_single: move |_| {
                            spawn(async move {
                                match delete_single_calendar_event(id()).await {
                                    Ok(()) => {
                                        on_refresh.call(());
                                    },
                                    Err(err) => {
                                        error_msg.set(Some(err.to_string()));
                                    },
                                }
                            });
                        },
                    }
                }
            }
        }

        // Modal shown when saving a recurring event — lets user choose scope
        if show_recurrent_scope_dialog() {
            RecurrentScopeDialog {
                on_only_this: move |_| {
                    show_recurrent_scope_dialog.set(false);
                    recurrent_scope.set(Some(RecurrentEditScope::OnlyThis));
                    if let EventFormMode::View(e) = action_mode() {
                        action_mode.set(EventFormMode::Edit(e));
                    }
                },
                on_all: move |_| {
                    show_recurrent_scope_dialog.set(false);
                    recurrent_scope.set(Some(RecurrentEditScope::All));
                    if is_recurrent {
                        if let EventFormMode::View(e) = action_mode() {
                            action_mode.set(EventFormMode::Edit(e));
                        }
                    } else {
                        // search parent and give to edit mode
                        let parent_event = events
                            .iter()
                            .find(|e| recurrence_exception().unwrap().recurrence_id.to_string() == e.id)
                            .and_then(|e| parse_calendar_event_light_to_calendar_event(e.clone()).ok());
                        if let Some(event) = parent_event {
                            action_mode.set(EventFormMode::Edit(Box::new(event)));
                        }
                    }
                },
                on_cancel: move |_| show_recurrent_scope_dialog.set(false),
            }
        }
    }
}

/// UI component with additional options for editing recurrence
#[component]
pub fn RecurrencePicker(
    recurrence: Signal<Option<Recurrent>>,
    from_date: Signal<DateTime<Utc>>,
    is_view: bool,
) -> Element {
    let is_active = use_memo(move || recurrence().is_some());

    let rrule_value = use_memo(move || match recurrence().unwrap_or_default().rrule {
        Rrule::Daily => "Daily",
        Rrule::Weekly => "Weekly",
        Rrule::Fortnight => "Fortnight",
        Rrule::OnWeekDays => "weekdays",
        Rrule::MonthlyOnDate => "monthly_on_date",
        Rrule::MonthlyOnWeekday => "monthly_on_weekday",
        Rrule::Annual => "Annual",
    });

    rsx! {
        div {
            class: "flex flex-col gap-3",

            div {
                class: "flex items-center gap-3",
                input {
                    r#type: "checkbox",
                    class: "w-4 h-4 accent-blue-500",
                    checked: is_active(),
                    onchange: move |_| {
                        if is_active() {
                            recurrence.set(None);
                        } else {
                            // Default to daily, ending 30 days from now
                            recurrence.set(Some(Recurrent { rrule: Rrule::Daily, recurrence_until: from_date() + Duration::days(30) }
                            ));
                        }
                    },
                        disabled: is_view,
                }
                label { class: "text-sm text-white/70", "Recurrence" }
            }

            if is_active() {
                div {
                    class: "flex flex-col gap-3 pl-7",

                    FormField {
                        label: "Frequency",
                        select {
                        class: if !is_view {field_input_class()} else {field_input_class_disabled()},
                            value: "{rrule_value}",
                            onchange: move |e| {
                                recurrence.set(Some(Recurrent { rrule: e.value().parse::<Rrule>().unwrap_or(Rrule::Daily), ..recurrence().unwrap_or_default() }));
                            },
                            disabled: is_view,
                            option { value: "Daily", style: "background: #1A1D2B", "Daily" }
                            option { value: "Weekly", style: "background: #1A1D2B", "Weekly" }
                            option { value: "Fortnight", style: "background: #1A1D2B", "Fortnight" }
                            option { value: "weekdays", style: "background: #1A1D2B", "On Week Days" }
                            option { value: "monthly_on_date", style: "background: #1A1D2B", "Monthly On Date"}
                            option { value: "monthly_on_weekday", style: "background: #1A1D2B", "Monthly On Weekday"}
                            option { value: "Annual", style: "background: #1A1D2B", "Annual" }
                        }
                    }

                    FormField {
                        label: "Repeat until",
                        input {
                        class: if !is_view {field_input_class()} else {field_input_class_disabled()},
                            r#type: "date",
                            value: "{recurrence().unwrap_or_default().recurrence_until.date_naive()}",
                            onchange: move |e| {
                                let current = recurrence().unwrap_or_default();
                                recurrence.set(Some(Recurrent {
                                // help with parsing by Github Copilot (GPT)
                                recurrence_until: NaiveDate::parse_from_str(&e.value(), "%Y-%m-%d")
                                    .unwrap_or_else(|_| current.recurrence_until.date_naive())
                                    .and_hms_opt(0, 0, 0)
                                    .unwrap() // and_hms_opt(0, 0, 0) is unfailable
                                    .and_utc(),
                                ..current }));
                            },
                            disabled: is_view,
                        }
                    }
                }
            }
        }
    }
}

/// Delete button for events
#[component]
fn DeleteButton(
    is_recurrent: bool,
    is_recurrence_exception: bool,
    is_loading: Signal<bool>,
    on_delete_instance: EventHandler<()>,
    on_delete_all: EventHandler<()>,
    on_delete_single: EventHandler<()>,
) -> Element {
    let mut show_delete_menu = use_signal(|| false);

    rsx! {
        div { class: "relative",
            button {
                class: "
                    w-full py-2.5 rounded-xl font-medium text-sm
                    border border-red-500/40 text-red-400
                    hover:bg-red-500/10 transition
                    disabled:opacity-40 disabled:cursor-not-allowed
                ",
                disabled: is_loading(),
                onclick: move |_| {
                    // Recurring events get a dropdown; single events delete immediately
                    if is_recurrent {
                        show_delete_menu.set(!show_delete_menu());
                    } else {
                        // check if is really single event or instance of recurring
                        if is_recurrence_exception {
                            on_delete_instance.call(());
                        } else {
                            on_delete_single.call(());
                        }
                    }
                },
                "Delete"
            }

            if show_delete_menu() {
                div {
                    class: "
                        absolute bottom-full mb-2 left-0 right-0
                        bg-[#1a1d2e] border border-white/10 rounded-xl
                        shadow-2xl overflow-hidden z-10
                    ",
                    button {
                        class: "w-full px-4 py-3 text-sm text-left text-white/70 hover:bg-white/5 transition",
                        onclick: move |_| {
                            show_delete_menu.set(false);
                            on_delete_instance.call(());
                        },
                        "Delete only this Event"
                    }
                    div { class: "h-px bg-white/10" }
                    button {
                        class: "w-full px-4 py-3 text-sm text-left text-red-400 hover:bg-white/5 transition",
                        onclick: move |_| {
                            show_delete_menu.set(false);
                            on_delete_all.call(());
                        },
                        "Delete all Events"
                    }
                }
            }
        }
    }
}

/// UI component for choosing if you want to adapt changes to only this event or all in the series
#[component]
fn RecurrentScopeDialog(
    on_only_this: EventHandler<()>,
    on_all: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 bg-black/70 z-[60] flex items-center justify-center",
            div {
                class: "
                    bg-[#0E1120] border border-white/10 rounded-2xl
                    p-6 w-[340px] shadow-2xl flex flex-col gap-5
                ",
                h3 { class: "text-white font-semibold text-base", "Edit Repeating Event" }
                p { class: "text-white/50 text-sm", "Do you want to change only this or all events?" }

                div { class: "flex flex-col gap-2",
                    button {
                        class: "w-full py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-sm text-white transition",
                        onclick: move |_| on_only_this.call(()),
                        "Only this event"
                    }
                    button {
                        class: "w-full py-2.5 rounded-xl bg-blue-600 hover:bg-blue-500 text-sm text-white font-medium transition",
                        onclick: move |_| on_all.call(()),
                        "All events"
                    }
                    button {
                        class: "text-sm text-white/40 hover:text-white/70 transition",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                }
            }
        }
    }
}

/// Single input field
#[component]
fn FormField(
    label: &'static str,
    #[props(default = false)] required: bool,
    children: Element,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1.5",
            label {
                class: "text-xs text-white/50 tracking-wider uppercase",
                "{label}"
                if required {
                    span { class: "text-blue-400 ml-0.5", " *" }
                }
            }
            {children}
        }
    }
}

fn field_input_class() -> &'static str {
    "
    w-full px-3 py-2.5 rounded-xl
    bg-white/5 border border-white/10
    text-white text-sm placeholder:text-white/30
    outline-none focus:border-blue-500/50
    transition
    "
}

fn field_input_class_disabled() -> &'static str {
    "
    w-full px-3 py-2.5 rounded-xl
    bg-white/5 border border-white/10
    text-gray-400 text-sm placeholder:text-white/30
    outline-none focus:border-blue-500/50
    transition
    "
}
