/*
Side Note Important! :  be aware that major parts of the css styling was made with LLM's (GroundLayer with ChatGpt & some details with Claude)
                        refactoring parts were consulted with LLM (Claude)
                        anything else is highlighted in the spot where it was used
*/

/*
Roles management tab for the group detail page.

Owners can promote/demote members, transfer ownership, and kick members.
Admins can kick regular members. Actions are dispatched through a
`pending_action` signal so that all `spawn` calls live in the parent scope
(required by Dioxus for signal access).
*/

use crate::database::local::init_fetch::init_fetch_local_db::{
    fetch_group_members_lokal_db, fetch_profiles_lokal_db,
};
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use dioxus::prelude::*;
use server_fn::error::ServerFnError;

// (user_id, username, role)
type MemberWithRole = (String, String, String);
type MembersRes = Resource<Result<Vec<MemberWithRole>, ServerFnError>>;

// (target_user_id, action_name, display_label)
type PendingAction = (String, String, String);

/// Roles tab showing all group members with role management actions.
///
/// Fetches members + profiles from local DB, resolves usernames,
/// and renders each row with context-dependent action buttons.
// rsx! macro does not support else if chains in class attributes
#[allow(clippy::collapsible_else_if)]
#[component]
pub fn RolesTab(group_id: String, current_user_id: String) -> Element {
    let group_id_for_fetch = group_id.clone();

    let mut members_res: MembersRes = use_resource(move || {
        let gid = group_id_for_fetch.clone();
        async move {
            let all_members = fetch_group_members_lokal_db().await?;
            let all_profiles = fetch_profiles_lokal_db().await?;

            // Build username lookup: profile_id -> username
            let username_map: std::collections::HashMap<String, String> = all_profiles
                .into_iter()
                .map(|p| (p.id, p.username))
                .collect();

            let result: Vec<MemberWithRole> = all_members
                .into_iter()
                .filter(|m| m.group_id == gid)
                .map(|m| {
                    let username = username_map
                        .get(&m.user_id)
                        .cloned()
                        .unwrap_or_else(|| "<unknown>".to_string());
                    (m.user_id, username, m.role)
                })
                .collect();

            Ok(result)
        }
    });

    // Determine the current user's role to decide which actions to show
    let current_user_role = members_res
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .and_then(|members| {
            members
                .iter()
                .find(|(uid, _, _)| uid == &current_user_id)
                .map(|(_, _, role)| role.clone())
        })
        .unwrap_or_else(|| "member".to_string());

    let is_owner = current_user_role == "owner";
    let is_admin = current_user_role == "admin";

    let mut action_status = use_signal(|| Option::<String>::None);
    let mut expanded_user = use_signal(|| Option::<String>::None);
    let mut pending_action = use_signal(|| Option::<PendingAction>::None);

    // Effect that picks up pending actions and executes them.
    // This pattern keeps spawn in the parent scope where all signals are accessible.
    let gid_for_action = group_id.clone();
    let uid_for_action = current_user_id.clone();
    use_effect(move || {
        let action_opt: Option<PendingAction> = { pending_action.read().clone() };
        if let Some((target, action, label)) = action_opt {
            let gid = gid_for_action.clone();
            let actor = uid_for_action.clone();

            // Clear immediately so the effect doesn't re-fire
            pending_action.set(None);

            spawn(async move {
                let (_, token) =
                    match crate::utils::functions::get_user_id_and_session_token().await {
                        Ok(t) => t,
                        Err(e) => {
                            action_status.set(Some(format!("Error: {}", e)));
                            return;
                        }
                    };

                let result = match action.as_str() {
                    "promote" => {
                        crate::groups::backend::roles::change_member_role(
                            gid,
                            target,
                            "admin".to_string(),
                            actor,
                            token,
                        )
                        .await
                    }
                    "demote" => {
                        crate::groups::backend::roles::change_member_role(
                            gid,
                            target,
                            "member".to_string(),
                            actor,
                            token,
                        )
                        .await
                    }
                    "transfer" => {
                        crate::groups::backend::roles::transfer_ownership(gid, target, actor, token)
                            .await
                    }
                    "kick" => {
                        crate::groups::backend::roles::kick_member(gid, target, actor, token).await
                    }
                    _ => Err(ServerFnError::new("Unknown action")),
                };

                match result {
                    Ok(_) => {
                        let _ = sync_local_to_remote_db().await;
                        action_status.set(Some(format!("✓ {}", label)));
                        members_res.restart();
                    }
                    Err(e) => {
                        action_status.set(Some(format!("Error: {}", e)));
                    }
                }
            });
        }
    });

    rsx! {
        div { class: "flex flex-col",
            div { class: "flex items-center justify-between mb-4",
                div {
                    div { class: "text-white/60 text-xs tracking-[0.18em]", "ROLES" }
                    div { class: "text-white/40 text-sm mt-1", "Manage member permissions" }
                }
            }

            // Status banner (success / error / in-progress)
            if let Some(status) = action_status.read().as_ref() {
                div {
                    class:        if status.starts_with("✓") {
                        "mb-3 p-3 rounded-xl bg-green-500/10 border border-green-400/30 text-green-300 text-sm"
                    } else if status.starts_with("Error") {
                        "mb-3 p-3 rounded-xl bg-red-500/10 border border-red-400/30 text-red-300 text-sm"
                    } else {
                        "mb-3 p-3 rounded-xl bg-white/5 border border-white/10 text-white/60 text-sm"
                    },
                    "{status}"
                }
            }

            match members_res.read().as_ref() {
                Some(Ok(members)) => rsx!(
                    div { class: "flex flex-col gap-2",
                        for (user_id, username, role) in members.iter() {
                            MemberRoleRow {
                                key: "{user_id}",
                                user_id: user_id.clone(),
                                username: username.clone(),
                                role: role.clone(),
                                current_user_id: current_user_id.clone(),
                                is_owner,
                                is_admin,
                                expanded_user,
                                on_action: move |(target, action, label): (String, String, String)| {
                                    expanded_user.set(None);
                                    action_status.set(Some(format!("{}...", label)));
                                    pending_action.set(Some((target, action, label)));
                                },
                            }
                        }
                    }
                ),
                Some(Err(e)) => rsx!(div { class: "text-red-400 text-sm", "{e}" }),
                None => rsx!(div { class: "text-white/40", "Loading members..." }),
            }
        }
    }
}

