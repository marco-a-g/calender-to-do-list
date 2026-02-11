use std::result;

/*
Groups UI module: list view and detail page
Contains two main pages:
- `GroupsPage`: Overview of user's groups with create group functionality
- `GroupDetailPage`: Single group view with tabs for members, files, and roles
*/
use dioxus::prelude::*;
use dioxus_router::use_navigator;

use crate::auth::backend::AuthStatus;
use crate::groups::backend::files::{delete_file, fetch_files, get_file_url, upload_file};
use crate::groups::frontend::invite_widget::{InvitesWidget, UserSearchDropdown};
use crate::groups::frontend::members::MembersTab;
use crate::groups::frontend::overview::{GroupsOverview, GroupsRes};
use crate::groups::frontend::roles_tab::RolesTab;
use crate::groups::{create_group, delete_group, fetch_group_by_id, fetch_groups};
use crate::utils::functions::get_user_id_and_session_token;
use crate::database::local::init_fetch::init_fetch_local_db::{fetch_groups_lokal_db, fetch_group_members_lokal_db,};
use crate::database::local::sync_local_db::sync_local_to_remote_db;

// Color palette for group creation (hex values matching DB format)
const GROUP_COLORS: [&str; 8] = [
    "#3A6BFF", "#A855F7", "#EC4899", "#10B981", "#F59E0B", "#06B6D4", "#EF4444", "#64748B",
];

