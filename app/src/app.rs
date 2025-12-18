#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

static CSS: Asset = asset!("/assets/styles.css");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub fn App() -> Element {
    let mut overlay_visible = use_signal(|| true);
    let mut move_mode = use_signal(|| false);
    let mut status_msg = use_signal(String::new);

    // Read signals once at the top to avoid multiple borrow conflicts
    let is_visible = overlay_visible();
    let is_move_mode = move_mode();
    let status = status_msg();

    let toggle_overlay = move |_| {
        let current = overlay_visible();
        let cmd = if current { "hide_overlay" } else { "show_overlay" };

        async move {
            let result = invoke(cmd, JsValue::NULL).await;
            if let Some(success) = result.as_bool() {
                if success {
                    let new_state = !current;
                    overlay_visible.set(new_state);
                    if !new_state {
                        move_mode.set(false);
                    }
                    status_msg.set(String::new());
                }
            } else if let Some(err) = result.as_string() {
                status_msg.set(format!("Error: {}", err));
            }
        }
    };

    let toggle_move = move |_| {
        let current = overlay_visible();

        async move {
            if !current {
                status_msg.set("Overlay must be visible first".to_string());
                return;
            }

            let result = invoke("toggle_move_mode", JsValue::NULL).await;
            if let Some(new_mode) = result.as_bool() {
                move_mode.set(new_mode);
                status_msg.set(String::new());
            } else if let Some(err) = result.as_string() {
                status_msg.set(format!("Error: {}", err));
            }
        }
    };

    rsx! {
        link { rel: "stylesheet", href: CSS }
        main { class: "container",
            h1 { "Baras" }
            p { class: "subtitle", "SWTOR Combat Log Parser" }

            div { class: "controls",

                button {
                    class: if is_visible { "btn btn-active" } else { "btn" },
                    onclick: toggle_overlay,
                    if is_visible { "Hide Overlay" } else { "Show Overlay" }
                }

                button {
                    class: if is_move_mode { "btn btn-warning" } else { "btn" },
                    disabled: !is_visible,
                    onclick: toggle_move,
                    if is_move_mode { "Lock Position" } else { "Move Overlay" }
                }
            }

            if !status.is_empty() {
                p { class: "error", "{status}" }
            }

            div { class: "status",
                p {
                    "Overlay: "
                    span { class: if is_visible { "status-on" } else { "status-off" },
                        if is_visible { "Visible" } else { "Hidden" }
                    }
                }
                p {
                    "Mode: "
                    span { class: if is_move_mode { "status-warning" } else { "" },
                        if is_move_mode { "Move Mode (drag to reposition)" } else { "Locked" }
                    }
                }
            }
        }
    }
}
