use crate::todos::backend::{ToDo, ToDoTransfer};
use crate::todos::frontend::filter_todos::FilterState;
use chrono::{Local, NaiveDate};
use dioxus::prelude::*;

#[component]
pub fn TaskListView(
    todos_list: Vec<ToDoTransfer>,
    selected_filter: FilterState,
    on_complete: EventHandler<i32>,
) -> Element {
    let (today_list, week_list, later_list) = categorize_todos(&todos_list);

    let title = match selected_filter {
        FilterState::All => "All To-Do's",
        FilterState::Personal => "Personal To-Do's",
        FilterState::Group(_) => "Group To-Do's",
    };

    rsx! {
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
                    h2 { style:
                        "margin: 0 0 4px 0;
                         font-size: 13px; 
                         letter-spacing: 0.08em; 
                         text-transform: uppercase; 
                         color: #9ca3af;", 
                         "To-Do List" }
                    h1 { style:
                        "margin: 0;
                         font-size: 24px; 
                         font-weight: 600; 
                         color: #f9fafb;", 
                         "{title}" }
                }

                div { class: "flex-1 overflow-y-auto pr-2 flex flex-col gap-3",

                    div { style:
                        "font-size: 12px;
                         color: #9ca3af; 
                         font-weight: 600; 
                         margin-top: 8px; 
                         margin-bottom: 4px; 
                         text-transform: uppercase; 
                         letter-spacing: 0.05em;", 
                         "Due Today / Overdue" }
                    if today_list.is_empty() { div { style: "font-size: 13px;
                                                             color: #4b5563; 
                                                             padding: 8px 0;",
                                                              "No To-Do's." } }
                    for item in today_list { ToDoItem { todo: map_transfer_to_todo(item), on_complete: move |id| on_complete.call(id) } }

                    div { style:
                        "font-size: 12px;
                         color: #9ca3af; 
                         font-weight: 600; 
                         margin-top: 24px; 
                         margin-bottom: 4px; 
                         text-transform: uppercase; 
                         letter-spacing: 0.05em;
                         ", "Due in the next 7 days" }
                    if week_list.is_empty() { div { style: "font-size: 13px;
                                                            color: #4b5563; 
                                                            padding: 8px 0;",
                                                            "No To-Do's." } }
                    for item in week_list { ToDoItem { todo: map_transfer_to_todo(item), on_complete: move |id| on_complete.call(id) } }

                    div { style:
                        "font-size: 12px;
                         color: #9ca3af; 
                         font-weight: 600; 
                         margin-top: 24px; 
                         margin-bottom: 4px; 
                         text-transform: uppercase; 
                         letter-spacing: 0.05em;", 
                         "Due Later" }
                    if later_list.is_empty() { div { style: "font-size: 13px;
                                                             color: #4b5563; 
                                                             padding: 8px 0;", 
                                                             "No To-Do's." } }
                    for item in later_list { ToDoItem { todo: map_transfer_to_todo(item), on_complete: move |id| on_complete.call(id) } }
                }
            }
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
                div { style:
                    "color: #f3f4f6;
                     font-weight: 500; 
                     font-size: 15px;", 
                     "{todo.title}" }
                div {
                    style:
                    "display: flex;
                     align-items: center; 
                     gap: 8px; 
                     margin-top: 4px;",
                    span { style: format!("font-size: 12px; color: {}; font-weight: {};", date_color, if date_color == "#ef4444" { "600" } else { "400" }), "Due to: {todo.due_date}" }
                    if let Some(name) = &todo.group_name {
                        if !name.is_empty() {
                            {
                                let color = todo.group_color.as_deref().unwrap_or("#3A6BFF");
                                rsx! { span { style: format!("font-size: 10px;
                                                              background: {}26; 
                                                              color: {}; 
                                                              padding: 2px 6px; 
                                                              border-radius: 4px; 
                                                              font-weight: 600; 
                                                              text-transform: uppercase;",
                                                               color, color), "{name}" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

//Helper Funkionen
fn categorize_todos(
    list: &Vec<ToDoTransfer>,
) -> (Vec<ToDoTransfer>, Vec<ToDoTransfer>, Vec<ToDoTransfer>) {
    let now = Local::now().date_naive();
    let next_week = now + chrono::Duration::days(7);
    let mut today = vec![];
    let mut week = vec![];
    let mut later = vec![];

    for item in list {
        if item.2 == "Heute" {
            today.push(item.clone());
        } else if let Ok(parsed) = NaiveDate::parse_from_str(&item.2, "%d.%m.%Y") {
            if parsed <= now {
                today.push(item.clone());
            } else if parsed <= next_week {
                week.push(item.clone());
            } else {
                later.push(item.clone());
            }
        } else {
            today.push(item.clone());
        }
    }
    (today, week, later)
}
//mapt Tupel ToDoTransfer aus Backend wieder auf ToDo
fn map_transfer_to_todo(item: ToDoTransfer) -> ToDo {
    ToDo {
        id: item.0,
        title: item.1,
        due_date: item.2,
        is_group: item.3,
        completed: item.4,
        group_id: item.5,
        group_name: item.6,
        group_color: item.7,
        completed_date: item.8,
    }
}
