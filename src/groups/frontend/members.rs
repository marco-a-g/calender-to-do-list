use crate::groups::backend::members::fetch_members;
use dioxus::prelude::*;

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

#[component]
pub fn MembersTab(group_id: String, mut open_invite_from_right: Signal<bool>) -> Element {
    let mut members = use_resource(move || {
        let gid = group_id.clone();
        async move { fetch_members(gid).await }
    });

    let member_count = members
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|v| v.len())
        .unwrap_or(0);

    rsx! {
        div { class: "w-full min-h-0 flex flex-col gap-4",
            div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 flex-none",
                div {
                    div { class: "text-white/60 text-xs tracking-[0.18em]", "MEMBERS" }
                    div { class: "text-white/40 text-sm mt-1", "{member_count} in this group" }
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

#[component]
fn MemberRow(username: String, user_id: String, role: String) -> Element {
    let (role_label, role_class) = role_badge_classes(&role);

    rsx! {
        div {
            class: "
                flex flex-col sm:flex-row sm:items-center sm:justify-between
                gap-3
                px-5 py-4 rounded-3xl
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
