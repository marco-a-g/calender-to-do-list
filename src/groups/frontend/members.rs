use crate::groups::backend::members::{
    fetch_members, invite_member, remove_member, update_member_role,
};
use dioxus::prelude::*;

fn role_badge_classes(role: &str) -> (&'static str, &'static str) {
    match role {
        "Owner" => (
            "OWNER",
            "bg-purple-500/15 text-purple-200 border-purple-400/20",
        ),
        "Admin" => ("ADMIN", "bg-blue-500/15 text-blue-200 border-blue-400/20"),
        _ => ("MEMBER", "bg-white/5 text-white/60 border-white/10"),
    }
}

/// Mock: später aus Auth/Group-State ableiten
fn current_user_role_mock() -> &'static str {
    "Admin" // "Member" | "Admin" | "Owner"
}

fn can_manage_members(current_role: &str) -> bool {
    current_role == "Admin" || current_role == "Owner"
}

#[component]
pub fn MembersTab(group_id: i32, mut open_invite_from_right: Signal<bool>) -> Element {
    let current_role = current_user_role_mock();
    let can_manage = can_manage_members(current_role);

    let mut invite_open = use_signal(|| false);
    let mut invite_name = use_signal(String::new);
    let mut invite_role = use_signal(|| "Member".to_string());

    let mut members = use_resource(move || async move { fetch_members(group_id).await });

    let member_count = members
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|v| v.len())
        .unwrap_or(0);
    {
        let mut invite_open = invite_open.clone();
        use_effect(move || {
            if open_invite_from_right() {
                if can_manage && !invite_open() {
                    invite_open.set(true);
                }
                open_invite_from_right.set(false);
            }
        });
    }

    rsx! {
        div { class: "w-full min-h-0 flex flex-col gap-4",
            div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 flex-none",
                div {
                    div { class: "text-white/60 text-xs tracking-[0.18em]", "MEMBERS" }
                    div { class: "text-white/40 text-sm mt-1", "{member_count} in this group" }
                }

                if can_manage {
                    button {
                        class: "
                            w-full sm:w-auto
                            px-4 py-2 rounded-2xl
                            bg-white/5 hover:bg-white/10 transition
                            border border-white/10
                            text-sm font-semibold
                        ",
                        onclick: move |_| invite_open.set(!invite_open()),
                        if invite_open() { "Close invite" } else { "+ Invite" }
                    }
                } else {
                    div { class: "text-white/30 text-xs", "Only admins can manage members" }
                }
            }

            // INVITE PANEL
            if can_manage && invite_open() {
                div { class: "rounded-3xl bg-white/5 border border-white/10 p-5 flex-none",
                    div { class: "text-white/60 text-xs tracking-[0.18em] mb-3", "INVITE MEMBER" }

                    div { class: "flex flex-col sm:flex-row gap-3 sm:items-center",
                        input {
                            class: "
                                w-full sm:flex-1
                                px-4 py-3 rounded-2xl
                                bg-black/20 border border-white/10
                                text-white placeholder:text-white/30
                                outline-none
                            ",
                            placeholder: "Name (mock invite)",
                            value: "{invite_name}",
                            oninput: move |e| invite_name.set(e.value()),
                        }

                        select {
                            class: "
                                w-full sm:w-auto
                                px-3 py-3 rounded-2xl
                                bg-black/20 border border-white/10
                                text-white/90
                                outline-none
                            ",
                            value: "{invite_role}",
                            onchange: move |e| invite_role.set(e.value()),
                            option { value: "Member", "Member" }
                            option { value: "Admin", "Admin" }
                            option { value: "Owner", "Owner" }
                        }

                        button {
                            class: "
                                w-full sm:w-auto
                                px-4 py-3 rounded-2xl
                                bg-blue-600/80 hover:bg-blue-500/80 transition
                                font-semibold
                            ",
                            onclick: move |_| async move {
                                let name = invite_name.read().trim().to_string();
                                if name.is_empty() { return; }

                                let role = invite_role.read().clone();
                                let _ = invite_member(group_id, name, role).await;

                                invite_name.set(String::new());
                                members.restart();
                            },
                            "Send"
                        }
                    }
                }
            }

            div { class: "w-full min-h-0",
                match members.read().as_ref() {
                    Some(Ok(list)) => rsx!(
                        div { class: "flex flex-col gap-2 pb-2",
                            for (g_id, user_id, name, role) in list.iter() {
                                MemberRow {
                                    key: "{g_id}-{user_id}",
                                    group_id,
                                    user_id: *user_id,
                                    name: name.clone(),
                                    role: role.clone(),
                                    can_manage,
                                    on_refresh: move |_| members.restart(),
                                }
                            }
                        }
                    ),
                    Some(Err(e)) => rsx!(div { class: "text-red-400", "{e}" }),
                    None => rsx!(div { class: "text-white/40", "Loading members…" }),
                }
            }
        }
    }
}

#[component]
fn MemberRow(
    group_id: i32,
    user_id: i32,
    name: String,
    role: String,
    can_manage: bool,
    on_refresh: EventHandler<()>,
) -> Element {
    let (role_label, role_class) = role_badge_classes(&role);
    let is_owner = role == "Owner";

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
                    "{name.chars().next().unwrap_or('?')}"
                }

                div { class: "min-w-0",
                    div { class: "text-white font-semibold truncate", "{name}" }
                    div { class: "mt-1 inline-flex flex-wrap items-center gap-2",
                        span { class: format!("px-2.5 py-1 rounded-full text-[11px] font-semibold border {}", role_class), "{role_label}" }
                        span { class: "text-white/30 text-xs", "user_id: {user_id}" }
                    }
                }
            }

            div { class: "flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 w-full sm:w-auto",
                if can_manage {
                    select {
                        class: "
                            w-full sm:w-auto
                            px-3 py-2 rounded-2xl
                            bg-black/20 border border-white/10
                            text-white/90 text-sm
                            outline-none
                        ",
                        disabled: is_owner,
                        value: "{role}",
                        onchange: {
                            let uid = user_id;
                            move |e| async move {
                                let _ = update_member_role(group_id, uid, e.value()).await;
                                on_refresh.call(());
                            }
                        },
                        option { value: "Member", "Member" }
                        option { value: "Admin", "Admin" }
                        option { value: "Owner", "Owner" }
                    }
                }

                if can_manage && !is_owner {
                    button {
                        class: "
                            w-full sm:w-auto
                            px-4 py-2 rounded-2xl
                            bg-red-500/15 hover:bg-red-500/20 transition
                            border border-red-400/20
                            text-red-200 text-sm font-semibold
                        ",
                        onclick: move |_| async move {
                            let _ = remove_member(group_id, user_id).await;
                            on_refresh.call(());
                        },
                        "Kick"
                    }
                }
            }
        }
    }
}