use crate::utils::structs::{GroupLight, TodoListLight};
use dioxus::prelude::*;
use uuid::Uuid;

#[derive(Clone, PartialEq, Debug)]
pub enum GroupFilter {
    AllGroups,
    Personal,
    Group(String),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ListFilter {
    AllLists,
    SpecificList(String),
}

#[component]
pub fn FilterSidebar(
    groups: Vec<GroupLight>,
    all_lists: Vec<TodoListLight>,
    selected_category: Signal<GroupFilter>,
    selected_list: Signal<ListFilter>,
) -> Element {
    //"Schattenlisten" raus filter, also Listen, die dem User/den Gruppen zugerdnet sind für die Todos die eigentlich keiner Liste zugeordnet sein sollen
    let visible_lists: Vec<TodoListLight> = all_lists
        .iter()
        .filter(|l| {
            //Name dieser Schattenlsten ist die ID des USers/der Grupper -> Schlägt ein Parsing des namen in UUid fehl -> ist diese Liste eine liste die man auch sehen soll
            Uuid::parse_str(&l.name).is_err()
        })
        .cloned()
        .collect();

    // Private Listen rausfiltern aus übergebenen listen
    let private_lists_all: Vec<TodoListLight> = visible_lists
        .iter()
        .filter(|l| l.list_type == "private")
        .cloned()
        .collect();

    // Aufteilen der privaten Listen in Tupel aus Event-basierte ToDo-Listen und "Standard"
    let (private_event_lists, private_standard_lists): (Vec<TodoListLight>, Vec<TodoListLight>) =
        private_lists_all
            .into_iter()
            .partition(|l| l.attached_to_calendar_event.is_some());

    // Gruppenlisten sammeln als Tupel mit Gruppe
    let groups_with_lists: Vec<(GroupLight, Vec<TodoListLight>)> = groups
        .into_iter()
        .map(|group| {
            let lists_for_thsi_group: Vec<TodoListLight> = visible_lists
                .iter()
                .filter(|l| l.group_id.as_deref() == Some(&group.id))
                .cloned()
                .collect();
            (group, lists_for_thsi_group)
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
            // Alle ToDos Header
            div { class: "px-3 mb-6",
                SidebarItem {
                    label: "All To-Do's".to_string(),
                    is_active: *selected_category.read() == GroupFilter::AllGroups,
                    is_header: true,
                    onclick: move |_| {
                        selected_category.set(GroupFilter::AllGroups);
                        selected_list.set(ListFilter::AllLists);
                    }
                }
            }

            // Personal ToDos Header
            div { class: "mb-6",
                div { class: "px-3",
                    SidebarItem {
                        label: "Personal To-Do's".to_string(),
                        is_active: *selected_category.read() == GroupFilter::Personal && *selected_list.read() == ListFilter::AllLists,
                        is_header: true,
                        onclick: move |_| {
                            selected_category.set(GroupFilter::Personal);
                            selected_list.set(ListFilter::AllLists);
                        }
                    }
                }

                // Personal ToDo-Lists
                div { class: "flex flex-col gap-1 mt-1 px-3",
                    // Standard Listen (nicht event based)
                    SectionHeader { label: "To-Do-Lists".to_string() }
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

                    // Personal Event-Linked Listen
                    SectionHeader { label: "Event-linked To-Do-Lists".to_string() }
                    {private_event_lists.into_iter().map(|list| {
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
                }
            }

            // Gruppen ToDos
            {groups_with_lists.into_iter().map(|(group, all_group_lists)| {
                let (event_lists, standard_lists): (Vec<TodoListLight>, Vec<TodoListLight>) = all_group_lists
                    .into_iter()
                    .partition(|l| l.attached_to_calendar_event.is_some());

                let group_id_header = group.id.clone();
                let group_id_std = group.id.clone();
                let group_id_evt = group.id.clone();

                rsx! {
                    div { class: "mb-6", key: "{group.id}",
                        // Headers für Groups
                        div { class: "px-3",
                            SidebarItem {
                                label: group.name.clone(),
                                is_active: if let GroupFilter::Group(g_id) = &*selected_category.read() {
                                    g_id == &group.id && *selected_list.read() == ListFilter::AllLists
                                } else { false },
                                is_header: true,
                                onclick: move |_| {
                                    selected_category.set(GroupFilter::Group(group_id_header.clone()));
                                    selected_list.set(ListFilter::AllLists);
                                }
                            }
                        }

                        // Standard Group-Listen (Nicht Event based)
                        div { class: "flex flex-col gap-1 mt-1 px-3",
                            SectionHeader { label: "To-Do-Lists".to_string() }
                            {standard_lists.into_iter().map(|list| {
                                let g_id = group_id_std.clone();
                                let l_id = list.id.clone();
                                rsx! {
                                    SidebarItem {
                                        label: list.name.clone(),
                                        is_active: if let ListFilter::SpecificList(id) = &*selected_list.read() { id == &list.id } else { false },
                                        is_header: false,
                                        indent: true,
                                        onclick: move |_| {
                                            selected_category.set(GroupFilter::Group(g_id.clone()));
                                            selected_list.set(ListFilter::SpecificList(l_id.clone()));
                                        }
                                    }
                                }
                            })}

                            // Group Event-based Listen
                            SectionHeader { label: "Event-linked To-Do-Lists".to_string() }
                            {event_lists.into_iter().map(|list| {
                                let g_id = group_id_evt.clone();
                                let l_id = list.id.clone();
                                rsx! {
                                    SidebarItem {
                                        label: list.name.clone(),
                                        is_active: if let ListFilter::SpecificList(id) = &*selected_list.read() { id == &list.id } else { false },
                                        is_header: false,
                                        indent: true,
                                        onclick: move |_| {
                                            selected_category.set(GroupFilter::Group(g_id.clone()));
                                            selected_list.set(ListFilter::SpecificList(l_id.clone()));
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
            })}
        }
    }
}

#[component]
fn SectionHeader(label: String) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 10px; margin: 12px 0 4px 0; padding-left: 14px; opacity: 0.8;",
            // Label
            span {
                style: "font-size: 10px; font-weight: 700; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em; white-space: nowrap;",
                "{label}"
            }
            // Linie
            div { style: "height: 1px; flex: 1; background: rgba(255,255,255,0.1);" }
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
            style: "cursor: pointer; padding: 6px 12px; padding-left: {indent_px}; border-radius: 6px; background-color: {bg_color}; color: {text_color}; font-weight: {font_weight}; font-size: {font_size}; transition: all 0.2s; display: flex; align-items: center; justify-content: space-between;",
            class: "hover:bg-[#1a1b26] hover:text-white",
            span { "{label}" }
        }
    }
}
