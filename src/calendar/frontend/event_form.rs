use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::utils::structs::{Calendar, CalendarEvent, Recurrent, Rrule};

#[derive(Debug, Clone, PartialEq)]
pub enum EventFormMode {
    Create,
    Edit(CalendarEvent),
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
    calendars: Vec<Calendar>,
    /// Pre-filled start date when form is opened via a day click
    prefilled_date: Option<DateTime<Utc>>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
    on_deleted: EventHandler<()>,
) -> Element {
    let initial_event = match &mode {
        EventFormMode::Edit(e) => e.clone(),
        EventFormMode::Create => CalendarEvent {
            id: Uuid::nil(),
            summary: String::new(),
            description: None,
            calendar_id: calendars.first().map(|c| c.id).unwrap_or(Uuid::nil()),
            created_at: Utc::now(),
            created_by: Uuid::nil(),
            from_date_time: prefilled_date.unwrap_or_else(Utc::now),
            to_date_time: None,
            attachment: None,
            recurrence: None,
            recurrence_exception: None,
            location: None,
            categories: None,
            is_all_day: false,
            last_mod: Utc::now(),
        },
    };

    let mut summary = use_signal(|| initial_event.summary.clone());
    let mut description = use_signal(|| initial_event.description.clone().unwrap_or_default());
    let mut selected_calendar_id = use_signal(|| initial_event.calendar_id);
    let mut from_date = use_signal(|| initial_event.from_date_time);
    let mut to_date: Signal<Option<DateTime<Utc>>> = use_signal(|| initial_event.to_date_time);
    let mut location = use_signal(|| initial_event.location.clone().unwrap_or_default());
    let mut is_all_day = use_signal(|| initial_event.is_all_day);
    let mut recurrence: Signal<Option<Recurrent>> = use_signal(|| initial_event.recurrence.clone());

    let mut show_recurrent_scope_dialog = use_signal(|| false);
    let mut recurrent_scope: Signal<Option<RecurrentEditScope>> = use_signal(|| None);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);
    let mut is_loading = use_signal(|| false);

    let is_edit = matches!(mode, EventFormMode::Edit(_));
    let is_recurrent = initial_event.recurrence.is_some();
    let is_recurrence_exception = initial_event.recurrence_exception.is_some();

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
                    if is_edit { "edit event" } else { "New Event" }
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
                        oninput: move |e| summary.set(e.value()),
                    }
                }

                FormField {
                    label: "Calender",
                    required: true,
                    select {
                        class: field_input_class(),
                        onchange: move |e| {
                            if let Ok(id) = Uuid::parse_str(&e.value()) {
                                selected_calendar_id.set(id);
                            }
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
                    label { class: "text-sm text-white/70", "all-day" }
                }

                FormField {
                    label: "from",
                    required: true,
                    input {
                        class: field_input_class(),
                        r#type: if is_all_day() { "date" } else { "datetime-local" },
                        // TODO: Bind value to `from_date` signal and parse on change
                    }
                }

                if !is_all_day() {
                    FormField {
                        label: "till (optional)",
                        input {
                            class: field_input_class(),
                            r#type: "datetime-local",
                            // TODO: Bind value to `to_date` signal and parse on change
                        }
                    }
                }

                FormField {
                    label: "Location (optional)",
                    input {
                        class: field_input_class(),
                        placeholder: "Location or Link",
                        value: "{location}",
                        oninput: move |e| location.set(e.value()),
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
                        oninput: move |e| description.set(e.value()),
                    }
                }

                RecurrencePicker { recurrence }

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
pub fn RecurrencePicker(recurrence: Signal<Option<Recurrent>>) -> Element {
    let is_active = use_memo(move || recurrence().is_some());

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
                            recurrence.set(Some(Recurrent {
                                rrule: Rrule::Daily,
                                recurrence_until: Utc::now() + chrono::Duration::days(30),
                            }));
                        }
                    },
                }
                label { class: "text-sm text-white/70", "Recurrance" }
            }

            if is_active() {
                div {
                    class: "flex flex-col gap-3 pl-7",

                    FormField {
                        label: "Frequency",
                        select {
                            class: field_input_class(),
                            onchange: move |e| {
                                // TODO: Parse Rrule variant from string and update recurrence signal
                            },
                            option { value: "Daily", "Daily" }
                            option { value: "Weekly", "Weekly" }
                            option { value: "Fortnight", "Fortnight" }
                            option { value: "OnWeekDays", "OnWeekDays" }
                            option { value: "MonthlyOnDate", "MonthlyOnDate"}
                            option { value: "MonthlyOnWeekday", "MonthlyOnWeekday"}
                            option { value: "Annual", "Annual" }
                        }
                    }

                    FormField {
                        label: "Repeat until",
                        input {
                            class: field_input_class(),
                            r#type: "date",
                            // TODO: Bind to recurrence_until and update recurrence signal on change
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

