use crate::todos::backend::create_todo_event;
use crate::utils::structs::{
    GroupLight, GroupMemberLight, ProfileLight, TodoEventLight, TodoListLight,
};
use chrono::Local;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn CreateToDoButton(onclick: EventHandler<MouseEvent>) -> Element {
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
            "Create New To-Do"
        }
    }
}

#[component]
pub fn CreateToDoModal(
    groups: Vec<GroupLight>,
    all_lists: Vec<TodoListLight>,
    all_profiles: Vec<ProfileLight>,
    all_group_members: Vec<GroupMemberLight>,
    show_modal: Signal<bool>,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut new_task_title = use_signal(|| String::new());
    let mut new_task_group_id = use_signal(|| String::new());
    let mut new_task_list_id = use_signal(|| String::new());
    let mut new_task_assignee = use_signal(|| String::new());
    let mut new_task_due_date = use_signal(|| String::new());

    let close_modal = move |_| {
        show_modal.set(false);
        new_task_title.set(String::new());
        new_task_group_id.set(String::new());
        new_task_list_id.set(String::new());
        new_task_assignee.set(String::new());
        new_task_due_date.set(String::new());
    };

    let all_lists_for_handler = all_lists.clone();

    let handle_create = move |_| {
        let all_lists_inner = all_lists_for_handler.clone();

        async move {
            if !new_task_title().is_empty() {
                let raw_date = new_task_due_date();
                let due_date_opt = if raw_date.is_empty() {
                    None
                } else {
                    Some(raw_date)
                };

                let final_list_id = if !new_task_list_id().is_empty() {
                    new_task_list_id()
                } else {
                    let gid = if new_task_group_id().is_empty() {
                        None
                    } else {
                        Some(new_task_group_id())
                    };
                    if let Some(l) = all_lists_inner.iter().find(|l| {
                        if gid.is_none() {
                            l.list_type == "private"
                        } else {
                            l.group_id.as_deref() == gid.as_deref()
                        }
                    }) {
                        l.id.clone()
                    } else {
                        "unknown-list-id".to_string()
                    }
                };

                let assignee_opt = if new_task_assignee().is_empty() {
                    None
                } else {
                    Some(new_task_assignee())
                };

                let new_todo = TodoEventLight {
                    id: Uuid::new_v4().to_string(),
                    todo_list_id: final_list_id,
                    summary: new_task_title(),
                    description: None,
                    completed: false,
                    due_datetime: due_date_opt,
                    priority: Some("normal".to_string()),
                    attachment: None,
                    rrule: None,
                    recurrence_id: None,
                    recurrence_until: None,
                    created_by: "local-user".to_string(),
                    created_at: Local::now().to_rfc3339(),
                    last_mod: Local::now().to_rfc3339(),
                    assigned_to_user: assignee_opt,
                    overrides_datetime: None,
                    skipped: false,
                };

                //Hier kommt dann insert in remote DB
                let _ = create_todo_event(new_todo).await;

                new_task_title.set(String::new());
                new_task_due_date.set(String::new());
                new_task_group_id.set(String::new());
                new_task_list_id.set(String::new());
                new_task_assignee.set(String::new());
                show_modal.set(false);
                on_refresh.call(());
            }
        }
    };

    //Listen filtern
    let filtered_lists: Vec<TodoListLight> = all_lists
        .iter()
        .filter(|l| {
            if new_task_group_id().is_empty() {
                l.list_type == "private"
            } else {
                l.group_id.as_deref() == Some(new_task_group_id().as_str())
            }
        })
        .cloned()
        .collect();

    //User filtern -> nur Mitglieder der Gruppe anzeigen
    let filtered_users: Vec<ProfileLight> = if new_task_group_id().is_empty() {
        vec![]
    } else {
        let current_group_id = new_task_group_id();
        let member_user_ids: Vec<String> = all_group_members
            .iter()
            .filter(|m| m.group_id == current_group_id)
            .map(|m| m.user_id.clone())
            .collect();

        all_profiles
            .iter()
            .filter(|profile| member_user_ids.contains(&profile.id))
            .cloned()
            .collect()
    };

    rsx! {
        div {
            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); backdrop-filter: blur(5px); z-index: 50; display: flex; align-items: center; justify-content: center;",

            div {
                style: "background: #171923; width: 400px; padding: 24px; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 20px 50px rgba(0,0,0,0.9); display: flex; flex-direction: column; gap: 16px;",

                h2 { style: "color: white; font-size: 18px; margin: 0 0 8px 0;", "Create New To-Do" }

                // Name
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "To-Do Name" }
                    input {
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none;",
                        value: "{new_task_title}",
                        oninput: move |evt| new_task_title.set(evt.value()),
                        placeholder: "e.g. Finish Report"
                    }
                }

                // Datum
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Due Date" }
                    input {
                        r#type: "date",
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; color-scheme: dark;",
                        value: "{new_task_due_date}",
                        oninput: move |evt| new_task_due_date.set(evt.value())
                    }
                }

                // Gruppenauswahl
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Assign to Group" }
                    select {
                        style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer;",
                        onchange: move |evt| {
                            new_task_group_id.set(evt.value());
                            new_task_list_id.set(String::new());
                            new_task_assignee.set(String::new());
                        },
                        option { value: "", "Personal (No Group)" }
                        for g in groups.clone() {
                            option { value: "{g.id}", "{g.name}" }
                        }
                    }
                }

                // Listenauswahl
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Select List (Optional)" }
                    select {
                        style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer;",
                        onchange: move |evt| new_task_list_id.set(evt.value()),
                        option { value: "", "Don't assign to specific List" }
                        for list in filtered_lists {
                            option { value: "{list.id}", "{list.name}" }
                        }
                    }
                }

                // User zuweisen
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Assign To User" }
                    select {
                        disabled: new_task_group_id().is_empty(),
                        style: format!("background: {}; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: {}; opacity: {};",
                            if new_task_group_id().is_empty() { "#1a1a1a" } else { "#171923" },
                            if new_task_group_id().is_empty() { "not-allowed" } else { "pointer" },
                            if new_task_group_id().is_empty() { "0.5" } else { "1" }
                        ),
                        onchange: move |evt| new_task_assignee.set(evt.value()),

                        option {
                            value: "",
                            if new_task_group_id().is_empty() { "Personal" } else { "Unassigned" }
                        }

                        for user in filtered_users {
                            option { value: "{user.id}", "{user.username}" }
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
                        "Create To-Do"
                    }
                }
            }
        }
    }
}
