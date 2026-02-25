use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum EditRecurrenceMode {
    OnlyInstance,
    WholeSeries,
}

/// UI-Element that renders a modal to determine the edit mode for a recurring task.
///
/// Acts as an "bridge" when a user attempts to edit a master recurring to-do.
/// Asks the user to specify whether changes should apply to the `WholeSeries`(updating the master record) or `OnlyInstance` (creating exception for the current date).
///
/// Conditionally rendered via `show_modal` signal and passes the user's choice back to the parent component using the `on_confirm`.
///
/// # Arguments
///
/// * `show_modal` - A signal controlling visibility of the modal.
/// * `on_close` - An event handler triggered when the user cancels the action or clicks the backdrop.
/// * `on_confirm` - An event handler that passes the chosen `EditRecurrenceMode`.
#[component]
pub fn EditRecurrenceChoiceModal(
    show_modal: Signal<bool>,
    on_close: EventHandler<()>,
    on_confirm: EventHandler<EditRecurrenceMode>,
) -> Element {
    if !show_modal() {
        return rsx! {};
    }

    rsx! {
        div {
            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); backdrop-filter: blur(5px); z-index: 70; display: flex; align-items: center; justify-content: center;",
            onclick: move |_| on_close.call(()),

            div {
                style: "background: #171923; width: 400px; padding: 24px; border-radius: 18px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 20px 50px rgba(0,0,0,0.9); display: flex; flex-direction: column; gap: 20px; text-align: center;",
                onclick: |e| e.stop_propagation(),

                // Icon / Header
                div {
                    div {
                        // NEU: SVG entfernt, font-size hinzugefügt, Emoji eingesetzt
                        style: "background: rgba(58, 107, 255, 0.1); width: 48px; height: 48px; border-radius: 50%; display: flex; align-items: center; justify-content: center; margin: 0 auto 16px auto; color: #3A6BFF; font-size: 24px;",
                        "🔄"
                    }
                    h2 { style: "color: white; font-size: 18px; margin: 0;", "Edit Recurring Task" }
                    p { style: "color: #9ca3af; font-size: 14px; margin-top: 8px; line-height: 1.5;",
                        "This is a repeating task. Do you want to change only this occurrence or the entire series?"
                    }
                }

                // Buttons
                div { class: "flex flex-col gap-3",

                    // Button für Bearbeitung nur dieser Instanz
                    button {
                        style: "background: transparent; border: 1px solid rgba(255,255,255,0.1); color: white; padding: 14px; border-radius: 10px; cursor: pointer; font-weight: 500; transition: background 0.2s; display: flex; flex-direction: column; align-items: start; gap: 4px; text-align: left;",
                        class: "hover:bg-white/5",
                        onclick: move |_| {
                            on_confirm.call(EditRecurrenceMode::OnlyInstance);
                            show_modal.set(false);
                        },
                        span { style: "font-size: 15px; font-weight: 600;", "This occurrence only" }
                        span { style: "font-size: 12px; color: #9ca3af;", "Creates an exception. The rest of the series remains unchanged." }
                    }

                    // Button für Bearbeitung der ganzen Reihe
                    button {
                        style: "background: rgba(58, 107, 255, 0.1); border: 1px solid #3A6BFF; color: white; padding: 14px; border-radius: 10px; cursor: pointer; font-weight: 500; transition: background 0.2s; display: flex; flex-direction: column; align-items: start; gap: 4px; text-align: left;",
                        class: "hover:bg-blue-600/20",
                        onclick: move |_| {
                            on_confirm.call(EditRecurrenceMode::WholeSeries);
                            show_modal.set(false);
                        },
                        span { style: "font-size: 15px; font-weight: 600; color: #3A6BFF;", "Entire series" }
                        span { style: "font-size: 12px; color: #9ca3af;", "Changes will apply to all future repetitions." }
                    }
                }

                // Cancel
                button {
                    style: "background: transparent; border: none; color: #6b7280; font-size: 14px; cursor: pointer; margin-top: 4px;",
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
            }
        }
    }
}
