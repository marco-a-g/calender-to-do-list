/*
Server functions for role and permission management within groups
Provides functionality for viewing member roles, promoting/demoting members,
transferring group ownership, and removing members from groups
All operations enforce permission checks (owner/admin privileges)
*/

use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use dioxus::prelude::*;
use serde::Deserialize;

// Member data for the roles UI: (user_id, username, role)
pub type MemberWithRole = (String, String, String);

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

#[derive(Deserialize)]
struct RoleCheck {
    role: String,
}

// Fetches a user's role within a specific group
async fn get_user_role(
    client: &reqwest::Client,
    auth: &str,
    group_id: &str,
    user_id: &str,
) -> Result<Option<String>, ServerFnError> {
    let url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}&select=role",
        SUPABASE_URL, group_id, user_id
    );

    let response = client
        .get(&url)
        .header("apikey", ANON_KEY)
        .header("Authorization", auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Role check error: {e}")))?;

    let roles: Vec<RoleCheck> = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Parse role error: {e}")))?;

    Ok(roles.first().map(|r| r.role.clone()))
}

#[derive(Deserialize)]
struct MemberRow {
    user_id: String,
    role: String,
}

#[derive(Deserialize)]
struct ProfileRow {
    id: String,
    username: Option<String>,
}

// Retrieves all group members with their usernames and roles
//#[server]
pub async fn fetch_members_with_roles(
    group_id: String,
    access_token: String,
) -> Result<Vec<MemberWithRole>, ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);

    // Fetch all members of the group
    let members_url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&select=user_id,role&order=joined_at.asc",
        SUPABASE_URL, group_id
    );

    let members_response = client
        .get(&members_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch members error: {e}")))?;

    if !members_response.status().is_success() {
        let err = members_response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Fetch members failed: {err}")));
    }

    let members: Vec<MemberRow> = members_response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Parse members error: {e}")))?;

    if members.is_empty() {
        return Ok(vec![]);
    }

    // Batch fetch usernames for all member user_ids
    let user_ids: Vec<&str> = members.iter().map(|m| m.user_id.as_str()).collect();
    let ids_filter = user_ids
        .iter()
        .map(|id| format!("\"{}\"", id))
        .collect::<Vec<_>>()
        .join(",");

    let profiles_url = format!(
        "{}/rest/v1/profiles?id=in.({}))&select=id,username",
        SUPABASE_URL, ids_filter
    );

    let profiles_response = client
        .get(&profiles_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch profiles error: {e}")))?;

    let profiles: Vec<ProfileRow> = if profiles_response.status().is_success() {
        profiles_response.json().await.unwrap_or_default()
    } else {
        vec![]
    };

    // Combine members with their usernames
    let result: Vec<MemberWithRole> = members
        .into_iter()
        .map(|m| {
            let username = profiles
                .iter()
                .find(|p| p.id == m.user_id)
                .and_then(|p| p.username.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            (m.user_id, username, m.role)
        })
        .collect();

    Ok(result)
}

// Changes a member's role (promote to admin or demote to member)
//#[server]
pub async fn change_member_role(
    group_id: String,
    target_user_id: String,
    new_role: String,
    actor_user_id: String,
    access_token: String,
) -> Result<(), ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);

    if !["member", "admin"].contains(&new_role.as_str()) {
        return Err(ServerFnError::new("Invalid role. Use 'member' or 'admin'."));
    }

    // Only owner can change roles
    let actor_role = get_user_role(&client, &auth, &group_id, &actor_user_id).await?;
    if actor_role.as_deref() != Some("owner") {
        return Err(ServerFnError::new("Only the owner can change roles."));
    }

    // Validate target user's current role
    let target_role = get_user_role(&client, &auth, &group_id, &target_user_id).await?;
    match target_role.as_deref() {
        None => return Err(ServerFnError::new("User is not a member of this group.")),
        Some("owner") => return Err(ServerFnError::new("Cannot change the owner's role.")),
        Some("invited") => return Err(ServerFnError::new("Cannot change role of invited user.")),
        _ => {}
    }

    let url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}",
        SUPABASE_URL, group_id, target_user_id
    );

    let body = serde_json::json!({ "role": new_role });

    let response = client
        .patch(&url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Change role error: {e}")))?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Change role failed: {err}")));
    }

    Ok(())
}

// Transfers group ownership to another member
//#[server]
pub async fn transfer_ownership(
    group_id: String,
    new_owner_id: String,
    current_owner_id: String,
    access_token: String,
) -> Result<(), ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);

    // Verify current user is the owner
    let actor_role = get_user_role(&client, &auth, &group_id, &current_owner_id).await?;
    if actor_role.as_deref() != Some("owner") {
        return Err(ServerFnError::new("Only the owner can transfer ownership."));
    }

    // Validate target user
    let target_role = get_user_role(&client, &auth, &group_id, &new_owner_id).await?;
    match target_role.as_deref() {
        None => return Err(ServerFnError::new("User is not a member of this group.")),
        Some("invited") => return Err(ServerFnError::new("Cannot transfer to invited user.")),
        Some("owner") => return Err(ServerFnError::new("User is already the owner.")),
        _ => {}
    }

    // Update owner_id in groups table
    let groups_url = format!("{}/rest/v1/groups?id=eq.{}", SUPABASE_URL, group_id);

    let resp = client
        .patch(&groups_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "owner_id": new_owner_id }))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Update groups error: {e}")))?;

    if !resp.status().is_success() {
        let err = resp.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Update groups failed: {err}")));
    }

    // Set new owner's role to 'owner'
    let new_owner_url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}",
        SUPABASE_URL, group_id, new_owner_id
    );

    client
        .patch(&new_owner_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "role": "owner" }))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Set new owner role error: {e}")))?;

    // Demote previous owner to 'admin'
    let old_owner_url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}",
        SUPABASE_URL, group_id, current_owner_id
    );

    client
        .patch(&old_owner_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "role": "admin" }))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Set old owner role error: {e}")))?;

    Ok(())
}

// Removes a member from the group
//#[server]
pub async fn kick_member(
    group_id: String,
    target_user_id: String,
    actor_user_id: String,
    access_token: String,
) -> Result<(), ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);

    // Check actor's permissions
    let actor_role = get_user_role(&client, &auth, &group_id, &actor_user_id).await?;
    let is_owner = actor_role.as_deref() == Some("owner");
    let is_admin = actor_role.as_deref() == Some("admin");

    if !is_owner && !is_admin {
        return Err(ServerFnError::new("Only owner or admin can kick members."));
    }

    // Validate target and enforce permission hierarchy
    let target_role = get_user_role(&client, &auth, &group_id, &target_user_id).await?;
    match target_role.as_deref() {
        None => return Err(ServerFnError::new("User is not a member of this group.")),
        Some("owner") => return Err(ServerFnError::new("Cannot kick the owner.")),
        Some("admin") if !is_owner => {
            return Err(ServerFnError::new("Only owner can kick admins."));
        }
        _ => {}
    }

    if target_user_id == actor_user_id {
        return Err(ServerFnError::new(
            "Cannot kick yourself. Use 'Leave Group' instead.",
        ));
    }

    let url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}",
        SUPABASE_URL, group_id, target_user_id
    );

    let response = client
        .delete(&url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Kick member error: {e}")))?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Kick failed: {err}")));
    }

    Ok(())
}
