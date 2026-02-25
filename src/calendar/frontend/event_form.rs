use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::{
    calendar::backend::create_calendar_event::create_calendar_event,
    utils::{
        functions::parse_calendar_event_light_to_calendar_event,
        structs::{CalendarEventLight, CalendarLight, Recurrent, Rrule},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum EventFormMode {
    Create,
    Edit(CalendarEventLight),
}

/// Whether a recurring event edit applies to one instance or the entire series
#[derive(Debug, Clone, PartialEq)]
pub enum RecurrentEditScope {
    OnlyThis,
    All,
}

#[component]
pub fn EventForm(
    mode: EventFormMode,
    /// All user calendars — used to populate the calendar selector dropdown
    calendars: Vec<CalendarLight>,
    /// Pre-filled start date when form is opened via a day click
    prefilled_date: Option<DateTime<Utc>>, // change to local
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
    on_deleted: EventHandler<()>,
) -> Element {
    // ----------Debugging and developing----------
    use_effect(move || {
        // on-change-prints
    });
    let testcalendar = CalendarLight {
        id: "e6abe79c-7403-45e5-aa1b-b573e4523826".to_string(),
        name: "0589386b-7fda-46a0-9c90-e1a2e47b24ac".to_string(),
        calendar_type: "private".to_string(),
        description: None,
        owner_id: Some("0589386b-7fda-46a0-9c90-e1a2e47b24ac".to_string()),
        group_id: None,
        created_at: "2026-02-07 11:51:56.54365+00".to_string(),
        created_by: "0589386b-7fda-46a0-9c90-e1a2e47b24ac".to_string(),
        last_mod: "2026-02-07 11:51:56.54365+00".to_string(),
    };
    calendars.push(testcalendar);
    // --------------------------------------------

    let initial_event = match &mode {
        EventFormMode::Edit(e) => e.clone(),
        EventFormMode::Create => CalendarEventLight {
            calendar_id: calendars
                .first()
                .map(|c| c.id.clone())
                .unwrap_or(Uuid::nil().to_string())
                .to_string(), // edit to correctly pick chosen calendar
            summary: String::new(),
            description: None,
            from_date_time: prefilled_date
                .unwrap_or_else(Utc::now)
                .format("%Y-%m-%dT%H:%M")
                .to_string(),
            to_date_time: Some(
                (prefilled_date.unwrap_or_else(Utc::now) + Duration::hours(1))
                    .format("%Y-%m-%dT%H:%M")
                    .to_string(),
            ),
            attachment: None,
            location: None,
            category: None,
            is_all_day: false,
            rrule: None,
            recurrence_until: None,
            recurrence_id: None,
            overrides_datetime: None,
            skipped: false,
            // Ignore: following fields are not needed for this function and/or should only be handled by supabase, but there to complete this struct
            id: Uuid::nil().to_string(),
            created_by: Uuid::nil().to_string(),
            created_at: Utc::now().to_string(),
            last_mod: Utc::now().to_string(),
        },
    };

    // needed event signals
    let mut summary = use_signal(|| initial_event.summary);
    let mut description = use_signal(|| initial_event.description.unwrap_or_default());
    let mut selected_calendar_id = use_signal(|| initial_event.calendar_id);
    let mut from_date = use_signal(|| initial_event.from_date_time);
    let mut to_date = use_signal(|| initial_event.to_date_time);
    let mut attachment = use_signal(|| initial_event.attachment);
    let mut location = use_signal(|| initial_event.location.unwrap_or_default());
    let mut categories = use_signal(|| initial_event.category.unwrap_or_default());
    let mut is_all_day = use_signal(|| initial_event.is_all_day);
    // not needed event signals
    let mut id = use_signal(|| initial_event.id);
    let mut created_at = use_signal(|| initial_event.created_at);
    let mut created_by = use_signal(|| initial_event.created_by);
    let mut last_mod = use_signal(|| initial_event.last_mod);
    // Recurrence signals
    let mut rrule = use_signal(|| initial_event.rrule);
    let mut recurrence_until = use_signal(|| initial_event.recurrence_until);
    let mut recurrence_id = use_signal(|| initial_event.recurrence_id);
    let mut overrides_datetime = use_signal(|| initial_event.overrides_datetime);
    let mut skipped = use_signal(|| initial_event.skipped);
    let is_recurrent = rrule().is_some();
    let is_recurrence_exception = overrides_datetime().is_some();
    // other signals
    let mut show_recurrent_scope_dialog = use_signal(|| false);
    let mut recurrent_scope: Signal<Option<RecurrentEditScope>> = use_signal(|| None);
    let is_edit = matches!(mode, EventFormMode::Edit(_));
    let mut is_loading = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);

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
                    if is_edit { "Edit Event" } else { "New Event" }
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
                        class: field_input_class(),
                        placeholder: "Event-Title (max. 25 characters)",
                        maxlength: 25,
                        value: "{summary}",
                        onchange: move |e| summary.set(e.value()),
                    }
                }

                FormField {
                    label: "Calender",
                    required: true,
                    select {
                        class: field_input_class(),
                        onchange: move |e| {
                            selected_calendar_id.set(e.value());
                        },
                        for cal in &calendars {
                            option {
                                value: "{cal.id}",
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
                    }
                    label { class: "text-sm text-white/70", "All Day" }
                }

                FormField {
                    label: "from",
                    required: true,
                    input {
                        class: field_input_class(),
                        r#type: if is_all_day() { "date" } else { "datetime-local" },
                        value: "{from_date}", // pre-set value doesn't work if is_all_day(), because you would give a datetime to a date
                        onchange: move |e| from_date.set(e.value()),
                    }
                }

                if !is_all_day() {
                    FormField {
                        label: "to (optional)",
                        input {
                            class: field_input_class(),
                            r#type: "datetime-local",
                            value: "{to_date().unwrap_or_default()}",
                            onchange: move |e| {
                                let value = e.value();
                                if value.is_empty() {
                                    to_date.set(None);
                                } else {
                                    to_date.set(Some(e.value()));
                                }
                            },
                        }
                    }
                }

                FormField {
                    label: "Categories (optional)",
                    input {
                        class: field_input_class(),
                        placeholder: "Categories (seperated by comma)",
                        value: "{categories}",
                        onchange: move |e| categories.set(e.value()),
                    }
                }

                FormField {
                    label: "Location (optional)",
                    input {
                        class: field_input_class(),
                        placeholder: "Location or Link",
                        value: "{location}",
                        onchange: move |e| location.set(e.value()),
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
                        value: "{description}",
                        onchange: move |e| description.set(e.value()),
                    }
                }

                RecurrencePicker { rrule, recurrence_until, from_date }

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
                        // Editing a recurring parent requires choosing the scope first
                        if is_edit && is_recurrent && !is_recurrence_exception {
                            show_recurrent_scope_dialog.set(true);
                        } else {
                            // TODO: Call create_calendar_event or edit_single_calendar_event
                            // create new CalendarEventLight to be able to use parse_calendar_event_light_to_calendar_event, because this function already has all the parsing in it
                            let new_light_event = CalendarEventLight {
                                id: id(),
                                calendar_id: selected_calendar_id(),
                                summary: summary(),
                                description: Some(description()),
                                from_date_time: format!("{}:00{}", from_date(), Local::now().format("%:z")), // add local timezone info; expects correct local datetime
                                to_date_time: match to_date() {
                                    Some(date) => Some(format!("{}:00{}", date, Local::now().format("%:z"))), // add local timezone info; expects correct local datetime
                                    None => None,
                                },
                                attachment: attachment(),
                                rrule: rrule(),
                                recurrence_until: recurrence_until(), // timezones
                                location: Some(location()),
                                category: Some(categories()),
                                is_all_day: is_all_day(),
                                recurrence_id: recurrence_id(),
                                overrides_datetime: overrides_datetime(), // timezones
                                skipped: skipped(),
                                created_at: created_at(), // timezones
                                created_by: created_by(),
                                last_mod: last_mod(), // timezones
                            };
                            println!(
                                "
                                new_light_event
                                id: {}\n
                                calendar_id: {}\n
                                summary: {}\n
                                description: {:?}\n
                                from_date_time: {}\n
                                to_date_time: {:?}\n
                                attachment: {:?}\n
                                rrule: {:?}\n
                                recurrence_until: {:?}\n
                                location: {:?}\n
                                category: {:?}\n
                                is_all_day: {}\n
                                recurrence_id: {:?}\n
                                overrides_datetime: {:?}\n
                                skipped: {}\n
                                created_at: {}\n
                                created_by: {}\n
                                last_mod: {}\n
                                ",
                                new_light_event.id,
                                new_light_event.calendar_id,
                                new_light_event.summary,
                                new_light_event.description,
                                new_light_event.from_date_time,
                                new_light_event.to_date_time,
                                new_light_event.attachment,
                                new_light_event.rrule,
                                new_light_event.recurrence_until,
                                new_light_event.location,
                                new_light_event.category,
                                new_light_event.is_all_day,
                                new_light_event.recurrence_id,
                                new_light_event.overrides_datetime,
                                new_light_event.skipped,
                                new_light_event.created_at,
                                new_light_event.created_by,
                                new_light_event.last_mod,

                            );
                            // put further control flow directly in correlation of result of parsing; i know it's shit but time's running out
                            match parse_calendar_event_light_to_calendar_event(new_light_event) {
                                Ok(event) => {
                                    // distinguish between creating and editing event and call function
                                    match &mode {
                                        EventFormMode::Create => {
                                            spawn(async move {
                                                match create_calendar_event(event.summary, event.description, event.calendar_id, event.from_date_time, event.to_date_time, event.attachment, event.recurrence, event.recurrence_exception, event.location, event.categories, event.is_all_day).await {
                                                Ok(()) => {
                                                    println!("Event erstellt");
                                                    on_close.call(());
                                                },
                                                Err(err) => {
                                                    error_msg.set(Some(err.to_string()));
                                                },
                                            }
                                            });
                                        },
                                        EventFormMode::Edit(e) => {
                                            todo!()
                                        },
                                    }
                                },
                                Err(err) => {
                                    error_msg.set(Some(err.to_string()));

                                }
                            };

                        }
                    },
                    if is_loading() { "Saving…" } else { "Save" }
                }

                if is_edit {
                    DeleteButton {
                        is_recurrent,
                        is_loading,
                        on_delete_instance: move |_| {
                            // TODO: Call delete_instance_of_recurrent_event
                        },
                        on_delete_all: move |_| {
                            // TODO: Call delete_calendar_event_with_all_instances
                        },
                        on_delete_single: move |_| {
                            // TODO: Call delete_single_calendar_event
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
                    // TODO: Call edit_instance_of_recurrent_event
                },
                on_all: move |_| {
                    show_recurrent_scope_dialog.set(false);
                    recurrent_scope.set(Some(RecurrentEditScope::All));
                    // TODO: Call edit_calendar_event with keep_overridings / keep_orphans flags
                },
                on_cancel: move |_| show_recurrent_scope_dialog.set(false),
            }
        }
    }
}

