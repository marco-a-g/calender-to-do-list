use crate::utils::structs::{GroupLight, GroupMemberLight};
use dioxus::prelude::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;
use supabase::Client;

pub async fn sync_groups_and_members(
    client: &Client,
    tx: &mut Transaction<'_, Sqlite>,
    user_id: &str,
) -> Result<Vec<String>, ServerFnError> {
    // Mitglieder laden
    println!("Loading Members...");
    let members_json = client
        .database()
        .from("group_members")
        .select("*")
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Members Error: {}", e)))?;

    //-----------
    println!("{:?}", members_json);// gibt [] zurück
    //-----
    //Mitglieder in Vec parsen
    let members: Vec<GroupMemberLight> = serde_json::from_value(serde_json::Value::Array(members_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Members: {}", e)))?;

    //group ids des derzeitigen users sammeln
    let user_group_ids: Vec<String> = members
        .iter()
        .filter(|m| m.user_id == user_id)
        .map(|m| m.group_id.clone())
        .collect();

    // Gruppen laden
    println!("Loading Groups...");
    let groups_json = client
        .database()
        .from("groups")
        .select("*")
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Groups Error: {}", e)))?;

    //Gruppen in Vec parsen
    let groups: Vec<GroupLight> = serde_json::from_value(serde_json::Value::Array(groups_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Groups: {}", e)))?;

    // Set Für Löschung von Gruppen IDs
    let mut remote_group_ids = HashSet::new();

    //nimmt remote Gruppen und packt sie in neues Set remote_group_ids
    for g in groups {
        remote_group_ids.insert(g.id.clone());
        sqlx::query(r#"
            INSERT INTO groups (id, name, owner_id, created_by, created_at) 
            VALUES (?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                name=excluded.name, 
                owner_id=excluded.owner_id, 
                created_by=excluded.created_by,
                created_at=excluded.created_at
            "#)
            .bind(g.id)
            .bind(g.name)
            .bind(g.owner_id)
            .bind(g.created_by)
            .bind(g.created_at)
            .execute(&mut **tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Group: {}", e)))?;
    }

    // Cleanup: erstelle Set aus lokalen gruppen ids
    let local_group_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM groups")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Group IDs: {}", e)))?;

    //Cleanup: ist local_id nicht in remote_ids -> löschen
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

    //speichert Mitglieder, die in Gruppen des users sind
   for m in members {
        if !user_group_ids.contains(&m.group_id) {
            continue;
        }
        sqlx::query(r#"
            INSERT INTO group_members (id, user_id, group_id, role, joined_at) 
            VALUES (?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                role=excluded.role, 
                group_id=excluded.group_id,
                joined_at=excluded.joined_at
            "#)
            .bind(m.id)
            .bind(m.user_id)
            .bind(m.group_id)
            .bind(m.role)
            .bind(m.joined_at)
            .execute(&mut **tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Member: {}", e)))?;
    }

    // Cleanup: löscht alle Members, die nicht zu Gruppen gehören die entfernt worden
    //set aus localen membern erstellen
    let local_member: Vec<(String, String)> =
        sqlx::query_as("SELECT id, group_id FROM group_members")
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| ServerFnError::new(format!("Fetch Local Members: {}", e)))?;
    //Cleanup: wenn member nicht in remote DB -> löschen
    for (mem_id, grp_id) in local_member {
        if !remote_group_ids.contains(&grp_id) {
            sqlx::query("DELETE FROM group_members WHERE id = ?")
                .bind(mem_id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }

    Ok(user_group_ids)
}
