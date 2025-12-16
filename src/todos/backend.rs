use dioxus::prelude::*;
use std::sync::{LazyLock, Mutex}; // für globalen State

#[server]
pub async fn fetch_my_groups() -> Result<Vec<(i32, String)>, ServerFnError> {
    // eigentliche Signatur: pub async fn fetch_my_groups() -> Result<Vec<(i32, String)>, ServerFnError>
    // Hier dann SQL Anfrage an Server

    //MOCK===========================
    let groups = MOCK_GROUPS.lock().unwrap();
    Ok(groups.iter().map(|g| (g.0, g.1.clone())).collect())
    //MOCK===========================
}

#[server]
pub async fn fetch_todos_filtered(filter_mode: i32) -> Result<Vec<ToDoTransfer>, ServerFnError> {
    // eigentliche Signatur: pub async fn fetch_todos_filtered(filter_mode: i32) -> Result<Vec<ToDo>, ServerFnError>
    // Hier dann SQL Anfrage an Server

    //MOCK===========================
    let todos = MOCK_TODOS.lock().unwrap();

    let filtered_data = todos.iter().filter(|t| match filter_mode {
        0 => !t.completed,
        -1 => !t.completed && t.group_id == 0,
        id if id > 0 => !t.completed && t.group_id == id,
        _ => false,
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

    let (group_name, group_color) = if group_id > 0 {
        if let Some(g) = groups.iter().find(|g| g.0 == group_id) {
            (Some(g.1.clone()), Some(g.2.clone()))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let new_task = ToDo {
        id: new_id,
        title,
        due_date: "Heute".to_string(),
        is_group: group_id > 0,
        completed: false,
        group_id,
        group_name,
        group_color,
    };

    todos.push(new_task);
    Ok(())
    //MOCK===========================
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
    //MOCK===========================
}

//=========================
// Mock Daten Struktur und funktionen
//=========================

type ToDoTransfer = (
    i32,
    String,
    String,
    bool,
    bool,
    i32,
    Option<String>,
    Option<String>,
);

#[derive(Clone, Debug, PartialEq)]
pub struct ToDo {
    pub id: i32,
    pub title: String,
    pub due_date: String,
    pub is_group: bool,
    pub completed: bool,
    pub group_id: i32,
    pub group_name: Option<String>,
    pub group_color: Option<String>,
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
        todo.group_color,
    )
}

// (ID, Name, HexColor)
static MOCK_GROUPS: LazyLock<Mutex<Vec<(i32, String, String)>>> = LazyLock::new(|| {
    Mutex::new(vec![
        (10, "Marketing Team".to_string(), "#A855F7".to_string()),
        (11, "Dev Squad".to_string(), "#3A6BFF".to_string()),
        (12, "Design Crew".to_string(), "#EC4899".to_string()),
        (13, "Finance & Ops".to_string(), "#10B981".to_string()),
        (14, "HR & People".to_string(), "#F59E0B".to_string()),
        (15, "Customer Support".to_string(), "#06B6D4".to_string()),
    ])
});

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
            group_color: None,
        },
        ToDo {
            id: 2,
            title: "Rust Tutorial beenden".into(),
            due_date: "18.12.2025".into(),
            is_group: false,
            completed: false,
            group_id: 0,
            group_name: None,
            group_color: None,
        },
        ToDo {
            id: 3,
            title: "Q4 Präsentation".into(),
            due_date: "20.12.2025".into(),
            is_group: true,
            completed: false,
            group_id: 10,
            group_name: Some("Marketing Team".into()),
            group_color: Some("#A855F7".into()),
        },
        ToDo {
            id: 4,
            title: "Server Deployment".into(),
            due_date: "21.12.2025".into(),
            is_group: true,
            completed: false,
            group_id: 11,
            group_name: Some("Dev Squad".into()),
            group_color: Some("#3A6BFF".into()),
        },
        ToDo {
            id: 5,
            title: "Website Mockups".into(),
            due_date: "22.12.2025".into(),
            is_group: true,
            completed: false,
            group_id: 12,
            group_name: Some("Design Crew".into()),
            group_color: Some("#EC4899".into()),
        },
        ToDo {
            id: 6,
            title: "Jahresabschluss prüfen".into(),
            due_date: "31.12.2025".into(),
            is_group: true,
            completed: false,
            group_id: 13,
            group_name: Some("Finance & Ops".into()),
            group_color: Some("#10B981".into()),
        },
        ToDo {
            id: 7,
            title: "Weihnachtsfeier Planen".into(),
            due_date: "15.12.2025".into(),
            is_group: true,
            completed: false,
            group_id: 14,
            group_name: Some("HR & People".into()),
            group_color: Some("#F59E0B".into()),
        },
        ToDo {
            id: 8,
            title: "Ticket #4092 Eskalation".into(),
            due_date: "Heute".into(),
            is_group: true,
            completed: false,
            group_id: 15,
            group_name: Some("Customer Support".into()),
            group_color: Some("#06B6D4".into()),
        },
        ToDo {
            id: 9,
            title: "Milch kaufen".into(),
            due_date: "14.12.2025".into(),
            is_group: false,
            completed: true,
            group_id: 0,
            group_name: None,
            group_color: None,
        },
        ToDo {
            id: 99,
            title: "Onboarding Prozess fixen".into(),
            due_date: "10.12.2025".into(),
            is_group: true,
            completed: true,
            group_id: 11,
            group_name: Some("Dev Squad".into()),
            group_color: Some("#3A6BFF".into()),
        },
    ])
});
