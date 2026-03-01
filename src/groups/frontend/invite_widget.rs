/*
Side Note Important! :  be aware that major parts of the css styling was made with LLM's (GroundLayer with ChatGpt & some details with Claude)
                        refactoring parts were consulted with LLM (Claude)
                        anything else is highlighted in the spot where it was used
*/

//! Group invitation UI widgets.
//!
//! Two components:
//! - InvitesWidget:        Shows pending invitations on the groups overview page.
//!                         Accept/decline buttons trigger backend calls and refresh the list.
//! - UserSearchDropdown:   Live-search dropdown on the group detail page.
//!                         Searches users by username and sends invites on click.

use crate::database::local::sync_local_db::sync_local_to_remote_db;
use dioxus::prelude::*;
use server_fn::error::ServerFnError;

/// Pending invitations panel for the current user.
///
/// Fetches invites on mount. After accepting an invite the parent's group
/// list is refreshed via `on_change`.
#[component]
pub fn InvitesWidget(user_id: String, on_change: EventHandler<()>) -> Element {
    let user_id_for_fetch = user_id.clone();

    let mut invites_res = use_resource(move || {
        let uid = user_id_for_fetch.clone();
        async move {
            let (_, token) = crate::utils::functions::get_user_id_and_session_token()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            crate::groups::backend::invites::fetch_my_invites(uid, token).await
        }
    });

    rsx! {
        div {
            class: "
                bg-white/5 border border-white/10
                backdrop-blur-xl rounded-3xl
                shadow-[0_20px_60px_rgba(0,0,0,0.55)]
                px-5 sm:px-6 lg:px-7 py-5 sm:py-6
                mt-4
            ",

            div { class: "text-white/60 text-xs tracking-[0.18em] mb-4", "INVITATIONS" }

            match invites_res.read().as_ref() {
                Some(Ok(invites)) if invites.is_empty() => rsx!(
                    div { class: "text-white/40 text-sm", "No pending invitations" }
                ),
                Some(Ok(invites)) => rsx!(
                    div { class: "flex flex-col gap-3",
                        for (group_id, group_name, group_color, _invited_by) in invites.iter() {
                            div {
                                key: "{group_id}",
                                class: "
                                    flex flex-col sm:flex-row sm:items-center sm:justify-between
                                    gap-3 p-4 rounded-2xl
                                    bg-white/5 border border-white/10
                                ",

                                div { class: "flex items-center gap-3",
                                    div {
                                        class: "w-3 h-3 rounded-full",
                                        style: format!("background: {};", group_color),
                                    }
                                    div {
                                        div { class: "text-white font-medium", "{group_name}" }
                                        div { class: "text-white/40 text-xs", "You've been invited" }
                                    }
                                }

                                div { class: "flex gap-2",
                                    button {
                                        class: "
                                            px-4 py-2 rounded-xl
                                            bg-green-500/20 hover:bg-green-500/30 transition
                                            border border-green-400/30
                                            text-green-200 text-sm font-semibold
                                        ",
                                        onclick: {
                                            let gid = group_id.clone();
                                            let uid = user_id.clone();

                                            move |_| {
                                                let gid = gid.clone();
                                                let uid = uid.clone();

                                                spawn(async move {
                                                    if let Ok((_, token)) = crate::utils::functions::get_user_id_and_session_token().await {
                                                        if crate::groups::backend::invites::accept_invite(gid, uid, token)
                                                            .await
                                                            .is_ok()
                                                        {
                                                            invites_res.restart();
                                                            on_change.call(());
                                                        }
                                                    }
                                                });
                                            }
                                        },
                                        "Accept"
                                    }

                                    button {
                                        class: "
                                            px-4 py-2 rounded-xl
                                            bg-red-500/20 hover:bg-red-500/30 transition
                                            border border-red-400/30
                                            text-red-200 text-sm font-semibold
                                        ",
                                        onclick: {
                                            let gid = group_id.clone();
                                            let uid = user_id.clone();

                                            move |_| {
                                                let gid = gid.clone();
                                                let uid = uid.clone();

                                                spawn(async move {
                                                    if let Ok((_, token)) = crate::utils::functions::get_user_id_and_session_token().await {
                                                        if crate::groups::backend::invites::decline_invite(gid, uid, token)
                                                            .await
                                                            .is_ok()
                                                        {
                                                            invites_res.restart();
                                                        }
                                                    }
                                                });
                                            }
                                        },
                                        "Decline"
                                    }
                                }
                            }
                        }
                    }
                ),
                Some(Err(e)) => rsx!(div { class: "text-red-400 text-sm", "{e}" }),
                None => rsx!(div { class: "text-white/40 text-sm", "Loading…" }),
            }
        }
    }
}

