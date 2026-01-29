use dioxus::prelude::*;
use dioxus_router::use_navigator;

use crate::auth::backend::AuthStatus;
use crate::groups::backend::files::{delete_file_mock, fetch_files, upload_file_mock};
use crate::groups::frontend::members::MembersTab;
use crate::groups::frontend::overview::{GroupsOverview, GroupsRes};
use crate::groups::{create_group, delete_group, fetch_group_by_id, fetch_groups};

const GROUP_COLORS: [&str; 8] = [
    "#3A6BFF", "#A855F7", "#EC4899", "#10B981", "#F59E0B", "#06B6D4", "#EF4444", "#64748B",
];

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

#[component]
pub fn GroupsPage(auth_status: Signal<AuthStatus>) -> Element {
    let mut groups_res: GroupsRes = use_resource({
        let auth_status = auth_status.clone();
        move || {
            let auth_status = auth_status.clone();
            async move {
                if let AuthStatus::Authenticated { user_id } = auth_status.read().clone() {
                    fetch_groups(user_id.to_string()).await
                } else {
                    Ok(vec![])
                }
            }
        }
    });

    let mut name = use_signal(String::new);
    let mut color = use_signal(|| "#3A6BFF".to_string());

    rsx! {
        PageShell {
            div { class: "w-full min-h-screen px-4 sm:px-6 lg:px-12 py-6 sm:py-8 lg:py-10",
                div { class: "mx-auto max-w-[1200px] w-full",
                    div { class: "grid grid-cols-1 lg:grid-cols-[520px_1px_520px] gap-6 lg:gap-10 items-start",
                        div { class: "flex flex-col",
                            div { class: "text-white/60 text-xs tracking-[0.18em] mb-6", "GROUPS" }
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
                                        let auth_status = auth_status.clone();

                                        spawn(async move {
                                            let n = name.read().trim().to_string();
                                            if n.is_empty() {
                                                return;
                                            }
                                            let c = color.read().trim().to_string();

                                            if let AuthStatus::Authenticated { user_id } = auth_status.read().clone() {
                                                let _ = create_group(n, c, user_id.to_string()).await;
                                                name.set(String::new());
                                                groups_res.restart();
                                            }
                                        });
                                    },
                                    "+ Create New Group"
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

#[component]
pub fn GroupDetailPage(id: String, auth_status: Signal<AuthStatus>) -> Element {
    let nav = use_navigator();

    let current_user_id = match auth_status.read().clone() {
        AuthStatus::Authenticated { user_id } => Some(user_id.to_string()),
        _ => None,
    };

    let group_res = use_resource({
        let id = id.clone();
        let auth_status = auth_status.clone();
        move || {
            let id = id.clone();
            let auth_status = auth_status.clone();
            async move {
                if let AuthStatus::Authenticated { user_id } = auth_status.read().clone() {
                    fetch_group_by_id(id, user_id.to_string()).await
                } else {
                    Ok(None)
                }
            }
        }
    });

    let mut tab = use_signal(|| DetailTab::Members);
    let mut open_invite_from_right = use_signal(|| false);

    let mut files_res = use_resource({
        let group_id = id.clone();
        move || {
            let group_id = group_id.clone();
            async move { fetch_files(group_id).await }
        }
    });

    let mut upload_open = use_signal(|| false);
    let mut upload_filename = use_signal(String::new);
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
                                    overflow-visible lg:overflow-hidden
                                    lg:h-full lg:min-h-0
                                ",

                                match group_res.read().as_ref() {
                                    Some(Ok(Some((gid, name, color)))) => {

                                        let gid_for_members = gid.clone();

                                        rsx!(
                                            div { class: "flex flex-col",
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

                                                div { class: "flex-1 min-h-0 overflow-visible lg:overflow-hidden",
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
                                                                        div { class: "text-white/40 text-sm mt-1", "Mock uploads" }
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

                                                                if upload_open() {
                                                                    div { class: "flex-none mb-4 rounded-3xl bg-white/5 border border-white/10 p-4",
                                                                        div { class: "text-white/60 text-xs tracking-[0.18em] mb-3", "UPLOAD (MOCK)" }

                                                                        div { class: "flex flex-col sm:flex-row gap-3 sm:items-center",
                                                                            input {
                                                                                class: "
                                                                                    flex-1 px-4 py-3 rounded-2xl
                                                                                    bg-black/20 border border-white/10
                                                                                    text-white placeholder:text-white/30
                                                                                    outline-none
                                                                                ",
                                                                                placeholder: "Filename (e.g. report.pdf)",
                                                                                value: "{upload_filename}",
                                                                                oninput: move |e| upload_filename.set(e.value()),
                                                                            }

                                                                            button {
                                                                                class: "
                                                                                    w-full sm:w-auto px-4 py-3 rounded-2xl
                                                                                    bg-blue-600/80 hover:bg-blue-500/80 transition
                                                                                    font-semibold
                                                                                ",
                                                                                onclick: {
                                                                                    let gid_upload = gid.clone();
                                                                                    let mut files_res = files_res.clone();
                                                                                    let mut upload_filename = upload_filename.clone();
                                                                                    let mut upload_status = upload_status.clone();

                                                                                    move |_| {
                                                                                        let gid_upload = gid_upload.clone();
                                                                                        let mut files_res = files_res.clone();
                                                                                        let mut upload_filename = upload_filename.clone();
                                                                                        let mut upload_status = upload_status.clone();

                                                                                        spawn(async move {
                                                                                            let filename = upload_filename.read().trim().to_string();
                                                                                            if filename.is_empty() {
                                                                                                upload_status.set(Some("Please enter a filename.".to_string()));
                                                                                                return;
                                                                                            }

                                                                                            match upload_file_mock(gid_upload, filename).await {
                                                                                                Ok(_) => {
                                                                                                    upload_filename.set(String::new());
                                                                                                    upload_status.set(Some("Uploaded (mock).".to_string()));
                                                                                                    files_res.restart();
                                                                                                }
                                                                                                Err(e) => upload_status.set(Some(format!("Upload failed: {e}"))),
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                },
                                                                                "Upload"
                                                                            }
                                                                        }

                                                                        if let Some(msg) = upload_status() {
                                                                            div { class: "text-white/40 text-sm mt-3", "{msg}" }
                                                                        }
                                                                    }
                                                                }

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

                                                                                        button {
                                                                                            class: "
                                                                                                w-full sm:w-auto px-4 py-2 rounded-2xl
                                                                                                bg-red-500/15 hover:bg-red-500/20 transition
                                                                                                border border-red-400/20
                                                                                                text-red-200 text-sm font-semibold
                                                                                            ",
                                                                                            onclick: {
                                                                                                let fid = *file_id;
                                                                                                let gid_delete = gid.clone();
                                                                                                let mut files_res = files_res.clone();

                                                                                                move |_| {
                                                                                                    let gid_delete = gid_delete.clone();
                                                                                                    let mut files_res = files_res.clone();

                                                                                                    spawn(async move {
                                                                                                        let _ = delete_file_mock(gid_delete, fid).await;
                                                                                                        files_res.restart();
                                                                                                    });
                                                                                                }
                                                                                            },
                                                                                            "Delete"
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
                                                            div { class: "text-white/40", "Roles UI comes next." }
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
                                    onclick: move |_| {
                                        tab.set(DetailTab::Members);
                                        open_invite_from_right.set(true);
                                    },
                                    "+ Invite Member"
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
                                        let current_user_id = current_user_id.clone();

                                        move |_| {
                                            let group_id = group_id.clone();
                                            let nav = nav.clone();
                                            let mut delete_status = delete_status.clone();
                                            let current_user_id = current_user_id.clone();

                                            spawn(async move {
                                                delete_status.set(None);

                                                let Some(user_id) = current_user_id else {
                                                    delete_status.set(Some("Not authenticated.".to_string()));
                                                    return;
                                                };

                                                match delete_group(group_id, user_id).await {
                                                    Ok(_) => {
                                                        let _ = nav.push("/Groups");
                                                    }
                                                    Err(e) => {
                                                        delete_status.set(Some(format!("Delete failed: {e}")));
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "Delete Group"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
