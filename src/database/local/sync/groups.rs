use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use crate::utils::structs::{GroupLight, GroupMemberLight};
use dioxus::prelude::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;

pub async fn sync_groups_and_members(
    tx: &mut Transaction<'_, Sqlite>,
    token: &str,
) -> Result<(), ServerFnError> {
    let http_client = reqwest::Client::new();
    let bearer_token = format!("Bearer {}", token);

    //Members laden
    println!("Loading Members...");

    //Config & Resonse von http-Anfrage
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

    //Gruppen laden
    println!("Loading Groups...");

    //Config & Resonse von http-Anfrage
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
            INSERT INTO groups (id, name, owner_id, created_by, created_at) 
            VALUES (?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                name=excluded.name, owner_id=excluded.owner_id, 
                created_by=excluded.created_by, created_at=excluded.created_at
            "#,
        )
        .bind(g.id)
        .bind(g.name)
        .bind(g.owner_id)
        .bind(g.created_by)
        .bind(g.created_at)
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
