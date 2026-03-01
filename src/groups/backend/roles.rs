//! Server functions for role and permission management within groups.
//! Provides functionality for viewing member roles, promoting/demoting members,
//! transferring group ownership, and removing members from groups.
//! All operations enforce permission checks (owner/admin privileges).

use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use dioxus::prelude::*;
use dioxus_logger::tracing::{debug, warn};
use serde::Deserialize;
use server_fn::error::ServerFnError;

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

#[derive(Deserialize)]
struct RoleCheck {
    role: String,
}

/// Fetches a user's role within a specific group.
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

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Role check failed ({status}): {body}"
        )));
    }

    let roles: Vec<RoleCheck> = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Parse role error: {e}")))?;

    Ok(roles.first().map(|r| r.role.clone()))
}

/// Changes a member's role (promote to admin or demote to member).
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
    debug!(
        "change_member_role: group_id={} target_user_id={} new_role={} actor_user_id={}",
        group_id, target_user_id, new_role, actor_user_id
    );

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

    debug!("change_member_role response status={}", response.status());

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Change role failed: {err}")));
    }

    Ok(())
}

/// Transfers group ownership to another member.
//#[server]
pub async fn transfer_ownership(
    group_id: String,
    new_owner_id: String,
    current_owner_id: String,
    access_token: String,
) -> Result<(), ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);
    debug!(
        "transfer_ownership: group_id={} new_owner_id={} current_owner_id={}",
        group_id, new_owner_id, current_owner_id
    );

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

    debug!(
        "transfer_ownership: update groups response status={}",
        resp.status()
    );

    if !resp.status().is_success() {
        let err = resp.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Update groups failed: {err}")));
    }

    // Set new owner's role to 'owner'
    let new_owner_url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}",
        SUPABASE_URL, group_id, new_owner_id
    );

    let new_owner_role_resp = client
        .patch(&new_owner_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "role": "owner" }))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Set new owner role error: {e}")))?;

    debug!(
        "transfer_ownership: set new owner role response status={}",
        new_owner_role_resp.status()
    );

    if !new_owner_role_resp.status().is_success() {
        let err = new_owner_role_resp.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Set new owner role failed: {err}"
        )));
    }

    // Demote previous owner to 'admin'
    let old_owner_url = format!(
        "{}/rest/v1/group_members?group_id=eq.{}&user_id=eq.{}",
        SUPABASE_URL, group_id, current_owner_id
    );

    let old_owner_role_resp = client
        .patch(&old_owner_url)
        .header("apikey", ANON_KEY)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "role": "admin" }))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Set old owner role error: {e}")))?;

    debug!(
        "transfer_ownership: set old owner role response status={}",
        old_owner_role_resp.status()
    );

    if !old_owner_role_resp.status().is_success() {
        let err = old_owner_role_resp.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Set old owner role failed: {err}"
        )));
    }

    Ok(())
}

/// Removes a member from the group.
//#[server]
pub async fn kick_member(
    group_id: String,
    target_user_id: String,
    actor_user_id: String,
    access_token: String,
) -> Result<(), ServerFnError> {
    let client = reqwest::Client::new();
    let auth = bearer(&access_token);
    debug!(
        "kick_member: group_id={} target_user_id={} actor_user_id={}",
        group_id, target_user_id, actor_user_id
    );

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

    debug!("kick_member response status={}", response.status());

    if !response.status().is_success() {
        warn!("kick_member failed with status={}", response.status());
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Kick failed: {err}")));
    }

    Ok(())
}