// Page wrapper providing consistent background styling
#[component]
fn PageShell(children: Element) -> Element {
    rsx! {
        div { class: "relative w-full min-h-screen overflow-y-auto text-white",
            div {
                class: "
                    fixed inset-0 -z-10
                    bg-gradient-to-b from-[#070B18] via-[#050914] to-black
                "
            }
            div { class: "relative w-full min-h-screen", {children} }
        }
    }
}

// Main groups page showing user's groups and create group form
#[component]
pub fn GroupsPage(auth_status: Signal<AuthStatus>) -> Element {
    let current_user_id = match auth_status.read().clone() {
        AuthStatus::Authenticated { user_id } => Some(user_id.to_string()),
        _ => None,
    };

    let mut groups_res: GroupsRes = use_resource(move || {
        async move {
            let groups = fetch_groups_lokal_db().await?;
            let all_members = fetch_group_members_lokal_db().await?;
            let result: Vec<(String, String, String, i32)> = groups.into_iter().map(|g| {
                let member_count = all_members                   .iter()
                    .filter(|m| m.group_id == g.id && m.role != "invited")
                    .count() as i32;
                let color = if g.color.is_empty() { "#3A6BFF".to_string() } else { g.color };
                (g.id, g.name, color, member_count)
                })
                .collect();

            Ok(result)
        }
    });

    // Create group form state
    let mut name = use_signal(String::new);
    let mut color = use_signal(|| "#3A6BFF".to_string());
    let mut create_error = use_signal(|| Option::<String>::None);

    rsx! {
        PageShell {
            div { class: "w-full min-h-screen px-4 sm:px-6 lg:px-12 py-6 sm:py-8 lg:py-10",
                div { class: "mx-auto max-w-[1200px] w-full",
                    div { class: "grid grid-cols-1 lg:grid-cols-[520px_1px_520px] gap-6 lg:gap-10 items-start",
                        div { class: "flex flex-col",
                            div { class: "text-white/60 text-xs tracking-[0.18em] mb-6", "GROUPS" }
                            if let Some(ref e) = *create_error.read() {
                                div { class: "mb-4 p-3 rounded-xl bg-red-500/10 border border-red-400/30 text-red-300 text-sm", "{e}" }
                            }
                            GroupsOverview { groups_res }
                        }

                        div { class: "hidden lg:block w-px bg-white/10 h-full" }

                        div {
                            div {
                                class: "
                                    bg-white/5 border border-white/10
                                    backdrop-blur-xl
                                    rounded-3xl
                                    shadow-[0_20px_60px_rgba(0,0,0,0.55)]
                                    px-5 sm:px-6 lg:px-7 py-5 sm:py-6
                                ",

                                div { class: "text-white/60 text-xs tracking-[0.18em] mb-4", "ACTIONS" }

                                input {
                                    class: "
                                        w-full px-4 py-3 rounded-2xl
                                        bg-black/20 border border-white/10
                                        text-white placeholder:text-white/30
                                        outline-none
                                        mb-5
                                    ",
                                    placeholder: "New group name",
                                    value: "{name}",
                                    oninput: move |e| name.set(e.value())
                                }

                                div { class: "flex items-center justify-between mb-2",
                                    div { class: "text-white/60 text-xs tracking-[0.18em]", "COLOR" }
                                    div { class: "text-white/40 text-xs", "{color}" }
                                }

                                div { class: "flex flex-wrap gap-2 mb-6",
                                    for c in GROUP_COLORS {
                                        button {
                                            r#type: "button",
                                            class: format!(
                                                "w-7 h-7 rounded-full border border-white/20 {}",
                                                if *color.read() == c { "ring-1 ring-white/60" } else { "" }
                                            ),
                                            style: format!("background: {};", c),
                                            onclick: move |_| color.set(c.to_string()),
                                        }
                                    }
                                }

                                button {
                                    class: "
                                        w-full py-3 rounded-2xl
                                        bg-blue-600/80 hover:bg-blue-500/80
                                        transition font-semibold
                                    ",
                                    onclick: move |_| {
                                        let mut groups_res = groups_res.clone();
                                        let mut name = name.clone();
                                        let color = color.clone();
                                        let mut create_error = create_error.clone();

                                        spawn(async move {
                                            create_error.set(None);
                                            let n = name.read().trim().to_string();
                                            if n.is_empty() {
                                                create_error.set(Some("Please enter a group name.".to_string()));
                                                return;
                                            }
                                            let c = color.read().trim().to_string();

                                            match get_user_id_and_session_token().await {
                                                Ok((user_id, token)) => {
                                                    match create_group(n.clone(), c, user_id.to_string(), token).await {
                                                        Ok(_) => {
                                                            sync_local_to_remote_db().await;
                                                            name.set(String::new());
                                                            create_error.set(None);
                                                            groups_res.restart();
                                                        }
                                                        Err(e) => {
                                                            create_error.set(Some(format!("Failed to create group: {e}")));
                                                        }
                                                    }
                                                }
                                                Err(_) => {
                                                    create_error.set(Some("Not logged in or session expired.".to_string()));
                                                }
                                            }
                                        });
                                    },
                                    "+ Create New Group"
                                }
                                if let Some(uid) = current_user_id.clone() {
                                    InvitesWidget {
                                        user_id: uid,
                                        on_change: move |_| groups_res.restart(),
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

#[derive(Clone, Copy, PartialEq)]
enum DetailTab {
    Members,
    Files,
    Roles,
}

// Detail page for a single group with tabs for members, files, and roles
#[component]
pub fn GroupDetailPage(id: String, auth_status: Signal<AuthStatus>) -> Element {
    let nav = use_navigator();

    let current_user_id = match auth_status.read().clone() {
        AuthStatus::Authenticated { user_id } => Some(user_id.to_string()),
        _ => None,
    };

    // Fetch group metadata (returns None if not found or not accessible via RLS)
    let group_res = use_resource({
        let id = id.clone();
        let auth_status = auth_status.clone();
        move || {
            let id = id.clone();
            let auth_status = auth_status.clone();
            async move {
                if !matches!(auth_status.read().clone(), AuthStatus::Authenticated { .. }) {
                    return Ok(None);
                }
                let (user_id, token) = match get_user_id_and_session_token().await {
                    Ok(t) => t,
                    Err(_) => return Ok(None),
                };
                fetch_group_by_id(id, user_id.to_string(), token).await
            }
        }
    });

    let mut tab = use_signal(|| DetailTab::Members);
    let mut open_invite_from_right = use_signal(|| false);
    let mut show_invite_dropdown = use_signal(|| false);

    // Files loaded separately to avoid fetching when not on Files tab
    let mut files_res = use_resource({
        let group_id = id.clone();
        move || {
            let group_id = group_id.clone();
            async move {
                match get_user_id_and_session_token().await {
                    Ok((_, token)) => fetch_files(group_id, token).await,
                    Err(e) => Err(e.to_string()),
                }
            }
        }
    });

    let mut upload_open = use_signal(|| false);
    let mut upload_status = use_signal(|| Option::<String>::None);
    let mut delete_status = use_signal(|| Option::<String>::None);

    rsx! {
        PageShell {
            div { class: "w-full min-h-screen px-4 sm:px-6 lg:px-12 py-6 sm:py-8 lg:py-10",
                div { class: "mx-auto max-w-[1200px] w-full",
                    div { class: "grid grid-cols-1 lg:grid-cols-[1fr_1px_520px] gap-6 lg:gap-10 items-start w-full",
                        div { class: "min-h-0",
                            div {
                                class: "
                                    bg-white/5 border border-white/10
                                    backdrop-blur-xl
                                    rounded-3xl
                                    shadow-[0_20px_60px_rgba(0,0,0,0.55)]
                                    px-5 sm:px-6 lg:px-7 py-5 sm:py-6
                                    flex flex-col
                                    overflow-hidden
                                    lg:h-full lg:min-h-0
                                ",

                                match group_res.read().as_ref() {
                                    Some(Ok(Some((gid, name, color)))) => {
                                        let gid_for_members = gid.clone();

                                        rsx!(
                                            div { class: "flex flex-col",
                                                // Group header
                                                div { class: "flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4 mb-6 flex-none",
                                                    div {
                                                        div { class: "text-white/60 text-xs tracking-[0.18em] mb-2", "GROUP OVERVIEW" }
                                                        div { class: "text-2xl sm:text-3xl font-semibold leading-tight", "{name}" }
                                                        div { class: "text-white/40 mt-1 text-sm", "Group ID: {gid}" }
                                                    }

                                                    div {
                                                        class: "px-3 py-1.5 rounded-full text-xs font-semibold bg-black/20 border border-white/10",
                                                        style: format!("color: {};", color),
                                                        "{color}"
                                                    }
                                                }

                                                // Tab navigation
                                                div { class: "flex gap-2 mb-4 flex-none overflow-x-auto whitespace-nowrap -mx-1 px-1",
                                                    button {
                                                        class: if tab() == DetailTab::Members {
                                                            "px-3 sm:px-4 py-2 rounded-2xl bg-white/10 border border-white/10 text-sm font-medium"
                                                        } else {
                                                            "px-3 sm:px-4 py-2 rounded-2xl text-sm text-white/40 hover:text-white/70 transition"
                                                        },
                                                        onclick: move |_| tab.set(DetailTab::Members),
                                                        "Members"
                                                    }

                                                    button {
                                                        class: if tab() == DetailTab::Files {
                                                            "px-3 sm:px-4 py-2 rounded-2xl bg-white/10 border border-white/10 text-sm font-medium"
                                                        } else {
                                                            "px-3 sm:px-4 py-2 rounded-2xl text-sm text-white/40 hover:text-white/70 transition"
                                                        },
                                                        onclick: move |_| {
                                                            tab.set(DetailTab::Files);
                                                            files_res.restart();
                                                        },
                                                        "Files"
                                                    }

                                                    button {
                                                        class: if tab() == DetailTab::Roles {
                                                            "px-3 sm:px-4 py-2 rounded-2xl bg-white/10 border border-white/10 text-sm font-medium"
                                                        } else {
                                                            "px-3 sm:px-4 py-2 rounded-2xl text-sm text-white/40 hover:text-white/70 transition"
                                                        },
                                                        onclick: move |_| tab.set(DetailTab::Roles),
                                                        "Roles"
                                                    }
                                                }

                                                // Tab content
                                                div { class: "flex-1 min-h-0 overflow-hidden",
                                                    match tab() {
                                                        DetailTab::Members => rsx!(
                                                            MembersTab {
                                                                group_id: gid_for_members,
                                                                open_invite_from_right: open_invite_from_right,
                                                            }
                                                        ),

                                                        DetailTab::Files => rsx!(
                                                            div { class: "min-h-0 flex flex-col overflow-visible lg:overflow-hidden",
                                                                div { class: "flex items-center justify-between flex-none mb-3",
                                                                    div {
                                                                        div { class: "text-white/60 text-xs tracking-[0.18em]", "FILES" }
                                                                        div { class: "text-white/40 text-sm mt-1", "File uploads" }
                                                                    }

                                                                    button {
                                                                        class: "
                                                                            px-4 py-2 rounded-2xl
                                                                            bg-white/5 hover:bg-white/10 transition
                                                                            border border-white/10
                                                                            text-sm font-semibold
                                                                        ",
                                                                        onclick: move |_| {
                                                                            upload_status.set(None);
                                                                            upload_open.set(!upload_open());
                                                                        },
                                                                        if upload_open() { "Close upload" } else { "+ Upload" }
                                                                    }
                                                                }

                                                                // Upload panel
                                                                if upload_open() {
                                                                    div { class: "flex-none mb-4 rounded-3xl bg-white/5 border border-white/10 p-4",
                                                                        div { class: "text-white/60 text-xs tracking-[0.18em] mb-3", "UPLOAD" }

                                                                        div { class: "flex flex-col gap-3",
                                                                            button {
                                                                                class: "
                                                                                    w-full px-4 py-3 rounded-2xl
                                                                                    bg-blue-500/20 hover:bg-blue-500/30
                                                                                    border border-blue-400/30
                                                                                    text-blue-200 font-semibold
                                                                                    transition
                                                                                ",
                                                                                onclick: {
                                                                                    let gid_upload = gid.clone();
                                                                                    let mut files_res = files_res.clone();
                                                                                    let mut upload_status = upload_status.clone();

                                                                                    move |_| {
                                                                                        let gid_upload = gid_upload.clone();
                                                                                        let mut files_res = files_res.clone();
                                                                                        let mut upload_status = upload_status.clone();

                                                                                        // Native file picker (desktop)
                                                                                        let picked = rfd::FileDialog::new().pick_file();
                                                                                        let Some(path) = picked else {
                                                                                            upload_status.set(Some("No file selected.".to_string()));
                                                                                            return;
                                                                                        };

                                                                                        let filename = path
                                                                                            .file_name()
                                                                                            .map(|s| s.to_string_lossy().to_string())
                                                                                            .unwrap_or_else(|| "upload.bin".to_string());

                                                                                        upload_status.set(Some(format!("Uploading {}...", filename)));

                                                                            spawn(async move {
                                                                                match tokio::fs::read(&path).await {
                                                                                    Ok(bytes) => {
                                                                                        match get_user_id_and_session_token().await {
                                                                                            Ok((_, token)) => {
                                                                                                match upload_file(
                                                                                                    gid_upload,
                                                                                                    filename,
                                                                                                    bytes,
                                                                                                    "application/octet-stream".to_string(),
                                                                                                    token,
                                                                                                )
                                                                                                .await
                                                                                                {
                                                                                                    Ok(_) => {
                                                                                                        sync_local_to_remote_db().await;
                                                                                                        upload_status.set(Some("Uploaded!".to_string()));
                                                                                                        files_res.restart();
                                                                                                    }
                                                                                                    Err(e) => {
                                                                                                        upload_status.set(Some(format!("Upload failed: {e}")));
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                            Err(e) => {
                                                                                                upload_status.set(Some(format!("Not authenticated: {e}")));
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    Err(e) => {
                                                                                        upload_status.set(Some(format!("Failed to read file: {e}")));
                                                                                    }
                                                                                }
                                                                            });
                                                                                    }
                                                                                },
                                                                                "Choose file…"
                                                                            }

                                                                            if let Some(msg) = upload_status() {
                                                                                div { class: "text-white/40 text-sm", "{msg}" }
                                                                            }
                                                                        }
                                                                    }
                                                                }

                                                                // File list
                                                                div { class: "flex-1 min-h-0 overflow-auto pr-1",
                                                                    match files_res.read().as_ref() {
                                                                        Some(Ok(list)) => rsx!(
                                                                            div { class: "flex flex-col gap-2 pb-2",
                                                                                for (group_id, file_id, filename, uploaded_at) in list.iter() {
                                                                                    div {
                                                                                        key: "{group_id}-{file_id}",
                                                                                        class: "
                                                                                            flex flex-col sm:flex-row sm:items-center sm:justify-between
                                                                                            gap-3
                                                                                            px-5 py-4 rounded-3xl
                                                                                            bg-white/5 border border-white/10
                                                                                            hover:bg-white/10 transition
                                                                                        ",

                                                                                        div {
                                                                                            div { class: "text-white font-semibold", "{filename}" }
                                                                                            div { class: "text-white/40 text-sm mt-1", "Uploaded: {uploaded_at}" }
                                                                                        }

                                                                                        div { class: "flex gap-2",
                                                                                            button {
                                                                                                class: "
                                                                                                    px-4 py-2 rounded-2xl
                                                                                                    bg-blue-500/15 hover:bg-blue-500/20 transition
                                                                                                    border border-blue-400/20
                                                                                                    text-blue-200 text-sm font-semibold
                                                                                                ",
                                                                                                onclick: {
                                                                                                    let filename = filename.clone();
                                                                                                    let gid_download = gid.clone();

                                                                                                    move |_| {
                                                                                                        let filename = filename.clone();
                                                                                                        let gid_download = gid_download.clone();

                                                                                                        spawn(async move {
                                                                                                            match get_user_id_and_session_token().await {
                                                                                                                Ok((_, token)) => match get_file_url(gid_download, filename, token).await {
                                                                                                                    Ok(url) => {
                                                                                                                        let _ = open::that(&url);
                                                                                                                    }
                                                                                                                    Err(e) => println!("Download error: {}", e),
                                                                                                                },
                                                                                                                Err(e) => println!("Not authenticated: {}", e),
                                                                                                            }
                                                                                                        });
                                                                                                    }
                                                                                                },
                                                                                                "Download"
                                                                                            }

                                                                                            button {
                                                                                                class: "
                                                                                                    px-4 py-2 rounded-2xl
                                                                                                    bg-red-500/15 hover:bg-red-500/20 transition
                                                                                                    border border-red-400/20
                                                                                                    text-red-200 text-sm font-semibold
                                                                                                ",
                                                                                                onclick: {
                                                                                                    let filename = filename.clone();
                                                                                                    let gid_delete = gid.clone();
                                                                                                    let mut files_res = files_res.clone();

                                                                                                    move |_| {
                                                                                                        let gid_delete = gid_delete.clone();
                                                                                                        let filename = filename.clone();
                                                                                                        let mut files_res = files_res.clone();

                                                                                                        spawn(async move {
                                                                                                            if let Ok((_, token)) = get_user_id_and_session_token().await {
                                                                                                                let _ = delete_file(gid_delete, filename, token).await;
                                                                                                            }
                                                                                                            files_res.restart();
                                                                                                        });
                                                                                                    }
                                                                                                },
                                                                                                "Delete"
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        ),
                                                                        Some(Err(e)) => rsx!(div { class: "text-red-400", "{e}" }),
                                                                        None => rsx!(div { class: "text-white/40", "Loading files…" }),
                                                                    }
                                                                }
                                                            }
                                                        ),

                                                        DetailTab::Roles => rsx!(
                                                            RolesTab {
                                                                group_id: gid.clone(),
                                                                current_user_id: current_user_id.clone().unwrap_or_default(),
                                                            }
                                                        ),
                                                    }
                                                }
                                            }
                                        )
                                    }
                                    Some(Ok(None)) => rsx!(div { class: "text-white/50", "Group not found." }),
                                    Some(Err(e)) => rsx!(div { class: "text-red-400", "{e}" }),
                                    None => rsx!(div { class: "text-white/50", "Loading…" }),
                                }
                            }
                        }

                        div { class: "hidden lg:block w-px bg-white/10 h-full" }

                        // Actions panel
                        div { class: "min-h-0",
                            div {
                                class: "
                                    bg-white/5 border border-white/10
                                    backdrop-blur-xl
                                    rounded-3xl
                                    shadow-[0_20px_60px_rgba(0,0,0,0.55)]
                                    px-5 sm:px-6 lg:px-7 py-5 sm:py-6
                                ",

                                div { class: "text-white/60 text-xs tracking-[0.18em] mb-4", "ACTIONS" }

                                button {
                                    class: "w-full py-3 rounded-2xl bg-white/5 hover:bg-white/10 transition border border-white/10 font-medium mb-3",
                                    onclick: {
                                        let mut show_invite_dropdown = show_invite_dropdown.clone();
                                        move |_| {
                                            let currently_open = show_invite_dropdown();
                                            show_invite_dropdown.set(!currently_open);
                                        }
                                    },
                                    if show_invite_dropdown() { "Close Invite" } else { "+ Invite Member" }
                                }

                                if show_invite_dropdown() {
                                    if let Some(uid) = current_user_id.clone() {
                                        UserSearchDropdown {
                                            group_id: id.clone(),
                                            current_user_id: uid,
                                            on_invite_sent: {
                                                let mut show_invite_dropdown = show_invite_dropdown.clone();
                                                move |_| {
                                                    show_invite_dropdown.set(false);
                                                }
                                            },
                                        }
                                    } else {
                                        div { class: "text-red-300/80 text-sm mb-3", "Not authenticated." }
                                    }
                                }

                                button {
                                    class: "w-full py-3 rounded-2xl bg-white/5 hover:bg-white/10 transition border border-white/10 font-medium mb-3",
                                    onclick: move |_| {
                                        tab.set(DetailTab::Files);
                                        files_res.restart();
                                    },
                                    "Open Files"
                                }

                                if let Some(msg) = delete_status() {
                                    div { class: "text-red-300/80 text-sm mb-3", "{msg}" }
                                }

                                button {
                                    class: "w-full py-3 rounded-2xl bg-red-500/20 hover:bg-red-500/25 transition border border-red-400/30 text-red-200 font-medium",
                                    onclick: {
                                        let group_id = id.clone();
                                        let nav = nav.clone();
                                        let mut delete_status = delete_status.clone();

                                        move |_| {
                                            let group_id = group_id.clone();
                                            let nav = nav.clone();
                                            let mut delete_status = delete_status.clone();

                                            spawn(async move {
                                                delete_status.set(None);

                                                match get_user_id_and_session_token().await {
                                                    Ok((user_id, token)) => {
                                                        match delete_group(group_id, user_id.to_string(), token).await {
                                                            Ok(_) => {
                                                                sync_local_to_remote_db().await;
                                                                let _ = nav.push("/Groups");
                                                            }
                                                            Err(e) => {
                                                                delete_status.set(Some(format!("Delete failed: {e}")));
                                                            }
                                                        }
                                                    }
                                                    Err(_) => {
                                                        delete_status.set(Some("Not authenticated.".to_string()));
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "Delete Group"
                                }

                                button {
                                    class: "w-full py-3 rounded-2xl bg-orange-500/20 hover:bg-orange-500/25 transition border border-orange-400/30 text-orange-200 font-medium mt-3",
                                    onclick: {
                                        let group_id = id.clone();
                                        let nav = nav.clone();

                                        move |_| {
                                            let group_id = group_id.clone();
                                            let nav = nav.clone();

                                            spawn(async move {
                                                match crate::utils::functions::get_user_id_and_session_token().await {
                                                    Ok((user_id, token)) => {
                                                        match crate::groups::leave_group(group_id, user_id.to_string(), token).await {
                                                            Ok(_) => {
                                                                sync_local_to_remote_db().await;
                                                                let _ = nav.push("/Groups");
                                                            }
                                                            Err(e) => {
                                                                println!("Leave failed: {e}");
                                                            }
                                                        }
                                                    }
                                                    Err(_) => {
                                                        println!("Not authenticated");
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "Leave Group"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
