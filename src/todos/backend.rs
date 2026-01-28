#![allow(dead_code)]

use crate::utils::structs::{
    CalendarEventLight, CalendarLight, GroupLight, GroupMemberLight, ProfileLight, TodoEventLight,
    TodoListLight,
};
use chrono::{DateTime, Local, NaiveDate};
use dioxus::{events, prelude::*};
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions},
};
use std::str::FromStr;
use std::sync::{LazyLock, Mutex, OnceLock};

// Local DB-Config
const DB_PATH: &str = "src/database/local/local_Database.db";
static CONNECTION_OPTIONS: LazyLock<SqliteConnectOptions> = LazyLock::new(|| {
    let url = format!("sqlite:{}", DB_PATH);
    SqliteConnectOptions::from_str(&url).expect("Fehler: DB URL Format falsch")
});
pub static POOL_LOCAL_DB: OnceLock<SqlitePool> = OnceLock::new();

//#[server]
pub async fn init_database() -> Result<(), ServerFnError> {
    if POOL_LOCAL_DB.get().is_some() {
        return Ok(());
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(7) //7 fetch funktionen zum Daten ziehen
        .connect_with(CONNECTION_OPTIONS.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("Konnte DB nicht verbinden: {}", e)))?;

    let _ = POOL_LOCAL_DB.set(pool); //OnceLock setzen, jetzt nicht mehr änderbar
    println!("Lokale Datenbank verbunden");

    Ok(())
}

fn get_pool() -> Result<&'static sqlx::SqlitePool, ServerFnError> {
    POOL_LOCAL_DB
        .get()
        .ok_or_else(|| ServerFnError::new("Datenbank getter fehlgeschlagen"))
}

//#[server]
pub async fn fetch_groups() -> Result<Vec<GroupLight>, ServerFnError> {
    let pool = get_pool()?;
    let groups = sqlx::query_as::<_, GroupLight>("SELECT * FROM groups")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Groups): {}", e)))?;
    Ok(groups)
}

//#[server]
pub async fn fetch_todo_lists() -> Result<Vec<TodoListLight>, ServerFnError> {
    let pool = get_pool()?;
    //hier anders weil in lokaler db heißt coloumn type und in structs list_type
    let sql = r#"
        SELECT *, 
               type AS list_type 
        FROM todo_lists
    "#;
    let lists = sqlx::query_as::<_, TodoListLight>(sql)
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Todo Lists): {}", e)))?;
    Ok(lists)
}

//#[server]
pub async fn fetch_todo_events() -> Result<Vec<TodoEventLight>, ServerFnError> {
    let pool = get_pool()?;
    let tasks = sqlx::query_as::<_, TodoEventLight>("SELECT * FROM todo_events")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Todo Events): {}", e)))?;
    Ok(tasks)
}

//#[server]
pub async fn fetch_group_members() -> Result<Vec<GroupMemberLight>, ServerFnError> {
    let pool = get_pool()?;
    let members = sqlx::query_as::<_, GroupMemberLight>("SELECT * FROM group_members")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Members): {}", e)))?;
    Ok(members)
}

//#[server]
pub async fn fetch_profiles() -> Result<Vec<ProfileLight>, ServerFnError> {
    let pool = get_pool()?;
    let profiles = sqlx::query_as::<_, ProfileLight>("SELECT * FROM profiles")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Profiles): {}", e)))?;
    Ok(profiles)
}

//#[server]
pub async fn fetch_calendar_events() -> Result<Vec<CalendarEventLight>, ServerFnError> {
    let pool = get_pool()?;
    let events = sqlx::query_as::<_, CalendarEventLight>("SELECT * FROM calendar_events")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Calendar Events): {}", e)))?;
    Ok(events)
}

//#[server]
pub async fn fetch_calendars() -> Result<Vec<CalendarLight>, ServerFnError> {
    let pool = get_pool()?;
    let calendars =
        sqlx::query_as::<_, CalendarLight>("SELECT *, type AS calendar_type FROM calendars")
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("SQL Fehler (Calendar): {}", e)))?;
    Ok(calendars)
}

//#[server]
pub async fn create_todo_event(todo: TodoEventLight) -> Result<(), ServerFnError> {
    //Hier Insert zu Remote-DB
    println!("Create Todo Server Funktion wurde aufgerufen{:?}", todo);
    //damit Server function akzeptiert wird-----
    let x = 1;
    match x {
        1 => Ok(()),
        _ => Err(ServerFnError::new("")),
    }
    //-------------------------------------------
}

//#[server]
pub async fn create_todo_list(list: TodoListLight) -> Result<(), ServerFnError> {
    //Hier Insert zu Remote-DB
    println!(
        "Create Todo-List Server Funktion wurde aufgerufen mit: {:?}",
        list
    );
    //damit Server function akzeptiert wird-----
    let x = 1;
    match x {
        1 => Ok(()),
        _ => Err(ServerFnError::new("")),
    }
    //-------------------------------------------
}

/* //Helper:
// Wandelt Timestamp in deutsche Darstellung um.
pub fn format_timestamp(raw_ts: &str) -> String {
    match DateTime::parse_from_rfc3339(&raw_ts) {
        Ok(dt_utc) => {
            let dt_local: DateTime<Local> = DateTime::from(dt_utc);
            dt_local.format("%d.%m.%Y %H:%M").to_string()
        }
        Err(_) => "Ungültiges Datum".to_string(),
    }
} */
