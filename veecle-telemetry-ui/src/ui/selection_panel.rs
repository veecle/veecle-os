use egui::RichText;

use crate::selection::{Item, SelectionState};
use crate::state::{AppState, PanelState};
use crate::store::{LogRef, Metadata, SpanRef, Store};
use crate::ui::panel::{collapsing_grid_ui, panel_content_ui};

pub fn selection_panel_ui(ui: &mut egui::Ui, app_state: &AppState, store: &Store) {
    let expanded = matches!(app_state.panel().selection_panel, PanelState::Expanded);

    let panel = egui::SidePanel::right("details_panel")
        .min_width(120.0)
        .frame(egui::Frame::default().fill(ui.style().visuals.panel_fill));

    panel.show_animated_inside(ui, expanded, |ui| {
        ui.add_space(8.0);

        panel_content_ui(ui, |ui| {
            ui.strong("Selection");
        });

        ui.separator();

        let Some(selected) = app_state.selection().get_selected() else {
            panel_content_ui(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.weak("Nothing selected");
                });
            });

            return;
        };

        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                selection_content_ui(ui, app_state.selection(), selected, store)
            });
    });
}

fn selection_content_ui(
    ui: &mut egui::Ui,
    selection_state: &SelectionState,
    selected: Item,
    store: &Store,
) {
    ui.scope(|ui| match selected {
        Item::Span(span_id) => {
            panel_content_ui(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("Span");
                    ui.monospace(format!("{span_id}"));
                });
            });

            let Some(span) = store.get_span(span_id) else {
                panel_content_ui(ui, |ui| {
                    ui.colored_label(egui::Color32::RED, "Span not found");
                });

                return;
            };

            panel_content_ui(ui, |ui| {
                span_details_ui(ui, selection_state, span);
            });
        }
        Item::Log(log_id) => {
            panel_content_ui(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("Log");
                    ui.monospace(format!("{log_id}"));
                });
            });

            let Some(log) = store.get_log(log_id) else {
                panel_content_ui(ui, |ui| {
                    ui.colored_label(egui::Color32::RED, "Log not found");
                });

                return;
            };

            panel_content_ui(ui, |ui| {
                log_details_ui(ui, selection_state, log);
            });
        }
    });
}

fn span_details_ui(ui: &mut egui::Ui, selection_state: &SelectionState, span: SpanRef) {
    metadata_details_ui(ui, &span.metadata);

    collapsing_grid_ui(ui, "Fields", |ui| {
        for (key, value) in span.fields.iter() {
            ui.monospace(key);
            ui.monospace(format!("{value}"));
            ui.end_row();
        }
    });

    collapsing_grid_ui(ui, "References", |ui| {
        ui.label("Parent");
        if let Some(parent) = span.parent {
            reference_link_ui(ui, selection_state, parent);
        } else {
            ui.weak("-");
        }
        ui.end_row();

        ui.label("Children");
        ui.horizontal(|ui| {
            ui.monospace("[");
            for item in span.children.iter() {
                reference_link_ui(ui, selection_state, *item);
            }
            ui.monospace("]");
        });
        ui.end_row();

        ui.label("Links");
        ui.horizontal(|ui| {
            ui.monospace("[");
            for item in span.links.iter() {
                reference_link_ui(ui, selection_state, *item);
            }
            ui.monospace("]");
        });
        ui.end_row();

        ui.label("Logs");
        ui.horizontal(|ui| {
            ui.monospace("[");
            for item in span.logs.iter() {
                reference_link_ui(ui, selection_state, *item);
            }
            ui.monospace("]");
        });
        ui.end_row();
    });

    collapsing_grid_ui(ui, "Timings", |ui| {
        let start = span.start.as_ms();
        let end = span.end.as_ms();
        let duration = span.duration_ms();

        let digits = start
            .floor()
            .log10()
            .max(end.floor().log10())
            .max(duration.floor().log10()) as usize;
        let width = digits + 5;

        ui.label("Start");
        ui.monospace(format!("{:width$.3} ms", span.start.as_ms(), width = width));
        ui.end_row();

        ui.label("End");
        ui.monospace(format!("{:width$.3} ms", span.end.as_ms(), width = width));
        ui.end_row();

        ui.label("Duration");
        ui.monospace(format!("{:width$.3} ms", span.duration_ms(), width = width));
        ui.end_row();
    });
}

fn log_details_ui(ui: &mut egui::Ui, selection_state: &SelectionState, log: LogRef) {
    metadata_details_ui(ui, &log.metadata);

    collapsing_grid_ui(ui, "Fields", |ui| {
        for (key, value) in log.fields.iter() {
            ui.monospace(key);
            ui.monospace(format!("{value}"));
            ui.end_row();
        }
    });

    collapsing_grid_ui(ui, "References", |ui| {
        ui.label("Span");
        reference_link_ui(ui, selection_state, log.span_context);
        ui.end_row();
    });

    collapsing_grid_ui(ui, "Timings", |ui| {
        ui.label("Timestamp");
        ui.monospace(format!("{:.3} ms", log.timestamp.as_ms()));
        ui.end_row();
    });
}

fn metadata_details_ui(ui: &mut egui::Ui, metadata: &Metadata) {
    collapsing_grid_ui(ui, "Metadata", |ui| {
        ui.label("Name");
        ui.monospace(&metadata.name);
        ui.end_row();

        ui.label("Target");
        ui.monospace(&metadata.target);
        ui.end_row();

        ui.label("Level");
        ui.label(
            RichText::new(metadata.level.as_str())
                .color(metadata.level.color())
                .monospace(),
        );
        ui.end_row();

        ui.label("File");
        optional_label_ui(ui, metadata.file.as_ref());
        ui.end_row();
    });
}

fn optional_label_ui(ui: &mut egui::Ui, value: Option<impl std::fmt::Display>) {
    if let Some(value) = value {
        ui.monospace(format!("{value}"));
    } else {
        ui.weak("-");
    }
}

fn reference_link_ui<T>(ui: &mut egui::Ui, selection_state: &SelectionState, item: T)
where
    T: Into<Item> + std::fmt::Display,
{
    let text = egui::RichText::new(format!("{item}")).monospace();
    let response = ui.link(text);

    let item = item.into();

    if response.hovered() {
        selection_state.set_hovered(item);
    }

    if response.clicked() {
        selection_state.set_selected(item);
    }
}
