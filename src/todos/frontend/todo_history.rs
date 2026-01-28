use crate::utils::structs::{GroupLight, TodoEventLight, TodoListLight};
use chrono::{DateTime, Local};
use dioxus::prelude::*;

#[component]
pub fn HistoryView(
    history_tasks: Vec<TodoEventLight>,
    all_lists: Vec<TodoListLight>,
    all_groups: Vec<GroupLight>,
) -> Element {
    rsx! {
        div {
            style: "flex: 1; background: linear-gradient(145deg, #1f222c 0%, #14161f 100%); border-radius: 18px; padding: 18px; box-shadow: 0 22px 45px rgba(0,0,0,0.8); border: 1px solid rgba(255,255,255,0.06); display: flex; flex-direction: column; overflow: hidden;",

            h2 { style: "margin: 0 0 12px 0; font-size: 13px; letter-spacing: 0.08em; text-transform: uppercase; color: #9ca3af;",
                 "Completed"
            }

            div { class: "flex-1 overflow-y-auto pr-1 flex flex-col gap-2",
                //über alle completed Tasks itterieren und dazugehörige liste und gruppe holen für Label und rendern
                for task in history_tasks {
                    {
                        let parent_list = all_lists.iter().find(|l| l.id == task.todo_list_id).cloned();
                        //hier über listen itterieren um über liste Gruppe finden -> Todo hat keine eigene Gruppe bis jetzt?
                        let parent_group = all_lists.iter()
                            .find(|l| l.id == task.todo_list_id)
                            .and_then(|l| l.group_id.as_ref())
                            .and_then(|gid| all_groups.iter().find(|g| &g.id == gid))
                            .cloned();

                        rsx! {
                            //Erledigte Tasks rendern mit Tags
                            HistoryItem {
                                key: "{task.id}",
                                task: task.clone(),
                                parent_list: parent_list,
                                parent_group: parent_group
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
    task: TodoEventLight,
    parent_list: Option<TodoListLight>,
    parent_group: Option<GroupLight>,
) -> Element {
    // Datum Formatierung
    let datetime_raw = task.due_datetime.clone().unwrap_or_default();
    let datetime_formatted = if datetime_raw.is_empty() {
        String::new()
    } else {
        if let Ok(dt_utc) = DateTime::parse_from_rfc3339(&datetime_raw) {
            let dt_local = dt_utc.with_timezone(&Local);
            dt_local.format("%d.%m.%Y").to_string()
        } else {
            datetime_raw //wenn nicht umfomratierbar dann raw-date String ausgeben
        }
    };

    //Label für Liste und Gruppe extrahieren als Tupel extrahieren
    let (group_label, list_label) = if let Some(list) = &parent_list {
        let group_text = if list.list_type == "private" {
            "Personal".to_string()
        } else {
            let g_name = parent_group
                .as_ref()
                .map(|g| g.name.clone())
                .unwrap_or("Group".to_string());
            format!("Group: {}", g_name)
        };
        let list_text = format!("List: {}", list.name);
        (Some(group_text), Some(list_text))
    } else {
        (None, None) //Sollte Nicht auch Gruppe ohne liste möglich sein? In JF fragen
    };

    //div für die einzelnen Abgeschlossenen Items
    rsx! {
        div {
            style: "display: flex; align-items: flex-start; gap: 10px; padding: 8px 0; border-bottom: 1px solid rgba(255,255,255,0.03);",
            // Checkbox
            div {
                style: "width: 16px; height: 16px; border-radius: 50%; background: rgba(16, 185, 129, 0.2); border: 1px solid rgba(16, 185, 129, 0.4); color: #10b981; display: flex; align-items: center; justify-content: center; font-size: 10px; flex-shrink: 0; margin-top: 2px;",
                "✓"
            }
            div {
                style: "display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0;",
                // Titel durchgestrichen
                span { style: "font-size: 13px; color: #6b7280; text-decoration: line-through; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{task.summary}"
                }
                // Metadaten
                div {
                    style: "display: flex; align-items: center; gap: 6px; flex-wrap: wrap;",
                    if !datetime_formatted.is_empty() {
                        span { style: "font-size: 10px; color: #4b5563;",
                            "Due: {datetime_formatted}"
                        }
                    }
                    // Zugehöriges Listen- & Gruppen-Badge, nur anzeigen if label vorhanden
                    if let Some(label) = group_label {
                        span {
                            style: "font-size: 9px; background: rgba(58, 107, 255, 0.15); color: #3A6BFF; padding: 1px 5px; border-radius: 3px; font-weight: 500; text-transform: uppercase;",
                            "{label}"
                        }
                    }
                    if let Some(label) = list_label {
                        span {
                            style: "font-size: 9px; background: rgba(58, 107, 255, 0.15); color: #3A6BFF; padding: 1px 5px; border-radius: 3px; font-weight: 500; text-transform: uppercase;",
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
