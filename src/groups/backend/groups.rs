//! Backend server functions for group management.
//! All functions use the user's access token for Supabase RLS authorization.
//! We intentionally do NOT use the service role key here - RLS enforces permissions.

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

/// Payload for creating a new group via PostgREST.
#[derive(Debug, Serialize)]
struct CreateGroupPayload {
    name: String,
    color: String,
    owner_id: String,
}

/// Fetches a single group by ID.
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

/// Creates a new group owned by the specified user.
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

/// Checks if the given user is the owner of the specified group.
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

/// Deletes a group and all its members.
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

    // Verify user is the owner
    if !is_owner(&client, url, key, &auth, &group_id, &user_id).await? {
        return Err(ServerFnError::new("Only the owner can delete this group."));
    }

    // Delete the group
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

    if user_is_owner {
        // Find next owner candidate (excluding pending invites)
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
            // Transfer ownership to next member
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

            // Update new owner's role in group_members
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
            // No other members - delete the entire group
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

    // Demote owner before deleting (RLS constraint workaround)
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

    // Remove user from group_members
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