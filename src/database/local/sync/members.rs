use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use crate::utils::structs::GroupMemberLight;
use dioxus::prelude::*;
use server_fn::error::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;

/// Synchronizes "member of Group entries" from remote database to local database within the provided transaction-queue. Is called by `sync_local_to_remote_db()` function, where the transaction-queue is created.
///
/// Fetches all "member of Group entries" from Supabase through REST API GET request.
/// Insers new "member of Group entries" or updates existing ones based on UUID.
/// Deletes local "member of Group entries" events that no longer exist in the remote database
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
/// - Parseing into a Vec of GroupMemberLight fails.
/// - Any Part of the SQL execution fails.
pub async fn sync_members(
    tx: &mut Transaction<'_, Sqlite>,
    token: &str,
) -> Result<(), ServerFnError> {
    let http_client = reqwest::Client::new();
    let bearer_token = format!("Bearer {}", token);

    //Members laden
    println!("Loading Members...");

    //Config & Response von http-Anfrage
    let url_members = format!("{}/rest/v1/group_members?select=*", SUPABASE_URL);
    let response_members = http_client
        .get(&url_members)
        .header("apikey", ANON_KEY)
        .header("Authorization", &bearer_token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Http Request Members Error: {}", e)))?;
    if !response_members.status().is_success() {
        let err = response_members.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Supabase Error Members: {}",
            err
        )));
    }

    //Response in Json parsen
    let members_text = response_members
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Text Error: {}", e)))?;

    //Json in Vec von GroupMembers parsen
    let members: Vec<GroupMemberLight> = serde_json::from_str(&members_text)
        .map_err(|e| ServerFnError::new(format!("JSON Parse Members: {}", e)))?;

    //neues set aus Remote-DB Id's für Löschung von verwaisten Einträgen
    let mut remote_member_ids = HashSet::new();

    for m in members {
        //id in Set aus remote IDs speichern
        remote_member_ids.insert(m.id.clone());
        sqlx::query(
            r#"
            INSERT INTO group_members (id, user_id, group_id, role, joined_at) 
            VALUES (?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                role=excluded.role, group_id=excluded.group_id, joined_at=excluded.joined_at
            "#,
        )
        .bind(m.id)
        .bind(m.user_id)
        .bind(m.group_id)
        .bind(m.role)
        .bind(m.joined_at)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error Member: {}", e)))?;
    }

    // Cleanup Members
    //Vec aus lokalen Members anhand ID
    let local_member_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM group_members")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Members: {}", e)))?;
    //Ist locale ID nicht in remote ID -> löschen
    for mem_id in local_member_ids {
        if !remote_member_ids.contains(&mem_id) {
            sqlx::query("DELETE FROM group_members WHERE id = ?")
                .bind(mem_id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }

    Ok(())
}
