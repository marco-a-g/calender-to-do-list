use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use crate::utils::structs::GroupLight;
use dioxus::prelude::*;
use server_fn::error::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;

/// Synchronizes Groups from remote database to local database within the provided transaction-queue. Is called by `sync_local_to_remote_db()` function, where the transaction-queue is created.
///
/// Fetches all Groups from Supabase through REST API GET request.
/// Insers new Groups or updates existing ones based on UUID.
/// Deletes local Groups events that no longer exist in the remote database
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
/// - Parseing into a Vec of GroupLight fails.
/// - Any Part of the SQL execution fails.
pub async fn sync_groups(
    tx: &mut Transaction<'_, Sqlite>,
    token: &str,
) -> Result<(), ServerFnError> {
    let http_client = reqwest::Client::new();
    let bearer_token = format!("Bearer {}", token);

    //Gruppen laden
    println!("Loading Groups...");

    //Config & Response von http-Anfrage
    let url_groups = format!("{}/rest/v1/groups?select=*", SUPABASE_URL);
    let response_groups = http_client
        .get(&url_groups)
        .header("apikey", ANON_KEY)
        .header("Authorization", &bearer_token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Http Request Groups Error: {}", e)))?;
    if !response_groups.status().is_success() {
        let err = response_groups.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Supabase Error Groups: {}",
            err
        )));
    }

    //Response in Json parsen
    let groups_text = response_groups
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Text Error: {}", e)))?;

    //Json in Vec von Groups parsen
    let groups: Vec<GroupLight> = serde_json::from_str(&groups_text)
        .map_err(|e| ServerFnError::new(format!("JSON Parse Groups: {}", e)))?;

    //neues set aus Remote-DB Id's für Löschung von verwaisten Einträgen
    let mut remote_group_ids = HashSet::new();

    for g in groups {
        //id in Set aus remote IDs speichern
        remote_group_ids.insert(g.id.clone());
        sqlx::query(
            r#"
            INSERT INTO groups (id, name, owner_id, created_by, created_at, color) 
            VALUES (?, ?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                name=excluded.name, owner_id=excluded.owner_id, 
                created_by=excluded.created_by, created_at=excluded.created_at,
                color=excluded.created_by, color=excluded.color
            "#,
        )
        .bind(g.id)
        .bind(g.name)
        .bind(g.owner_id)
        .bind(g.created_by)
        .bind(g.created_at)
        .bind(g.color)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error Group: {}", e)))?;
    }

    // Cleanup Groups
    //Vec aus lokalen Gruppen anhand ID
    let local_group_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM groups")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Group IDs: {}", e)))?;

    //Ist locale ID nicht in remote ID -> löschen
    for local_id in local_group_ids {
        if !remote_group_ids.contains(&local_id) {
            println!("Deleting orphan group: {}", local_id);
            sqlx::query("DELETE FROM groups WHERE id = ?")
                .bind(local_id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }
    Ok(())
}
