use crate::todos::backend::delete_todo::delete_todo_event;
use crate::utils::date_formatting::db_to_display_only_date;
use crate::utils::structs::{GroupLight, ProfileLight, TodoEventLight, TodoListLight};
use dioxus::prelude::*;
use std::str::FromStr;
use uuid::Uuid;

#[component]
pub fn ToDoDetailModal(
    selected_todo: Signal<Option<TodoEventLight>>,
    groups: Vec<GroupLight>,
    all_lists: Vec<TodoListLight>,
    all_profiles: Vec<ProfileLight>,
    on_refresh: EventHandler<()>,
    on_edit: EventHandler<TodoEventLight>,
) -> Element {
    // Ausgewähltes ToDo refenzieren
    let current_todo_ref = selected_todo.read();
    let Some(todo) = &*current_todo_ref else {
        return rsx! {};
    }; //kein ausgewähltes ToDo -> nichts rendern

    // Daten aufbereiten
    let title = todo.summary.clone();
    let description = todo.description.clone().unwrap_or_default();

    let list_opt = all_lists.iter().find(|l| l.id == todo.todo_list_id);
    let list_name = list_opt
        .map(|l| l.name.clone())
        .unwrap_or("Unknown List".to_string());

    let group_name = if let Some(list) = list_opt {
        if list.list_type == "private" {
            "Personal (No Group)".to_string()
        } else if let Some(gid) = &list.group_id {
            groups
                .iter()
                .find(|g| &g.id == gid)
                .map(|g| g.name.clone())
                .unwrap_or("Unknown Group".to_string())
        } else {
            "Personal".to_string()
        }
    } else {
        "-".to_string()
    };

    let assignee_name = if let Some(uid) = &todo.assigned_to_user {
        all_profiles
            .iter()
            .find(|p| &p.id == uid)
            .map(|p| p.username.clone())
            .unwrap_or("Unknown User".to_string())
    } else {
        "Unassigned".to_string()
    };

    let due_date_display = match db_to_display_only_date(&todo.due_datetime) {
        Ok(s) if !s.is_empty() => s,
        _ => "-".to_string(),
    };

    // Prio parsen
    let priority = todo.priority.clone().unwrap_or("normal".to_string());
    let priority_display = match priority.to_lowercase().as_str() {
        "low" => "Low",
        "high" => "High",
        "top" => "Top",
        _ => "Normal",
    };

    // Reccurence
    let rrule_raw = todo.rrule.clone().unwrap_or_default();

    let recurrence_text = if rrule_raw.is_empty() {
        "Not recurring".to_string()
    } else {
        let rule_name = match rrule_raw.as_str() {
            "daily" => "Daily",
            "weekly" => "Weekly",
            "fortnight" => "Fortnight (2 Weeks)",
            "onweekdays" => "On Weekdays (Mon-Fri)",
            "monthly" => "Monthly",
            "annual" => "Annual",
            _ => rrule_raw.as_str(),
        };

        let until_formatted = db_to_display_only_date(&todo.recurrence_until).unwrap_or_default();
        if !until_formatted.is_empty() {
            format!("{}, until: {}", rule_name, until_formatted)
        } else {
            rule_name.to_string()
        }
    };

    // Für Handler vorbereiten
    let todo_for_delete = todo.clone();

    let handle_delete = move |_| {
        let todo_to_delete = todo_for_delete.clone();
        spawn(async move {
            if delete_todo_event(todo_to_delete).await.is_ok() {
                selected_todo.set(None);
                on_refresh.call(());
            } else {
                println!("Error on deleting");
            }
        });
    };

    let todo_for_edit = todo.clone();
    /*let handle_edit = move |_| {
        let todo_to_edit = todo_for_edit.clone();
        spawn(async move {
            if delete_todo_event(todo_to_edit).await.is_ok() {
                selected_todo.set(None);
                on_refresh.call(());
            } else {
                println!("Error on editing");
            }
        });
    }; */

    let input_style = "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; width: 100%; display: block; min-height: 42px; display: flex; align-items: center;";
    let label_style = "font-size: 12px; color: #9ca3af; text-transform: uppercase;";

    rsx! {
        div {
            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); backdrop-filter: blur(5px); z-index: 50; display: flex; align-items: center; justify-content: center;",
            onclick: move |_| selected_todo.set(None),

            div {
                style: "background: #171923; width: 450px; padding: 24px; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 20px 50px rgba(0,0,0,0.9); display: flex; flex-direction: column; gap: 16px; max-height: 90vh; overflow-y: auto;",
                onclick: |e| e.stop_propagation(),

                // Header
                div { class: "flex justify-between items-start",
                    h2 { style: "color: white; font-size: 18px; margin: 0;", "To-Do Details" }
                    button {
                        style: "background: transparent; border: none; color: #9ca3af; cursor: pointer; font-size: 18px;",
                        onclick: move |_| selected_todo.set(None),
                        "✕"
                    }
                }

                // Name
                div { class: "flex flex-col gap-2",
                    label { style: "{label_style}", "To-Do Name" }
                    div { style: "{input_style} font-weight: 600;", "{title}" }
                }

                // Beschreibung
                if !description.is_empty() {
                    div { class: "flex flex-col gap-2",
                        label { style: "{label_style}", "Description" }
                        div {
                            style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: #e5e7eb; min-height: 60px; white-space: pre-wrap; font-family: sans-serif;",
                            "{description}"
                        }
                    }
                }

                // Prio und Fälligkeitsdatum
                div { style: "display: flex; gap: 10px;",
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "{label_style}", "Due Date" }
                        div { style: "{input_style}", "{due_date_display}" }
                    }
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "{label_style}", "Priority" }
                        div {
                            style: format!("{}; color: {};", input_style, match priority.to_lowercase().as_str() {
                                "high" | "top" => "#ef4444",
                                "low" => "#3b82f6",
                                _ => "white"
                            }),
                            "{priority_display}"
                        }
                    }
                }

                // Recurrence
                div { class: "flex flex-col gap-2",
                    label { style: "{label_style}", "Recurrence" }
                    div { style: "{input_style}", "{recurrence_text}" }
                }

                // Gruppe
                div { class: "flex flex-col gap-2",
                    label { style: "{label_style}", "Group" }
                    div { style: "{input_style}", "{group_name}" }
                }

                // Liste
                div { class: "flex flex-col gap-2",
                    label { style: "{label_style}", "List" }
                    div { style: "{input_style}", "{list_name}" }
                }

                // Zugewiesener User
                div { class: "flex flex-col gap-2",
                    label { style: "{label_style}", "Assigned To" }
                    div { style: "{input_style}", "{assignee_name}" }
                }

                // Buttons
                div {
                    style: "display: flex; gap: 10px; margin-top: 10px;",

                    button {
                        style: "flex: 1; padding: 10px; border-radius: 8px; border: 1px solid rgba(239, 68, 68, 0.3); color: #fca5a5; background: rgba(239, 68, 68, 0.1); cursor: pointer; font-weight: 600;",
                        onclick: handle_delete,
                        "Delete"
                    }

                    button {
                        style: "flex: 1; padding: 10px; border-radius: 8px; background: #3A6BFF; color: white; border: none; font-weight: 600; cursor: pointer;",
                        onclick: move |_| on_edit.call(todo_for_edit.clone()),
                        "Edit To-Do"
                    }
                }
            }
        }
    }
}
