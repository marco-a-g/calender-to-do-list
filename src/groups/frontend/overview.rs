use dioxus::prelude::*;
use dioxus_router::use_navigator;

use crate::Route;

pub type GroupsRes = Resource<Result<Vec<(i32, String, String, i32)>, ServerFnError>>;

#[component]
pub fn GroupsOverview(groups_res: GroupsRes) -> Element {
    rsx! {
        div {
            div { class: "text-white/60 text-xs tracking-[0.18em] mb-4", "" }

            match groups_res.read().as_ref() {
                Some(Ok(list)) => rsx!(
                    div { class: "flex flex-col gap-2",
                        for (id, name, color, member_count) in list.iter() {
                            GroupRow {
                                id: *id,
                                name: name.clone(),
                                color: color.clone(),
                                member_count: *member_count
                            }
                        }
                    }
                ),
                Some(Err(e)) => rsx!(div { class: "text-red-400", "{e}" }),
                None => rsx!(div { class: "text-white/50", "Loading…" }),
            }
        }
    }
}

#[component]
fn GroupRow(id: i32, name: String, color: String, member_count: i32) -> Element {
    let nav = use_navigator();

    rsx! {
        div {
            onclick: move |_| {
                nav.push(Route::GroupDetail { id });
            },
            class: "cursor-pointer
                    bg-white/5 border border-white/10 
                    backdrop-blur-xl rounded-2xl 
                    px-4 py-3 
                    hover:bg-white/10",

            div { class: "flex justify-between items-center",
                div { class: "flex items-center gap-3",
                    span { class: "w-2.5 h-2.5 rounded-full", style: format!("background: {color};") }
                    span { class: "text-white font-semibold text-[15px]", "{name}" }
                }
                span { class: "text-white/40 text-sm", "{member_count}" }
            }
        }
    }
}
