use crate::todos::backend::create_todo;
use dioxus::prelude::*;

#[component]
pub fn CreateButton(onclick: EventHandler<MouseEvent>) -> Element {
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
pub fn CreateModal(
    groups: Vec<(i32, String)>,
    show_modal: Signal<bool>,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut new_task_title = use_signal(|| String::new());
    let mut new_task_group_id = use_signal(|| 0);
    let mut new_task_due_date = use_signal(|| String::new());

    let close_modal = move |_| {
        show_modal.set(false);
        new_task_title.set(String::new());
        new_task_group_id.set(0);
        new_task_due_date.set(String::new());
    };

    let handle_create = move |_| async move {
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
            show_modal.set(false);
            on_refresh.call(());
        }
    };

    rsx! {
        div {
            style:
            "position: absolute;
             top: 0;
             left: 0;
             width: 100%;
             height: 100%;
             background: rgba(0,0,0,0.7); 
             backdrop-filter: blur(5px); 
             z-index: 50;
             display: flex; 
             align-items: center; 
             justify-content: center;",
            div {
                style: "background: #171923;
                        width: 400px; 
                        padding: 24px; 
                        border-radius: 18px;
                        border: 1px solid rgba(255,255,255,0.1); 
                        box-shadow: 0 20px 50px rgba(0,0,0,0.9);
                        display: flex; 
                        flex-direction: column;
                        gap: 16px;",
                h2 { style:
                    "color: white;
                     font-size: 18px; 
                     margin: 0 0 8px 0;", 
                     "Create New To-Do" }

                div { class: "flex flex-col gap-2",
                    label { style:
                        "font-size: 12px;
                         color: #9ca3af; 
                         text-transform: uppercase;",
                          "To-Do Name" }
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
                    label { style:
                        "font-size: 12px;
                         color: #9ca3af; 
                         text-transform: uppercase;", 
                         "Due Date" }
                    input {
                        r#type: "date",
                        style: "background: rgba(255,255,255,0.05);
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
                    label { style:
                        "font-size: 12px;
                         color: #9ca3af; 
                         text-transform: uppercase;", 
                         "Assign to Group" }
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
                        for g in groups.clone() {
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
                        onclick: close_modal,
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
                        onclick: handle_create,
                        "Create To-Do"
                    }
                }
            }
        }
    }
}
