/*
Side Note Important! :  be aware that major parts of the css styling was made with LLM's (GroundLayer with ChatGpt & some details with Claude)
                        refactoring parts were consulted with LLM (Claude)
                        anything else is highlighted in the spot where it was used
*/

//! Groups overview UI component.
//!
//! Displays a list of groups as clickable rows. Clicking a group navigates
//! to its detail page. This component is purely presentational — all data
//! fetching is handled by the parent component.

use dioxus::prelude::*;
use dioxus_router::use_navigator;
use server_fn::error::ServerFnError;

use crate::Route;

/// Resource type for group list data.
/// Tuple: (group_id, name, color_hex, member_count)
pub type GroupsRes = Resource<Result<Vec<(String, String, String, i32)>, ServerFnError>>;

/// Renders the groups list or a loading/error state based on resource status.
#[component]
pub fn GroupsOverview(groups_res: GroupsRes) -> Element {
    rsx! {
        div {
            match groups_res.read().as_ref() {
                Some(Ok(list)) => rsx!(
                    div { class: "flex flex-col gap-2",
                        for (id, name, color, member_count) in list.iter() {
                            GroupRow {
                                id: id.clone(),
                                name: name.clone(),
                                color: color.clone(),
                                member_count: *member_count
                            }
                        }
                    }
                ),
                Some(Err(e)) => rsx!(div { class: "text-red-400", "{e}" }),
                None => rsx!(div { class: "text-white/50", "Loading..." }),
            }
        }
    }
}

/// Single group row. Navigates to group detail page on click.
#[component]
fn GroupRow(id: String, name: String, color: String, member_count: i32) -> Element {
    let nav = use_navigator();

    rsx! {
        div {
            onclick: {
                let id = id.clone();
                move |_| {
                    nav.push(Route::GroupDetail { id: id.clone() });
                }
            },
            class: "
                cursor-pointer
                bg-white/5 border border-white/10
                backdrop-blur-xl rounded-2xl
                px-4 py-3
                hover:bg-white/10
            ",

            div { class: "flex justify-between items-center",
                div { class: "flex items-center gap-3",
                    span {
                        class: "w-2.5 h-2.5 rounded-full",
                        style: format!("background: {color};")
                    }
                    span { class: "text-white font-semibold text-[15px]", "{name}" }
                }
                span { class: "text-white/40 text-sm", "{member_count}" }
            }
        }
    }
}
