use crate::utils::structs::{GroupLight, TodoListLight};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum GroupFilter {
    All,
    Personal,
    Group(String),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ListFilter {
    AllInContext,
    SpecificList(String),
}

#[component]
pub fn FilterSidebar(
    groups: Vec<GroupLight>,
    all_lists: Vec<TodoListLight>,
    selected_category: Signal<GroupFilter>,
    selected_list: Signal<ListFilter>,
) -> Element {
    //Private Listen
    let private_lists_all: Vec<TodoListLight> = all_lists
        .iter()
        .filter(|l| l.list_type == "private")
        .cloned()
        .collect();

    //aufteilen von privaten Listen in Vectoren von KalenderEvent-Listen und Rest
    let (private_event_lists, private_standard_lists): (Vec<TodoListLight>, Vec<TodoListLight>) =
        private_lists_all
            .into_iter()
            .partition(|l| l.attached_to_calendar_event.is_some());

    // Sammelt TodoListen die zu Gruppen gehören ink. Gruppen
    let groups_with_lists: Vec<(GroupLight, Vec<TodoListLight>)> = groups
        .into_iter()
        .map(|group| {
            let lists_for_this_group: Vec<TodoListLight> = all_lists
                .iter()
                .filter(|l| l.group_id.as_deref() == Some(&group.id))
                .cloned()
                .collect();
            (group, lists_for_this_group)
        })
        .collect();

    rsx! {
        div {
            style: "
                width: 260px; 
                height: 100%; 
                display: flex; 
                flex-direction: column; 
                padding: 20px 0; 
                overflow-y: auto;
                background: linear-gradient(145deg, #1f222c 0%, #14161f 100%);
                border-radius: 18px;
                box-shadow: 0 18px 40px rgba(0,0,0,0.85);
                border: 1px solid rgba(255,255,255,0.06);
            ",

            // Ansicht: All To-Do's
            div { class: "px-3 mb-6",
                SidebarItem {
                    label: "All To-Do's".to_string(),
                    is_active: *selected_category.read() == GroupFilter::All,
                    is_header: true,
                    onclick: move |_| {
                        selected_category.set(GroupFilter::All);
                        selected_list.set(ListFilter::AllInContext);
                    }
                }
            }

            // Ansicht: Personal To-Do's
            div { class: "mb-6",
                // Header
                div { class: "px-3",
                    SidebarItem {
                        label: "Personal To-Do's".to_string(),
                        is_active: *selected_category.read() == GroupFilter::Personal && *selected_list.read() == ListFilter::AllInContext,
                        is_header: true,
                        onclick: move |_| {
                            selected_category.set(GroupFilter::Personal);
                            selected_list.set(ListFilter::AllInContext);
                        }
                    }
                }

                // Wrapper
                div { class: "flex flex-col gap-1 mt-1 px-3",
                    // Standard Listen
                    {private_standard_lists.into_iter().map(|list| {
                        let list_id = list.id.clone();
                        rsx! {
                            SidebarItem {
                                label: list.name.clone(),
                                is_active: if let ListFilter::SpecificList(id) = &*selected_list.read() { id == &list.id } else { false },
                                is_header: false,
                                indent: true,
                                onclick: move |_| {
                                    selected_category.set(GroupFilter::Personal);
                                    selected_list.set(ListFilter::SpecificList(list_id.clone()));
                                }
                            }
                        }
                    })}

                    // Trenner
                    if !private_event_lists.is_empty() {
                        // Querstrich
                        div { style: "height: 1px; background: rgba(255,255,255,0.1); margin: 8px 4px 8px 24px;" }

                        {private_event_lists.into_iter().map(|list| {
                            let list_id = list.id.clone();
                            rsx! {
                                SidebarItem {
                                    // Event-Label
                                    label: format!("Event: {}", list.name),
                                    is_active: if let ListFilter::SpecificList(id) = &*selected_list.read() { id == &list.id } else { false },
                                    is_header: false,
                                    indent: true,
                                    onclick: move |_| {
                                        selected_category.set(GroupFilter::Personal);
                                        selected_list.set(ListFilter::SpecificList(list_id.clone()));
                                    }
                                }
                            }
                        })}
                    }
                }
            }

            // Ansicht: GruppenListen
            {groups_with_lists.into_iter().map(|(group, all_group_lists)| {
                let (event_lists, standard_lists): (Vec<TodoListLight>, Vec<TodoListLight>) = all_group_lists
                    .into_iter()
                    .partition(|l| l.attached_to_calendar_event.is_some());
                let group_id_for_header = group.id.clone();
                let group_id_for_std_loop = group.id.clone();
                let group_id_for_evt_loop = group.id.clone();

                rsx! {
                    div { class: "mb-6", key: "{group.id}",
                        // Header
                        div { class: "px-3",
                            SidebarItem {
                                label: group.name.clone(),
                                is_active: if let GroupFilter::Group(g_id) = &*selected_category.read() {
                                    g_id == &group.id && *selected_list.read() == ListFilter::AllInContext
                                } else { false },
                                is_header: true,
                                onclick: move |_| {
                                    selected_category.set(GroupFilter::Group(group_id_for_header.clone()));
                                    selected_list.set(ListFilter::AllInContext);
                                }
                            }
                        }

                        // Wrapper
                        div { class: "flex flex-col gap-1 mt-1 px-3",

                            // StandardListen
                            {standard_lists.into_iter().map(|list| {
                                let group_id_item = group_id_for_std_loop.clone();
                                let list_id_item = list.id.clone();
                                rsx! {
                                    SidebarItem {
                                        label: list.name.clone(),
                                        is_active: if let ListFilter::SpecificList(id) = &*selected_list.read() { id == &list.id } else { false },
                                        is_header: false,
                                        indent: true,
                                        onclick: move |_| {
                                            selected_category.set(GroupFilter::Group(group_id_item.clone()));
                                            selected_list.set(ListFilter::SpecificList(list_id_item.clone()));
                                        }
                                    }
                                }
                            })}

                            // Trenner
                            if !event_lists.is_empty() {
                                // Querstrich
                                div { style: "height: 1px; background: rgba(255,255,255,0.1); margin: 8px 4px 8px 24px;" }

                                {event_lists.into_iter().map(|list| {
                                    let group_id_item = group_id_for_evt_loop.clone();
                                    let list_id_item = list.id.clone();
                                    rsx! {
                                        SidebarItem {
                                            // EventLabel
                                            label: format!("Event: {}", list.name),
                                            is_active: if let ListFilter::SpecificList(id) = &*selected_list.read() { id == &list.id } else { false },
                                            is_header: false,
                                            indent: true,
                                            onclick: move |_| {
                                                selected_category.set(GroupFilter::Group(group_id_item.clone()));
                                                selected_list.set(ListFilter::SpecificList(list_id_item.clone()));
                                            }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }
            })}
        }
    }
}

#[component]
fn SidebarItem(
    label: String,
    is_active: bool,
    is_header: bool,
    indent: Option<bool>,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let indent_px = if indent.unwrap_or(false) {
        "24px"
    } else {
        "10px"
    };

    let (bg_color, text_color, font_weight, font_size) = if is_active {
        (
            "#2b2c33",
            "#ffffff",
            "600",
            if is_header { "14px" } else { "13px" },
        )
    } else {
        (
            "transparent",
            if is_header { "#e5e7eb" } else { "#9ca3af" },
            if is_header { "600" } else { "400" },
            if is_header { "14px" } else { "13px" },
        )
    };

    rsx! {
        div {
            onclick: move |evt| onclick.call(evt),
            style: "cursor: pointer; padding: 6px 12px; padding-left: {indent_px}; border-radius: 6px; background-color: {bg_color}; color: {text_color}; font-weight: {font_weight}; font-size: {font_size}; transition: all 0.2s; display: flex; align-items: center;",
            class: "hover:bg-[#1a1b26] hover:text-white",
            "{label}"
        }
    }
}
