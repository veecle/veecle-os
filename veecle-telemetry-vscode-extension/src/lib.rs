//! `veecle-telemetry-ui` bindings for the extension.

#![forbid(unsafe_code)]
#![cfg(target_arch = "wasm32")]

use std::sync::mpsc;

use eframe::egui;
use serde::Deserialize;
use tsify::Tsify;
use veecle_telemetry_ui::app::VeecleTelemetryApp;
use veecle_telemetry_ui::connection::file_contents::{FileContents, FileContentsConnection};
use veecle_telemetry_ui::connection::{Connection, ConnectionMessage};
use wasm_bindgen::prelude::*;
use web_sys::MessageEvent;

/// Startup options for `veecle-telemetry-ui`.
#[derive(Tsify, Deserialize, Debug)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct StartupOptions {
    /// Whether to hide the menu bar in the app.
    pub is_preview: bool,
}

/// A document passed in from VS Code.
#[derive(Tsify, Deserialize, Debug)]
#[tsify(from_wasm_abi)]
pub struct Document {
    /// File name.
    pub name: String,
    /// File content in bytes.
    pub bytes: String,
}

/// Messages passed from the extension to `veecle-telemetry-ui`.
#[derive(Tsify, Deserialize, Debug)]
#[tsify(from_wasm_abi)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToWebviewMessage {
    /// An updated document.
    Document {
        /// The document.
        document: Document,
    },
}

#[derive(Debug)]
struct CodeConnection {
    receiver: mpsc::Receiver<Document>,

    connection: Box<dyn Connection>,
}

impl CodeConnection {
    fn new_boxed(receiver: mpsc::Receiver<Document>) -> Box<dyn Connection> {
        Box::new(Self {
            receiver,
            connection: FileContentsConnection::new_boxed(FileContents {
                name: String::new(),
                bytes: Default::default(),
            }),
        })
    }
}

impl Connection for CodeConnection {
    fn try_recv(&mut self) -> Option<ConnectionMessage> {
        if let Ok(document) = self.receiver.try_recv() {
            self.connection = FileContentsConnection::new_boxed(FileContents {
                name: document.name,
                bytes: document.bytes.into_bytes().into_boxed_slice().into(),
            });

            return Some(ConnectionMessage::Restart);
        }

        self.connection.try_recv()
    }

    fn is_continuous(&self) -> bool {
        self.connection.is_continuous()
    }

    fn is_done(&self) -> bool {
        self.connection.is_done()
    }
}

impl std::fmt::Display for CodeConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "vscode custom editor ({})", self.connection)
    }
}

/// Start `VeecleTelemetryApp`, should only be called once.
#[wasm_bindgen]
pub async fn start(options: StartupOptions) {
    re_log::setup_logging();

    let canvas = get_canvas();

    let web_options = eframe::WebOptions::default();

    let start_result = eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(move |cc| {
                let startup_options = if options.is_preview {
                    veecle_telemetry_ui::app::StartupOptions {
                        hide_menu: true,
                        connection: Some(create_connection(cc.egui_ctx.clone())),
                    }
                } else {
                    Default::default()
                };

                Ok(Box::new(VeecleTelemetryApp::new(cc, startup_options)))
            }),
        )
        .await;

    // Remove the loading text and spinner:
    let document = web_sys::window()
        .expect("No window")
        .document()
        .expect("No document");
    if let Some(loading_text) = document.get_element_by_id("loading-text") {
        match start_result {
            Ok(_) => {
                loading_text.remove();
            }
            Err(e) => {
                loading_text.set_inner_html(
                    "<p> The app has crashed. See the developer console for details. </p>",
                );
                panic!("Failed to start eframe: {e:?}");
            }
        }
    }
}

fn create_connection(egui_ctx: egui::Context) -> Box<dyn Connection> {
    let (sender, receiver) = mpsc::channel();

    on_message(move |event: MessageEvent| {
        let message = ToWebviewMessage::from_js(event.data()).expect("invalid webview document");

        match message {
            ToWebviewMessage::Document { document } => {
                sender
                    .send(document)
                    .expect("channel should never be closed");
                egui_ctx.request_repaint();
            }
        }
    });

    CodeConnection::new_boxed(receiver)
}

fn on_message(callback_fn: impl FnMut(MessageEvent) + 'static) {
    let cb = Closure::wrap(Box::new(callback_fn) as Box<dyn FnMut(_)>);

    web_sys::window()
        .expect("No window")
        .add_event_listener_with_callback("message", cb.as_ref().unchecked_ref())
        .expect("error adding event listener");

    cb.forget();
}

fn get_canvas() -> web_sys::HtmlCanvasElement {
    let document = web_sys::window()
        .expect("No window")
        .document()
        .expect("No document");

    document
        .get_element_by_id("app")
        .expect("Failed to find app_canvas")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("app_canvas was not a HtmlCanvasElement")
}
