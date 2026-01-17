#![allow(dead_code)]
#![allow(unused_imports)]

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
use crate::auth::backend::{AuthError, get_client};
use crate::utils::structs::{
    CalendarEventLight, CalendarLight, GroupLight, GroupMemberLight, ProfileLight, TodoEventLight,
    TodoListLight,
};

//Sync api function
//#[server]
pub async fn sync_remote_to_local() -> Result<(), ServerFnError> {
    sync_function().await
}

// Sync Logik
pub async fn sync_function() -> Result<(), ServerFnError> {
    let client = match get_client() {
        Ok(c) => c,
        Err(e) => return Err(ServerFnError::new(format!("get_client Fehler: {}", e))),
    };

    if !client.auth().is_authenticated() {
        println!("Sync skipped: User not logged in.");
        return Ok(());
    }

    println!("Start sync for logged in User...");

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

    //öffnet "Änderungs-Warteschlange", läuft querys ab hier durch und ändert erst ab .commit die Inhalte, bisschen wie ein Lock
    let mut transaction_queue = pool_local_db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Transaction Error: {}", e)))?;

    //Profile synchronisieren
    sync_profiles(&client, &mut transaction_queue).await?;
    //Gruppen und Mitglieder synchronisieren
    sync_groups_and_members(&client, &mut transaction_queue).await?;
    // Kalender und Events synchronisieren
    sync_calendars_and_events(&client, &mut transaction_queue).await?;
    // To-Do Listen und Einträge synchronisieren
    sync_todos(&client, &mut transaction_queue).await?;

    //Hier Änderungsqueue zusammenfügen und "commiten"
    transaction_queue
        .commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Commit Error: {}", e)))?;

    println!("Sync completed");
    Ok(())
}
