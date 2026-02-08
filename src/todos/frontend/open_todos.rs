use crate::todos::frontend::filter_todos::{GroupFilter, ListFilter};
use crate::utils::date_handling::db_to_display_only_date;
use crate::utils::structs::{
    CalendarEventLight, GroupLight, ProfileLight, TodoEventLight, TodoListLight,
};
use chrono::{Duration, Local, NaiveDate};
use dioxus::prelude::*;

#[component]
pub fn OpenToDoView(
    todos: Vec<TodoEventLight>,
    all_lists: Vec<TodoListLight>,
    groups: Vec<GroupLight>,
    all_profiles: Vec<ProfileLight>,
    all_events: Vec<CalendarEventLight>,
    selected_category: GroupFilter,
    selected_list_filter: ListFilter,
    on_complete: EventHandler<TodoEventLight>,
    on_select_todo: EventHandler<TodoEventLight>,
    on_edit_list: EventHandler<TodoListLight>,
) -> Element {
    // Über die Übergebenen Todos alle Listen und Gruppen für Sidebar heraussuchen
    let filtered_tasks: Vec<TodoEventLight> = todos
        .iter()
        .filter(|task| {
            let parent_list_opt = all_lists.iter().find(|l| l.id == task.todo_list_id);
            match parent_list_opt {
                Some(parent_list) => {
                    let category_match = match &selected_category {
                        GroupFilter::AllGroups => true,
                        GroupFilter::Personal => parent_list.list_type == "private",
                        GroupFilter::Group(g_id) => parent_list.group_id.as_deref() == Some(g_id),
                    };
                    let list_match = match &selected_list_filter {
                        ListFilter::AllLists => true,
                        ListFilter::SpecificList(l_id) => &parent_list.id == l_id,
                    };
                    category_match && list_match
                }
                None => {
                    matches!(selected_category, GroupFilter::AllGroups)
                        && matches!(selected_list_filter, ListFilter::AllLists)
                }
            }
        })
        .cloned()
        .collect();

    //Todos filtern nach due date mit Hilfsfunktion unten
    let (today_list, week_list, later_list) = categorize_todos(&filtered_tasks);

    let current_list_obj = if let ListFilter::SpecificList(id) = &selected_list_filter {
        all_lists.iter().find(|l| &l.id == id).cloned()
    } else {
        None
    };

    // Titel der Ansicht je nach Auswahl der Filter
    let title = match &selected_list_filter {
        // Spezifische Liste ausgewählt:
        ListFilter::SpecificList(id) => {
            //
            if let Some(list) = all_lists.iter().find(|l| &l.id == id) {
                if list.list_type == "private" {
                    // Personal Liste: "Personal To-Do-List 'Name'"
                    format!("Personal To-Do-List \"{}\"", list.name)
                } else {
                    // Gruppen Liste: "To-Do-List 'Name' from Group: Name", oder "Unknown Group", aber sollte eig. nicht passieren
                    let group_name = list
                        .group_id
                        .as_ref()
                        .and_then(|gid| groups.iter().find(|g| &g.id == gid))
                        .map(|g| g.name.clone())
                        .unwrap_or("Unknown Group".to_string());
                    format!("List \"{}\" (Group: {})", list.name, group_name)
                }
            } else {
                //Keine zugehörige Liste gefunden -> Unknown List, sollte aber auch nicht passieren -> Sollte extra Liste pro User/Gruppe geben, die "nicht Listen zugewiesene ToDos" hat
                "Unknown List".to_string()
            }
        }
        // Gruppenauswahl Header
        ListFilter::AllLists => match &selected_category {
            GroupFilter::AllGroups => "All To-Do's".to_string(),
            GroupFilter::Personal => "Personal To-Do's".to_string(),
            GroupFilter::Group(g_id) => {
                let group_name = groups
                    .iter()
                    .find(|g| &g.id == g_id)
                    .map(|g| g.name.clone())
                    .unwrap_or("Unknown Group".to_string());

                format!("To-Do's for \"{}\"", group_name)
            }
        },
    };

    // Liste Metadaten für Header extrahieren
    let (list_due_display, list_description, list_priority) =
        if let ListFilter::SpecificList(id) = &selected_list_filter {
            if let Some(list) = all_lists.iter().find(|l| &l.id == id) {
                // Due Datum der Liste
                let date_str = db_to_display_only_date(&list.due_datetime)
                    .ok()
                    .filter(|s| !s.is_empty());

                // Beschreibung
                let description = list.description.clone().filter(|d| !d.is_empty());

                // Prio
                let raw_prio = list.priority.clone().unwrap_or("normal".to_string());
                let (p_label, p_color) = match raw_prio.to_lowercase().as_str() {
                    "low" => ("Low", "#3b82f6"),
                    "high" => ("High", "#f59e0b"),
                    "top" => ("Top", "#ef4444"),
                    _ => ("Normal", "#9ca3af"),
                };
                (date_str, description, Some((p_label, p_color)))
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

    rsx! {
        div {
            style: "flex: 1; padding: 24px; display: flex; flex-direction: column; background: #080910;",
            div {
                style: "background: linear-gradient(145deg, #1f222c 0%, #14161f 100%); border-radius: 18px; padding: 24px; box-shadow: 0 18px 40px rgba(0,0,0,0.85); border: 1px solid rgba(255,255,255,0.06); flex: 1; display: flex; flex-direction: column; gap: 16px; overflow: hidden;",
                div {
                    style: "border-bottom: 1px solid rgba(255,255,255,0.06); padding-bottom: 16px; margin-bottom: 8px;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: flex-start;",
                        // Titel Links im Header
                        div {
                            h2 {
                                style: "margin: 0 0 4px 0; font-size: 13px; letter-spacing: 0.08em; text-transform: uppercase; color: #9ca3af;",
                                "Tasks"
                            }
                            h1 {
                                style: "margin: 0; font-size: 24px; font-weight: 600; color: #f9fafb;",
                                "{title}"
                            }
                        }
                        // Metadaten Rechts im Header (Description, Priority, Due Date)
                        div {
                            style: "display: flex; gap: 24px; text-align: right;",
                            // Description der Liste
                            if let Some(desc) = list_description {
                                div {
                                    style: "max-width: 300px;",
                                    span {
                                        style: "font-size: 11px; color: #6b7280; text-transform: uppercase; display: block; margin-bottom: 2px;",
                                        "Description"
                                    }
                                    span {
                                        style: "font-size: 13px; color: #e5e7eb; display: block; line-height: 1.4;",
                                        "{desc}"
                                    }
                                }
                            }
                            // Priority de Liste
                            if let Some((label, color)) = list_priority {
                                div {
                                    span {
                                        style: "font-size: 11px; color: #6b7280; text-transform: uppercase; display: block; margin-bottom: 2px;",
                                        "Priority"
                                    }
                                    span {
                                        style: "font-size: 14px; color: {color}; font-weight: 600;",
                                        "{label}"
                                    }
                                }
                            }
                            // Due Date der Liste
                            if let Some(due_date) = list_due_display {
                                div {
                                    span {
                                        style: "font-size: 11px; color: #6b7280; text-transform: uppercase; display: block; margin-bottom: 2px;",
                                        "List Due Date"
                                    }
                                    span {
                                        style: "font-size: 14px; color: #f3f4f6; font-weight: 500;",
                                        "{due_date}"
                                    }
                                }
                            }
                            //Bearbeiten symbol für todoliste
                            if let Some(list_obj) = current_list_obj {
                                div {
                                    style: "margin-left: 10px;",
                                    button {
                                        onclick: move |_| on_edit_list.call(list_obj.clone()),
                                        title: "Edit List",
                                        style: "background: rgba(255, 255, 255, 0.1); border: 1px solid rgba(255, 255, 255, 0.2); cursor: pointer; color: white; border-radius: 8px; width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; transition: all 0.2s;",
                                        class: "hover:bg-blue-600 hover:border-blue-500",
                                        span { style: "font-size: 14px;", "✏️" }
                                    }
                        }
                    }
                }
            }
        }
                div {
                    class: "flex-1 overflow-y-auto pr-2 flex flex-col gap-3",
                    // Due heute oder overdue
                    div {
                        style: "font-size: 12px; color: #9ca3af; font-weight: 600; margin-top: 8px; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.05em;",
                        "Due Today / Overdue"
                    }
                    if today_list.is_empty() {
                        div { style: "font-size: 13px; color: #4b5563; padding: 8px 0;", "No Tasks." }
                    }
                    for task in today_list {
                        // alle due heute/overdue ToDos rendern
                        ToDoItem {
                            key: "{task.id}",
                            task: task.clone(),
                            parent_list: all_lists.iter().find(|l| l.id == task.todo_list_id).cloned(),
                            parent_group: all_lists //alle Gruppen > ToDoList > ToDo finden
                                .iter()
                                .find(|l| l.id == task.todo_list_id)
                                .and_then(|l| l.group_id.as_ref())
                                .and_then(|gid| groups.iter().find(|g| &g.id == gid).cloned()),
                            all_profiles: all_profiles.clone(),
                            all_events: all_events.clone(),
                            on_complete: move |t| on_complete.call(t),
                            on_click: move |t| on_select_todo.call(t)
                        }
                    }

                    // Due in einer Woche
                    div {
                        style: "font-size: 12px; color: #9ca3af; font-weight: 600; margin-top: 24px; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.05em;",
                        "Due in the next 7 days"
                    }
                    if week_list.is_empty() {
                        div { style: "font-size: 13px; color: #4b5563; padding: 8px 0;", "No Tasks." }
                    }
                    for task in week_list {
                    // alle due in einer woche ToDos rendern
                        ToDoItem {
                            key: "{task.id}",
                            task: task.clone(),
                            parent_list: all_lists.iter().find(|l| l.id == task.todo_list_id).cloned(),
                            parent_group: all_lists //alle Gruppen > ToDoList > ToDo finden
                                .iter()
                                .find(|l| l.id == task.todo_list_id)
                                .and_then(|l| l.group_id.as_ref())
                                .and_then(|gid| groups.iter().find(|g| &g.id == gid).cloned()),
                            all_profiles: all_profiles.clone(),
                            all_events: all_events.clone(),
                            on_complete: move |t| on_complete.call(t),
                            on_click: move |t| on_select_todo.call(t)
                        }
                    }
                    // Due Later
                    div {
                        style: "font-size: 12px; color: #9ca3af; font-weight: 600; margin-top: 24px; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.05em;",
                        "Due Later or no Due-Date"
                    }
                    if later_list.is_empty() {
                        div { style: "font-size: 13px; color: #4b5563; padding: 8px 0;", "No Tasks." }
                    }
                    for task in later_list {
                        //alle Due Later todos rendern
                        ToDoItem {
                            key: "{task.id}",
                            task: task.clone(),
                            parent_list: all_lists.iter().find(|l| l.id == task.todo_list_id).cloned(),
                            parent_group: all_lists //alle Gruppen > ToDoList > ToDo finden
                                .iter()
                                .find(|l| l.id == task.todo_list_id)
                                .and_then(|l| l.group_id.as_ref())
                                .and_then(|gid| groups.iter().find(|g| &g.id == gid).cloned()),
                            all_profiles: all_profiles.clone(),
                            all_events: all_events.clone(),
                            on_complete: move |t| on_complete.call(t),
                            on_click: move |t| on_select_todo.call(t)
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
    all_events: Vec<CalendarEventLight>,
    on_complete: EventHandler<TodoEventLight>,
    on_click: EventHandler<TodoEventLight>,
) -> Element {
    //Feststellen ob master für Mastersymbol
    let is_recurring = task.rrule.is_some() || task.recurrence_id.is_some();
    // Auf ToDo klicken -> Task_cklick auf dieses Todo setzen (Für Detailansicht)
    let task_click = task.clone();
    // ToDo Completen > task_complete auf dieses todo setzen
    let task_complete = task.clone();
    // Datum
    let now = Local::now().date_naive();
    let (date_color, font_weight, display_date) = match db_to_display_only_date(&task.due_datetime)
    {
        Ok(german_format) if !german_format.is_empty() => {
            if let Ok(parsed_date) = NaiveDate::parse_from_str(&german_format, "%d.%m.%Y") {
                if parsed_date <= now {
                    ("#ef4444", "600", german_format)
                } else {
                    ("#6b7280", "400", german_format)
                }
            } else {
                ("#6b7280", "400", german_format) //falls Fehler beim Parsen, sollte aber nicht
            }
        }
        // Bei Fehler, None oder leerem String nicht anzeigen
        _ => ("#6b7280", "400", String::new()),
    };

    // Prio
    let raw_priority = task.priority.clone().unwrap_or("normal".to_string());
    let (priority_label, priority_color) = match raw_priority.to_lowercase().as_str() {
        "low" => ("Low", "#3b82f6"),
        "high" => ("High", "#f59e0b"),
        "top" => ("Top", "#ef4444"),
        _ => ("Normal", "#9ca3af"),
    };

    // Beschreibung
    let raw_desc = task.description.clone().unwrap_or_default();
    let (desc_text, desc_color) = if raw_desc.is_empty() {
        ("-".to_string(), "#4b5563")
    } else {
        (raw_desc, "#e5e7eb")
    };

    // Label fürs ToDo
    let (group_badge, list_badge) = if let Some(list) = &parent_list {
        let g_text = if list.list_type == "private" {
            "Personal".to_string()
        } else {
            let g_name = parent_group
                .as_ref()
                .map(|g| g.name.clone())
                .unwrap_or("Group".to_string());
            format!("{}", g_name)
        };
        let l_text = format!("{}", list.name);
        (Some(g_text), Some(l_text))
    } else {
        (None, None) //Sollte auch Liste ohne Gruppe möglich sein oder? In JF besprechen
    };
    //Wenn ToDo-Liste zu einem Event gehört Event label bei ToDo rendern (Über Parent list gehen -> hat es event id -> Event anhanf von id finden -> Summary/Name mit ausgeben)
    let event_badge = if let Some(list) = &parent_list {
        if let Some(event_id) = &list.attached_to_calendar_event {
            if let Some(event) = all_events.iter().find(|e| &e.id == event_id) {
                Some(format!("Event: {}", event.summary))
            } else {
                Some("Event".to_string())
            }
        } else {
            None
        }
    } else {
        None
    };
    // Zugewiesener User
    let (assignee_label, assignee_color) = if let Some(user_id) = &task.assigned_to_user {
        if let Some(user) = all_profiles.iter().find(|p| &p.id == user_id) {
            (user.username.clone(), "#e5e7eb")
        } else {
            ("Unassigned".to_string(), "#9ca3af")
        }
    } else {
        ("No one".to_string(), "#4b5563") //Sollte eig nicht passieren
    };
    rsx! {
        div {
            style: "background: #181b24; border-radius: 14px; border: 1px solid rgba(255,255,255,0.06); box-shadow: 0 4px 12px rgba(0,0,0,0.2); padding: 16px; display: flex; align-items: center; gap: 14px; transition: border-color 0.2s;",
            // Checkbox
            div {
                // Complete Task
                onclick: move |_| on_complete.call(task_complete.clone()),
                style: "width: 20px; height: 20px; border-radius: 50%; border: 2px solid #4b5563; cursor: pointer; flex-shrink: 0; transition: border-color 0.2s;",
                class: "hover:border-blue-500"
            }
            // Name und Datum (anklickbar für Detailansicht) und rec symbol
            div {
                onclick: move |_| on_click.call(task_click.clone()),
                style: "flex: 1; display: flex; flex-direction: column; gap: 4px; min-width: 0; cursor: pointer;",
                class: "group",
            div {
                style: "display: flex; align-items: center; gap: 6px; overflow: hidden;",
                span {
                    style: "color: #f3f4f6; font-weight: 500; font-size: 15px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                    class: "group-hover:text-blue-400 transition-colors",
                    "{task.summary}"
                }
                // Rec symbol nur für master
                 if is_recurring {
                span {
                     title: "Master of a recurring todo",
                     style: "font-size: 12px; flex-shrink: 0;", // flex-shrink verhindert, dass das Icon zerdrückt wird
                    "🔄"
                    }
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 8px;",
                    if !display_date.is_empty() {
                        span {
                            style: "font-size: 12px; color: {date_color}; font-weight: {font_weight};",
                            "Due: {display_date}"
                        }
                    }
                    if let Some(label) = group_badge {
                        span {
                            style: "font-size: 10px; background: rgba(58, 107, 255, 0.15); color: #3A6BFF; padding: 2px 6px; border-radius: 4px; font-weight: 600; text-transform: uppercase;",
                            "{label}"
                        }
                    }
                    if let Some(label) = list_badge {
                        if uuid::Uuid::parse_str(&label).is_err() {
                            span {
                                style: "font-size: 10px; background: rgba(58, 107, 255, 0.15); color: #3A6BFF; padding: 2px 6px; border-radius: 4px; font-weight: 600; text-transform: uppercase;",
                                "{label}"
                            }
                        }
                    }
                    if let Some(evt_label) = event_badge {
                        span {
                            style: "font-size: 10px; background: rgba(255, 255, 255, 0.1); color: #9ca3af; padding: 2px 6px; border-radius: 4px; font-weight: 600; text-transform: uppercase;",
                            "{evt_label}"
                        }
                    }
                }
            }

            // Metadaten des Todos
            div {
                style: "display: flex; align-items: center; gap: 16px;",
                // Beschreibung
                div {
                    style: "text-align: right; display: flex; flex-direction: column; align-items: flex-end; min-width: 80px; max-width: 150px;",
                    span {
                        style: "font-size: 10px; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em;",
                        "Description"
                    }
                    span {
                        style: "font-size: 13px; color: {desc_color}; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; width: 100%; text-align: right;",
                        title: "{desc_text}",
                        "{desc_text}"
                    }
                }
                div { style: "width: 1px; height: 24px; background: rgba(255,255,255,0.1);" }
                // Prio
                div {
                    style: "text-align: right; display: flex; flex-direction: column; align-items: flex-end; min-width: 60px;",
                    span {
                        style: "font-size: 10px; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em;",
                        "Priority"
                    }
                    span {
                        style: "font-size: 13px; color: {priority_color}; font-weight: 600;",
                        "{priority_label}"
                    }
                }
                div { style: "width: 1px; height: 24px; background: rgba(255,255,255,0.1);" }
                // Zugewiesener User
                div {
                    style: "text-align: right; display: flex; flex-direction: column; align-items: flex-end; min-width: 80px;",
                    span {
                        style: "font-size: 10px; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em;",
                        "Assigned to"
                    }
                    span {
                        style: "font-size: 13px; color: {assignee_color}; font-weight: 500;",
                        "{assignee_label}"
                    }
                }
            }
        }
    }
}

fn categorize_todos(
    list: &Vec<TodoEventLight>,
) -> (
    Vec<TodoEventLight>,
    Vec<TodoEventLight>,
    Vec<TodoEventLight>,
) {
    //Datum und Einteilung
    let now_date = Local::now().date_naive();
    let next_week_date = now_date + Duration::days(7);
    //Vecs je nach Due Date erstellen
    let mut today = vec![];
    let mut week = vec![];
    let mut later = vec![];

    for todo in list {
        let todo_to_sort = todo.clone();
        // Alles Ungültige (None, leer, Parse-Fehler) wird hier zu None.
        let valid_date: Option<_> = db_to_display_only_date(&todo.due_datetime)
            .ok()
            .filter(|s| !s.is_empty())
            .and_then(|s| NaiveDate::parse_from_str(&s, "%d.%m.%Y").ok());

        // In jeweilige vecs pushen (Parse Fehler, None, oder leerer String wird zu later)
        if let Some(item_date) = valid_date {
            if item_date <= now_date {
                today.push(todo_to_sort);
            } else if item_date <= next_week_date {
                week.push(todo_to_sort);
            } else {
                later.push(todo_to_sort);
            }
        } else {
            later.push(todo_to_sort);
        }
    }
    today.sort_by(|a, b| a.due_datetime.cmp(&b.due_datetime));
    week.sort_by(|a, b| a.due_datetime.cmp(&b.due_datetime));
    later.sort_by(|a, b| a.due_datetime.cmp(&b.due_datetime));

    (today, week, later)
}
