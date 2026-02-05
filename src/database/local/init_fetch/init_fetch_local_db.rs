use crate::utils::structs::{
    CalendarEventLight, CalendarLight, GroupLight, GroupMemberLight, ProfileLight, TodoEventLight,
    TodoListLight,
};
use dioxus::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::{LazyLock, OnceLock};

// Local DB-Config
const DB_PATH: &str = "src/database/local/local_Database.db";
static CONNECTION_OPTIONS: LazyLock<SqliteConnectOptions> = LazyLock::new(|| {
    let url = format!("sqlite:{}", DB_PATH);
    SqliteConnectOptions::from_str(&url)
        .expect("Fehler: DB URL Format falsch")
        .create_if_missing(true)
});
pub static POOL_LOCAL_DB: OnceLock<SqlitePool> = OnceLock::new();

//#[server]
pub async fn init_database() -> Result<(), ServerFnError> {
    if POOL_LOCAL_DB.get().is_some() {
        return Ok(());
    }
    //====der Block kann bei finaler Verssion dann raus eigentlich, bisher ist es einfacher Änderungen in lokaler db-struktur hier im SQL skript zu ändern=====
    println!("Initialisiere reset lokale Datenbankdateien...");
    let db_path = Path::new(DB_PATH);
    if db_path.exists() {
        if let Err(e) = fs::remove_file(db_path) {
            eprintln!("Fehler: alte db nicht gelöscht: {}", e);
        } else {
            println!("Alte Datenbank gelöscht.");
        }
    }
    //==============================================================================================================================================================
    let pool = SqlitePoolOptions::new()
        .max_connections(7) //7 fetch funktionen zum Daten ziehen
        .connect_with(CONNECTION_OPTIONS.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("Konnte DB nicht verbinden: {}", e)))?;
    println!("Lokale Datenbank verbunden");

    let schema_sql = r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS profiles (
            id TEXT PRIMARY KEY NOT NULL,
            username TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL DEFAULT 'New Group',
            owner_id TEXT NOT NULL,
            created_by TEXT,
            created_at TEXT NOT NULL,
            color TEXT NULL DEFAULT '#3A6BFF',
            FOREIGN KEY (owner_id) REFERENCES profiles(id) ON DELETE CASCADE,
            FOREIGN KEY (created_by) REFERENCES profiles(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS group_members (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            group_id TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('owner', 'admin', 'member', 'invited')) DEFAULT 'member',
            joined_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES profiles(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS calendars (
            id TEXT PRIMARY KEY NOT NULL,
            type TEXT NOT NULL CHECK(type IN ('private', 'group')) DEFAULT 'private',
            owner_id TEXT,
            group_id TEXT,
            name TEXT NOT NULL DEFAULT 'New Calendar',
            description TEXT,
            created_at TEXT NOT NULL,
            created_by TEXT,
            last_mod TEXT NOT NULL,
            FOREIGN KEY (owner_id) REFERENCES profiles(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
            FOREIGN KEY (created_by) REFERENCES profiles(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS calendar_events (
            id TEXT PRIMARY KEY NOT NULL,
            summary TEXT NOT NULL DEFAULT 'New Event',
            description TEXT,
            created_at TEXT NOT NULL,
            created_by TEXT,
            calendar_id TEXT NOT NULL,
            from_date_time TEXT NOT NULL,
            to_date_time TEXT,
            attachment TEXT,
            last_mod TEXT NOT NULL,
            rrule TEXT CHECK(rrule IN ('daily', 'weekly', 'fortnight', 'weekdays', 'monthly_on_weekday', 'monthly_on_date', 'annual')),
            recurrence_id TEXT,
            recurrence_until TEXT,
            location TEXT,
            category TEXT,
            is_all_day INTEGER NOT NULL DEFAULT 0, 
            overrides_datetime TEXT, 
            skipped INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (calendar_id) REFERENCES calendars(id) ON DELETE CASCADE,
            FOREIGN KEY (created_by) REFERENCES profiles(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS todo_lists (
            id TEXT PRIMARY KEY NOT NULL,
            type TEXT NOT NULL CHECK(type IN ('private', 'group')) DEFAULT 'private',
            owner_id TEXT,
            group_id TEXT,
            name TEXT NOT NULL DEFAULT 'New ToDo-List',
            description TEXT,
            created_at TEXT NOT NULL,
            created_by TEXT,
            due_datetime TEXT,
            priority TEXT CHECK(priority IN ('low', 'normal', 'high', 'top')) DEFAULT 'normal',
            last_mod TEXT NOT NULL,
            rrule TEXT CHECK(rrule IN ('daily', 'weekly', 'fortnight', 'weekdays', 'monthly_on_weekday', 'monthly_on_date', 'annual')),
            recurrence_id TEXT,
            recurrence_until TEXT,
            attached_to_calendar_event TEXT, 
            attachment TEXT, 
            overrides_datetime TEXT, 
            skipped INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (owner_id) REFERENCES profiles(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
            FOREIGN KEY (created_by) REFERENCES profiles(id) ON DELETE SET NULL,
            FOREIGN KEY (attached_to_calendar_event) REFERENCES calendar_events(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS todo_events (
            id TEXT PRIMARY KEY NOT NULL,
            summary TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL,
            created_by TEXT,
            todo_list_id TEXT NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0,
            due_datetime TEXT,
            priority TEXT NOT NULL CHECK(priority IN ('low', 'normal', 'high', 'top')) DEFAULT 'normal',
            attachment TEXT,
            last_mod TEXT NOT NULL,
            rrule TEXT CHECK(rrule IN ('daily', 'weekly', 'fortnight', 'weekdays', 'monthly_on_weekday', 'monthly_on_date', 'annual')),
            recurrence_id TEXT,
            recurrence_until TEXT,
            assigned_to_user TEXT, 
            overrides_datetime TEXT, 
            skipped INTEGER NOT NULL DEFAULT 0, 
            FOREIGN KEY (created_by) REFERENCES profiles(id) ON DELETE SET NULL,
            FOREIGN KEY (todo_list_id) REFERENCES todo_lists(id) ON DELETE CASCADE
        );
    "#;

    sqlx::query(schema_sql)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Fehler beim Erstellen der Tabellen: {}", e)))?;

    let _ = POOL_LOCAL_DB.set(pool); //OnceLock setzen, jetzt nicht mehr änderbar
    println!("Lokale Datenbank verbunden und Tabellen erstellt.");
    Ok(())
}

fn get_pool_lokal_db() -> Result<&'static sqlx::SqlitePool, ServerFnError> {
    POOL_LOCAL_DB
        .get()
        .ok_or_else(|| ServerFnError::new("Datenbank getter fehlgeschlagen"))
}

//#[server]
pub async fn fetch_groups_lokal_db() -> Result<Vec<GroupLight>, ServerFnError> {
    let pool = get_pool_lokal_db()?;
    let groups = sqlx::query_as::<_, GroupLight>("SELECT * FROM groups")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Groups): {}", e)))?;
    Ok(groups)
}

//#[server]
pub async fn fetch_todo_lists_lokal_db() -> Result<Vec<TodoListLight>, ServerFnError> {
    let pool = get_pool_lokal_db()?;
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
pub async fn fetch_todo_events_lokal_db() -> Result<Vec<TodoEventLight>, ServerFnError> {
    let pool = get_pool_lokal_db()?;
    let tasks = sqlx::query_as::<_, TodoEventLight>("SELECT * FROM todo_events")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Todo Events): {}", e)))?;
    Ok(tasks)
}

//#[server]
pub async fn fetch_group_members_lokal_db() -> Result<Vec<GroupMemberLight>, ServerFnError> {
    let pool = get_pool_lokal_db()?;
    let members = sqlx::query_as::<_, GroupMemberLight>("SELECT * FROM group_members")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Members): {}", e)))?;
    Ok(members)
}

//#[server]
pub async fn fetch_profiles_lokal_db() -> Result<Vec<ProfileLight>, ServerFnError> {
    let pool = get_pool_lokal_db()?;
    let profiles = sqlx::query_as::<_, ProfileLight>("SELECT * FROM profiles")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Profiles): {}", e)))?;
    Ok(profiles)
}

//#[server]
pub async fn fetch_calendar_events_lokal_db() -> Result<Vec<CalendarEventLight>, ServerFnError> {
    let pool = get_pool_lokal_db()?;
    let events = sqlx::query_as::<_, CalendarEventLight>("SELECT * FROM calendar_events")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Fehler (Calendar Events): {}", e)))?;
    Ok(events)
}

//#[server]
pub async fn fetch_calendars_lokal_db() -> Result<Vec<CalendarLight>, ServerFnError> {
    let pool = get_pool_lokal_db()?;
    let calendars =
        sqlx::query_as::<_, CalendarLight>("SELECT *, type AS calendar_type FROM calendars")
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("SQL Fehler (Calendar): {}", e)))?;
    Ok(calendars)
}
