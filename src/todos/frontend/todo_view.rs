#![allow(unused_mut)]

use super::create_todo::{CreateToDoButton, CreateToDoModal};
use super::create_todolist::{CreateListButton, CreateListModal};
use super::filter_todos::{FilterSidebar, GroupFilter, ListFilter};
use super::open_todos::OpenToDoView;
use super::todo_history::HistoryView;
use crate::todos::backend::{
    fetch_calendar_events, fetch_calendars, fetch_group_members, fetch_groups, fetch_profiles,
    fetch_todo_events, fetch_todo_lists, init_database,
};
use crate::utils::structs::{
    CalendarEventLight, CalendarLight, GroupLight, GroupMemberLight, ProfileLight, TodoEventLight,
    TodoListLight,
};
use chrono::Local;
use dioxus::prelude::*;
use sqlx::Error;
use tokio::join;

#[component]
pub fn ToDoView() -> Element {
    let today_date = use_signal(|| Local::now().format("%A, %d.%m.%Y").to_string());

    let mut selected_category = use_signal(|| GroupFilter::All);
    let mut selected_list_filter = use_signal(|| ListFilter::AllInContext);
    let mut show_create_todo_modal = use_signal(|| false);
    let mut show_create_list_modal = use_signal(|| false);
    let mut tasks_signal = use_signal(|| Vec::<TodoEventLight>::new());
    let mut full_data_resource = use_resource(move || async move {
        match init_database().await {
            Ok(_) => println!("Frontend: DB Init OK"),
            Err(e) => println!("Frontend: DB Init FEHLER: {:?}", e),
        };

        let results = join!(
            fetch_groups(),
            fetch_todo_lists(),
            fetch_todo_events(),
            fetch_profiles(),
            fetch_group_members(),
            fetch_calendar_events(),
            fetch_calendars()
        );

        results
    });

    use_effect(move || {
        if let Some((_, _, Ok(tasks), _, _, _, _)) = &*full_data_resource.read() {
            if tasks_signal.read().is_empty() {
                tasks_signal.set(tasks.clone());
            }
        }
    });

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

    let history_data: Vec<TodoEventLight> = tasks_signal
        .read()
        .iter()
        .filter(|t| t.completed)
        .cloned()
        .collect();
    let active_tasks_data: Vec<TodoEventLight> = tasks_signal
        .read()
        .iter()
        .filter(|t| !t.completed)
        .cloned()
        .collect();

    let handle_refresh_from_cache = move |_| {
        full_data_resource.restart();
    };

    let handle_complete_task = move |task_id: String| {
        let mut tasks = tasks_signal.write();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.completed = true;
            task.due_datetime = Some(Local::now().format("%Y-%m-%d").to_string());
        }
    };

    rsx! {
        div {
            style: "width: 100%; height: 100%; background: #05060b; display: flex; overflow: hidden; font-family: sans-serif; position: relative;",

            if show_create_todo_modal() {
                CreateToDoModal {
                    groups: groups_data.clone(),
                    all_lists: lists_data.clone(),
                    all_profiles: profiles_data.clone(),
                    all_group_members: members_data.clone(),
                    show_modal: show_create_todo_modal,
                    on_refresh: handle_refresh_from_cache
                }
            }
            if show_create_list_modal() {
                CreateListModal {
                    groups: groups_data.clone(),
                    all_events: events_data.clone(),
                    all_calendars: calendars_data.clone(),
                    show_modal: show_create_list_modal,
                    on_refresh: handle_refresh_from_cache
                }
            }

            div {
                style: "height: 100%; padding: 24px 0 24px 24px;",
                FilterSidebar {
                    groups: groups_data.clone(),
                    all_lists: lists_data.clone(),
                    selected_category: selected_category,
                    selected_list: selected_list_filter
                }
            }

            OpenToDoView {
                todos_list: active_tasks_data,
                all_lists: lists_data.clone(),
                groups: groups_data.clone(),
                all_profiles: profiles_data.clone(),
                selected_category: selected_category(),
                selected_list_filter: selected_list_filter(),
                on_complete: handle_complete_task
            }

            div {
                style: "width: 320px; padding: 24px 24px 24px 0; display: flex; flex-direction: column; gap: 24px; background: #080910;",

                div {
                    style: "background: linear-gradient(145deg, #222531 0%, #171923 100%); border-radius: 18px; padding: 18px; box-shadow: 0 22px 45px rgba(0,0,0,0.8); border: 1px solid rgba(255,255,255,0.06);",
                    h2 { style: "margin: 0 0 4px 0; font-size: 13px; letter-spacing: 0.08em; text-transform: uppercase; color: #9ca3af;", "Today" }
                    h3 { style: "margin: 0; font-size: 20px; font-weight: 600; color: #f9fafb;", "{today_date}" }
                }

                div {
                    style: "background: linear-gradient(145deg, #222531 0%, #171923 100%); border-radius: 18px; padding: 18px; box-shadow: 0 22px 45px rgba(0,0,0,0.8); border: 1px solid rgba(255,255,255,0.06); display: flex; flex-direction: column; gap: 14px;",
                    h2 { style: "margin: 0; font-size: 13px; letter-spacing: 0.08em; text-transform: uppercase; color: #9ca3af;", "Actions" }
                    CreateToDoButton { onclick: move |_| show_create_todo_modal.set(true) }
                    CreateListButton { onclick: move |_| show_create_list_modal.set(true) }
                }

                div {
                    style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
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
