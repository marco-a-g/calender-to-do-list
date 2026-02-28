/*
Local members backend (SQLite).

Reads group members from the local database and enriches them with
usernames from the profiles table. Returns a flat tuple for the frontend.
*/

use server_fn::error::ServerFnError;

use crate::database::local::init_fetch::init_fetch_local_db::{
    fetch_group_members_lokal_db, fetch_profiles_lokal_db,
};
use dioxus::prelude::*;

// Compact DTO returned to the frontend.
// (group_id, user_id, username, role)
pub type MemberTransfer = (String, String, String, String);

/// Returns all members for the given group from the local DB.
/// Filters out pending invites and resolves usernames via the profiles table.
//#[server]
pub async fn fetch_members(group_id: String) -> Result<Vec<MemberTransfer>, ServerFnError> {
    let all_members = fetch_group_members_lokal_db().await?;
    let all_profiles = fetch_profiles_lokal_db().await?;

    // Build a username lookup: profile_id -> username
    let username_map: std::collections::HashMap<String, String> = all_profiles
        .into_iter()
        .map(|p| (p.id, p.username))
        .collect();

    let result: Vec<MemberTransfer> = all_members
        .into_iter()
        .filter(|m| m.group_id == group_id && m.role != "invited")
        .map(|m| {
            let username = username_map
                .get(&m.user_id)
                .cloned()
                .unwrap_or_else(|| "<no profile>".to_string());
            (group_id.clone(), m.user_id, username, m.role)
        })
        .collect();

    Ok(result)
}