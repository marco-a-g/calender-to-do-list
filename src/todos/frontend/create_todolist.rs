use crate::todos::backend::create_todo_list;
use crate::utils::structs::{
    CalendarEventLight, CalendarLight, GroupLight, GroupMemberLight, ProfileLight, TodoListLight,
};
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
    //zunächst alle Felder auf leeren String setzen
    let mut new_list_title = use_signal(|| String::new());
    let mut new_list_group_id = use_signal(|| String::new()); //Keine Gruppen id -> privat -> user id braucht man nicht
    let mut new_list_event_id = use_signal(|| String::new());
    let mut new_list_due_date = use_signal(|| String::new());
    //description
    //priority
    //rrule zeugs, wie bei todoliste?

    let close_modal = move |_| {
        show_modal.set(false);
        new_list_title.set(String::new());
        new_list_group_id.set(String::new());
        new_list_event_id.set(String::new());
        new_list_due_date.set(String::new());
    };

    let handle_create = move |_| async move {
        if !new_list_title().is_empty() {
            let gid_opt = if new_list_group_id().is_empty() {
                None
            } else {
                Some(new_list_group_id())
            };
            let event_id_opt = if new_list_event_id().is_empty() {
                None
            } else {
                Some(new_list_event_id())
            };

            let raw_date = new_list_due_date();
            let due_date_opt = if raw_date.is_empty() {
                None
            } else {
                Some(raw_date)
            };

            let list_type = if gid_opt.is_some() {
                "group".to_string()
            } else {
                "private".to_string()
            };

            let new_list = TodoListLight {
                id: Uuid::new_v4().to_string(),
                name: new_list_title(),
                list_type: list_type,
                owner_id: Some("local-user".to_string()),
                group_id: gid_opt,
                attached_to_calendar_event: event_id_opt,
                description: None,
                due_datetime: due_date_opt,
                priority: Some("normal".to_string()),
                attachment: None,
                rrule: None,
                recurrence_id: None,
                recurrence_until: None,
                created_by: "local-user".to_string(),
                created_at: Local::now().to_rfc3339(),
                last_mod: Local::now().to_rfc3339(),
                overrides_datetime: None,
                skipped: false,
            };

            let _ = create_todo_list(new_list).await; //Hier kommt dann insert in remote DB mit new_list?

            new_list_title.set(String::new());
            new_list_group_id.set(String::new());
            new_list_event_id.set(String::new());
            new_list_due_date.set(String::new());

            show_modal.set(false);
            on_refresh.call(());
        }
    };

    // Kalenderevents Filtern
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
                style: "background: #171923; width: 400px; padding: 24px; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 20px 50px rgba(0,0,0,0.9); display: flex; flex-direction: column; gap: 16px;",

                h2 { style: "color: white; font-size: 18px; margin: 0 0 8px 0;",
                    "Create New To-Do List"
                }

                // Name
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "List Name" }
                    input {
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none;",
                        value: "{new_list_title}",
                        oninput: move |evt| new_list_title.set(evt.value()),
                        placeholder: "e.g. Shopping, Project X"
                    }
                }

                // Datum
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Due Date (Optional)" }
                    input {
                        r#type: "date",
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; color-scheme: dark;",
                        value: "{new_list_due_date}",
                        oninput: move |evt| new_list_due_date.set(evt.value())
                    }
                }

                // Gruppe
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Assign to Group" }
                    select {
                        style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer;",
                        onchange: move |evt| {
                            new_list_group_id.set(evt.value());
                            new_list_event_id.set(String::new());
                        },
                        option { value: "", "Personal (No Group)" }
                        for g in groups.clone() {
                            option { value: "{g.id}", "{g.name}" }
                        }
                    }
                }

                // Event
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Attach to Calendar Event (Optional)" }
                    select {
                        disabled: filtered_events.is_empty(),
                        style: format!("background: {}; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: {}; opacity: {};",
                            if filtered_events.is_empty() { "#1a1a1a" } else { "#171923" },
                            if filtered_events.is_empty() { "not-allowed" } else { "pointer" },
                            if filtered_events.is_empty() { "0.5" } else { "1" }
                        ),
                        onchange: move |evt| new_list_event_id.set(evt.value()),

                        option { value: "", "Don't assign to event" }

                        for evt in filtered_events.clone() {
                            option { value: "{evt.id}", "{evt.summary}" }
                        }
                    }
                }

                // Buttons
                div {
                    style: "display: flex; gap: 10px; margin-top: 10px;",
                    button {
                        style: "flex: 1; padding: 10px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); color: #9ca3af; background: transparent; cursor: pointer;",
                        onclick: close_modal,
                        "Cancel"
                    }
                    button {
                        style: "flex: 1; padding: 10px; border-radius: 8px; background: #3A6BFF; color: white; border: none; font-weight: 600; cursor: pointer;",
                        onclick: handle_create,
                        "Create List"
                    }
                }
            }
        }
    }
}
