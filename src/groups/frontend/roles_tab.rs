/*
Roles management tab for group detail page
Allows owners to manage member permissions:
- Promote members to admin
- Demote admins to member
- Transfer group ownership
- Kick members from the group
Admins can only kick regular members, not other admins or the owner
*/

use dioxus::prelude::*;

// Roles tab showing all members with role management actions
#[component]
pub fn RolesTab(group_id: String, current_user_id: String) -> Element {
    let group_id_for_fetch = group_id.clone();

    let mut members_res = use_resource(move || {
        let gid = group_id_for_fetch.clone();
        async move {
            let (_, token) = crate::utils::functions::get_user_id_and_session_token()
                .await
                .map_err(|e| dioxus::prelude::ServerFnError::new(e.to_string()))?;
            crate::groups::backend::roles::fetch_members_with_roles(gid, token).await
        }
    });

    // Determine current user's role for permission checks
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
    let mut dropdown_open = use_signal(|| Option::<String>::None);

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "flex items-center justify-between mb-4",
                div {
                    div { class: "text-white/60 text-xs tracking-[0.18em]", "ROLES" }
                    div { class: "text-white/40 text-sm mt-1", "Manage member permissions" }
                }
            }

            // Action feedback message
            if let Some(status) = action_status.read().as_ref() {
                div {
                    class: if status.starts_with("✓") {
                        "mb-3 p-3 rounded-xl bg-green-500/10 border border-green-400/30 text-green-300 text-sm"
                    } else if status.starts_with("Error") {
                        "mb-3 p-3 rounded-xl bg-red-500/10 border border-red-400/30 text-red-300 text-sm"
                    } else {
                        "mb-3 p-3 rounded-xl bg-white/5 border border-white/10 text-white/60 text-sm"
                    },
                    "{status}"
                }
            }

            // Members list
            div { class: "flex-1 overflow-auto",
                match members_res.read().as_ref() {
                    Some(Ok(members)) => rsx!(
                        div { class: "flex flex-col gap-2",
                            for (user_id, username, role) in members.iter() {
                                MemberRoleRow {
                                    key: "{user_id}",
                                    user_id: user_id.clone(),
                                    username: username.clone(),
                                    role: role.clone(),
                                    group_id: group_id.clone(),
                                    current_user_id: current_user_id.clone(),
                                    current_user_role: current_user_role.clone(),
                                    is_owner: is_owner,
                                    is_admin: is_admin,
                                    dropdown_open: dropdown_open.clone(),
                                    action_status: action_status.clone(),
                                    on_refresh: move |_| members_res.restart(),
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
}

// Single member row with role badge and action dropdown.
#[component]
fn MemberRoleRow(
    user_id: String,
    username: String,
    role: String,
    group_id: String,
    current_user_id: String,
    current_user_role: String,
    is_owner: bool,
    is_admin: bool,
    dropdown_open: Signal<Option<String>>,
    action_status: Signal<Option<String>>,
    on_refresh: EventHandler<()>,
) -> Element {
    let is_self = user_id == current_user_id;
    let is_dropdown_open = dropdown_open.read().as_ref() == Some(&user_id);

    // Permission checks for available actions
    let can_change_role = is_owner && !is_self && role != "owner" && role != "invited";
    let can_kick = (is_owner || (is_admin && role == "member")) && !is_self && role != "owner";
    let can_transfer = is_owner && !is_self && role != "invited";
    let has_actions = can_change_role || can_kick || can_transfer;

    // Role badge styling
    let role_color = match role.as_str() {
        "owner" => "bg-yellow-500/20 text-yellow-300 border-yellow-400/30",
        "admin" => "bg-purple-500/20 text-purple-300 border-purple-400/30",
        "member" => "bg-blue-500/20 text-blue-300 border-blue-400/30",
        "invited" => "bg-gray-500/20 text-gray-300 border-gray-400/30",
        _ => "bg-white/10 text-white/60 border-white/20",
    };

    rsx! {
        div {
            class: "
                flex items-center justify-between
                px-4 py-3 rounded-2xl
                bg-white/5 border border-white/10
                hover:bg-white/10 transition
                relative
            ",

            // User info
            div { class: "flex items-center gap-3",
                // Avatar placeholder
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

            // Role badge and actions dropdown
            div { class: "flex items-center gap-2",
                div {
                    class: format!("px-3 py-1 rounded-full text-xs font-semibold border {}", role_color),
                    "{role}"
                }

                // Dropdown button (only shown if user has available actions)
                if has_actions {
                    div { class: "relative",
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
                                let mut dropdown_open = dropdown_open.clone();
                                move |e: MouseEvent| {
                                    e.stop_propagation();
                                    let current = dropdown_open.read().clone();
                                    if current.as_ref() == Some(&uid) {
                                        dropdown_open.set(None);
                                    } else {
                                        dropdown_open.set(Some(uid.clone()));
                                    }
                                }
                            },
                            "▼"
                        }

                        // Dropdown menu
                        if is_dropdown_open {
                            div {
                                class: "
                                    absolute right-0 top-full mt-1
                                    bg-[#0a0e1a] border border-white/10
                                    rounded-xl overflow-hidden
                                    min-w-[160px]
                                    z-50
                                    shadow-xl
                                ",

                                // Promote/demote options (owner only)
                                if can_change_role {
                                    if role == "member" {
                                        ActionButton {
                                            label: "Make Admin",
                                            group_id: group_id.clone(),
                                            target_user_id: user_id.clone(),
                                            current_user_id: current_user_id.clone(),
                                            action: "promote".to_string(),
                                            dropdown_open: dropdown_open.clone(),
                                            action_status: action_status.clone(),
                                            on_refresh: on_refresh.clone(),
                                        }
                                    }
                                    if role == "admin" {
                                        ActionButton {
                                            label: "Make Member",
                                            group_id: group_id.clone(),
                                            target_user_id: user_id.clone(),
                                            current_user_id: current_user_id.clone(),
                                            action: "demote".to_string(),
                                            dropdown_open: dropdown_open.clone(),
                                            action_status: action_status.clone(),
                                            on_refresh: on_refresh.clone(),
                                        }
                                    }
                                }

                                // Transfer ownership (owner only)
                                if can_transfer {
                                    ActionButton {
                                        label: "Transfer Ownership",
                                        group_id: group_id.clone(),
                                        target_user_id: user_id.clone(),
                                        current_user_id: current_user_id.clone(),
                                        action: "transfer".to_string(),
                                        dropdown_open: dropdown_open.clone(),
                                        action_status: action_status.clone(),
                                        on_refresh: on_refresh.clone(),
                                    }
                                }

                                // Kick member
                                if can_kick {
                                    ActionButton {
                                        label: "Kick",
                                        group_id: group_id.clone(),
                                        target_user_id: user_id.clone(),
                                        current_user_id: current_user_id.clone(),
                                        action: "kick".to_string(),
                                        dropdown_open: dropdown_open.clone(),
                                        action_status: action_status.clone(),
                                        on_refresh: on_refresh.clone(),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Button for a single role management action (promote, demote, transfer, kick)
#[component]
fn ActionButton(
    label: String,
    group_id: String,
    target_user_id: String,
    current_user_id: String,
    action: String,
    dropdown_open: Signal<Option<String>>,
    action_status: Signal<Option<String>>,
    on_refresh: EventHandler<()>,
) -> Element {
    let is_dangerous = action == "kick" || action == "transfer";

    rsx! {
        button {
            class: if is_dangerous {
                "w-full px-4 py-2 text-left text-sm text-red-300 hover:bg-red-500/20 transition"
            } else {
                "w-full px-4 py-2 text-left text-sm text-white/80 hover:bg-white/10 transition"
            },
            onclick: {
                let gid = group_id.clone();
                let target = target_user_id.clone();
                let actor = current_user_id.clone();
                let action = action.clone();
                let label = label.clone();
                let mut dropdown_open = dropdown_open.clone();
                let mut action_status = action_status.clone();
                let on_refresh = on_refresh.clone();

                move |_| {
                    let gid = gid.clone();
                    let target = target.clone();
                    let actor = actor.clone();
                    let action = action.clone();
                    let label = label.clone();
                    let mut dropdown_open = dropdown_open.clone();
                    let mut action_status = action_status.clone();
                    let on_refresh = on_refresh.clone();

                    dropdown_open.set(None);
                    action_status.set(Some(format!("{}...", label)));

                    spawn(async move {
                        let result = match crate::utils::functions::get_user_id_and_session_token().await {
                            Ok((_, token)) => {
                                match action.as_str() {
                                    "promote" => {
                                        crate::groups::backend::roles::change_member_role(
                                            gid, target, "admin".to_string(), actor, token
                                        ).await
                                    }
                                    "demote" => {
                                        crate::groups::backend::roles::change_member_role(
                                            gid, target, "member".to_string(), actor, token
                                        ).await
                                    }
                                    "transfer" => {
                                        crate::groups::backend::roles::transfer_ownership(
                                            gid, target, actor, token
                                        ).await
                                    }
                                    "kick" => {
                                        crate::groups::backend::roles::kick_member(
                                            gid, target, actor, token
                                        ).await
                                    }
                                    _ => Err(dioxus::prelude::ServerFnError::new("Unknown action"))
                                }
                            }
                            Err(e) => Err(dioxus::prelude::ServerFnError::new(e.to_string()))
                        };

                        match result {
                            Ok(_) => {
                                action_status.set(Some(format!("✓ {}", label)));
                                on_refresh.call(());
                            }
                            Err(e) => {
                                action_status.set(Some(format!("Error: {}", e)));
                            }
                        }
                    });
                }
            },
            "{label}"
        }
    }
}