/// Live-search dropdown for inviting users to a group.
///
/// Fires a search request after every keystroke (minimum 2 characters).
/// Clicking a result sends the invite and collapses the dropdown.
// rsx! macro does not support else if chains in class attributes
#[allow(clippy::collapsible_else_if)]
#[component]
pub fn UserSearchDropdown(
    group_id: String,
    current_user_id: String,
    on_invite_sent: EventHandler<()>,
) -> Element {
    let mut search_query = use_signal(String::new);
    let mut search_results = use_signal(Vec::<(String, String)>::new);
    let mut is_searching = use_signal(|| false);
    let mut is_inviting = use_signal(|| false);
    let mut invite_status = use_signal(|| Option::<String>::None);

    // Re-run search whenever the query text changes
    let current_user_id_search = current_user_id.clone();
    let _ = use_effect(move || {
        let query = search_query.read().clone();
        let uid = current_user_id_search.clone();

        spawn(async move {
            if query.trim().len() < 2 {
                search_results.set(vec![]);
                return;
            }

            is_searching.set(true);

            if let Ok((_, token)) = crate::utils::functions::get_user_id_and_session_token().await {
                match crate::groups::backend::invites::search_users_by_username(query, uid, token)
                    .await
                {
                    Ok(results) => search_results.set(results),
                    Err(_) => search_results.set(vec![]),
                }
            }

            is_searching.set(false);
        });
    });

    rsx! {
        div { class: "relative mb-4",
            div { class: "text-white/60 text-xs tracking-[0.18em] mb-2", "INVITE USER" }

            input {
                class: "
                    w-full px-4 py-3 rounded-2xl
                    bg-black/20 border border-white/10
                    text-white placeholder:text-white/30
                    outline-none
                ",
                placeholder: "Search by username...",
                value: "{search_query}",
                oninput: move |e| search_query.set(e.value()),
            }

            // Results dropdown (positioned absolutely below the input)
            if !search_results.read().is_empty() {
                div {
                    class: "
                        absolute top-full left-0 right-0 mt-2
                        bg-[#0a0e1a] border border-white/10
                        rounded-2xl overflow-hidden
                        max-h-48 overflow-y-auto
                        z-50
                    ",

                    for (user_id, username) in search_results.read().iter() {
                        button {
                            key: "{user_id}",
                            class: "
                                w-full px-4 py-3
                                hover:bg-white/10 transition
                                text-left flex items-center justify-between
                            ",
                            onclick: {
                                let uid = user_id.clone();
                                let uname = username.clone();
                                let gid = group_id.clone();
                                let inviter_id = current_user_id.clone();

                                move |_| {
                                    if is_inviting() { return; }
                                    let uid = uid.clone();
                                    let uname = uname.clone();
                                    let gid = gid.clone();
                                    let inviter_id = inviter_id.clone();

                                    is_inviting.set(true);
                                    search_query.set(String::new());
                                    search_results.set(vec![]);

                                    spawn(async move {
                                        invite_status.set(Some(format!("Inviting {}...", uname)));

                                        if let Ok((_, token)) = crate::utils::functions::get_user_id_and_session_token().await {
                                            match crate::groups::backend::invites::invite_user(gid, uid, inviter_id, token).await {
                                                Ok(_) => {
                                                    let _ = sync_local_to_remote_db().await;
                                                    invite_status.set(Some(format!("✓ Invited {}", uname)));
                                                    on_invite_sent.call(());
                                                }
                                                Err(e) => {
                                                    let msg = if e.to_string().contains("unique_user_per_group") {
                                                        "Error: User is already in this group".to_string()
                                                    } else {
                                                        format!("Error: {}", e)
                                                    };
                                                    invite_status.set(Some(msg));
                                                }
                                            }
                                        }
                                        is_inviting.set(false);
                                    });
                                }
                            },

                            span { class: "text-white", "{username}" }
                            span { class: "text-white/40 text-sm", "Invite →" }
                        }
                    }
                }
            }

            if *is_searching.read() {
                div { class: "text-white/40 text-sm mt-2", "Searching..." }
            }

            if let Some(status) = invite_status.read().as_ref() {
                div {
                    class:        if status.starts_with("✓") {
                        "text-green-400 text-sm mt-2"
                    } else if status.starts_with("Error") {
                        "text-red-400 text-sm mt-2"
                    } else {
                        "text-white/40 text-sm mt-2"
                    },
                    "{status}"
                }
            }
        }
    }
}