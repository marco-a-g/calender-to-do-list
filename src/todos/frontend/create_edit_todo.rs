use crate::todos::backend::create_todo::{
    create_todo_event, frontend_input_to_todo, todo_event_into_to_do_transfer,
};
use crate::todos::backend::edit_todo::edit_todo_event;
use crate::utils::date_handling::db_to_html_input;
use crate::utils::structs::{
    GroupLight, GroupMemberLight, ProfileLight, TodoEventLight, TodoListLight,
};
use chrono::Local;
use dioxus::prelude::*;
use uuid::Uuid;
#[component]
pub fn CreateToDoButton(onclick: EventHandler<MouseEvent>) -> Element {
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

//Umfunktioniert in Create ToDo + Edit ToDo Maske -> nur header ändert sich
#[component]
pub fn CreateEditToDoModal(
    groups: Vec<GroupLight>,
    all_lists: Vec<TodoListLight>,
    all_profiles: Vec<ProfileLight>,
    all_group_members: Vec<GroupMemberLight>,
    show_modal: Signal<bool>,
    on_refresh: EventHandler<()>,
    todo_to_edit: Signal<Option<TodoEventLight>>,
    edit_series_mode: Signal<bool>,
) -> Element {
    //Standards fürs erstellen setzen
    let mut new_task_title = use_signal(|| String::new());
    let mut new_task_description = use_signal(|| String::new());
    let mut new_task_group_id = use_signal(|| String::new());
    let mut new_task_list_id = use_signal(|| String::new());
    let mut new_task_assignee = use_signal(|| String::new());
    let mut new_task_due_date = use_signal(|| String::new());
    let mut new_task_priority = use_signal(|| "normal".to_string());
    let mut new_task_rrule = use_signal(|| String::new());
    let mut new_task_recurrence_until = use_signal(|| String::new());

    //für edit Maske alle listen nutzen, clonen
    let lists_for_effect = all_lists.clone();

    use_effect(move || {
        // Prüfen, ob ein Todo zum Bearbeiten übergeben wurde
        if let Some(todo) = todo_to_edit() {
            // Titel
            new_task_title.set(todo.summary.clone());
            // Beschreibung
            new_task_description.set(todo.description.clone().unwrap_or_default());
            // Prio
            new_task_priority.set(todo.priority.clone().unwrap_or("normal".to_string()));
            // Rrule
            new_task_rrule.set(todo.rrule.clone().unwrap_or_default());
            // Zugewiesener nutzer
            new_task_assignee.set(todo.assigned_to_user.clone().unwrap_or_default());
            // Listen ID
            let current_list_id = todo.todo_list_id.clone();

            //über Listen itterieren und suchen ob es sich um Shadow List handelt
            if let Some(list) = lists_for_effect.iter().find(|l| l.id == current_list_id) {
                if Uuid::parse_str(&list.name).is_ok() {
                    // Lässt sich name der Liste in uuid parsen ist es shadow list -> new Task_list_id auf leeren String setzen
                    new_task_list_id.set(String::new());
                } else {
                    // Es ist existierende Liste, dann deren id hernehmen
                    new_task_list_id.set(current_list_id.clone());
                }

                // Gruppen ID der Liste finden
                if let Some(gid) = &list.group_id {
                    new_task_group_id.set(gid.clone());
                } else {
                    new_task_group_id.set(String::new());
                }
            } else {
                // falls Liste nicht gefunden, sollte aber  nicht passieren
                new_task_list_id.set(String::new());
                new_task_group_id.set(String::new());
            }
            //Due Datum
            new_task_due_date.set(db_to_html_input(&todo.due_datetime).unwrap_or_default());
            // Rec-Until Datum
            new_task_recurrence_until
                .set(db_to_html_input(&todo.recurrence_until).unwrap_or_default());
        } else {
            // Bei keinem Todo_to_edit -> Werte bleiben Standardwerte
        }
    });

    //bei close modal Standards wieder zurücksetzen
    let close_modal = move |_| {
        show_modal.set(false);
        todo_to_edit.set(None);
        new_task_title.set(String::new());
        new_task_description.set(String::new());
        new_task_group_id.set(String::new());
        new_task_list_id.set(String::new());
        new_task_assignee.set(String::new());
        new_task_due_date.set(String::new());
        new_task_priority.set("normal".to_string());
        new_task_rrule.set(String::new());
        new_task_recurrence_until.set(String::new());
        edit_series_mode.set(true);
    };

    //let all_lists_for_handler = all_lists.clone();

    // Header des Modals
    let modal_title = if todo_to_edit().is_some() {
        "Edit To-Do"
    } else {
        "Create New To-Do"
    };

    //Button des Modals rechts (Create oder Edit)
    let button_text = if todo_to_edit().is_some() {
        "Save Changes"
    } else {
        "Create To-Do"
    };

    // Validierung für eingabemaske, gültig wenn: Titel nicht leer || RRule eingaben gültig
    let is_form_valid = !new_task_title().is_empty()
        && (new_task_rrule().is_empty() || !new_task_recurrence_until().is_empty());

    let handle_create = move |_| {
        //let all_lists_inner = all_lists_for_handler.clone();
        //Werte für neues ToDo-Setzen
        async move {
            if !new_task_title().is_empty() {
                //Due Date
                let due_date = if new_task_due_date().is_empty() {
                    None
                } else {
                    Some(new_task_due_date())
                };
                //Rrule
                let rrule = if new_task_rrule().is_empty() {
                    None
                } else {
                    Some(new_task_rrule())
                };
                //Recurrence until
                let recurrence_until =
                    if new_task_recurrence_until().is_empty() || new_task_rrule().is_empty() {
                        None
                    } else {
                        Some(new_task_recurrence_until())
                    };
                //Beschreibung
                let description = if new_task_description().is_empty() {
                    None
                } else {
                    Some(new_task_description())
                };
                //Listen&Gruppe
                let list_id = if !new_task_list_id().is_empty() {
                    // Eine Liste wurde ausgewählt -> diese nutzen
                    new_task_list_id()
                } else if !new_task_group_id().is_empty() {
                    // Keine Liste und eine Gruppe wurde ausgewählt -> Gruppen ID übergeben und in Backend zu ShadowListe für die Gruppe mappen
                    new_task_group_id()
                } else {
                    // weder Liste noch Gruppe ausgewählt -> ShadowListe des Nutzers -> In Backend mappen
                    "".to_string()
                };

                //zugewiesener User
                let assignee = if new_task_assignee().is_empty() {
                    None
                } else {
                    Some(new_task_assignee())
                };

                // Unterscheidung für Edit und Create hier
                if let Some(existing_todo) = todo_to_edit() {
                    //:________________________________________________________________
                    //Hier Edit
                    //Unterscheidung für Ganze reihe oder nur diese Instanz
                    // Prüfen: Ist es ein Master, der sich wiederholt?
                    let is_master_recurring =
                        existing_todo.recurrence_id.is_none() && existing_todo.rrule.is_some();

                    //uuid, rrule, und overides datetime extrahieren, anhand von Fallunterscheidung in: ist recurring todo und soll ganze serie bearbeiten
                    let (target_rec_id, target_rrule, target_overrides) = if is_master_recurring {
                        // Ganze Reihe editieren -> Auf Mastereintrag arbeiten
                        if edit_series_mode() {
                            (None, rrule, None)
                        } else {
                            // Nur einzelne Instanz eines recurring todos ändern
                            (
                                Some(existing_todo.id.clone()),
                                None,
                                existing_todo.due_datetime.clone(),
                            )
                        }
                    } else {
                        // Nicht Recurring todo ändern oder existierende Exception eines Masters
                        (
                            existing_todo.recurrence_id.clone(),
                            rrule,
                            existing_todo.overrides_datetime.clone(),
                        )
                    };
                    //:________________________________________________________________

                    // Edit-Modus
                    let updated_todo = TodoEventLight {
                        //Hier light creieren und in backend funktion in Transfer Objekt wandeln
                        id: existing_todo.id.clone(),
                        todo_list_id: list_id,
                        summary: new_task_title(),
                        description: description,
                        completed: existing_todo.completed,
                        due_datetime: due_date,
                        priority: Some(new_task_priority()),
                        attachment: existing_todo.attachment.clone(),
                        rrule: target_rrule, //rrule aus Fallunterscheidung oben nutzen
                        recurrence_until: recurrence_until,
                        recurrence_id: target_rec_id, //recid  aus Fallunterscheidung oben nutzen
                        created_by: existing_todo.created_by.clone(),
                        created_at: existing_todo.created_at.clone(),
                        last_mod: Local::now().to_rfc3339(),
                        assigned_to_user: assignee,
                        skipped: existing_todo.skipped,
                        overrides_datetime: target_overrides, //overrides datetime aus Fallunterscheidung oben nutzen
                    };

                    let _ = edit_todo_event(updated_todo).await; // Edit Funkrion an Remote -DB, kümmert sich selbst um Fallunterscheidung anhand eingabeparameter nach Fallunterscheidung oben
                } else {
                    // Create-Modus
                    let new_todo_list_id = list_id;
                    let new_summary = new_task_title();
                    let new_description = description;
                    let new_due_datetime = due_date;
                    let new_priority = Some(new_task_priority());
                    let new_rrule = rrule;
                    let new_recurrence_until = recurrence_until;
                    let new_assigned_to_user = assignee;

                    //lässt sich input erfolgreich in ToDoEvent umwandeln
                    match frontend_input_to_todo(
                        new_todo_list_id,
                        new_summary,
                        new_description,
                        new_due_datetime,
                        new_priority,
                        new_rrule,
                        new_recurrence_until,
                        new_assigned_to_user,
                    ) {
                        //Wenn ja in ToDoTransfer umwandeln
                        Ok(new_todo_struct) => {
                            match todo_event_into_to_do_transfer(new_todo_struct) {
                                Ok(todo_transfer) => {
                                    // Erfolgreich umgewandelt -> create damit aufrufen
                                    let _ = create_todo_event(todo_transfer).await;
                                } //Sollte Fehler beim umwandeln entstehen Error werfen
                                Err(e) => {
                                    println!("Fehler beim Erstellen des Transfer-Objekts: {}", e);
                                }
                            }
                        }
                        //Falls Umwandlung in ToDoTransfer fehlschlägt
                        Err(e) => {
                            println!(
                                "Fehler bei den Eingabedaten (z.B. falsches Datumsformat): {}",
                                e
                            );
                        }
                    }
                }

                //Nach erstellen eines Todos Standardwerte der Maske wieder zurücksetzen
                new_task_title.set(String::new());
                new_task_description.set(String::new());
                new_task_due_date.set(String::new());
                new_task_group_id.set(String::new());
                new_task_list_id.set(String::new());
                new_task_assignee.set(String::new());
                new_task_priority.set("normal".to_string());
                new_task_rrule.set(String::new());
                new_task_recurrence_until.set(String::new());
                todo_to_edit.set(None);
                show_modal.set(false);
                on_refresh.call(());
                edit_series_mode.set(true);
            }
        }
    };

    // Listen Filtern für Auswahl in Drop Down
    let filtered_lists: Vec<TodoListLight> = all_lists
        .iter()
        .filter(|l| {
            // Prv oder Gruppe ausgewählt
            let private_or_matching_group = if new_task_group_id().is_empty() {
                //Zeigt nur private Listen
                l.list_type == "private"
            } else {
                //Zeigt nur GruppenListen der ausgewählten Gruppe
                l.group_id.as_deref() == Some(new_task_group_id().as_str())
            };

            // Check ist es shadowList
            let is_real_list = Uuid::parse_str(&l.name).is_err();
            //nur Listen die keine shadows sind, private listen des users oder gruppenlisten des users sind mit rein nehmen
            private_or_matching_group && is_real_list
        })
        .cloned()
        .collect();

    //User Filtern für Auswahl in Drop Down bei USer assign
    let filtered_users: Vec<ProfileLight> = if new_task_group_id().is_empty() {
        vec![]
    } else {
        let current_group_id = new_task_group_id();
        let member_user_ids: Vec<String> = all_group_members
            .iter()
            .filter(|m| m.group_id == current_group_id)
            .map(|m| m.user_id.clone())
            .collect();
        all_profiles
            .iter()
            .filter(|profile| member_user_ids.contains(&profile.id))
            .cloned()
            .collect()
    };

    //Für Edit mode Unterscheidung in ist master oder ist rec.instanz
    let is_recurrence_instance = if let Some(todo) = todo_to_edit() {
        todo.recurrence_id.is_some()
    } else {
        false
    };

    //Create ToDo Maske
    rsx! {
        div {
            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); backdrop-filter: blur(5px); z-index: 50; display: flex; align-items: center; justify-content: center;",

            div {
                style: "background: #171923; width: 450px; padding: 24px; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 20px 50px rgba(0,0,0,0.9); display: flex; flex-direction: column; gap: 16px; max-height: 90vh; overflow-y: auto;",

                // Titel dynamisch (Create oder Edit)
                h2 { style: "color: white; font-size: 18px; margin: 0 0 8px 0;", "{modal_title}" }

                // Name setzen
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "To-Do Name" }
                    input {
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none;",
                        value: "{new_task_title}",
                        oninput: move |evt| new_task_title.set(evt.value()),
                        placeholder: "e.g. Fix Error in sync-function"
                    }
                }

                // Beschreibung eingeben
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Description (Optional)" }
                    textarea {
                        style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; min-height: 60px; resize: vertical; font-family: sans-serif;",
                        value: "{new_task_description}",
                        oninput: move |evt| new_task_description.set(evt.value()),
                        placeholder: "Add details here..."
                    }
                }

                // Due Date setzen
                div { style: "display: flex; gap: 10px;",
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Due Date" }
                        input {
                            r#type: "date",
                            style: "background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; color-scheme: dark; width: 100%;",
                            value: "{new_task_due_date}",
                            oninput: move |evt| new_task_due_date.set(evt.value())
                        }
                    }
                    //Prio setzen
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Priority" }
                        select {
                            style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer; width: 100%;",
                            value: "{new_task_priority}",
                            onchange: move |evt| new_task_priority.set(evt.value()),
                            option { value: "low", "Low" }
                            option { value: "normal", "Normal" }
                            option { value: "high", "High" }
                            option { value: "top", "Top" }
                        }
                    }
                }
            // Recurrance instanzen können RRule nicht ändern -> Andere Anzeige im Modal
            if is_recurrence_instance {
                div {
                    style: "background: rgba(255,255,255,0.05); border: 1px dashed rgba(255,255,255,0.2); padding: 12px; border-radius: 8px; color: #9ca3af; font-size: 13px; text-align: center; margin-bottom: 8px;",
                    "Recurring settings are managed by the Master To-Do."
                    br {}
                    "Please edit the original task to change the schedule."
                }
            } else {
                //Masterinstanzen können RRule schon ändern
                // Rrule setzen
                div { style: "display: flex; gap: 10px;",
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Recurrence" }
                        select {
                            style: format!("background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; outline: none; cursor: pointer; width: 100%; color: {};",
                                if new_task_rrule().is_empty() { "#9ca3af" } else { "white" }),
                            value: "{new_task_rrule}",
                            onchange: move |evt| new_task_rrule.set(evt.value()),
                            option { value: "", style: "color: #9ca3af;", "Not recurring" }
                            option { value: "daily", "Daily" }
                            option { value: "weekly", "Weekly" }
                            option { value: "fortnight", "Fortnight (2 Weeks)" }
                            option { value: "weekdays", "On Weekdays (Mon-Fri)" }
                            option { value: "monthly_on_weekday", "Monthly (Weekday)" }
                            option { value: "monthly_on_date", "Monthly (Date)" }
                            option { value: "annual", "Annual" }
                        }
                    }
                    //Reccurence until setzen
                    div { class: "flex flex-col gap-2 flex-1",
                        label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Repeat Until" }
                        input {
                            r#type: "date",
                            disabled: new_task_rrule().is_empty(),
                            style: format!("background: {}; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; color-scheme: dark; width: 100%; opacity: {}; cursor: {};",
                                if new_task_rrule().is_empty() { "#1a1a1a" } else { "rgba(255,255,255,0.05)" },
                                if new_task_rrule().is_empty() { "0.3" } else { "1" },
                                if new_task_rrule().is_empty() { "not-allowed" } else { "pointer" }
                            ),
                            value: "{new_task_recurrence_until}",
                            oninput: move |evt| new_task_recurrence_until.set(evt.value())
                        }
                    }
                }
            }

            // Gruppe zuweisen / Auswhal
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Assign to Group" }
                    select {
                        style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer;",
                        onchange: move |evt| { new_task_group_id.set(evt.value()); new_task_list_id.set(String::new()); new_task_assignee.set(String::new()); },
                        value: "{new_task_group_id}",
                        option { value: "", "Personal (No Group)" }
                        for g in groups.clone() { option { value: "{g.id}", "{g.name}" } }
                    }
                }

                // Liste zuweisen / Auswahl
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Select List (Optional)" }
                    select {
                        style: "background: #171923; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: pointer;",
                        onchange: move |evt| new_task_list_id.set(evt.value()),
                        value: "{new_task_list_id}",
                        option { value: "", "Don't assign to specific List" } //Hier nochmal in JF reden
                        for list in filtered_lists { option { value: "{list.id}", "{list.name}" } }
                    }
                }

                // User zuweisen / Auswahl
                div { class: "flex flex-col gap-2",
                    label { style: "font-size: 12px; color: #9ca3af; text-transform: uppercase;", "Assign To User" }
                    select {
                        disabled: new_task_group_id().is_empty(),
                        style: format!("background: {}; color-scheme: dark; border: 1px solid rgba(255,255,255,0.1); padding: 10px; border-radius: 8px; color: white; outline: none; cursor: {}; opacity: {};", if new_task_group_id().is_empty() { "#1a1a1a" } else { "#171923" }, if new_task_group_id().is_empty() { "not-allowed" } else { "pointer" }, if new_task_group_id().is_empty() { "0.5" } else { "1" }),
                        onchange: move |evt| new_task_assignee.set(evt.value()),
                        value: "{new_task_assignee}",
                        option { value: "", if new_task_group_id().is_empty() { "Personal" } else { "Unassigned" } }
                        for user in filtered_users { option { value: "{user.id}", "{user.username}" } }
                    }
                }

                // Buttons: Create bzw. Edit und Cancel
                div {
                    style: "display: flex; gap: 10px; margin-top: 10px;",
                    button {
                        style: "flex: 1; padding: 10px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); color: #9ca3af; background: transparent; cursor: pointer;",
                        onclick: close_modal,
                        "Cancel"
                    }
                    button {
                        // Save bzw. Edit Button nur gültig, wenn RRule und Runtil gültig
                        style: format!(
                            "flex: 1; padding: 10px; border-radius: 8px; background: {}; color: white; border: none; font-weight: 600; cursor: {}; opacity: {};",
                            if is_form_valid { "#3A6BFF" } else { "#4b5563" }, // Blau oder Grau
                            if is_form_valid { "pointer" } else { "not-allowed" },
                            if is_form_valid { "1" } else { "0.5" }
                        ),
                        // Button deaktivieren, wenn Formular ungültig
                        disabled: !is_form_valid,
                        onclick: handle_create,
                        "{button_text}"
                    }
                }
            }
        }
    }
}
