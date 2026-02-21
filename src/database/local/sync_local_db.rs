#![allow(unused_imports)]

use crate::{
    auth::backend::get_client,
    database::local::sync::{
        calendar_events::sync_calendar_events, calendars::sync_calendars, groups::sync_groups,
        members::sync_members, profiles::sync_profiles, todolists::sync_todolists,
        todos::sync_todos,
    },
};
use dioxus::prelude::*;
use sqlx::{
    ConnectOptions,
    sqlite::{SqlitePool, SqlitePoolOptions},
};
use std::str::FromStr;
use supabase::Client;

//#[server] -> funktioniert vorerst noch nicht mit #server // soll ja auch nicht auf server sondern db localer rechner speichern!
pub async fn sync_local_to_remote_db() -> Result<(), ServerFnError> {
    let is_syncing = try_consume_context::<Signal<bool>>();

    if let Some(mut sig) = is_syncing {
        sig.set(true);
    }
    //Client holen und Auth checken
    let client = match get_client() {
        Ok(c) => c,
        Err(e) => return Err(ServerFnError::new(format!("get_client Fehler: {}", e))),
    };
    if !client.auth().is_authenticated() {
        println!("Sync skipped: User not authenticated.");
        return Ok(());
    }
    //Token holen für weitergabe an Unterfunktionen
    let session = client
        .auth()
        .get_session()
        .map_err(|e| ServerFnError::new(format!("Session Error: {}", e)))?;
    let token_str = session.access_token.clone();

    println!("Start sync for logged in User...");

    //Pfad local DB
    let db_path = "sqlite:src/database/local/local_Database.db";

    //Connectionoptions; Foreign Keys deaktivieren sonst geht es nicht?
    let connect_options_local_db =
        sqlx::sqlite::SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))
            .map_err(|e| ServerFnError::new(format!("Path Error: {}", e)))?
            .create_if_missing(true)
            .foreign_keys(false)
            .disable_statement_logging();

    // connection zur local db mit error
    let pool_local_db = SqlitePoolOptions::new()
        .connect_with(connect_options_local_db)
        .await
        .map_err(|e| ServerFnError::new(format!("DB Connect Error: {}.", e)))?;

    //öffnet "Änderungs-Warteschlange", läuft querys ab hier durch und ändert erst ab .commit die Inhalte, bisschen wie eine Art Lock
    let mut transaction_queue = pool_local_db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Transaction Error: {}", e)))?;

    //Profile synchronisieren
    sync_profiles(&mut transaction_queue, &token_str).await?;
    //Gruppen synchronisieren
    sync_groups(&mut transaction_queue, &token_str).await?;
    // Mitglieder synchronisieren
    sync_members(&mut transaction_queue, &token_str).await?;
    // Kalender synchronisieren
    sync_calendars(&mut transaction_queue, &token_str).await?;
    // Kalender-Events synchronisieren
    sync_calendar_events(&mut transaction_queue, &token_str).await?;
    // To-Do Einträge synchronisieren
    sync_todos(&mut transaction_queue, &token_str).await?;
    // To-Do Listen  synchronisieren
    sync_todolists(&mut transaction_queue, &token_str).await?;

    //Änderungsqueue zusammenfügen und "commiten"
    transaction_queue
        .commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Commit Error: {}", e)))?;

    println!("Sync completed successfully.");
    if let Some(mut sig) = is_syncing {
        sig.set(false);
    }
    Ok(())
}

#[component]
pub fn SyncIndicator() -> Element {
    let is_syncing = use_context::<Signal<bool>>();
    if !is_syncing() {
        return rsx! {};
    }
    rsx! {
    div {
            class: "fixed bottom-6 right-6 z-[9999] flex items-center gap-3 bg-[#171923] border border-white/10 text-white px-4 py-3 rounded-xl shadow-2xl animate-in fade-in slide-in-from-bottom-4 duration-300",
        div {
            class: "w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"
        }
            span {
                class: "text-xs font-medium tracking-wide text-gray-300", "Sync in progress..."
            }
        }
    }
}
