#![allow(unused_mut)]

use crate::todos::backend::{
    complete_task, fetch_completed_history, fetch_groups, fetch_todos_filtered,
};
use chrono::Local;
use dioxus::prelude::*;

use super::create_todo::CreateModal;
use super::filter_todos::FilterView;
use super::open_todos::TaskListView;
use super::todo_history::HistoryView;
use crate::todos::frontend::filter_todos::FilterState;

#[component]
pub fn ToDoView() -> Element {
    let today_date = use_signal(|| Local::now().format("%A, %d.%m.%Y").to_string());
    let mut selected_filter = use_signal(|| FilterState::All);
    let mut show_create_modal = use_signal(|| false);
    let groups = use_resource(fetch_groups);

    let mut todos_resource = use_resource(move || {
        let mode = match selected_filter() {
            FilterState::All => 0,
            FilterState::Personal => -1,
            FilterState::Group(id) => id,
        };
        fetch_todos_filtered(mode)
    });
    let mut history = use_resource(fetch_completed_history);

    let current_groups = match &*groups.read() {
        Some(Ok(list)) => list.clone(),
        _ => vec![],
    };

    let history_data = match &*history.read() {
        Some(Ok(list)) => list.clone(),
        _ => vec![],
    };

    let todos_data = match &*todos_resource.read() {
        Some(Ok(list)) => list.clone(),
        _ => vec![],
    };

    let handle_complete = move |id: i32| async move {
        let _ = complete_task(id).await;
        todos_resource.restart();
        history.restart();
    };

    rsx! {
        div {
            style:
            "width: 100%;
             height: 100%; 
             background: #05060b; 
             display: flex; 
             overflow: hidden; 
             font-family: sans-serif; 
             position: relative; 
             overscroll-behavior: none;",

            if show_create_modal() {
                CreateModal {
                    groups: current_groups.clone(),
                    show_modal: show_create_modal,
                    on_refresh: move |_| todos_resource.restart()
                }
            }

            FilterView {
                groups: current_groups,
                selected_filter: selected_filter
            }

            TaskListView {
                todos_list: todos_data,
                selected_filter: selected_filter(),
                on_complete: handle_complete
            }

            HistoryView {
                today_date: today_date(),
                history_data: history_data,
                on_open_create: move |_| show_create_modal.set(true)
            }
        }
    }
}
