#![allow(unused_mut)]
#![allow(unused_imports)]

use super::create_edit_todo::{CreateEditToDoModal, CreateToDoButton};
use super::create_todolist::{CreateListButton, CreateListModal};
use super::filter_todos::{FilterSidebar, GroupFilter, ListFilter};
use super::open_todos::OpenToDoView;
use super::todo_detail::ToDoDetailModal;
use super::todo_history::HistoryView;
use crate::database::local::init_fetch::init_fetch_local_db::{
    fetch_calendar_events_lokal_db, fetch_calendars_lokal_db, fetch_group_members_lokal_db,
    fetch_groups_lokal_db, fetch_profiles_lokal_db, fetch_todo_events_lokal_db,
    fetch_todo_lists_lokal_db,
};
use crate::todos::backend::complete_todo::complete_todo_event;
use crate::todos::backend::handle_recurrence_todos::expand_recurring_todos;
use crate::utils::structs::{
    CalendarEventLight, CalendarLight, GroupLight, GroupMemberLight, ProfileLight, TodoEventLight,
    TodoListLight,
};

use chrono::Local;
use dioxus::prelude::*;
use std::str::FromStr;
use tokio::join;
use uuid::Uuid;

#[component]
pub fn ToDoDashboard() -> Element {
    let today_date = use_signal(|| Local::now().format("%A, %d.%m.%Y").to_string());

    //Standardwerte für ToDoView setzen
    let mut selected_category = use_signal(|| GroupFilter::AllGroups);
    let mut selected_list_filter = use_signal(|| ListFilter::AllLists);
    let mut show_create_todo_modal = use_signal(|| false);
    let mut show_create_list_modal = use_signal(|| false);
    let mut selected_todo_for_detail = use_signal(|| None::<TodoEventLight>);
    let mut todo_to_edit = use_signal(|| None::<TodoEventLight>);

    //leeres Set aus Tasks erstellen um u.a. nachher geladenen tasks trennen zu können in erledigt / nicht erledigt
    let mut tasks_signal = use_signal(|| Vec::<TodoEventLight>::new());

    //alle Daten aus lokaler Datenbank ziehen und joinen
    let mut full_data_resource = use_resource(move || async move {
        let results = join!(
            fetch_groups_lokal_db(),
            fetch_todo_lists_lokal_db(),
            fetch_todo_events_lokal_db(),
            fetch_profiles_lokal_db(),
            fetch_group_members_lokal_db(),
            fetch_calendar_events_lokal_db(),
            fetch_calendars_lokal_db()
        );
        results
    });

    //läuft sobald sich Abhänigkeiten ändern, lädt todo daten neu wenn full_data_resource fertig geladen hat und schreibt todos in tasks_signal
    use_effect(move || {
        if let Some((_, _, Ok(raw_tasks_from_db), _, _, _, _)) = &*full_data_resource.read() {
            let input_tasks = raw_tasks_from_db.clone();
            //recurrence handeln in todos
            match expand_recurring_todos(input_tasks) {
                Ok(expanded_tasks) => {
                    // Expanding klappt -> expandete tasks in tasks_signal setzen
                    //let test_tasks = expanded_tasks.clone();
                    tasks_signal.set(expanded_tasks);
                    //println!("DU SOLLTEST DOCH FUNKTIONIEREN :{:?}", test_tasks);
                }
                Err(e) => {
                    // wenn expanden nicht klappt unexpandete ausgeben
                    println!("Error expanding recurring tasks: {}", e);
                    tasks_signal.set(raw_tasks_from_db.clone());
                }
            }
        }
    });

    //solange datenbankdaten noch nicht geladen sind Ladeanimation
    if full_data_resource.read().is_none() {
        return rsx! {
            div {
                style: "width: 100%; height: 100%; background: #05060b; color: white; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 15px;",
                div { style: "width: 30px; height: 30px; border: 3px solid #3A6BFF; border-top-color: transparent; border-radius: 50%; animation: spin 1s linear infinite;" }
                p { style: "color: #6b7280; font-size: 14px;", "Lade To-Dos..." }
                style { "@keyframes spin {{ 0% {{ transform: rotate(0deg); }} 100% {{ transform: rotate(360deg); }} }}" }
            }
        };
    }

    //Hier die gejointen Datenbankdaten auseinander ziehen und in ein jeweils eigenen Vec //Todos oben ja schon
    let data_lock = full_data_resource.read();
    let (groups_res, lists_res, _, profiles_res, members_res, events_res, calendars_res) =
        data_lock.as_ref().unwrap();

    let groups_data = match groups_res {
        Ok(data) => data.clone(),
        Err(e) => {
            println!("Fehler Gruppen: {:?}", e);
            vec![]
        }
    };
    let lists_data = match lists_res {
        Ok(data) => data.clone(),
        Err(e) => {
            println!("Fehler Listen: {:?}", e);
            vec![]
        }
    };
    let profiles_data = match profiles_res {
        Ok(data) => data.clone(),
        Err(e) => {
            println!("Fehler Profile: {:?}", e);
            vec![]
        }
    };
    let members_data = match members_res {
        Ok(data) => data.clone(),
        Err(e) => {
            println!("Fehler Members: {:?}", e);
            vec![]
        }
    };
    let events_data = match events_res {
        Ok(data) => data.clone(),
        Err(e) => {
            println!("Fehler Events: {:?}", e);
            vec![]
        }
    };
    let calendars_data = match calendars_res {
        Ok(data) => data.clone(),
        Err(e) => {
            println!("Fehler Calendars: {:?}", e);
            vec![]
        }
    };

    //über ToDo-Events itterieren und erledigte sammeln
    let history_data: Vec<TodoEventLight> = tasks_signal
        .read()
        .iter()
        .filter(|t| t.completed)
        .cloned()
        .collect();
    //über ToDo-Events itterieren und offene sammeln
    let active_tasks_data: Vec<TodoEventLight> = tasks_signal
        .read()
        .iter()
        .filter(|t| !t.completed)
        .cloned()
        .collect();

    //Handler

    //Datenbankdaten neu laden
    let handle_refresh = move |_| {
        println!("Refreshing Data...");
        full_data_resource.restart();
    };

    // ToDo-Bearbeiten Handler
    let handle_edit_request = move |todo: TodoEventLight| {
        //Setzt Detailansicht eines ToDos auf aus
        selected_todo_for_detail.set(None);
        //setzt zum bearbeitendes Todo in dafür vorgesehene var
        todo_to_edit.set(Some(todo));
        // Eigabemaske für TODO-Erstellung anzeigen (unterscheidet dann zw. edit und create)
        show_create_todo_modal.set(true);
    };

    //Selected ToDo setzen
    let handle_select_todo = move |todo: TodoEventLight| {
        selected_todo_for_detail.set(Some(todo));
    };

    //Complete ToDo Handler
    let handle_complete_task = move |task_id: String| {
        let id_for_db = task_id.clone();
        // Supabase Update Senden
        spawn(async move {
            // String ID in echte UUID umwandeln
            if let Ok(uuid) = Uuid::from_str(&id_for_db) {
                match complete_todo_event(uuid).await {
                    Ok(_) => {
                        println!(
                            "Insert update for field 'completed' for Todo: {} done.",
                            id_for_db
                        );
                        full_data_resource.restart();
                    }
                    Err(e) => {
                        println!("Error on inserting completion update: {}", e);
                    }
                }
            } else {
                println!("Invalid id for todo: {}", id_for_db);
            }
        });
    };

    rsx! {
        div {
            style: "width: 100%; height: 100%; background: #05060b; display: flex; overflow: hidden; font-family: sans-serif; position: relative;",

            if show_create_todo_modal() {
                CreateEditToDoModal {
                    //create ToDo-Komponente rendern und Listen übergeben
                    groups: groups_data.clone(),
                    all_lists: lists_data.clone(),
                    all_profiles: profiles_data.clone(),
                    all_group_members: members_data.clone(),
                    show_modal: show_create_todo_modal,
                    on_refresh: handle_refresh,
                    todo_to_edit: todo_to_edit
                }
            }
            if show_create_list_modal() {
                //create Liste-Komponente rendern und Listen übergeben
                CreateListModal {
                    groups: groups_data.clone(),
                    all_events: events_data.clone(),
                    all_calendars: calendars_data.clone(),
                    show_modal: show_create_list_modal,
                    on_refresh: handle_refresh
                }
            }

            // Detailansicht von Todos rendern und ToDo und Listen übergeben
            ToDoDetailModal {
                selected_todo: selected_todo_for_detail,
                on_refresh: handle_refresh,
                on_edit: handle_edit_request,
                groups: groups_data.clone(),
                all_lists: lists_data.clone(),
                all_profiles: profiles_data.clone(),
            }

            div {
                //Sidebar-Komponente rendern und Listen übergeben
                style: "height: 100%; padding: 24px 0 24px 24px;",
                FilterSidebar {
                    groups: groups_data.clone(),
                    all_lists: lists_data.clone(),
                    selected_category: selected_category,
                    selected_list: selected_list_filter
                }
            }

            OpenToDoView {
                //Offene ToDos-Komponente rendern und Listen übergeben
                todos: active_tasks_data,
                all_lists: lists_data.clone(),
                groups: groups_data.clone(),
                all_profiles: profiles_data.clone(),
                all_events: events_data.clone(),
                selected_category: selected_category(),
                selected_list_filter: selected_list_filter(),
                on_complete: handle_complete_task,
                on_select_todo: handle_select_todo
            }

            div {
                style: "width: 320px; padding: 24px 24px 24px 0; display: flex; flex-direction: column; gap: 24px; background: #080910;",

                div {//Heutiges Datum
                    style: "background: linear-gradient(145deg, #222531 0%, #171923 100%); border-radius: 18px; padding: 18px; box-shadow: 0 22px 45px rgba(0,0,0,0.8); border: 1px solid rgba(255,255,255,0.06);",
                    h2 { style: "margin: 0 0 4px 0; font-size: 13px; letter-spacing: 0.08em; text-transform: uppercase; color: #9ca3af;", "Today" }
                    h3 { style: "margin: 0; font-size: 20px; font-weight: 600; color: #f9fafb;", "{today_date}" }
                }

                div {//Button Bereich
                    style: "background: linear-gradient(145deg, #222531 0%, #171923 100%); border-radius: 18px; padding: 18px; box-shadow: 0 22px 45px rgba(0,0,0,0.8); border: 1px solid rgba(255,255,255,0.06); display: flex; flex-direction: column; gap: 14px;",
                    h2 { style: "margin: 0; font-size: 13px; letter-spacing: 0.08em; text-transform: uppercase; color: #9ca3af;", "Actions" }

                    CreateToDoButton { onclick: move |_| {
                        //Setzt geöffnete Detailansicht auf aus
                        todo_to_edit.set(None);
                        //öffnet create todo eingabemaske
                        show_create_todo_modal.set(true);
                    }}
                    CreateListButton { onclick: move |_| //öffnet create List eingabemaske
                        show_create_list_modal.set(true) }
                }

                div {
                    style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                    //History-Komponente rendern und Listen übergeben
                    HistoryView {
                        history_tasks: history_data,
                        all_lists: lists_data.clone(),
                        all_groups: groups_data.clone(),
                    }
                }
            }
        }
    }
}