/// Single member row with expandable action buttons.
///
/// Actions shown depend on the viewer's role:
/// - Owner sees promote/demote, transfer, kick
/// - Admin sees kick (members only)
/// - Members see no actions
#[component]
fn MemberRoleRow(
    user_id: String,
    username: String,
    role: String,
    current_user_id: String,
    is_owner: bool,
    is_admin: bool,
    expanded_user: Signal<Option<String>>,
    on_action: EventHandler<(String, String, String)>,
) -> Element {
    let is_self = user_id == current_user_id;
    let is_expanded = expanded_user.read().as_ref() == Some(&user_id);

    // Permission checks
    let can_change_role = is_owner && !is_self && role != "owner" && role != "invited";
    let can_kick = (is_owner || (is_admin && role == "member")) && !is_self && role != "owner";
    let can_transfer = is_owner && !is_self && role != "invited";
    let has_actions = can_change_role || can_kick || can_transfer;

    let role_color = match role.as_str() {
        "owner" => "bg-yellow-500/20 text-yellow-300 border-yellow-400/30",
        "admin" => "bg-purple-500/20 text-purple-300 border-purple-400/30",
        "member" => "bg-blue-500/20 text-blue-300 border-blue-400/30",
        "invited" => "bg-gray-500/20 text-gray-300 border-gray-400/30",
        _ => "bg-white/10 text-white/60 border-white/20",
    };

    rsx! {
        div {
            class: "rounded-2xl bg-white/5 border border-white/10 hover:bg-white/[0.07] transition",

            // Header row: avatar, name, role badge, expand toggle
            div {
                class: "flex items-center justify-between px-4 py-3",

                div { class: "flex items-center gap-3",
                    div {
                        class: "w-8 h-8 rounded-full bg-white/10 flex items-center justify-center text-white/60 text-sm font-medium",
                        "{username.chars().next().unwrap_or('?').to_uppercase()}"
                    }
                    div {
                        div { class: "text-white font-medium flex items-center gap-2",
                            "{username}"
                            if is_self {
                                span { class: "text-white/40 text-xs", "(you)" }
                            }
                        }
                    }
                }

                div { class: "flex items-center gap-2",
                    div {
                        class: format!("px-3 py-1 rounded-full text-xs font-semibold border {}", role_color),
                        "{role}"
                    }

                    if has_actions {
                        button {
                            class: "
                                w-8 h-8 rounded-lg
                                bg-white/5 hover:bg-white/10 transition
                                border border-white/10
                                flex items-center justify-center
                                text-white/60 hover:text-white
                            ",
                            onclick: {
                                let uid = user_id.clone();
                                move |_| {
                                    let current = expanded_user.read().clone();
                                    if current.as_ref() == Some(&uid) {
                                        expanded_user.set(None);
                                    } else {
                                        expanded_user.set(Some(uid.clone()));
                                    }
                                }
                            },
                            if is_expanded { "▲" } else { "▼" }
                        }
                    }
                }
            }

            // Expandable action row
            if is_expanded && has_actions {
                div {
                    class: "flex flex-wrap gap-2 px-4 pb-3 pt-1 border-t border-white/5",

                    if can_change_role {
                        if role == "member" {
                            ActionBtn {
                                label: "Make Admin",
                                user_id: user_id.clone(),
                                action: "promote".to_string(),
                                dangerous: false,
                                on_action,
                            }
                        }
                        if role == "admin" {
                            ActionBtn {
                                label: "Make Member",
                                user_id: user_id.clone(),
                                action: "demote".to_string(),
                                dangerous: false,
                                on_action,
                            }
                        }
                    }

                    if can_transfer {
                        ActionBtn {
                            label: "Transfer Ownership",
                            user_id: user_id.clone(),
                            action: "transfer".to_string(),
                            dangerous: true,
                            on_action,
                        }
                    }

                    if can_kick {
                        ActionBtn {
                            label: "Kick",
                            user_id: user_id.clone(),
                            action: "kick".to_string(),
                            dangerous: true,
                            on_action,
                        }
                    }
                }
            }
        }
    }
}

/// Small action button that fires the callback upward (no spawning here).
#[component]
fn ActionBtn(
    label: String,
    user_id: String,
    action: String,
    dangerous: bool,
    on_action: EventHandler<(String, String, String)>,
) -> Element {
    let btn_class = if dangerous {
        "px-3 py-1.5 rounded-xl text-sm font-medium bg-red-500/10 border border-red-400/20 text-red-300 hover:bg-red-500/20 transition"
    } else {
        "px-3 py-1.5 rounded-xl text-sm font-medium bg-white/5 border border-white/10 text-white/70 hover:bg-white/10 transition"
    };

    rsx! {
        button {
            class: btn_class,
            onclick: {
                let uid = user_id.clone();
                let action = action.clone();
                let label = label.clone();
                move |_| {
                    on_action.call((uid.clone(), action.clone(), label.clone()));
                }
            },
            "{label}"
        }
    }
}
