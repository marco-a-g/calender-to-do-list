#![allow(dead_code)]
#![allow(unused_imports)]

use dioxus::prelude::*;
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};
use sqlx::{
    ConnectOptions,
    sqlite::{SqlitePool, SqlitePoolOptions},
};
use std::str::FromStr;
use supabase::Client;

use super::sync::calendars::sync_calendars_and_events;
use super::sync::groups::sync_groups_and_members;
use super::sync::profiles::sync_profiles;
use super::sync::todos::sync_todos;

// Config -> Später raus sobald auth steht?
const SUPABASE_URL: &str = "https://tixtjdlkhnnxvneduxvb.supabase.co";
const SUPABASE_ANON_KEY: &str = "sb_publishable_NjcEUTm3xiKWvKkrcLBJsQ_tPIEgO90";
const MOCK_EMAIL: &str = "sarah.dev@example.com";
const MOCK_PASSWORD: &str = "passwort123";
const MOCK_USER_ID: &str = "d8671d1a-4262-446f-9f08-e8ffc802da73";
/* Mock User Zugangsdaten:
    Passwort für alle: passwort123
    Sarah, Email: sarah.dev@example.com, id: 730485e6-83c2-41c6-bdd7-6677269b4ae9
    Mike, Email: mike.po@example.com, uid: 8414c78d-5de8-46ba-8457-0833ccd86939
    Emma , Email: emma.mkt@example.com, uid: ce2c830e-fc8a-4f09-830b-95cc4727358f
    Tom, Email: tom.design@example.com, uid: 68b4a3af-55ab-4c8a-9af9-75be3f88672d
    Lisa, Email: lisa.intern@example.com, uid: d6e0f119-ecfe-4e70-bf43-4044162a3d92
*/

// Data-Stucts; später eher global wo definieren
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub owner_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: String,
    pub user_id: String,
    pub group_id: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub calendar_type: String,
    pub description: Option<String>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
    pub last_mod: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub calendar_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub date: String,
    pub from_time: Option<String>,
    pub to_time: Option<String>,
    pub seq: bool,
    pub rrule: Option<String>,
    pub last_mod: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoList {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub list_type: String,
    pub description: Option<String>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
    pub due_datetime: Option<String>,
    pub priority: Option<String>,
    pub last_mod: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoEvent {
    pub id: String,
    pub todo_list_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub completed: bool,
    pub due_datetime: Option<String>,
    pub priority: Option<String>,
    pub seq: bool,
    pub last_mod: String,
}

// Sync

//Sync api function
#[server]
pub async fn sync_remote_to_local() -> Result<(), ServerFnError> {
    sync_function().await
}

// Sync Logik
pub async fn sync_function() -> Result<(), ServerFnError> {
    println!("Start sync for User: {}", MOCK_USER_ID);

    //Client aufsetzen, Später dann in main?
    let client = Client::new(SUPABASE_URL, SUPABASE_ANON_KEY)
        .map_err(|e| ServerFnError::new(format!("Supabase Init Error: {}", e)))?;

    println!("Login successful!");

    //Pfad local DB
    let db_path = "sqlite:src/database/local/local_Database.db";

    //Connectionoptions; Foreign Keys deaktivieren sonst geht es nicht? Keine Ahnung...
    let connect_options_local_db =
        sqlx::sqlite::SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))
            .map_err(|e| ServerFnError::new(format!("Path Error: {}", e)))?
            .create_if_missing(true)
            .foreign_keys(false)
            .disable_statement_logging(); //sonst logging vorerst zu ausführlich

    // connection zur local db mit error
    let pool_local_db = SqlitePoolOptions::new()
        .connect_with(connect_options_local_db)
        .await
        .map_err(|e| ServerFnError::new(format!("DB Connect Error: {}.", e)))?;

    //öffnet "Änderungs-Warteschlange", tx = transaction, läuft querys ab hier durch und ändert erst ab tx.commit die Inhalte, bisschen wie ein Lock
    let mut transaction_queue = pool_local_db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Transaction Error: {}", e)))?;

    //Profile synchronisieren
    sync_profiles(&client, &mut transaction_queue).await?;

    //Gruppen und Mitglieder synchronisieren
    let user_group_ids =
        sync_groups_and_members(&client, &mut transaction_queue, MOCK_USER_ID).await?;

    // Kalender und Events synchronisieren
    sync_calendars_and_events(
        &client,
        &mut transaction_queue,
        MOCK_USER_ID,
        &user_group_ids,
    )
    .await?;

    // To-Do Listen und Einträge synchronisieren
    sync_todos(
        &client,
        &mut transaction_queue,
        MOCK_USER_ID,
        &user_group_ids,
    )
    .await?;

    //Hier Änderungsqueue zusammenfügen und "commiten"
    transaction_queue
        .commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Commit Error: {}", e)))?;

    println!("Sync completed");
    Ok(())
}
