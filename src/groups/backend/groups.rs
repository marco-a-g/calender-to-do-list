use crate::auth::backend::{ANON_KEY, SUPABASE_URL as AUTH_SUPABASE_URL};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{ConnectOptions, SqlitePool};
use std::str::FromStr;

pub type GroupTransfer = (String, String, String, i32);

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub color: String,
}

pub fn to_transfer(g: &Group, member_count: i32) -> GroupTransfer {
    (g.id.clone(), g.name.clone(), g.color.clone(), member_count)
}

#[derive(Debug, Serialize)]
struct CreateGroupPayload {
    name: String,
    color: String,
    owner_id: String,
    created_by: String,
}

#[derive(Debug, Deserialize)]
struct CreatedGroupRow {
    id: String,
}

#[derive(Debug, Serialize)]
struct CreateGroupMemberPayload {
    user_id: String,
    group_id: String,
    role: String,
}

async fn get_local_db_pool() -> Result<SqlitePool, ServerFnError> {
    let db_path = "sqlite:src/database/local/local_Database.db";
    let connect_options = sqlx::sqlite::SqliteConnectOptions::from_str(db_path)
        .map_err(|e| ServerFnError::new(format!("DB path error: {e}")))?
        .create_if_missing(false)
        .foreign_keys(true)
        .disable_statement_logging();

    let pool = SqlitePool::connect_with(connect_options)
        .await
        .map_err(|e| ServerFnError::new(format!("DB connection error: {e}")))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_group_preferences (
            user_id TEXT NOT NULL,
            group_id TEXT NOT NULL,
            color TEXT,
            tag TEXT,
            PRIMARY KEY (user_id, group_id),
            FOREIGN KEY (user_id) REFERENCES profiles(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to create preferences table: {e}")))?;

    Ok(pool)
}

#[server]
pub async fn fetch_groups(user_id: String) -> Result<Vec<GroupTransfer>, ServerFnError> {
    let pool = get_local_db_pool().await?;

    let rows: Vec<(String, String, Option<String>, i64)> = sqlx::query_as(
        r#"
        SELECT
            g.id,
            g.name,
            ugp.color AS user_color,
            COUNT(gm2.user_id) AS member_count
        FROM group_members gm
        INNER JOIN groups g ON g.id = gm.group_id
        LEFT JOIN user_group_preferences ugp
            ON ugp.user_id = gm.user_id AND ugp.group_id = g.id
        LEFT JOIN group_members gm2
            ON gm2.group_id = g.id
        WHERE gm.user_id = ?
        GROUP BY g.id, g.name, ugp.color
        ORDER BY g.created_at DESC
        "#,
    )
    .bind(&user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB query error (fetch_groups): {e}")))?;

    let result = rows
        .into_iter()
        .map(|(id, name, user_color, member_count)| {
            (
                id,
                name,
                user_color.unwrap_or_else(|| "#3A6BFF".to_string()),
                member_count as i32,
            )
        })
        .collect();

    Ok(result)
}

#[server]
pub async fn fetch_group_by_id(
    id: String,
    user_id: String,
) -> Result<Option<(String, String, String)>, ServerFnError> {
    let pool = get_local_db_pool().await?;

    let row: Option<(String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT
            g.id,
            g.name,
            ugp.color AS user_color
        FROM group_members gm
        INNER JOIN groups g ON g.id = gm.group_id
        LEFT JOIN user_group_preferences ugp
            ON ugp.user_id = gm.user_id AND ugp.group_id = g.id
        WHERE gm.user_id = ? AND g.id = ?
        LIMIT 1
        "#,
    )
    .bind(&user_id)
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB query error (fetch_group_by_id): {e}")))?;

    Ok(row.map(|(gid, name, user_color)| {
        (
            gid,
            name,
            user_color.unwrap_or_else(|| "#3A6BFF".to_string()),
        )
    }))
}

#[server]
pub async fn create_group(
    name: String,
    color: String,
    user_id: String,
) -> Result<String, ServerFnError> {
    let url = AUTH_SUPABASE_URL;
    let key = ANON_KEY;
    let client = reqwest::Client::new();

    let groups_endpoint = format!("{url}/rest/v1/groups?select=id");

    let payload = CreateGroupPayload {
        name,
        color,
        owner_id: user_id.clone(),
        created_by: user_id.clone(),
    };

    let response = client
        .post(groups_endpoint)
        .header("apikey", key)
        .header("Authorization", format!("Bearer {key}"))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .error_for_status()
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut created: Vec<CreatedGroupRow> = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let group_id = created
        .pop()
        .ok_or_else(|| ServerFnError::new("Supabase returned no created group row".to_string()))?
        .id;

    let members_endpoint = format!("{url}/rest/v1/group_members");

    let member_payload = CreateGroupMemberPayload {
        user_id: user_id.clone(),
        group_id: group_id.clone(),
        role: "owner".to_string(),
    };

    client
        .post(members_endpoint)
        .header("apikey", key)
        .header("Authorization", format!("Bearer {key}"))
        .header("Content-Type", "application/json")
        .json(&member_payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .error_for_status()
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(group_id)
}

#[derive(Deserialize)]
struct OwnerRow {
    owner_id: String,
}

async fn is_owner(
    client: &reqwest::Client,
    url: &str,
    key: &str,
    group_id: &str,
    user_id: &str,
) -> Result<bool, ServerFnError> {
    let endpoint = format!("{url}/rest/v1/groups?select=owner_id&id=eq.{group_id}&limit=1");

    let resp = client
        .get(&endpoint)
        .header("apikey", key)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .error_for_status()
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut rows: Vec<OwnerRow> = resp
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows.pop().map(|r| r.owner_id == user_id).unwrap_or(false))
}

#[server]
pub async fn delete_group(group_id: String, user_id: String) -> Result<(), ServerFnError> {
    let url = AUTH_SUPABASE_URL;
    let key = ANON_KEY;
    let client = reqwest::Client::new();

    let check_owner_endpoint = format!(
        "{}/rest/v1/groups?select=owner_id&id=eq.{}&limit=1",
        url, group_id
    );

    #[derive(Deserialize)]
    struct OwnerRow {
        owner_id: String,
    }

    let owner_response = client
        .get(&check_owner_endpoint)
        .header("apikey", key)
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Owner check request failed: {e}")))?
        .error_for_status()
        .map_err(|e| ServerFnError::new(format!("Owner check failed: {e}")))?;

    let mut owner_rows: Vec<OwnerRow> = owner_response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Owner check json failed: {e}")))?;

    let Some(owner) = owner_rows.pop() else {
        return Err(ServerFnError::new("Group not found."));
    };

    if owner.owner_id != user_id {
        return Err(ServerFnError::new("Only the owner can delete this group."));
    }

    let delete_endpoint = format!("{}/rest/v1/groups?id=eq.{}", url, group_id);

    client
        .delete(&delete_endpoint)
        .header("apikey", key)
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Delete request failed: {e}")))?
        .error_for_status()
        .map_err(|e| ServerFnError::new(format!("Delete failed: {e}")))?;

    Ok(())
}

#[derive(Deserialize)]
struct MemberRow {
    user_id: String,
}

#[server]
pub async fn leave_group(group_id: String, user_id: String) -> Result<(), ServerFnError> {
    let url = AUTH_SUPABASE_URL;
    let key = ANON_KEY;
    let client = reqwest::Client::new();

    let user_is_owner = is_owner(&client, url, key, &group_id, &user_id).await?;

    if user_is_owner {
        let members_endpoint = format!(
            "{url}/rest/v1/group_members?select=user_id,joined_at&group_id=eq.{group_id}&order=joined_at.asc"
        );

        let resp = client
            .get(&members_endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {key}"))
            .send()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .error_for_status()
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let members: Vec<MemberRow> = resp
            .json()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let next_owner = members
            .iter()
            .find(|m| m.user_id != user_id)
            .map(|m| m.user_id.clone());

        if let Some(new_owner_id) = next_owner {
            #[derive(Serialize)]
            struct UpdateOwnerPayload {
                owner_id: String,
            }

            let update_group_endpoint = format!("{url}/rest/v1/groups?id=eq.{group_id}");

            client
                .patch(&update_group_endpoint)
                .header("apikey", key)
                .header("Authorization", format!("Bearer {key}"))
                .header("Content-Type", "application/json")
                .json(&UpdateOwnerPayload {
                    owner_id: new_owner_id.clone(),
                })
                .send()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?
                .error_for_status()
                .map_err(|e| ServerFnError::new(e.to_string()))?;

            // 2) set new owner role in group_members
            #[derive(Serialize)]
            struct UpdateRolePayload {
                role: String,
            }

            let update_member_endpoint = format!(
                "{url}/rest/v1/group_members?group_id=eq.{group_id}&user_id=eq.{new_owner_id}"
            );

            client
                .patch(&update_member_endpoint)
                .header("apikey", key)
                .header("Authorization", format!("Bearer {key}"))
                .header("Content-Type", "application/json")
                .json(&UpdateRolePayload {
                    role: "owner".to_string(),
                })
                .send()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?
                .error_for_status()
                .map_err(|e| ServerFnError::new(e.to_string()))?;
        } else {
            let delete_group_endpoint = format!("{url}/rest/v1/groups?id=eq.{group_id}");

            client
                .delete(&delete_group_endpoint)
                .header("apikey", key)
                .header("Authorization", format!("Bearer {key}"))
                .send()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?
                .error_for_status()
                .map_err(|e| ServerFnError::new(e.to_string()))?;

            return Ok(());
        }
    }

    let delete_member_endpoint =
        format!("{url}/rest/v1/group_members?group_id=eq.{group_id}&user_id=eq.{user_id}");

    client
        .delete(&delete_member_endpoint)
        .header("apikey", key)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .error_for_status()
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn set_group_color(
    user_id: String,
    group_id: String,
    color: String,
) -> Result<(), ServerFnError> {
    let pool = get_local_db_pool().await?;

    sqlx::query(
        r#"
        INSERT INTO user_group_preferences (user_id, group_id, color)
        VALUES (?, ?, ?)
        ON CONFLICT(user_id, group_id) DO UPDATE SET color = excluded.color
        "#,
    )
    .bind(&user_id)
    .bind(&group_id)
    .bind(&color)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to set group color: {e}")))?;

    Ok(())
}

#[server]
pub async fn get_group_color(
    user_id: String,
    group_id: String,
) -> Result<Option<String>, ServerFnError> {
    let pool = get_local_db_pool().await?;

    let result: Option<(String,)> = sqlx::query_as(
        "SELECT color FROM user_group_preferences WHERE user_id = ? AND group_id = ? LIMIT 1",
    )
    .bind(&user_id)
    .bind(&group_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to get group color: {e}")))?;

    Ok(result.map(|(color,)| color))
}
