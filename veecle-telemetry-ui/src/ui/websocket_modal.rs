use egui::{Id, Key};

use crate::command::SystemCommand;
use crate::connection::websocket::WebSocketConnection;
use crate::state::AppState;

#[derive(Debug)]
pub struct WebSocketModal {
    visible: bool,

    url: String,
}

impl Default for WebSocketModal {
    fn default() -> Self {
        WebSocketModal {
            visible: false,

            url: "ws://127.0.0.1:9000".to_string(),
        }
    }
}

impl WebSocketModal {
    pub fn show(&mut self) {
        self.visible = true;
    }

    fn connect(&mut self, ctx: egui::Context, app_state: &AppState) {
        match WebSocketConnection::new_boxed(self.url.clone(), ctx) {
            Ok(connection) => {
                app_state.send_system(SystemCommand::Connect(connection));
            }
            Err(error) => {
                log::error!(
                    "Failed to create WebSocket connection to {:?}:\n{}",
                    &self.url,
                    error
                );
            }
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, app_state: &AppState) {
        if !self.visible {
            return;
        }

        egui::Modal::new(Id::new("web_socket_connect_modal")).show(ctx, |ui| {
            // Check _before_ we add the `TextEdit`, so it doesn't steal it.
            let enter_pressed = ui.input_mut(|i| i.consume_key(Default::default(), Key::Enter));

            ui.heading("Connect to WebSocket");

            ui.label("URL:");
            ui.text_edit_singleline(&mut self.url);

            ui.separator();

            egui::Sides::new().show(
                ui,
                |_| {},
                |ui| {
                    if ui.button("Connect").clicked() || enter_pressed {
                        self.connect(ctx.clone(), app_state);
                        self.visible = false;
                    }

                    if ui.button("Cancel").clicked() {
                        self.visible = false;
                    }
                },
            );
        });
    }
}
