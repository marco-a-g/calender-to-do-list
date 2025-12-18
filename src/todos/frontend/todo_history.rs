use super::create_todo::CreateButton;
use crate::todos::backend::ToDoTransfer;
use dioxus::prelude::*;

#[component]
pub fn HistoryView(
    today_date: String,
    history_data: Vec<ToDoTransfer>,
    on_open_create: EventHandler<()>,
) -> Element {
    rsx! {
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

                h2 { style:
                     "margin: 0 0 4px 0;
                      font-size: 13px; 
                      letter-spacing: 0.08em; 
                      text-transform: uppercase;
                      color: #9ca3af;",
                       "Today" }
                h3 { style:
                    "margin: 0;
                     font-size: 20px; 
                     font-weight: 600; 
                     color: #f9fafb;", 
                     "{today_date}" }
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
                h2 { style:
                    "margin: 0;
                     font-size: 13px; 
                     letter-spacing: 0.08em; 
                     text-transform: uppercase; 
                     color: #9ca3af;", 
                     "Actions" }
                CreateButton { onclick: move |_| on_open_create.call(()) }
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

                h2 { style:
                    "margin: 0 0 12px 0;
                     font-size: 13px; 
                     letter-spacing: 0.08em; 
                     text-transform: uppercase; 
                     color: #9ca3af;",
                      "Completed" }
                div { class: "flex-1 overflow-y-auto pr-1 flex flex-col gap-2",
                    for item in history_data {
                        HistoryItem {
                            title: item.1.clone(),
                            date: item.8.clone().unwrap_or(item.2.clone()),
                            group_name: item.6.clone(),
                            group_color: item.7.clone()
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
                 margin-top: 2px;", "✓"
            }
            div {
                style:
                "display: flex;
                 flex-direction: column; 
                 gap: 2px; 
                 flex: 1; 
                 min-width: 0;",
                span { style:
                    "font-size: 13px;
                     color: #6b7280; 
                     text-decoration: line-through; 
                     overflow: hidden; 
                     text-overflow: ellipsis; 
                     white-space: nowrap;", 
                     "{title}" }
                div {
                    style:
                    "display: flex;
                     align-items: center; 
                     gap: 6px; 
                     flex-wrap: wrap;",
                    span { style:
                        "font-size: 10px;
                         color: #4b5563;",
                         "completed at: {date}" }
                    if let Some(name) = &group_name {
                        if !name.is_empty() {
                            {
                                let color = group_color.as_deref().unwrap_or("#3A6BFF");
                                rsx! { span { style: format!("font-size: 9px;
                                                                background: {}26; 
                                                                color: {}; 
                                                                padding: 1px 5px; 
                                                                border-radius: 3px; 
                                                                font-weight: 500; 
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
