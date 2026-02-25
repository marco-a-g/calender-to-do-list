use dioxus::prelude::*;

//Mit LLM generierter Newsfeed mit Artikeln von dev.to die #Rust als Tag haben, um leeren Platz im Dashboard vorübergehend zu füllen
/// UI-Element that renders a Placeholder for a Chat feature in the future
///
/// Completely generated with LLM to temporarily cover empty space in Dasboard.
#[component]
pub fn DashboardChat() -> Element {
    rsx! {
        div {
            style: "text-align: center; color: rgba(255,255,255,0.3);",
            div { style: "font-size: 40px; margin-bottom: 10px;", "💬" }
            div { style: "font-size: 18px; font-weight: 600;", "Team Chat" }
            div { style: "font-size: 14px; margin-top: 4px;", "Coming soon..." }
        }
    }
}
