use chrono::{Local, NaiveDate};
use dioxus::prelude::*;
use std::sync::{LazyLock, Mutex};

#[server]
pub async fn fetch_groups() -> Result<Vec<(i32, String)>, ServerFnError> {
    //Eigentlich hier SQL Abfrage an Server-DB
    //MOCK===========================
    let groups = MOCK_GROUPS.lock().unwrap();
    Ok(groups.iter().map(|g| (g.0, g.1.clone())).collect())
    //MOCK===========================
}

#[server]
pub async fn fetch_todos_filtered(filter_mode: i32) -> Result<Vec<ToDoTransfer>, ServerFnError> {
    //Eigentlich hier SQL Abfrage an Server-DB
    //MOCK===========================
    let todos = MOCK_TODOS.lock().unwrap();
    let mut filtered_data: Vec<ToDo> = todos
        .iter()
        .filter(|t| match filter_mode {
            0 => !t.completed,
            -1 => !t.completed && t.group_id == 0,
            id if id > 0 => !t.completed && t.group_id == id,
            _ => false,
        })
        .cloned()
        .collect();

    filtered_data.sort_by(|a, b| {
        let date_a = parse_date_sortable(Some(&a.due_date));
        let date_b = parse_date_sortable(Some(&b.due_date));
        date_a.cmp(&date_b)
    });
    Ok(filtered_data.into_iter().map(to_transfer).collect())
    //MOCK===========================
}

#[server]
pub async fn fetch_completed_history() -> Result<Vec<ToDoTransfer>, ServerFnError> {
    //Eigentlich hier SQL Abfrage an Server-DB
    //MOCK===========================
    let todos = MOCK_TODOS.lock().unwrap();

    let mut history: Vec<ToDo> = todos.iter().filter(|t| t.completed).cloned().collect();

    history.sort_by(|a, b| {
        let date_a = parse_date_sortable(a.completed_date.as_deref());
        let date_b = parse_date_sortable(b.completed_date.as_deref());
        date_b.cmp(&date_a)
    });

    Ok(history.into_iter().map(to_transfer).collect())
    //MOCK===========================
}

// Hilfsfunktion zum Sortieren deutscher Daten (DD.MM.YYYY)??
fn parse_date_sortable(date_str: Option<&str>) -> NaiveDate {
    match date_str {
        Some("Heute") => Local::now().date_naive(),
        Some(s) => NaiveDate::parse_from_str(s, "%d.%m.%Y")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
        None => NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
    }
}

#[server]
pub async fn create_todo(
    title: String,
    group_id: i32,
    due_date: String,
) -> Result<(), ServerFnError> {
    //Eigentlich hier SQL Abfrage an Server-DB
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
        due_date,
        is_group: group_id > 0,
        completed: false,
        completed_date: None,
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
    //Eigentlich hier SQL Abfrage an Server-DB
    //MOCK===========================
    let mut todos = MOCK_TODOS.lock().unwrap();

    if let Some(task) = todos.iter_mut().find(|t| t.id == id) {
        task.completed = true;
        task.completed_date = Some(Local::now().format("%d.%m.%Y").to_string());
        println!("Completed task: {:?}", task);
    }

    Ok(())
    //MOCK===========================
}

//=========================
// Mock Daten Struktur und funktionen // kann nachher alls raus
//=========================

pub type ToDoTransfer = (
    i32,
    String,
    String,
    bool,
    bool,
    i32,
    Option<String>,
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
    pub completed_date: Option<String>,
    pub group_id: i32,
    pub group_name: Option<String>,
    pub group_color: Option<String>,
}

pub fn to_transfer(todo: ToDo) -> ToDoTransfer {
    (
        todo.id,
        todo.title,
        todo.due_date,
        todo.is_group,
        todo.completed,
        todo.group_id,
        todo.group_name,
        todo.group_color,
        todo.completed_date,
    )
}

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
            completed_date: None,
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
            completed_date: None,
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
            completed_date: None,
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
            completed_date: None,
            group_id: 11,
            group_name: Some("Dev Squad".into()),
            group_color: Some("#3A6BFF".into()),
        },
        ToDo {
            id: 9,
            title: "Milch kaufen".into(),
            due_date: "14.12.2025".into(),
            is_group: false,
            completed: true,
            completed_date: Some("14.12.2025".into()),
            group_id: 0,
            group_name: None,
            group_color: None,
        },
        ToDo {
            id: 10,
            title: "SEP Abgabe".into(),
            due_date: "14.12.2026".into(),
            is_group: true,
            completed: false,
            completed_date: None,
            group_id: 11,
            group_name: Some("Dev Squad".into()),
            group_color: Some("#3A6BFF".into()),
        },
        ToDo {
            id: 99,
            title: "Onboarding Prozess fixen".into(),
            due_date: "10.12.2025".into(),
            is_group: true,
            completed: true,
            completed_date: Some("11.12.2025".into()),
            group_id: 11,
            group_name: Some("Dev Squad".into()),
            group_color: Some("#3A6BFF".into()),
        },
    ])
});
