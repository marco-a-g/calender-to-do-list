/* Server functions for group invitation management
Handles user search, sending invites, and accepting/declining invitations
All functions use the user's access token for Supabase RLS authentication */

use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use dioxus::prelude::*;
use serde::Deserialize;

// Search result containing user ID and username
pub type UserSearchResult = (String, String);

// Invite data: (group_id, group_name, group_color, invited_by_username)
pub type InviteTransfer = (String, String, String, String);

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

#[derive(Debug, Deserialize)]
struct ProfileRow {
    id: String,
    username: Option<String>,
}

// Searches for users by username using case-insensitive partial matching
#[server]
pub async fn search_users_by_username(
    query: String,
    exclude_user_id: String,
    access_token: String,
) -> Result<Vec<UserSearchResult>, ServerFnError> {
    if query.trim().len() < 2 {
        return Ok(vec![]);
    }

    let client = reqwest::Client::new();
    let auth = bearer(&access_token);

    let url = format!(
        "{}/rest/v1/profiles?username=ilike.*{}*&id=neq.{}&select=id,username&limit=10",
        SUPABASE_URL,
        query.trim(),
        exclude_user_id
    );

    let response = client
        .get(&url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Search request error: {e}")))?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Search error: {err}")));
    }

    let profiles: Vec<ProfileRow> = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Parse error: {e}")))?;

    let results: Vec<UserSearchResult> = profiles
        .into_iter()
        .filter_map(|p| p.username.map(|username| (p.id, username)))
        .collect();

    Ok(results)
}

#[derive(Deserialize)]
struct RoleCheck {
    role: String,
}

#[derive(Deserialize)]
struct ExistsCheck {
    id: String,
}

// Sends a group invitation to a user
#[server]
pub async fn invite_user(
    group_id: String,
    invited_user_id: String,
    invited_by_user_id: String,
    access_token: String,
) -> Result<(), ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);

    // Verify inviter has permission (must be owner or admin)
    let check_url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}&select=role",
        SUPABASE_URL, group_id, invited_by_user_id
    );

    let check_response = client
        .get(&check_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Permission check error: {e}")))?;

    let roles: Vec<RoleCheck> = check_response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Parse role error: {e}")))?;

    let is_privileged = roles
        .first()
        .map(|r| r.role == "owner" || r.role == "admin")
        .unwrap_or(false);

    if !is_privileged {
        return Err(ServerFnError::new("Only owner or admin can invite users"));
    }

    // Check if user is already a member or has pending invite
    let exists_url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}&select=id",
        SUPABASE_URL, group_id, invited_user_id
    );

    let exists_response = client
        .get(&exists_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Exists check error: {e}")))?;

    let existing: Vec<ExistsCheck> = exists_response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Parse exists error: {e}")))?;

    if !existing.is_empty() {
        return Err(ServerFnError::new("User is already a member or invited"));
    }

    // Create invite (role defaults to 'invited' in database)
    let insert_url = format!("{}/rest/v1/group_members", SUPABASE_URL);

    let body = serde_json::json!({
        "group_id": group_id,
        "user_id": invited_user_id
    });

    let insert_response = client
        .post(&insert_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Invite insert error: {e}")))?;

    if !insert_response.status().is_success() {
        let err = insert_response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Invite failed: {err}")));
    }

    Ok(())
}

#[derive(Deserialize)]
struct GroupInfo {
    id: String,
    name: String,
    color: Option<String>,
}

#[derive(Deserialize)]
struct InviteRow {
    group_id: String,
    groups: Option<GroupInfo>,
}

// Retrieves all pending group invitations for the current user
#[server]
pub async fn fetch_my_invites(
    user_id: String,
    access_token: String,
) -> Result<Vec<InviteTransfer>, ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);

    let url = format!(
        "{}/rest/v1/group_members?user_id=eq.{}&role=eq.invited&select=group_id,groups(id,name,color)",
        SUPABASE_URL, user_id
    );

    let response = client
        .get(&url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch invites error: {e}")))?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Fetch invites failed: {err}")));
    }

    let rows: Vec<InviteRow> = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Parse invites error: {e}")))?;

    let invites: Vec<InviteTransfer> = rows
        .into_iter()
        .filter_map(|r| {
            r.groups.map(|g| {
                (
                    g.id,
                    g.name,
                    g.color.unwrap_or_else(|| "#3A6BFF".to_string()),
                    "Someone".to_string(), // TODO: track who sent the invite
                )
            })
        })
        .collect();

    Ok(invites)
}

// Accepts a group invitation by changing the user's role from 'invited' to 'member'
#[server]
pub async fn accept_invite(
    group_id: String,
    user_id: String,
    access_token: String,
) -> Result<(), ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);

    let url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}&role=eq.invited",
        SUPABASE_URL, group_id, user_id
    );

    let body = serde_json::json!({ "role": "member" });

    let response = client
        .patch(&url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Accept invite error: {e}")))?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Accept failed: {err}")));
    }

    Ok(())
}

// Declines a group invitation by removing the group_members entry
#[server]
pub async fn decline_invite(
    group_id: String,
    user_id: String,
    access_token: String,
) -> Result<(), ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);

    let url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}&role=eq.invited",
        SUPABASE_URL, group_id, user_id
    );

    let response = client
        .delete(&url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Decline invite error: {e}")))?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Decline failed: {err}")));
    }

    Ok(())
}
