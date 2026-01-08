use dioxus::prelude::*;
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
const SUPABASE_URL: &str = "https://wyqawnnkpusgtnhmeebn.supabase.co";
const SUPABASE_Service_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Ind5cWF3bm5rcHVzZ3RuaG1lZWJuIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc2NTg0MzkyOSwiZXhwIjoyMDgxNDE5OTI5fQ.s3Gmfv0u89h5ZjguByboQbfjPADR3p9iVfcIeYyAoFY";
const MOCK_USER_ID: &str = "0cbc387b-984c-4dde-9c7c-281a07d4ce39"; //User Sarah

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

    //Client aufsetzen
    let client = Client::new(SUPABASE_URL, SUPABASE_Service_KEY)
        .map_err(|e| ServerFnError::new(format!("Supabase Init Error: {}", e)))?;

    //Pfad local DB
    let db_path = "sqlite:src/database/local/local_Database.db";

    //Connectionoptions; Foreign Keys aktivieren sonst geht es nicht? Keine Ahnung...
    let connect_options_local_db =
        sqlx::sqlite::SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))
            .map_err(|e| ServerFnError::new(format!("Path Error: {}", e)))?
            .create_if_missing(true)
            .foreign_keys(true)
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
