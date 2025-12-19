use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum FilterState {
    All,
    Personal,
    Group(i32),
}

#[component]
pub fn FilterView(groups: Vec<(i32, String)>, selected_filter: Signal<FilterState>) -> Element {
    rsx! {
        div {
            style:
            "width: 260px;
             background: linear-gradient(180deg, #11121b 0%, #05060b 100%); 
             border-right: 1px solid rgba(255,255,255,0.06); 
             display: flex; 
             flex-direction: column; 
             padding: 24px 16px; 
             gap: 20px;",

            h2 { style:
                "margin: 0 0 8px 12px;
                 font-size: 11px; 
                 letter-spacing: 0.12em; 
                 text-transform: uppercase; 
                 color: #9ca3af; 
                 font-weight: 600;", 
                 "Filters" }

            div { class: "flex flex-col gap-3",
                FilterButton { label: "All To-Do's".to_string(), active: selected_filter() == FilterState::All, onclick: move |_| selected_filter.set(FilterState::All) }
                FilterButton { label: "Personal To-Do's".to_string(), active: selected_filter() == FilterState::Personal, onclick: move |_| selected_filter.set(FilterState::Personal) }
            }

            div { style:
                "height: 1px;
                 background: rgba(255,255,255,0.06); 
                 margin: 0 8px;" }

            h2 { style:
                "margin: 8px 0 8px 12px;
                 font-size: 11px; 
                 letter-spacing: 0.12em; 
                 text-transform: uppercase; 
                 color: #9ca3af; 
                 font-weight: 600;", 
                 "Groups" }

            div { class: "flex-1 overflow-y-auto flex flex-col gap-3 pr-2",
                for g in groups {
                    FilterButton { label: g.1, active: selected_filter() == FilterState::Group(g.0), onclick: move |_| selected_filter.set(FilterState::Group(g.0)) }
                }
            }
        }
    }
}

#[component]
fn FilterButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div {
            onclick: move |evt| onclick.call(evt),
            style: format!(
                "position: relative;
                 padding: 12px 16px; 
                 border-radius: 12px; 
                 cursor: pointer; 
                 transition: all 0.2s ease;
                 background: {}; 
                 border: 1px solid {}; 
                 box-shadow: {};
                 display: flex; 
                 align-items: center; 
                 justify-content: space-between;
                 color: {}; 
                 font-weight: 500; 
                 font-size: 14px;",
                if active { "#2b2c33" } else { "transparent" },
                if active { "rgba(255,255,255,0.06)" } else { "transparent" },
                if active { "0 4px 14px rgba(0,0,0,0.4)" } else { "none" },
                if active { "#ffffff" } else { "#9ca3af" }
            ),
            div {
                style: format!(
                    "position: absolute; left: 0; top: 50%; transform: translateY(-50%);
                     width: 3px; height: 20px; border-radius: 0 2px 2px 0;
                     background: #3A6BFF; opacity: {}; transition: opacity 0.2s ease;", 
                     if active { "1" } else { "0" }
                )
            }
            "{label}"
        }
    }
}
