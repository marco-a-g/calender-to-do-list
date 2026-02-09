use crate::utils::date_handling::db_to_display_only_date;
use crate::utils::structs::{GroupLight, TodoEventLight, TodoListLight};
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
    let datetime_formatted = db_to_display_only_date(&task.due_datetime).unwrap_or_default();

    //Label für Liste und Gruppe als Tupel extrahieren
    let (group_label, list_label) = if let Some(list) = &parent_list {
        //wenn private Liste = Personal Label
        let group_text = if list.list_type == "private" {
            "Personal".to_string()
        } else {
            let g_name = parent_group
                .as_ref()
                .map(|g| g.name.clone())
                .unwrap_or("Group".to_string());
            format!("{}", g_name)
        };
        let list_text = format!("{}", list.name);
        (Some(group_text), Some(list_text))
    } else {
        (None, None) //Sollte Nicht auch Gruppe ohne liste möglich sein? In JF fragen
    };

    let group_badge_color = parent_group
        .as_ref()
        .map(|g| g.color.clone())
        .unwrap_or_else(|| "#9ca3af".to_string());

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
                    //Color des Labels nun an Gruppencolor angepasst
                    if let Some(label_group) = group_label {
                        span {
                            style: format!("font-size: 10px; background: color-mix(in srgb, {}, transparent 85%); color: {}; padding: 2px 6px; border-radius: 4px; font-weight: 600; text-transform: uppercase;",
                                    group_badge_color,
                                    group_badge_color),
                                     "{label_group}"}
                        }
                    if let Some(label) = list_label {
                        if uuid::Uuid::parse_str(&label).is_err() {
                            span {
                                style: "font-size: 10px; background: rgba(255, 255, 255, 0.1); color: #9ca3af; padding: 2px 6px; border-radius: 4px; font-weight: 600; text-transform: uppercase;",
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
