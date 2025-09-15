use egui::RichText;
use egui_remixicon::icons;

use crate::command::SystemCommand;
use crate::state::{AppState, PanelState};
use crate::store::{Level, Store};
use crate::ui::panel::panel_content_ui;

pub fn filter_panel_ui(ui: &mut egui::Ui, app_state: &AppState, store: &Store) {
    let expanded = matches!(app_state.panel().filter_panel, PanelState::Expanded);

    let panel = egui::SidePanel::left("filter_panel")
        .min_width(120.0)
        .frame(egui::Frame::default().fill(ui.style().visuals.panel_fill));

    panel.show_animated_inside(ui, expanded, |ui| {
        ui.add_space(8.0);

        panel_content_ui(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Filter");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::MenuBar::new().ui(ui, |ui| {
                        if ui
                            .button(icons::FILTER_OFF_FILL)
                            .on_hover_text("Clear filter")
                            .clicked()
                        {
                            app_state.send_system(SystemCommand::ClearFilter);
                        }
                    });
                });
            });
        });

        ui.separator();

        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| filter_content_ui(ui, app_state, store));
    });
}

fn filter_content_ui(ui: &mut egui::Ui, app_state: &AppState, store: &Store) {
    panel_content_ui(ui, |ui| {
        ui.add_space(8.0);

        ui.label("Message");
        string_filter_ui(
            ui,
            app_state,
            app_state.filter().message.clone(),
            SystemCommand::SetMessageFilter,
        );

        ui.add_space(8.0);

        ui.scope(|ui| {
            ui.label("Severity");

            level_filter_checkbox_ui(ui, app_state, Level::Error);
            level_filter_checkbox_ui(ui, app_state, Level::Warn);
            level_filter_checkbox_ui(ui, app_state, Level::Info);
            level_filter_checkbox_ui(ui, app_state, Level::Debug);
            level_filter_checkbox_ui(ui, app_state, Level::Trace);
        });

        ui.add_space(8.0);

        ui.scope(|ui| {
            ui.label("Actor");

            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

            for actor in store.actors() {
                actor_filter_checkbox_ui(ui, app_state, actor);
            }
        });

        ui.add_space(8.0);

        ui.label("Target");
        string_filter_ui(
            ui,
            app_state,
            app_state.filter().target.clone(),
            SystemCommand::SetTargetFilter,
        );

        ui.add_space(8.0);

        ui.label("File");
        string_filter_ui(
            ui,
            app_state,
            app_state.filter().file.clone(),
            SystemCommand::SetFileFilter,
        );
    });
}

fn string_filter_ui(
    ui: &mut egui::Ui,
    app_state: &AppState,
    mut value: String,
    create_message: impl FnOnce(String) -> SystemCommand,
) {
    let response = egui::TextEdit::singleline(&mut value).show(ui).response;

    if response.changed() {
        app_state.send_system(create_message(value));
    }
}

fn level_filter_checkbox_ui(ui: &mut egui::Ui, app_state: &AppState, level: Level) {
    let mut checked = app_state.filter().level.contains(&level);

    if ui
        .checkbox(&mut checked, severity_label_text(level))
        .clicked()
    {
        let mut level_filter = app_state.filter().level.clone();

        if checked {
            level_filter.insert(level);
        } else {
            level_filter.remove(&level);
        }

        app_state.send_system(SystemCommand::SetLevelFilter(level_filter));
    }
}

fn actor_filter_checkbox_ui(ui: &mut egui::Ui, app_state: &AppState, actor: &str) {
    let mut checked = app_state.filter().actor.contains(actor);

    if ui
        .checkbox(&mut checked, RichText::new(actor).monospace())
        .clicked()
    {
        let mut actor_filter = app_state.filter().actor.clone();

        if checked {
            actor_filter.insert(actor.to_string());
        } else {
            actor_filter.remove(actor);
        }

        app_state.send_system(SystemCommand::SetActorFilter(actor_filter));
    }
}

fn severity_label_text(value: Level) -> RichText {
    RichText::new(value.as_str())
        .color(value.color())
        .monospace()
}
