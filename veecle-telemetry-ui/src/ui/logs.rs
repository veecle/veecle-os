//! Components which display log data.
//!
//! See [log_ui].

use egui::{Align, Label, Layout, RichText, Sense, Widget};
use egui_extras::{Column, TableBuilder};

use crate::state::AppState;
use crate::store::Store;

/// Displays a table with all logs.
pub fn log_ui(ui: &mut egui::Ui, app_state: &AppState, store: &Store) {
    egui::Frame::new().inner_margin(4).show(ui, |ui| {
        ui.expand_to_include_rect(ui.max_rect());

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().at_least(70.0).resizable(true))
            .column(Column::auto().at_least(40.0).resizable(true))
            .column(Column::auto().resizable(true).clip(true))
            .column(Column::remainder().clip(true).resizable(true))
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .sense(Sense::click());

        table
            .header(text_height, |mut header| {
                header.col(|ui| {
                    ui.strong("Timestamp");
                });
                header.col(|ui| {
                    ui.strong("Level");
                });
                header.col(|ui| {
                    ui.strong("Actor");
                });
                header.col(|ui| {
                    ui.strong("Message");
                });
            })
            .body(|body| {
                let logs = app_state.filter().filter_logs(store).collect::<Vec<_>>();
                body.rows(text_height, logs.len(), |mut row| {
                    let log = logs
                        .get(row.index())
                        .expect("length should not be exceeded");

                    if app_state.selection().is_hovered(log.span_context.into()) {
                        row.set_hovered(true);
                    }

                    let is_selected = app_state.selection().is_selected(log.id.into());
                    if is_selected {
                        row.set_selected(true);
                    }

                    row.col(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            monospace_table_label_ui(ui, log.timestamp.as_ns().to_string());
                        });
                    });
                    row.col(|ui| {
                        Label::new(
                            RichText::new(log.metadata.level.as_str())
                                .color(log.metadata.level.color())
                                .monospace(),
                        )
                        .selectable(false)
                        .ui(ui);
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                        monospace_table_label_ui(ui, log.actor.as_str());
                    });
                    row.col(|ui| {
                        monospace_table_label_ui(ui, log.body.as_str());
                    });

                    if row.response().hovered() {
                        app_state.selection().set_hovered(log.id.into());
                    }

                    if row.response().clicked() {
                        if is_selected {
                            app_state.selection().clear_selected();
                        } else {
                            app_state.selection().set_selected(log.id.into());
                        }
                    }
                });
            });
    });
}

/// Create a monospace label that's not selectable so hover and click events go to the table row.
fn monospace_table_label_ui(ui: &mut egui::Ui, text: impl Into<RichText>) -> egui::Response {
    Label::new(text.into().monospace()).selectable(false).ui(ui)
}
