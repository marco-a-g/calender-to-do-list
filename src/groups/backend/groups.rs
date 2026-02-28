/*
Backend server functions for group management.

All functions use the user's access token for Supabase RLS authorization.
We intentionally do NOT use the service role key — RLS enforces permissions.
*/

use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use crate::utils::functions::get_user_id_and_session_token;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::error::ServerFnError;

const DEFAULT_GROUP_COLOR: &str = "#3A6BFF";

#[derive(Debug, Deserialize)]
struct SupabaseGroupRow {
    id: String,
    name: String,
    #[serde(default)]
    color: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateGroupPayload {
    name: String,
    color: String,
    owner_id: String,
}

/// Fetches a single group by ID. Returns None if not found or RLS blocks access.
//#[server]
pub async fn fetch_group_by_id(
    id: String,
    _user_id: String,
    _access_token: String,
) -> Result<Option<(String, String, String)>, ServerFnError> {
    let url = SUPABASE_URL;
    let key = ANON_KEY;
    let token = get_user_id_and_session_token().await?.1;
    let client = reqwest::Client::new();

    let endpoint = format!("{url}/rest/v1/groups?id=eq.{id}&select=id,name,color&limit=1");
    let response = client
        .get(&endpoint)
        .header("apikey", key)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("fetch_group_by_id: {e}")))?
        .error_for_status()
        .map_err(|e| ServerFnError::new(format!("fetch_group_by_id Supabase: {e}")))?;

    let rows: Vec<SupabaseGroupRow> = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("fetch_group_by_id json: {e}")))?;

    Ok(rows.into_iter().next().map(|g| {
        (
            g.id,
            g.name,
            g.color
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| DEFAULT_GROUP_COLOR.to_string()),
        )
    }))
}

/// Creates a new group owned by the given user.
//#[server]
pub async fn create_group(
    name: String,
    color: String,
    user_id: String,
    _access_token: String,
) -> Result<(), ServerFnError> {
    let url = SUPABASE_URL;
    let key = ANON_KEY;
    let token = get_user_id_and_session_token().await?.1;
    let client = reqwest::Client::new();

    let groups_endpoint = format!("{url}/rest/v1/groups");

    let payload = CreateGroupPayload {
        name,
        color,
        owner_id: user_id.clone(),
    };

    let response = client
        .post(&groups_endpoint)
        .header("apikey", key)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("create_group request: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "(no body)".to_string());
        return Err(ServerFnError::new(format!(
            "create_group Supabase {}: {}",
            status, body
        )));
    }
    Ok(())
}

#[derive(Deserialize)]
struct OwnerRow {
    owner_id: String,
}

/// Checks whether the given user is the owner of a group.
async fn is_owner(
    client: &reqwest::Client,
    url: &str,
    key: &str,
    auth: &str,
    group_id: &str,
    user_id: &str,
) -> Result<bool, ServerFnError> {
    let endpoint = format!("{url}/rest/v1/groups?select=owner_id&id=eq.{group_id}&limit=1");

    let resp = client
        .get(&endpoint)
        .header("apikey", key)
        .header("Authorization", auth)
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

/// Deletes a group. Only the owner is allowed to do this.
/// Member rows are cascade-deleted by the database.
//#[server]
pub async fn delete_group(
    group_id: String,
    user_id: String,
    _access_token: String,
) -> Result<(), ServerFnError> {
    let url = SUPABASE_URL;
    let key = ANON_KEY;
    let token = get_user_id_and_session_token().await?.1;
    let auth = format!("Bearer {}", token);
    let client = reqwest::Client::new();

    if !is_owner(&client, url, key, &auth, &group_id, &user_id).await? {
        return Err(ServerFnError::new("Only the owner can delete this group."));
    }

    let delete_endpoint = format!("{}/rest/v1/groups?id=eq.{}", url, group_id);

    let group_response = client
        .delete(&delete_endpoint)
        .header("apikey", key)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Delete group request failed: {e}")))?;

    if !group_response.status().is_success() {
        let err = group_response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Delete group failed: {err}")));
    }

    Ok(())
}

#[derive(Deserialize)]
struct MemberRow {
    user_id: String,
}

/// Removes the current user from a group.
///
/// If the user is the owner, ownership is automatically transferred to the
/// longest-standing member (by joined_at). If no other members remain the
/// group is deleted entirely. A self-demotion step is needed before the
/// delete because of an RLS constraint on owners.
//#[server]
pub async fn leave_group(
    group_id: String,
    user_id: String,
    _access_token: String,
) -> Result<(), ServerFnError> {
    let url = SUPABASE_URL;
    let key = ANON_KEY;
    let token = get_user_id_and_session_token().await?.1;
    let auth = format!("Bearer {}", token);
    let client = reqwest::Client::new();

    let user_is_owner = is_owner(&client, url, key, &auth, &group_id, &user_id).await?;

    //LLM consulted: owner transfer logic on leave
    if user_is_owner {
        // Pick the next owner: oldest non-invited member that isn't us
        let members_endpoint = format!(
            "{url}/rest/v1/group_members?select=user_id,joined_at&group_id=eq.{group_id}&role=neq.invited&order=joined_at.asc"
        );

        let resp = client
            .get(&members_endpoint)
            .header("apikey", key)
            .header("Authorization", &auth)
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
            // Transfer ownership: update groups table + set new owner's role
            #[derive(Serialize)]
            struct UpdateOwnerPayload {
                owner_id: String,
            }

            let update_group_endpoint = format!("{url}/rest/v1/groups?id=eq.{group_id}");

            client
                .patch(&update_group_endpoint)
                .header("apikey", key)
                .header("Authorization", &auth)
                .header("Content-Type", "application/json")
                .json(&UpdateOwnerPayload {
                    owner_id: new_owner_id.clone(),
                })
                .send()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?
                .error_for_status()
                .map_err(|e| ServerFnError::new(e.to_string()))?;

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
                .header("Authorization", &auth)
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
            // Last member standing — just delete the group
            let delete_group_endpoint = format!("{url}/rest/v1/groups?id=eq.{group_id}");

            client
                .delete(&delete_group_endpoint)
                .header("apikey", key)
                .header("Authorization", &auth)
                .send()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?
                .error_for_status()
                .map_err(|e| ServerFnError::new(e.to_string()))?;

            return Ok(());
        }
    }

    // Demote ourselves first (RLS won't let an owner delete their own row)
    if user_is_owner {
        let demote_self_endpoint =
            format!("{url}/rest/v1/group_members?group_id=eq.{group_id}&user_id=eq.{user_id}");

        client
            .patch(&demote_self_endpoint)
            .header("apikey", key)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({"role": "member"}))
            .send()
            .await
            .ok();
    }

    // Finally remove ourselves from the group
    let delete_member_endpoint =
        format!("{url}/rest/v1/group_members?group_id=eq.{group_id}&user_id=eq.{user_id}");

    client
        .delete(&delete_member_endpoint)
        .header("apikey", key)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .error_for_status()
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}