#[component]
pub fn RecurrencePicker(
    rrule: Signal<Option<String>>,
    recurrence_until: Signal<Option<String>>,
    from_date: Signal<String>,
) -> Element {
    let is_active = use_memo(move || rrule().is_some());

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
                            rrule.set(None);
                            recurrence_until.set(None);
                        } else {
                            // Default to daily, ending 30 days from now
                            rrule.set(Some("Daily".to_string()));
                            let naive = NaiveDate::parse_from_str(&from_date(), "%Y-%m-%d").unwrap_or_else(|_| Utc::now().date_naive());
                            let until = naive + Duration::days(30);
                            recurrence_until.set(Some(until.to_string()));
                        }
                    },
                }
                label { class: "text-sm text-white/70", "Recurrence" }
            }

            if is_active() {
                div {
                    class: "flex flex-col gap-3 pl-7",

                    FormField {
                        label: "Frequency",
                        select {
                            class: field_input_class(),
                            value: "{rrule().unwrap_or_else(|| \"Daily\".to_string())}",
                            onchange: move |e| {
                                rrule.set(Some(e.value()));
                            },
                            option { value: "Daily", style: "background: #1A1D2B", "Daily" }
                            option { value: "Weekly", style: "background: #1A1D2B", "Weekly" }
                            option { value: "Fortnight", style: "background: #1A1D2B", "Fortnight" }
                            option { value: "OnWeekDays", style: "background: #1A1D2B", "On Week Days" }
                            option { value: "MonthlyOnDate", style: "background: #1A1D2B", "Monthly On Date"}
                            option { value: "MonthlyOnWeekday", style: "background: #1A1D2B", "Monthly On Weekday"}
                            option { value: "Annual", style: "background: #1A1D2B", "Annual" }
                        }
                    }

                    FormField {
                        label: "Repeat until",
                        input {
                            class: field_input_class(),
                            r#type: "date",
                            value: "{recurrence_until().unwrap_or_default()}",
                            onchange: move |e| {
                                recurrence_until.set(Some(e.value()));
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DeleteButton(
    is_recurrent: bool,
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
                        on_delete_single.call(());
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
                h3 { class: "text-white font-semibold text-base", "edit repeating event" }
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
