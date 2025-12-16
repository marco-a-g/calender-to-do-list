use dioxus::prelude::*;
use std::sync::{LazyLock, Mutex}; // für globalen State

#[server]
pub async fn fetch_my_groups() -> Result<Vec<(i32, String)>, ServerFnError> {
    // eigentliche Signatur: pub async fn fetch_my_groups() -> Result<Vec<(i32, String)>, ServerFnError>
    // Hier dann SQL Anfrage an Server

    //MOCK===========================
    let groups = MOCK_GROUPS.lock().unwrap();
    Ok(groups.clone())
    //MOCK===========================
}

#[server]
pub async fn fetch_todos_filtered(filter_mode: i32) -> Result<Vec<ToDoTransfer>, ServerFnError> {
    // eigentliche Signatur: pub async fn fetch_todos_filtered(filter_mode: i32) -> Result<Vec<ToDo>, ServerFnError>
    // Hier dann SQL Anfrage an Server

    //MOCK===========================
    let todos = MOCK_TODOS.lock().unwrap();

    let filtered_data = todos.iter().filter(|t| {
        match filter_mode {
            0 => !t.completed,                                // All (nur offene)
            -1 => !t.completed && t.group_id == 0,            // Personal
            id if id > 0 => !t.completed && t.group_id == id, // Specific Group
            _ => false,
        }
    });

    Ok(filtered_data.map(|t| to_transfer(t.clone())).collect())
    //MOCK===========================
}

#[server]
pub async fn fetch_completed_history() -> Result<Vec<ToDoTransfer>, ServerFnError> {
    // eigentliche Signatur: pub async fn fetch_completed_history() -> Result<Vec<ToDo>, ServerFnError>
    // Hier dann SQL Anfrage an Server

    //MOCK===========================
    let todos = MOCK_TODOS.lock().unwrap();
    let history = todos.iter().filter(|t| t.completed);
    Ok(history.map(|t| to_transfer(t.clone())).collect())
    //MOCK===========================
}

#[server]
pub async fn create_todo(title: String, group_id: i32) -> Result<(), ServerFnError> {
    // eigentliche Signatur: pub async fn create_todo(title: String, group_id: i32) -> Result<(), ServerFnError>
    // Hier dann SQL Anfrage an Server

    //MOCK===========================
    let mut todos = MOCK_TODOS.lock().unwrap();
    let groups = MOCK_GROUPS.lock().unwrap();

    let new_id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;

    let group_name = if group_id > 0 {
        groups.iter().find(|g| g.0 == group_id).map(|g| g.1.clone())
    } else {
        None
    };

    let new_task = ToDo {
        id: new_id,
        title,
        due_date: "Heute".to_string(),
        is_group: group_id > 0,
        completed: false,
        group_id,
        group_name,
    };

    todos.push(new_task);
    Ok(())
}

#[server]
pub async fn complete_task(id: i32) -> Result<(), ServerFnError> {
    // eigentliche Signatur: pub async fn complete_task(id: i32) -> Result<(), ServerFnError>
    // Hier dann SQL Anfrage an Server

    //MOCK===========================
    let mut todos = MOCK_TODOS.lock().unwrap();

    if let Some(task) = todos.iter_mut().find(|t| t.id == id) {
        task.completed = true;
        println!("Completed task: {:?}", task);
    }

    Ok(())
}

//=========================
// Mock Daten Struktur und funktionen
//=========================

// Transfer Type: (id, title, due_date, is_group, completed, group_id, group_name)
type ToDoTransfer = (i32, String, String, bool, bool, i32, Option<String>);

#[derive(Clone, Debug, PartialEq)]
pub struct ToDo {
    pub id: i32,
    pub title: String,
    pub due_date: String,
    pub is_group: bool,
    pub completed: bool,
    pub group_id: i32,
    pub group_name: Option<String>,
}

fn to_transfer(todo: ToDo) -> ToDoTransfer {
    (
        todo.id,
        todo.title,
        todo.due_date,
        todo.is_group,
        todo.completed,
        todo.group_id,
        todo.group_name,
    )
}

static MOCK_GROUPS: LazyLock<Mutex<Vec<(i32, String)>>> = LazyLock::new(|| {
    Mutex::new(vec![
        (10, "Marketing Team".to_string()),
        (11, "Dev Squad".to_string()),
    ])
});

// GLOBALE VARIABLE für ToDos (Mutable State)
static MOCK_TODOS: LazyLock<Mutex<Vec<ToDo>>> = LazyLock::new(|| {
    Mutex::new(vec![
        ToDo {
            id: 1,
            title: "Zahnarzt Termin".into(),
            due_date: "16.12.2025".into(),
            is_group: false,
            completed: false,
            group_id: 0,
            group_name: None,
        },
        ToDo {
            id: 2,
            title: "Rust Tutorial beenden".into(),
            due_date: "18.12.2025".into(),
            is_group: false,
            completed: false,
            group_id: 0,
            group_name: None,
        },
        ToDo {
            id: 3,
            title: "Q4 Präsentation".into(),
            due_date: "20.12.2025".into(),
            is_group: true,
            completed: false,
            group_id: 10,
            group_name: Some("Marketing".into()),
        },
        ToDo {
            id: 4,
            title: "Server Deployment".into(),
            due_date: "21.12.2025".into(),
            is_group: true,
            completed: false,
            group_id: 11,
            group_name: Some("Dev Squad".into()),
        },
        ToDo {
            id: 9,
            title: "Milch kaufen".into(),
            due_date: "14.12.2025".into(),
            is_group: false,
            completed: true,
            group_id: 0,
            group_name: None,
        },
    ])
});
