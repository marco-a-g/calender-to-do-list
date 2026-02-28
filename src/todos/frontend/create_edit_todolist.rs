use crate::todos::backend::create_todolist::{
    create_todo_list, frontend_input_to_todo_list, todo_list_into_todo_list_transfer,
};
use crate::todos::backend::delete_todolist::delete_todo_list;
use crate::todos::backend::edit_todolist::edit_todo_list;
use crate::todos::frontend::filter_todos::{GroupFilter, ListFilter};
use crate::utils::date_handling::db_to_html_input;
use crate::utils::functions::get_user_id_and_session_token;
use crate::utils::structs::{CalendarEventLight, CalendarLight, GroupLight, TodoListLight};
use chrono::Local;
use dioxus::prelude::*;
use uuid::Uuid;
#[component]
pub fn CreateListButton(onclick: EventHandler<MouseEvent>) -> Element {
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
            "Create New List"
        }
    }
}

#[component]
pub fn CreateEditListModal(
    groups: Vec<GroupLight>,
    all_events: Vec<CalendarEventLight>,
    all_calendars: Vec<CalendarLight>,
    show_modal: Signal<bool>,
    on_refresh: EventHandler<()>,
    list_to_edit: Signal<Option<TodoListLight>>,
    selected_category: Signal<GroupFilter>,
    selected_list: Signal<ListFilter>,
) -> Element {
    //Standardwerte für neue ToDoListe setzen
    let mut new_list_title = use_signal(String::new);
    let mut new_list_description = use_signal(String::new);
    let mut new_list_group_id = use_signal(String::new);
    let mut new_list_event_id = use_signal(String::new);
    let mut new_list_due_date = use_signal(String::new);
    let mut new_list_priority = use_signal(|| "normal".to_string());
    //Recurrance bei Listen vorerst nicht
    //let mut new_list_rrule = use_signal(|| String::new());
    //let mut new_list_recurrence_until = use_signal(|| String::new());

    use_effect(move || {
        if let Some(list) = list_to_edit() {
            new_list_title.set(list.name.clone());
            new_list_description.set(list.description.clone().unwrap_or_default());
            new_list_priority.set(list.priority.clone().unwrap_or("normal".to_string()));
            new_list_group_id.set(list.group_id.clone().unwrap_or_default());
            new_list_event_id.set(list.attached_to_calendar_event.clone().unwrap_or_default());
            new_list_due_date.set(db_to_html_input(&list.due_datetime).unwrap_or_default());
        }
    });

    //Bei schließen der Maske wieder Werte auf Standard setzen
    let close_modal = move |_| {
        show_modal.set(false);
        list_to_edit.set(None); // Reset Edit State
        new_list_title.set(String::new());
        new_list_description.set(String::new());
        new_list_group_id.set(String::new());
        new_list_event_id.set(String::new());
        new_list_due_date.set(String::new());
        new_list_priority.set("normal".to_string());
        //new_list_rrule.set(String::new());
        //new_list_recurrence_until.set(String::new());
    };

    //Buttontexte je nach Maske (Edit oder Create)
    let modal_title = if list_to_edit().is_some() {
        "Edit List"
    } else {
        "Create New To-Do List"
    };
    let button_text = if list_to_edit().is_some() {
        "Save Changes"
    } else {
        "Create List"
    };

    //Liste Löschen handhaben
    let handle_delete = move |_| {
        if let Some(list) = list_to_edit() {
            let list_id_str = list.id.clone();
            spawn(async move {
                if let Ok(list_uuid) = Uuid::parse_str(&list_id_str) {
                    if delete_todo_list(list_uuid).await.is_ok() {
                        println!("List deleted successfully");
                        show_modal.set(false);
                        list_to_edit.set(None);
                        //Nachdem Liste gelöscht wurde Ansicht wieder auf All-Todos gesetzt werden
                        selected_category.set(GroupFilter::AllGroups);
                        selected_list.set(ListFilter::AllLists);
                        on_refresh.call(());
                    } else {
                        println!("Error deleting list");
                    }
                } else {
                    println!("Error parsing list id for list '{}'", list_id_str);
                }
            });
        }
    };

    //Erstellte Liste handhaben
    let handle_create = move |_| async move {
        if !new_list_title().is_empty() {
            let title = new_list_title();
            let desc_str = new_list_description();
            let group_id_str = new_list_group_id();
            let event_id_str = new_list_event_id();
            let due_date_str = new_list_due_date();
            let prio_str = new_list_priority();
            //let rrule_str = new_list_rrule();
            //let until_str = new_list_recurrence_until();
            // User-ID holen
            let (user_id, _token) = match get_user_id_and_session_token().await {
                Ok(data) => data,
                Err(e) => {
                    println!("Nicht authentifiziert: {:?}", e);
                    return;
                }
            };
            let user_id_str = format!("{:?}", user_id);

            let description = if desc_str.is_empty() {
                None
            } else {
                Some(desc_str)
            };

            let event_id_opt = if event_id_str.is_empty() {
                None
            } else {
                Some(event_id_str)
            };

            let due_date = if due_date_str.is_empty() {
                None
            } else {
                Some(due_date_str)
            };

            let priority_opt = Some(prio_str);

            /*let rrule_opt = None;
            let recurrence_until_opt = None; */

            // Wenn in Edit Mode
            if let Some(existing_list) = list_to_edit() {
                let edited_list = TodoListLight {
                    id: existing_list.id.clone(),
                    name: title,
                    description,
                    list_type: if group_id_str.is_empty() {
                        "private".to_string()
                    } else {
                        "group".to_string()
                    },
                    group_id: if group_id_str.is_empty() {
                        None
                    } else {
                        Some(group_id_str)
                    },
                    due_datetime: due_date,
                    priority: priority_opt,
                    attachment: existing_list.attachment.clone(),
                    created_at: existing_list.created_at.clone(),
                    created_by: existing_list.created_by.clone(),
                    owner_id: existing_list.owner_id.clone(),
                    attached_to_calendar_event: event_id_opt,
                    rrule: None,
                    recurrence_until: None,
                    recurrence_id: None,
                    overrides_datetime: None,
                    skipped: false,
                    last_mod: Local::now().to_rfc3339(),
                };
                let _ = edit_todo_list(edited_list).await; //Backend aufruf

                // Maskenwerte wieder zurücksetzen
                new_list_title.set(String::new());
                new_list_description.set(String::new());
                new_list_group_id.set(String::new());
                new_list_event_id.set(String::new());
                new_list_due_date.set(String::new());
                new_list_priority.set("normal".to_string());
                list_to_edit.set(None);
                show_modal.set(false);
                on_refresh.call(());
            } else {
                //Create Modal
                //input in ToDoList parsen
                match frontend_input_to_todo_list(
                    title,
                    description,
                    group_id_str,
                    user_id_str,
                    due_date,
                    priority_opt,
                    //rrule_opt,
                    //recurrence_until_opt,
                    event_id_opt,
                ) {
                    // Parsing klappt
                    Ok(new_list_struct) => {
                        //ToDoList in ToDoListTransfer umwandeln
                        match todo_list_into_todo_list_transfer(new_list_struct) {
                            // Parsing klappt
                            Ok(transfer_obj) => {
                                // ToDoListTransfer an Supabase senden
                                let _ = create_todo_list(transfer_obj).await;

                                // Eingabemaske zurück setzen
                                new_list_title.set(String::new());
                                new_list_description.set(String::new());
                                new_list_group_id.set(String::new());
                                new_list_event_id.set(String::new());
                                new_list_due_date.set(String::new());
                                new_list_priority.set("normal".to_string());
                                //new_list_rrule.set(String::new());
                                //new_list_recurrence_until.set(String::new());
                                show_modal.set(false);
                                on_refresh.call(());
                            }
                            // Falls TransferObjekt erstellung nicht klappen sollte
                            Err(e) => {
                                println!("Fehler beim Erstellen des Transfer-Objekts: {}", e);
                            }
                        }
                    }
                    // Falls Input nicht in ToDoList geparsed werden konnte
                    Err(e) => {
                        println!("Fehler bei den Eingabedaten: {}", e);
                    }
                };
            }
        };
    };
    //Listen für Dropdown bei Gruppenzuweisung filtern
    let filtered_events: Vec<CalendarEventLight> = all_events
        .iter()
        .filter(|evt| {
            if let Some(calendar) = all_calendars.iter().find(|c| c.id == evt.calendar_id) {
                if new_list_group_id().is_empty() {
                    calendar.calendar_type == "private" || calendar.group_id.is_none()
                } else {
                    calendar.group_id.as_deref() == Some(new_list_group_id().as_str())
                }
            } else {
                false
            }
        })
        .cloned()
        .collect();

    rsx! {
        div {
            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); backdrop-filter: blur(5px); z-index: 60; display: flex; align-items: center; justify-content: center;",
            onclick: close_modal,
            div {
                style: "background: #171923; width: 450px; padding: 24px; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 20px 50px rgba(0,0,0,0.9); display: flex; flex-direction: column; gap: 16px; max-height: 90vh; overflow-y: auto;",
                onclick: |e| e.stop_propagation(),
                // Header
                div { class: "flex justify-between items-start",
                    // Titel Links
                    h2 { style: "color: white; font-size: 18px; margin: 0;", "{modal_title}" }
                    div { class: "flex items-center gap-3",
                        //Delete todoliste button
                        if list_to_edit().is_some() {
                            button {
                                style: "background: transparent; border: none; color: #ef4444; cursor: pointer; font-size: 16px; transition: color 0.2s;",
                                title: "Delete List",
                                class: "hover:text-red-400",
                                onclick: handle_delete,
                                span { style: "font-size: 14px;", "🗑️" }
                            }
                        }
                        // x-Button
                        button {
                            style: "background: transparent; border: none; color: #9ca3af; cursor: pointer; font-size: 18px;",
                            onclick: close_modal,
                            "✕"
                        }
                    }
                }
                // Name setzen
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "List Name" }
                    input {
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none;",
                        value: "{new_list_title}",
                        oninput: move |evt| new_list_title.set(evt.value()),
                        placeholder: "e.g. Project X, Sprint 1"
                    }
                }

                // Description schreiben
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Description (Optional)" }
                    textarea {
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; min-height: 60px; resize: vertical; font-family: sans-serif;",
                        value: "{new_list_description}",
                        oninput: move |evt| new_list_description.set(evt.value()),
                        placeholder: "Add details here..."
                    }
                }

                // Due Date auswählen
                div { style: "display: flex; gap: 10px;",
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Due Date (Optional)" }
                        input {
                            r#type: "date",
                            style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; color-scheme: dark; width: 100%;",
                            value: "{new_list_due_date}",
                            oninput: move |evt| new_list_due_date.set(evt.value())
                        }
                    }
                    //Prio auswählen
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Priority" }
                        select {
                            style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer; width: 100%;",
                            value: "{new_list_priority}",
                            onchange: move |evt| new_list_priority.set(evt.value()),
                            option { value: "low", "Low" }
                            option { value: "normal", "Normal" }
                            option { value: "high", "High" }
                            option { value: "top", "Top" }
                        }
                    }
                }

               /* // RRule auswählen//vorerst doch nicht nötig für Listen, nur Todos
                div { style: "display: flex; gap: 10px;",
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Recurrence" }
                        select {
                            style: format!("background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; outline: none; cursor: pointer; width: 100%; color: {};",
                                if new_list_rrule().is_empty() { "#9ca3af" } else { "white" }),
                            value: "{new_list_rrule}",
                            onchange: move |evt| new_list_rrule.set(evt.value()),
                            option { value: "", style: "color: #9ca3af;", "Not recurring" }
                            option { value: "daily", "Daily" }
                            option { value: "weekly", "Weekly" }
                            option { value: "fortnight", "Fortnight (2 Weeks)" }
                            option { value: "weekdays", "On Weekdays (Mon-Fri)" }
                            option { value: "monthly_on_weekday", "Monthly (Weekday)" }
                            option { value: "monthly_on_date", "Monthly (Date)" }
                            option { value: "monthly", "Monthly" }
                            option { value: "annual", "Annual" }
                        }
                    }
                     //Recurrence until auswählen //vorerst doch nicht nötig für Listen, nur Todos
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Repeat Until" }
                        input {
                            r#type: "date",
                            disabled: new_list_rrule().is_empty(),
                            style: format!("background: {}; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; color-scheme: dark; width: 100%; opacity: {}; cursor: {};",
                                if new_list_rrule().is_empty() { "#1a1a1a" } else { "rgba(255,255,255,0.05)" },
                                if new_list_rrule().is_empty() { "0.3" } else { "1" },
                                if new_list_rrule().is_empty() { "not-allowed" } else { "pointer" }
                            ),
                            value: "{new_list_recurrence_until}",
                            oninput: move |evt| new_list_recurrence_until.set(evt.value())
                        }
                    }
                }*/

                // Gruppen zur zuweisung auflisten
                if list_to_edit().is_none() {
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Assign to Group" }
                    select {
                        style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer;",
                        onchange: move |evt| { new_list_group_id.set(evt.value()); new_list_event_id.set(String::new()); },
                        option { value: "", "Personal (No Group)" }
                        for g in groups.clone() { option { value: "{g.id}", "{g.name}" } }
                    }
                }
            }
                // Events auflisten zum zuordnen
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Attach to Calendar Event (Optional)" }
                    select {
                        disabled: filtered_events.is_empty(),
                        style: format!("background: {}; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: {}; opacity: {};", if filtered_events.is_empty() { "#1a1a1a" } else { "#171923" }, if filtered_events.is_empty() { "not-allowed" } else { "pointer" }, if filtered_events.is_empty() { "0.5" } else { "1" }),
                        onchange: move |evt| new_list_event_id.set(evt.value()),
                        option { value: "", "Don't assign to event" }
                        for evt in filtered_events.clone() { option { value: "{evt.id}", "{evt.summary}" } }
                    }
                }

                // Buttons: Create und Cancel
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
                        "{button_text}"
                    }
                }
            }
        }
    }
}
