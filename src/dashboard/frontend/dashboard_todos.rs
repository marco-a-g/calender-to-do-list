use crate::utils::date_handling::db_to_display_only_date;
use dioxus::prelude::*;

/// UI-Element that renders a dashboard widget summarizing the user's upcoming tasks for the week.
///
/// Displays a read-only list of to-do items. If there are no ToDos assigned to the user and due this week it renders a placeholder message.
/// Todos are rendered with due date and group badge.
///
/// ## Arguments
///
/// * `todos` - A vector of tuples containing the metadata for each task: `(task_name, optional_due_date, group_name, group_color)`.
#[component]
pub fn DashboardTodos(
    //Nimmt Tupel aus Todo Metadaten entgegen, hier braucht es keine Structs selber, da keine Interaktion im Dashboard
    todos: Vec<(String, Option<String>, String, String)>,
) -> Element {
    rsx! {
        div {
            style: "padding: 20px; height: 100%; display: flex; flex-direction: column;",
            div {
                style: "margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;",
                h3 { style: "margin: 0; color: white; font-size: 16px; font-weight: 600;", "My Tasks this week" }
                span {
                    style: "background: rgba(255,255,255,0.1); color: #9ca3af; padding: 2px 8px; border-radius: 6px; font-size: 12px;",
                    "{todos.len()}"
                }
            }

            //TodoListe
            div {
                style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; padding-right: 4px;",
                //wenn keine todos bekommen, zeige exttra ansicht an
                if todos.is_empty() {
                    div {
                        style: "height: 100%; display: flex; align-items: center; justify-content: center; color: #6b7280; font-size: 14px;",
                        "Nothing to do! 🎉"
                    }
                }

                //über  Tupel iterieren (mit enumerate Zähler i für Ansicht oben rechts) und für jedes Rendern
                {todos.iter().enumerate().map(|(i, (name, due_date, group_name, group_color))| {
                    let date_str = db_to_display_only_date(due_date).unwrap_or_default();
                    rsx! {
                        div {
                            key: "{i}",
                            style: "background: rgba(255,255,255,0.03); padding: 10px 12px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.05); display: flex; align-items: center; justify-content: space-between; transition: background 0.2s;",
                            class: "hover:bg-white/5",

                            // Name und Due Date
                            div { style: "display: flex; flex-direction: column; gap: 4px; overflow: hidden;",
                                span {
                                    style: "color: white; font-size: 14px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                    "{name}"
                                }
                                //Falls dateformatting nicht fehlschlägt oder leer ist zeigen wir das due date an
                                if !date_str.is_empty() {
                                    div { style: "display: flex; align-items: center; gap: 4px; color: #9ca3af; font-size: 11px;",
                                        span { "🕒" }
                                        "{date_str}"
                                    }
                                }
                            }

                            // Gruppen Badge
                            div {
                                style: format!(
                                    "font-size: 10px; font-weight: 700; text-transform: uppercase; padding: 4px 8px; border-radius: 4px; color: {}; background: color-mix(in srgb, {}, transparent 85%); white-space: nowrap; margin-left: 12px;",
                                    group_color, group_color
                                ),
                                "{group_name}"
                            }
                        }
                    }
                })}
            }
        }
    }
}
