use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use crate::utils::structs::ProfileLight;
use dioxus::prelude::*;
use server_fn::error::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;

/// Synchronizes profiles from remote database to local database within the provided transaction-queue. Is called by `sync_local_to_remote_db()` function, where the transaction-queue is created.
///
/// Fetches all profiles from Supabase through REST API GET request.
/// Insers new profiles or updates existing ones based on UUID.
/// Deletes local profiles events that no longer exist in the remote database
///
/// ## Arguments
///
/// * `tx` - Reference to active SQLite transaction.
/// * `token` - Access token of the authenticated user.
///
/// ## Errors
///
/// Returns a `ServerFnError` if:
/// - The HTTP request to Supabase fails.
/// - Parseing into a Vec of ProfileLight fails.
/// - Any Part of the SQL execution fails.
pub async fn sync_profiles(
    tx: &mut Transaction<'_, Sqlite>,
    token: &str,
) -> Result<(), ServerFnError> {
    let http_client = reqwest::Client::new();
    let bearer_token = format!("Bearer {}", token);
    //Profile laden
    println!("Loading Profiles...");

    //Config & Response von http-Anfrage
    let url = format!("{}/rest/v1/profiles?select=*", SUPABASE_URL);
    let response = http_client
        .get(&url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &bearer_token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Http Request Profiles Error: {}", e)))?;
    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Supabase Error Profiles: {}",
            err
        )));
    }

    //Response in Json parsen
    let response_text = response
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Text Error: {}", e)))?;

    //Json in Vec von Profiles parsen
    let profiles: Vec<ProfileLight> = serde_json::from_str(&response_text)
        .map_err(|e| ServerFnError::new(format!("JSON Parse Profiles: {}", e)))?;

    //neues set aus Remote-DB Id's für Löschung von verwaisten Einträgen
    let mut remote_profiles_ids = HashSet::new();

    //über Vec mit Profilen itterieren und in local DB (erst in Transactionswarteschlange, noch nicht direkt) speichern
    for p in profiles {
        //id in Set aus remote IDs speichern
        remote_profiles_ids.insert(p.id.clone());
        sqlx::query("INSERT INTO profiles (id, username, created_at) VALUES (?, ?, ?) ON CONFLICT(id) DO UPDATE SET username = excluded.username, created_at = excluded.created_at")
            .bind(p.id)
            .bind(p.username)
            .bind(p.created_at)
            .execute(&mut **tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Profile: {}", e)))?;
    }

    // CleanUp: Profile die lokal noch da sind aber nicht mehr in remote -> löschen
    //Vec aus lokalen Profiles anhand ID
    let local_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM profiles")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Profile IDs: {}", e)))?;

    //Ist locale ID nicht in remote ID -> löschen
    for id in local_ids {
        if !remote_profiles_ids.contains(&id) {
            sqlx::query("DELETE FROM profiles WHERE id = ?")
                .bind(id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }
    Ok(())
}
