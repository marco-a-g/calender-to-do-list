use crate::todos::frontend::filter_todos::{GroupFilter, ListFilter};
use crate::utils::structs::{GroupLight, ProfileLight, TodoEventLight, TodoListLight};
use chrono::{DateTime, Duration, Local, NaiveDate};
use dioxus::prelude::*;

#[component]
pub fn OpenToDoView(
    todos_list: Vec<TodoEventLight>,
    all_lists: Vec<TodoListLight>,
    groups: Vec<GroupLight>,
    all_profiles: Vec<ProfileLight>,
    selected_category: GroupFilter,
    selected_list_filter: ListFilter,
    on_complete: EventHandler<String>,
) -> Element {
    //Tasks filtern
    let filtered_tasks: Vec<TodoEventLight> = todos_list
        .iter()
        .filter(|task| {
            // GPT-Ergänzung: Falls du Tasks ohne Liste anzeigen willst, müsstest du hier "|| task.todo_list_id.is_none()" erlauben und unten im ToDoItem den Fall parent_list = None behandeln.
            if let Some(parent_list) = all_lists.iter().find(|l| l.id == task.todo_list_id) {
                let category_match = match &selected_category {
                    GroupFilter::All => true,
                    GroupFilter::Personal => parent_list.list_type == "private",
                    GroupFilter::Group(g_id) => parent_list.group_id.as_deref() == Some(g_id),
                };
                let list_match = match &selected_list_filter {
                    ListFilter::AllInContext => true,
                    ListFilter::SpecificList(l_id) => &parent_list.id == l_id,
                };
                return category_match && list_match;
            }
            false
        })
        .cloned()
        .collect();

    let (today_list, week_list, later_list) = categorize_todos(&filtered_tasks);

    //Titel
    let title = match &selected_list_filter {
        ListFilter::SpecificList(id) => all_lists
            .iter()
            .find(|l| &l.id == id)
            .map(|l| l.name.clone())
            .unwrap_or("Unknown List".to_string()),
        ListFilter::AllInContext => match &selected_category {
            GroupFilter::All => "All To-Do's".to_string(),
            GroupFilter::Personal => "Personal To-Do's".to_string(),
            GroupFilter::Group(_) => "Group To-Do's".to_string(),
        },
    };

    rsx! {
        div {
            style: "flex: 1; padding: 24px; display: flex; flex-direction: column; background: #080910;",
            div {
                style: "background: linear-gradient(145deg, #1f222c 0%, #14161f 100%); border-radius: 18px; padding: 24px; box-shadow: 0 18px 40px rgba(0,0,0,0.85); border: 1px solid rgba(255,255,255,0.06); flex: 1; display: flex; flex-direction: column; gap: 16px; overflow: hidden;",

                // Header
                div {
                    style: "border-bottom: 1px solid rgba(255,255,255,0.06); padding-bottom: 16px; margin-bottom: 8px;",
                    h2 { style: "margin: 0 0 4px 0; font-size: 13px; letter-spacing: 0.08em; text-transform: uppercase; color: #9ca3af;", "Tasks" }
                    h1 { style: "margin: 0; font-size: 24px; font-weight: 600; color: #f9fafb;", "{title}" }
                }

                // Liste
                div { class: "flex-1 overflow-y-auto pr-2 flex flex-col gap-3",

                    // Due Today oder overdue
                    div { style: "font-size: 12px; color: #9ca3af; font-weight: 600; margin-top: 8px; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.05em;", "Due Today / Overdue" }
                    if today_list.is_empty() { div { style: "font-size: 13px; color: #4b5563; padding: 8px 0;", "No Tasks." } }
                    for task in today_list {
                        ToDoItem {
                            key: "{task.id}",
                            task: task.clone(),
                            parent_list: all_lists.iter().find(|l| l.id == task.todo_list_id).cloned(),
                            parent_group: all_lists.iter().find(|l| l.id == task.todo_list_id).and_then(|l| l.group_id.as_ref()).and_then(|gid| groups.iter().find(|g| &g.id == gid).cloned()),
                            all_profiles: all_profiles.clone(),
                            on_complete: move |id| on_complete.call(id)
                        }
                    }

                    //  Due in den nächsten 7 Tagen
                    div { style: "font-size: 12px; color: #9ca3af; font-weight: 600; margin-top: 24px; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.05em;", "Due in the next 7 days" }
                    if week_list.is_empty() { div { style: "font-size: 13px; color: #4b5563; padding: 8px 0;", "No Tasks." } }
                    for task in week_list {
                        ToDoItem {
                            key: "{task.id}",
                            task: task.clone(),
                            parent_list: all_lists.iter().find(|l| l.id == task.todo_list_id).cloned(),
                            parent_group: all_lists.iter().find(|l| l.id == task.todo_list_id).and_then(|l| l.group_id.as_ref()).and_then(|gid| groups.iter().find(|g| &g.id == gid).cloned()),
                            all_profiles: all_profiles.clone(),
                            on_complete: move |id| on_complete.call(id)
                        }
                    }

                    // Due mehr als 1 Woche in Zukunft
                    div { style: "font-size: 12px; color: #9ca3af; font-weight: 600; margin-top: 24px; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.05em;", "Due Later" }
                    if later_list.is_empty() { div { style: "font-size: 13px; color: #4b5563; padding: 8px 0;", "No Tasks." } }
                    for task in later_list {
                        ToDoItem {
                            key: "{task.id}",
                            task: task.clone(),
                            parent_list: all_lists.iter().find(|l| l.id == task.todo_list_id).cloned(),
                            parent_group: all_lists.iter().find(|l| l.id == task.todo_list_id).and_then(|l| l.group_id.as_ref()).and_then(|gid| groups.iter().find(|g| &g.id == gid).cloned()),
                            all_profiles: all_profiles.clone(),
                            on_complete: move |id| on_complete.call(id)
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ToDoItem(
    task: TodoEventLight,
    parent_list: Option<TodoListLight>,
    parent_group: Option<GroupLight>,
    all_profiles: Vec<ProfileLight>,
    on_complete: EventHandler<String>,
) -> Element {
    let due_str_raw = task.due_datetime.clone().unwrap_or_default();
    let now = Local::now().date_naive();

    let (date_color, font_weight, display_date) = if due_str_raw.is_empty() {
        ("#6b7280", "400", String::new())
    } else {
        // Parse als DateTime mit Zeitzone (RFC3339?)
        if let Ok(dt_utc) = DateTime::parse_from_rfc3339(&due_str_raw) {
            let dt_local = dt_utc.with_timezone(&Local);
            let parsed_date = dt_local.date_naive();
            let german_format = dt_local.format("%d.%m.%Y").to_string();
            if parsed_date <= now {
                ("#ef4444", "600", german_format) // Rot, fett
            } else {
                ("#6b7280", "400", german_format) // Grau, normal
            }
        } else {
            // Wenn Parsing fehlschlägt, zeige String
            ("#6b7280", "400", due_str_raw)
        }
    };

    // Tags für Todos
    let (context_badge_label, is_event) = if let Some(list) = &parent_list {
        let is_evt = list.attached_to_calendar_event.is_some();
        let label = if list.list_type == "private" {
            format!("Personal: {}", list.name)
        } else {
            let group_name = parent_group
                .as_ref()
                .map(|g| g.name.clone())
                .unwrap_or("Group".to_string());
            format!("{}: {}", group_name, list.name)
        };
        (Some(label), is_evt)
    } else {
        (None, false)
    };

    // Zugeiwesener User
    let (assignee_label, assignee_color) = if let Some(user_id) = &task.assigned_to_user {
        if let Some(user) = all_profiles.iter().find(|p| &p.id == user_id) {
            (user.username.clone(), "#e5e7eb")
        } else {
            ("Unassigned".to_string(), "#9ca3af")
        }
    } else {
        ("No one".to_string(), "#4b5563")
    };

    rsx! {
        div {
            style: "background: #181b24; border-radius: 14px; border: 1px solid rgba(255,255,255,0.06); box-shadow: 0 4px 12px rgba(0,0,0,0.2); padding: 16px; display: flex; align-items: center; gap: 14px; transition: border-color 0.2s;",

            // Checkbox
            div {
                onclick: move |_| on_complete.call(task.id.clone()),
                style: "width: 20px; height: 20px; border-radius: 50%; border: 2px solid #4b5563; cursor: pointer; flex-shrink: 0; transition: border-color 0.2s;",
                class: "hover:border-blue-500"
            }

            // Tags und Titel von Todo
            div { style: "flex: 1; display: flex; flex-direction: column; gap: 4px;",
                div { style: "color: #f3f4f6; font-weight: 500; font-size: 15px;", "{task.summary}" }

                div { style: "display: flex; align-items: center; gap: 8px;",
                    if !display_date.is_empty() {
                        span { style: "font-size: 12px; color: {date_color}; font-weight: {font_weight};", "Due: {display_date}" }
                    }

                    if let Some(label) = context_badge_label {
                         span { style: "font-size: 10px; background: rgba(58, 107, 255, 0.15); color: #3A6BFF; padding: 2px 6px; border-radius: 4px; font-weight: 600; text-transform: uppercase;", "{label}" }
                    }

                    if is_event {
                         span { style: "font-size: 10px; background: rgba(255, 255, 255, 0.1); color: #9ca3af; padding: 2px 6px; border-radius: 4px; font-weight: 600; text-transform: uppercase;", "Event" }
                    }
                }
            }

            // Assigned to
            div { style: "text-align: right; display: flex; flex-direction: column; align-items: flex-end; min-width: 80px;",
                span { style: "font-size: 10px; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em;", "Assigned to:" }
                span { style: "font-size: 13px; color: {assignee_color}; font-weight: 500;", "{assignee_label}" }
            }
        }
    }
}

// Todos kategorisieren
fn categorize_todos(
    list: &Vec<TodoEventLight>,
) -> (
    Vec<TodoEventLight>,
    Vec<TodoEventLight>,
    Vec<TodoEventLight>,
) {
    let now_date = Local::now().date_naive();
    let next_week_date = now_date + Duration::days(7);

    let mut today = vec![];
    let mut week = vec![];
    let mut later = vec![];

    for item in list {
        let owned_item = item.clone();

        if let Some(due_str) = &item.due_datetime {
            if due_str.is_empty() {
                later.push(owned_item);
                continue;
            }

            match DateTime::parse_from_rfc3339(&due_str) {
                Ok(dt_utc) => {
                    let item_date = dt_utc.with_timezone(&Local).date_naive();

                    if item_date <= now_date {
                        today.push(owned_item);
                    } else if item_date <= next_week_date {
                        week.push(owned_item);
                    } else {
                        later.push(owned_item);
                    }
                }
                Err(_) => {
                    //Fehler beim Parsen -> in later rein
                    later.push(owned_item);
                }
            }
        }
    }
    (today, week, later)
}
