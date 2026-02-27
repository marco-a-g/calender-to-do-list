/*
Side Note Important! :  be aware that major parts of the css styling was made with LLM's (GroundLayer with ChatGpt & some details with Claude)
                        refactoring parts were consulted with LLM (Claude)
                        anything else is highlighted in the spot where it was used
*/

/*
Members tab for the group detail page.

Displays all group members with their roles fetched from the local SQLite
cache (offline-first). The list updates reactively when the resource resolves.
*/

use crate::groups::backend::members::fetch_members;
use dioxus::prelude::*;

/// Returns (display_label, tailwind_classes) for a given role string.
fn role_badge_classes(role: &str) -> (&'static str, &'static str) {
    match role {
        "owner" => (
            "OWNER",
            "bg-purple-500/15 text-purple-200 border-purple-400/20",
        ),
        "admin" => ("ADMIN", "bg-blue-500/15 text-blue-200 border-blue-400/20"),
        _ => ("MEMBER", "bg-white/5 text-white/60 border-white/10"),
    }
}

/// Shows all members of a group in a scrollable list.
#[component]
pub fn MembersTab(group_id: String, open_invite_from_right: Signal<bool>) -> Element {
    let group_id_for_fetch = group_id.clone();

    let members = use_resource(move || {
        let gid = group_id_for_fetch.clone();
        async move { fetch_members(gid).await }
    });

    rsx! {
        div { class: "w-full min-h-0 flex flex-col gap-4",
            div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 flex-none",
                div {
                    div { class: "text-white/60 text-xs tracking-[0.18em]", "MEMBERS" }
                    div { class: "text-white/40 text-sm mt-1",
                        match members.read().as_ref() {
                            Some(Ok(list)) => format!("{} in this group", list.len()),
                            _ => "Loading...".to_string(),
                        }
                    }
                }
            }

            div { class: "w-full min-h-0",
                match members.read().as_ref() {
                    Some(Ok(list)) => rsx!(
                        div { class: "flex flex-col gap-2 pb-2",
                            for (g_id, user_id, username, role) in list.iter() {
                                MemberRow {
                                    key: "{g_id}-{user_id}",
                                    username: username.clone(),
                                    user_id: user_id.clone(),
                                    role: role.clone(),
                                }
                            }
                        }
                    ),
                    Some(Err(e)) => rsx!(div { class: "text-red-400", "Error: {e}" }),
                    None => rsx!(div { class: "text-white/40", "Loading members…" }),
                }
            }
        }
    }
}

/// Single member row showing avatar initial, username, and role badge.
#[component]
fn MemberRow(username: String, user_id: String, role: String) -> Element {
    let (role_label, role_class) = role_badge_classes(&role);

    rsx! {
        div {
            class: "
                flex flex-col sm:flex-row sm:items-center sm:justify-between
                gap-3 px-5 py-4 rounded-3xl
                bg-white/5 border border-white/10
                hover:bg-white/10 transition
            ",

            div { class: "flex items-center gap-4 min-w-0",
                div {
                    class: "
                        w-10 h-10 rounded-2xl
                        bg-white/10 border border-white/10
                        flex items-center justify-center
                        text-sm font-bold text-white/90
                        flex-none
                    ",
                    "{username.chars().next().unwrap_or('?').to_uppercase()}"
                }

                div { class: "min-w-0",
                    div { class: "text-white font-semibold truncate", "{username}" }
                    div { class: "mt-1 inline-flex flex-wrap items-center gap-2",
                        span {
                            class: format!("px-2.5 py-1 rounded-full text-[11px] font-semibold border {}", role_class),
                            "{role_label}"
                        }
                    }
                }
            }
        }
    }
}
