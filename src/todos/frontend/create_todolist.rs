use crate::todos::backend::create_todolist::create_todo_list;
use crate::utils::structs::{CalendarEventLight, CalendarLight, GroupLight, TodoListLight};
use chrono::Local;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn CreateListButton(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            style:
            "background: linear-gradient(180deg, #3A6BFF 0%, #244BC5 100%);
            border: 1px solid rgba(255,255,255,0.1);
            border-radius: 10px; 
            padding: 14px;
            color: white; 
            font-weight: 600; 
            cursor: pointer;
            box-shadow: 0 4px 12px rgba(58, 107, 255, 0.3);
            transition: transform 0.1s;
            display: flex;
            justify-content: center;
            align-items: center; gap: 8px;",
            onclick: move |evt| onclick.call(evt),
            span { style: "font-size: 18px; line-height: 1;", "+" }
            "Create New List"
        }
    }
}

#[component]
pub fn CreateListModal(
    groups: Vec<GroupLight>,
    all_events: Vec<CalendarEventLight>,
    all_calendars: Vec<CalendarLight>,
    show_modal: Signal<bool>,
    on_refresh: EventHandler<()>,
) -> Element {
    //Standardwerte für neue ToDoListe setzen
    let mut new_list_title = use_signal(|| String::new());
    let mut new_list_description = use_signal(|| String::new());
    let mut new_list_group_id = use_signal(|| String::new());
    let mut new_list_event_id = use_signal(|| String::new());
    let mut new_list_due_date = use_signal(|| String::new());
    let mut new_list_priority = use_signal(|| "normal".to_string());
    let mut new_list_rrule = use_signal(|| String::new());
    let mut new_list_recurrence_until = use_signal(|| String::new());

    //Bei schließen der Maske wieder Werte auf Standard setzen
    let close_modal = move |_| {
        show_modal.set(false);
        new_list_title.set(String::new());
        new_list_description.set(String::new());
        new_list_group_id.set(String::new());
        new_list_event_id.set(String::new());
        new_list_due_date.set(String::new());
        new_list_priority.set("normal".to_string());
        new_list_rrule.set(String::new());
        new_list_recurrence_until.set(String::new());
    };

    //Erstellte Liste handhaben
    let handle_create = move |_| async move {
        if !new_list_title().is_empty() {
            //Zugewiesene Gruppe
            let gid = if new_list_group_id().is_empty() {
                None
            } else {
                Some(new_list_group_id())
            };
            //Zugewiesenes Evebt
            let event_id_opt = if new_list_event_id().is_empty() {
                None
            } else {
                Some(new_list_event_id())
            };
            //Due-Date
            let due_date = if new_list_due_date().is_empty() {
                None
            } else {
                Some(new_list_due_date())
            };
            //Beschreibung
            let description = if new_list_description().is_empty() {
                None
            } else {
                Some(new_list_description())
            };
            //Recurence Rule
            let rrule_opt = if new_list_rrule().is_empty() {
                None
            } else {
                Some(new_list_rrule())
            };
            //Recurrence bis
            let recurrence_until_opt =
                if new_list_recurrence_until().is_empty() || new_list_rrule().is_empty() {
                    None
                } else {
                    Some(new_list_recurrence_until())
                };

            //Ist Liste Privat oder Gruppe
            let list_type = if gid.is_some() {
                "group".to_string()
            } else {
                "private".to_string()
            };

            //Neue Listen Felder zusammenfügen
            let new_list = TodoListLight {
                id: Uuid::new_v4().to_string(), //bzw leer? Supabase handhaben lassen
                name: new_list_title(),
                list_type: list_type,
                owner_id: Some("local-user(hier noch uid rausfischen)".to_string()), //bzw leer? Supabase handhaben lassen
                group_id: gid,
                attached_to_calendar_event: event_id_opt,
                description: description,
                due_datetime: due_date,
                priority: Some(new_list_priority()),
                attachment: None,
                rrule: rrule_opt,
                recurrence_until: recurrence_until_opt,
                recurrence_id: None,
                created_by: "local-user(hier noch uid rausfischen)".to_string(), //bzw leer? Supabase handhaben lassen
                created_at: Local::now().to_rfc3339(), //bzw leer? Supabase handhaben lassen
                last_mod: Local::now().to_rfc3339(),   //bzw leer? Supabase handhaben lassen
                skipped: false,
                overrides_datetime: None,
            };

            let _ = create_todo_list(new_list).await; //Backend Funktion in Remote DB

            //Bei Kreieren einer Liste wieder Werte auf Standard setzen
            new_list_title.set(String::new());
            new_list_description.set(String::new());
            new_list_group_id.set(String::new());
            new_list_event_id.set(String::new());
            new_list_due_date.set(String::new());
            new_list_priority.set("normal".to_string());
            new_list_rrule.set(String::new());
            new_list_recurrence_until.set(String::new());

            show_modal.set(false);
            on_refresh.call(());
        }
    };

    //Listen für Dropdown bei Gruppenzuweisung filtern
    let filtered_events: Vec<CalendarEventLight> = all_events
        .iter()
        .filter(|evt| {
            if let Some(calendar) = all_calendars.iter().find(|c| c.id == evt.calendar_id) {
                if new_list_group_id().is_empty() {
                    calendar.calendar_type == "private" || calendar.group_id.is_none()
                } else {
                    calendar.group_id.as_deref() == Some(new_list_group_id().as_str())
                }
            } else {
                false
            }
        })
        .cloned()
        .collect();

    rsx! {
        div {
            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); backdrop-filter: blur(5px); z-index: 60; display: flex; align-items: center; justify-content: center;",
            div {
                style: "background: #171923; width: 450px; padding: 24px; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 20px 50px rgba(0,0,0,0.9); display: flex; flex-direction: column; gap: 16px; max-height: 90vh; overflow-y: auto;",

                h2 { style: "color: white; font-size: 18px; margin: 0 0 8px 0;",
                    "Create New To-Do List"
                }

                // Name setzen
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "List Name" }
                    input {
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none;",
                        value: "{new_list_title}",
                        oninput: move |evt| new_list_title.set(evt.value()),
                        placeholder: "e.g. Project X, Sprint 1"
                    }
                }

                // Description schreiben
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Description (Optional)" }
                    textarea {
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; min-height: 60px; resize: vertical; font-family: sans-serif;",
                        value: "{new_list_description}",
                        oninput: move |evt| new_list_description.set(evt.value()),
                        placeholder: "Add details here..."
                    }
                }

                // Due Date auswählen
                div { style: "display: flex; gap: 10px;",
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Due Date (Optional)" }
                        input {
                            r#type: "date",
                            style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; color-scheme: dark; width: 100%;",
                            value: "{new_list_due_date}",
                            oninput: move |evt| new_list_due_date.set(evt.value())
                        }
                    }
                    //Prio auswählen
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Priority" }
                        select {
                            style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer; width: 100%;",
                            value: "{new_list_priority}",
                            onchange: move |evt| new_list_priority.set(evt.value()),
                            option { value: "low", "Low" }
                            option { value: "normal", "Normal" }
                            option { value: "high", "High" }
                            option { value: "top", "Top" }
                        }
                    }
                }

                // RRule auswählen
                div { style: "display: flex; gap: 10px;",
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Recurrence" }
                        select {
                            style: format!("background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; outline: none; cursor: pointer; width: 100%; color: {};",
                                if new_list_rrule().is_empty() { "#9ca3af" } else { "white" }),
                            value: "{new_list_rrule}",
                            onchange: move |evt| new_list_rrule.set(evt.value()),
                            option { value: "", style: "color: #9ca3af;", "Not recurring" }
                            option { value: "daily", "Daily" }
                            option { value: "weekly", "Weekly" }
                            option { value: "fortnight", "Fortnight (2 Weeks)" }
                            option { value: "onweekdays", "On Weekdays (Mon-Fri)" }
                            option { value: "monthly", "Monthly" }
                            option { value: "annual", "Annual" }
                        }
                    }
                    //Recurrence until auswählen
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Repeat Until" }
                        input {
                            r#type: "date",
                            disabled: new_list_rrule().is_empty(),
                            style: format!("background: {}; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; color-scheme: dark; width: 100%; opacity: {}; cursor: {};",
                                if new_list_rrule().is_empty() { "#1a1a1a" } else { "rgba(255,255,255,0.05)" },
                                if new_list_rrule().is_empty() { "0.3" } else { "1" },
                                if new_list_rrule().is_empty() { "not-allowed" } else { "pointer" }
                            ),
                            value: "{new_list_recurrence_until}",
                            oninput: move |evt| new_list_recurrence_until.set(evt.value())
                        }
                    }
                }

                // Gruppen zur zuweisung auflisten
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Assign to Group" }
                    select {
                        style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer;",
                        onchange: move |evt| { new_list_group_id.set(evt.value()); new_list_event_id.set(String::new()); },
                        option { value: "", "Personal (No Group)" }
                        for g in groups.clone() { option { value: "{g.id}", "{g.name}" } }
                    }
                }

                // Events auflisten zum zuordnen
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Attach to Calendar Event (Optional)" }
                    select {
                        disabled: filtered_events.is_empty(),
                        style: format!("background: {}; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: {}; opacity: {};", if filtered_events.is_empty() { "#1a1a1a" } else { "#171923" }, if filtered_events.is_empty() { "not-allowed" } else { "pointer" }, if filtered_events.is_empty() { "0.5" } else { "1" }),
                        onchange: move |evt| new_list_event_id.set(evt.value()),
                        option { value: "", "Don't assign to event" }
                        for evt in filtered_events.clone() { option { value: "{evt.id}", "{evt.summary}" } }
                    }
                }

                // Buttons: Create und Cancel
                div {
                    style: "display: flex; gap: 10px; margin-top: 10px;",
                    button {
                        style: "flex: 1; padding: 10px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); color: #9ca3af; background: transparent; cursor: pointer;",
                        onclick: close_modal,
                        "Cancel"
                    }
                    button {
                        style: "flex: 1; padding: 10px; border-radius: 8px; background: #3A6BFF; color: white; border: none; font-weight: 600; cursor: pointer;",
                        onclick: handle_create, //ruft handle create oben auf
                        "Create List"
                    }
                }
            }
        }
    }
}
