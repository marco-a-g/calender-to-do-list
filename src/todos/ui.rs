use crate::todos::backend::{
    //ToDo ist noch Moch-Struktur
    ToDo,
    complete_task,
    create_todo,
    fetch_completed_history,
    fetch_groups,
    fetch_todos_filtered,
};
use chrono::*;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
enum FilterState {
    All,
    Personal,
    Group(i32),
}

#[component]
pub fn ToDoView() -> Element {
    let today_date = use_signal(|| Local::now().format("%A, %d.%m.%Y").to_string());

    let mut selected_filter = use_signal(|| FilterState::All);

    let mut show_create_modal = use_signal(|| false);
    let mut new_task_title = use_signal(|| String::new());
    let mut new_task_group_id = use_signal(|| 0);
    let mut new_task_due_date = use_signal(|| String::new());

    let groups = use_resource(fetch_groups);

    let current_groups = match &*groups.read() {
        Some(Ok(list)) => list.clone(),
        _ => vec![],
    };

    let mut todos_resource = use_resource(move || {
        let mode = match selected_filter() {
            FilterState::All => 0,
            FilterState::Personal => -1,
            FilterState::Group(id) => id,
        };
        fetch_todos_filtered(mode)
    });

    let mut history = use_resource(fetch_completed_history);

    let handle_complete = move |id: i32| async move {
        let _ = complete_task(id).await;
        todos_resource.restart();
        history.restart();
    };

    rsx! {
        div {
            style:
            "width: 100%;
             height: 100%;
             background: #05060b;
             display: flex;
             overflow: hidden;
             font-family: sans-serif;
             position: relative;",

            if show_create_modal() {
                div {
                    style:
                    "position: absolute;
                     top: 0; left: 0;
                     width: 100%;
                     height: 100%;
                     background: rgba(0,0,0,0.7);
                     backdrop-filter: blur(5px);
                     z-index: 50;
                     display: flex;
                     align-items: center;
                     justify-content: center;",

                    div {
                        style:
                        "background: #171923;
                         width: 400px;
                         padding: 24px;
                         border-radius: 18px;
                         border: 1px solid rgba(255,255,255,0.1);
                         box-shadow: 0 20px 50px rgba(0,0,0,0.9);
                         display: flex;
                         flex-direction: column;
                         gap: 16px;",

                        h2 {
                            style:
                            "color: white;
                             font-size: 18px;
                             margin: 0 0 8px 0;",
                            "Create New To-Do"
                        }

                        div { class: "flex flex-col gap-2",
                            label {
                                style:
                                "font-size: 12px;
                                 color: #9ca3af;
                                 text-transform: uppercase;",
                                "To-Do Name"
                            }
                            input {
                                style:
                                "background: rgba(255,255,255,0.05);
                                 border: 1px solid rgba(255,255,255,0.1);
                                 padding: 10px;
                                 border-radius: 8px;
                                 color: white;
                                 outline: none;",
                                value: "{new_task_title}",
                                oninput: move |evt| new_task_title.set(evt.value()),
                                placeholder: "e.g. Finish Report"
                            }
                        }

                        div { class: "flex flex-col gap-2",
                            label {
                                style:
                                "font-size: 12px;
                                 color: #9ca3af;
                                 text-transform: uppercase;",
                                "Due Date"
                            }
                            input {
                                r#type: "date",
                                style:
                                "background: rgba(255,255,255,0.05);
                                 border: 1px solid rgba(255,255,255,0.1);
                                 padding: 10px;
                                 border-radius: 8px;
                                 color: white;
                                 outline: none;
                                 color-scheme: dark;",
                                value: "{new_task_due_date}",
                                oninput: move |evt| new_task_due_date.set(evt.value())
                            }
                        }

                        div { class: "flex flex-col gap-2",
                            label {
                                style:
                                "font-size: 12px;
                                 color: #9ca3af;
                                 text-transform: uppercase;",
                                "Assign to Group"
                            }

                            select {
                                style:
                                "background: #171923;
                                 color-scheme: dark;
                                 border: 1px solid rgba(255,255,255,0.1);
                                 padding: 10px;
                                 border-radius: 8px;
                                 color: white;
                                 outline: none;
                                 cursor: pointer;",

                                onchange: move |evt| {
                                    if let Ok(id) = evt.value().parse::<i32>() {
                                        new_task_group_id.set(id);
                                    }
                                },
                                option { value: "0", "Personal (No Group)" }
                                for g in current_groups.clone() {
                                    option { value: "{g.0}", "{g.1}" }
                                }
                            }
                        }

                        div {
                            style:
                            "display: flex;
                             gap: 10px;
                             margin-top: 10px;",

                            button {
                                style:
                                "flex: 1;
                                 padding: 10px;
                                 border-radius: 8px;
                                 border: 1px solid rgba(255,255,255,0.1);
                                 color: #9ca3af;
                                 background: transparent;
                                 cursor: pointer;",
                                onclick: move |_| {
                                    show_create_modal.set(false);
                                    new_task_title.set(String::new());
                                    new_task_due_date.set(String::new());
                                },
                                "Cancel"
                            }
                            button {
                                style:
                                "flex: 1;
                                 padding: 10px;
                                 border-radius: 8px;
                                 background: #3A6BFF;
                                 color: white;
                                 border: none;
                                 font-weight: 600;
                                 cursor: pointer;",
                                onclick: move |_| async move {
                                    if !new_task_title().is_empty() {
                                        let raw_date = new_task_due_date();
                                        let formatted_date = if raw_date.is_empty() {
                                            "Heute".to_string()
                                        } else {
                                            let parts: Vec<&str> = raw_date.split('-').collect();
                                            if parts.len() == 3 {
                                                format!("{}.{}.{}", parts[2], parts[1], parts[0])
                                            } else {
                                                raw_date
                                            }
                                        };

                                        let _ = create_todo(new_task_title(), new_task_group_id(), formatted_date).await;

                                        new_task_title.set(String::new());
                                        new_task_due_date.set(String::new());
                                        new_task_group_id.set(0);
                                        show_create_modal.set(false);
                                        todos_resource.restart();
                                    }
                                },
                                "Create To-Do"
                            }
                        }
                    }
                }
            }

            div {
                style:
                "width: 260px;
                 background: linear-gradient(180deg, #11121b 0%, #05060b 100%);
                 border-right: 1px solid rgba(255,255,255,0.06);
                 display: flex;
                 flex-direction: column;
                 padding: 24px 16px;
                 gap: 20px;",

                h2 {
                    style:
                    "margin: 0 0 8px 12px;
                     font-size: 11px;
                     letter-spacing: 0.12em;
                     text-transform: uppercase;
                     color: #9ca3af;
                     font-weight: 600;",
                    "Filters"
                }

                div { class: "flex flex-col gap-3",
                    FilterButton { label: "All To-Do's".to_string(), active: selected_filter() == FilterState::All, onclick: move |_| selected_filter.set(FilterState::All) }
                    FilterButton { label: "Personal To-Do's".to_string(), active: selected_filter() == FilterState::Personal, onclick: move |_| selected_filter.set(FilterState::Personal) }
                }

                div {
                    style:
                    "height: 1px;
                     background: rgba(255,255,255,0.06);
                     margin: 0 8px;"
                }

                h2 {
                    style:
                    "margin: 8px 0 8px 12px;
                     font-size: 11px;
                     letter-spacing: 0.12em;
                     text-transform: uppercase;
                     color: #9ca3af;
                     font-weight: 600;",
                    "Groups"
                }

                div {
                    class: "flex-1 overflow-y-auto flex flex-col gap-3 pr-2",
                    for g in current_groups {
                        FilterButton { label: g.1, active: selected_filter() == FilterState::Group(g.0), onclick: move |_| selected_filter.set(FilterState::Group(g.0)) }
                    }
                }
            }

            div {
                style:
                "flex: 1;
                 padding: 24px;
                 display: flex;
                 flex-direction: column;
                 background: #080910;",

                div {
                    style:
                    "background: linear-gradient(145deg, #1f222c 0%, #14161f 100%);
                     border-radius: 18px;
                     padding: 24px;
                     box-shadow: 0 18px 40px rgba(0,0,0,0.85);
                     border: 1px solid rgba(255,255,255,0.06);
                     flex: 1;
                     display: flex;
                     flex-direction: column;
                     gap: 16px;
                     overflow: hidden;",

                    div {
                        style:
                        "border-bottom: 1px solid rgba(255,255,255,0.06);
                         padding-bottom: 16px;
                         margin-bottom: 8px;",

                        h2 {
                            style:
                            "margin: 0 0 4px 0;
                             font-size: 13px;
                             letter-spacing: 0.08em;
                             text-transform: uppercase;
                             color: #9ca3af;",
                            "To-Do List"
                        }
                        h1 {
                            style:
                            "margin: 0;
                             font-size: 24px;
                             font-weight: 600;
                             color: #f9fafb;",
                            match selected_filter() {
                                FilterState::All => "All To-Do's",
                                FilterState::Personal => "Personal To-Do's",
                                FilterState::Group(_) => "Group To-Do's",
                            }
                        }
                    }

                    div {
                        class: "flex-1 overflow-y-auto pr-2 flex flex-col gap-3",
                        match &*todos_resource.read() {
                            Some(Ok(list)) => {
                                let now = Local::now().date_naive();
                                let next_week = now + chrono::Duration::days(7);
                                let mut today_list = vec![];
                                let mut week_list = vec![];
                                let mut later_list = vec![];

                                for item in list {
                                    if item.2 == "Heute" {
                                        today_list.push(item);
                                    } else if let Ok(parsed) = NaiveDate::parse_from_str(&item.2, "%d.%m.%Y") {
                                        if parsed <= now {
                                            today_list.push(item);
                                        } else if parsed <= next_week {
                                            week_list.push(item);
                                        } else {
                                            later_list.push(item);
                                        }
                                    } else {
                                        today_list.push(item);
                                    }
                                }

                                rsx! {
                                    div {
                                        style: "font-size: 12px; color: #9ca3af; font-weight: 600; margin-top: 8px; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.05em;",
                                        "Due Today / Overdue"
                                    }
                                    if today_list.is_empty() {
                                        div { style: "font-size: 13px; color: #4b5563; padding: 8px 0;", "No To-Do's." }
                                    }
                                    for item in today_list {
                                        ToDoItem {
                                            todo: ToDo {
                                                id: item.0, title: item.1.clone(), due_date: item.2.clone(),
                                                is_group: item.3, completed: item.4, group_id: item.5,
                                                group_name: item.6.clone(), group_color: item.7.clone(),
                                                completed_date: item.8.clone()
                                            },
                                            on_complete: handle_complete
                                        }
                                    }

                                    div {
                                        style: "font-size: 12px; color: #9ca3af; font-weight: 600; margin-top: 24px; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.05em;",
                                        "Due in the next 7 days"
                                    }
                                    if week_list.is_empty() {
                                        div { style: "font-size: 13px; color: #4b5563; padding: 8px 0;", "No To-Do's." }
                                    }
                                    for item in week_list {
                                        ToDoItem {
                                            todo: ToDo {
                                                id: item.0, title: item.1.clone(), due_date: item.2.clone(),
                                                is_group: item.3, completed: item.4, group_id: item.5,
                                                group_name: item.6.clone(), group_color: item.7.clone(),
                                                completed_date: item.8.clone()
                                            },
                                            on_complete: handle_complete
                                        }
                                    }

                                    div {
                                        style: "font-size: 12px; color: #9ca3af; font-weight: 600; margin-top: 24px; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.05em;",
                                        "Due Later"
                                    }
                                    if later_list.is_empty() {
                                        div { style: "font-size: 13px; color: #4b5563; padding: 8px 0;", "No To-Do's." }
                                    }
                                    for item in later_list {
                                        ToDoItem {
                                            todo: ToDo {
                                                id: item.0, title: item.1.clone(), due_date: item.2.clone(),
                                                is_group: item.3, completed: item.4, group_id: item.5,
                                                group_name: item.6.clone(), group_color: item.7.clone(),
                                                completed_date: item.8.clone()
                                            },
                                            on_complete: handle_complete
                                        }
                                    }
                                }
                            },
                            Some(Err(e)) => rsx! { div { style: "color: #ef4444;", "Error: {e}" } },
                            None => rsx! { div { style: "color: #6b7280;", "Loading..." } }
                        }
                    }
                }
            }

            div {
                style:
                "width: 320px;
                 padding: 24px 24px 24px 0;
                 display: flex;
                 flex-direction: column;
                 gap: 24px;
                 background: #080910;",

                div {
                    style:
                    "background: linear-gradient(145deg, #222531 0%, #171923 100%);
                     border-radius: 18px;
                     padding: 18px;
                     box-shadow: 0 22px 45px rgba(0,0,0,0.8);
                     border: 1px solid rgba(255,255,255,0.06);",

                    h2 {
                        style:
                        "margin: 0 0 4px 0;
                         font-size: 13px;
                         letter-spacing: 0.08em;
                         text-transform: uppercase;
                         color: #9ca3af;",
                        "Today"
                    }

                    h3 {
                        style:
                        "margin: 0;
                         font-size: 20px;
                         font-weight: 600;
                         color: #f9fafb;",
                        "{today_date()}"
                    }
                }

                div {
                    style:
                    "background: linear-gradient(145deg, #222531 0%, #171923 100%);
                     border-radius: 18px;
                     padding: 18px;
                     box-shadow: 0 22px 45px rgba(0,0,0,0.8);
                     border: 1px solid rgba(255,255,255,0.06);
                     display: flex;
                     flex-direction: column;
                     gap: 14px;",

                    h2 {
                        style:
                        "margin: 0;
                         font-size: 13px;
                         letter-spacing: 0.08em;
                         text-transform: uppercase;
                         color: #9ca3af;",
                        "Actions"
                    }

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
                         align-items: center;
                         gap: 8px;",
                        onclick: move |_| show_create_modal.set(true),
                        span { style: "font-size: 18px; line-height: 1;", "+" }
                        "Create New To-Do"
                    }
                }

                div {
                    style:
                    "flex: 1;
                     background: linear-gradient(145deg, #1f222c 0%, #14161f 100%);
                     border-radius: 18px;
                     padding: 18px;
                     box-shadow: 0 22px 45px rgba(0,0,0,0.8);
                     border: 1px solid rgba(255,255,255,0.06);
                     display: flex;
                     flex-direction: column;
                     overflow: hidden;",

                    h2 {
                        style:
                        "margin: 0 0 12px 0;
                         font-size: 13px;
                         letter-spacing: 0.08em;
                         text-transform: uppercase;
                         color: #9ca3af;",
                        "Completed"
                    }

                    div {
                        class: "flex-1 overflow-y-auto pr-1 flex flex-col gap-2",
                        match &*history.read() {
                            Some(Ok(list)) => rsx! {
                                for item in list {
                                    HistoryItem {
                                        title: item.1.clone(),
                                        date: item.8.clone().unwrap_or(item.2.clone()),
                                        group_name: item.6.clone(),
                                        group_color: item.7.clone()
                                    }
                                }
                            },
                            _ => rsx! { "..." }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FilterButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div {
            onclick: move |evt| onclick.call(evt),
            style: format!(
                "position: relative;
                 padding: 12px 16px;
                 border-radius: 12px;
                 cursor: pointer;
                 transition: all 0.2s ease;
                 background: {};
                 border: 1px solid {};
                 box-shadow: {};
                 display: flex;
                 align-items: center;
                 justify-content: space-between;
                 color: {};
                 font-weight: 500;
                 font-size: 14px;",
                if active { "#2b2c33" } else { "transparent" },
                if active { "rgba(255,255,255,0.06)" } else { "transparent" },
                if active { "0 4px 14px rgba(0,0,0,0.4)" } else { "none" },
                if active { "#ffffff" } else { "#9ca3af" }
            ),
            div {
                style: format!(
                    "position: absolute;
                     left: 0; top: 50%;
                     transform: translateY(-50%);
                     width: 3px;
                     height: 20px;
                     border-radius: 0 2px 2px 0;
                     background: #3A6BFF;
                     opacity: {};
                     transition: opacity 0.2s ease;", if active { "1" } else { "0" })
            }
            "{label}"
        }
    }
}

#[component]
fn ToDoItem(todo: ToDo, on_complete: EventHandler<i32>) -> Element {
    let date_color = if todo.due_date == "Heute" {
        "#ef4444"
    } else {
        if let Ok(parsed_due) = NaiveDate::parse_from_str(&todo.due_date, "%d.%m.%Y") {
            if Local::now().date_naive() >= parsed_due {
                "#ef4444"
            } else {
                "#6b7280"
            }
        } else {
            "#6b7280"
        }
    };

    rsx! {
        div {
            style:
            "background: #181b24;
             border-radius: 14px;
             border: 1px solid rgba(255,255,255,0.06);
             box-shadow: 0 4px 12px rgba(0,0,0,0.2);
             padding: 16px;
             display: flex;
             align-items: center;
             gap: 14px;
             transition: border-color 0.2s;",

            div {
                onclick: move |_| on_complete.call(todo.id),
                style:
                "width: 20px;
                 height: 20px;
                 border-radius: 50%;
                 border: 2px solid #4b5563;
                 cursor: pointer;
                 flex-shrink: 0;
                 transition: border-color 0.2s;",
                class: "hover:border-blue-500"
            }

            div { style: "flex: 1;",
                div {
                    style:
                    "color: #f3f4f6;
                     font-weight: 500;
                     font-size: 15px;",
                    "{todo.title}"
                }

                div {
                    style:
                    "display: flex;
                     align-items: center;
                     gap: 8px;
                     margin-top: 4px;",

                    span {
                        style: format!(
                            "font-size: 12px; color: {}; font-weight: {};",
                            date_color,
                            if date_color == "#ef4444" { "600" } else { "400" }
                        ),
                        "Due to: {todo.due_date}"
                    }

                    if let Some(name) = &todo.group_name {
                        if !name.is_empty() {
                            {
                                let color = todo.group_color.as_deref().unwrap_or("#3A6BFF");
                                rsx! {
                                    span {
                                        style: format!(
                                            "font-size: 10px;
                                             background: {}26;
                                             color: {};
                                             padding: 2px 6px;
                                             border-radius: 4px;
                                             font-weight: 600;
                                             text-transform: uppercase;",
                                            color, color
                                        ),
                                        "{name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HistoryItem(
    title: String,
    date: String,
    group_name: Option<String>,
    group_color: Option<String>,
) -> Element {
    rsx! {
        div {
            style:
            "display: flex;
             align-items: flex-start;
             gap: 10px;
             padding: 8px 0;
             border-bottom: 1px solid rgba(255,255,255,0.03);",

            div {
                style:
                "width: 16px;
                 height: 16px;
                 border-radius: 50%;
                 background: rgba(16, 185, 129, 0.2);
                 border: 1px solid rgba(16, 185, 129, 0.4);
                 color: #10b981;
                 display: flex;
                 align-items: center;
                 justify-content: center;
                 font-size: 10px;
                 flex-shrink: 0;
                 margin-top: 2px;",
                "✓"
            }

            div {
                style:
                "display: flex;
                 flex-direction: column;
                 gap: 2px;
                 flex: 1;
                 min-width: 0;",

                span {
                    style:
                    "font-size: 13px;
                     color: #6b7280;
                     text-decoration: line-through;
                     overflow: hidden;
                     text-overflow: ellipsis;
                     white-space: nowrap;",
                    "{title}"
                }

                div {
                    style:
                    "display: flex;
                     align-items: center;
                     gap: 6px;
                     flex-wrap: wrap;",

                    span {
                        style:
                        "font-size: 10px;
                         color: #4b5563;",
                        "completed at: {date}"
                    }

                    if let Some(name) = &group_name {
                        if !name.is_empty() {
                            {
                                let color = group_color.as_deref().unwrap_or("#3A6BFF");
                                rsx! {
                                    span {
                                        style: format!(
                                            "font-size: 9px;
                                             background: {}26;
                                             color: {};
                                             padding: 1px 5px;
                                             border-radius: 3px;
                                             font-weight: 500;
                                             text-transform: uppercase;",
                                            color, color
                                        ),
                                        "{name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
