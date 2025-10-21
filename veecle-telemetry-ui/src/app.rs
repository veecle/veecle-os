//! TODO: document

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use egui::{Color32, Shadow};
use egui_notify::{Toast, ToastLevel, Toasts};
use egui_remixicon::icons;
use log::Level;
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use crate::command::{CommandReceiver, SystemCommand, UICommand, command_channel};
#[cfg(not(target_arch = "wasm32"))]
use crate::connection::file::FileConnection;
use crate::connection::file_contents::{FileContents, FileContentsConnection};
use crate::connection::{Connection, ConnectionMessage};
use crate::state::AppState;
use crate::store::Store;
use crate::ui::filter_panel::filter_panel_ui;
use crate::ui::logs::log_ui;
use crate::ui::selection_panel::selection_panel_ui;
use crate::ui::timeline::TraceTimelinePanel;
use crate::ui::toggle_button::toggle_button_ui;
use crate::ui::websocket_modal::WebSocketModal;

/// Data processing budget each frame.
const MAX_PROCESSING_DURATION: Duration = Duration::from_millis(1);

/// Options passed when starting the application.
#[derive(Debug, Default)]
pub struct StartupOptions {
    /// Hide the menu bar with the file menu.
    pub hide_menu: bool,
    /// Pass a connection to use right away.
    pub connection: Option<Box<dyn Connection>>,
}

/// The egui app for `veecle-telemetry-ui`.
#[allow(missing_debug_implementations)]
pub struct VeecleTelemetryApp {
    startup_options: StartupOptions,

    store: Store,
    state: AppState,

    trace_timeline_panel: TraceTimelinePanel,

    web_socket_modal: WebSocketModal,

    connection: Option<Box<dyn Connection>>,
    connection_error: bool,

    /// Listens to the local text log stream
    text_log_rx: std::sync::mpsc::Receiver<re_log::LogMsg>,
    toasts: Toasts,

    command_receiver: CommandReceiver,

    #[cfg(target_arch = "wasm32")]
    open_file_promise: Option<poll_promise::Promise<Option<FileContents>>>,
}

fn add_icon_font(fonts: &mut egui::FontDefinitions) {
    let font_data = std::sync::Arc::new(egui::FontData::from_static(egui_remixicon::FONT));
    fonts.font_data.insert("remixicon".into(), font_data);

    if let Some(font_keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        font_keys.push("remixicon".into());
    }
}

impl VeecleTelemetryApp {
    /// Creates a new `VeecleTelemetryApp`.
    ///
    /// Takes a receiver for tracing data and a path to open by default.
    pub fn new(cc: &eframe::CreationContext<'_>, mut startup_options: StartupOptions) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        add_icon_font(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

        let (logger, text_log_rx) = re_log::ChannelLogger::new(re_log::LevelFilter::Info);
        re_log::add_boxed_logger(Box::new(logger)).expect("Failed to add logger");

        let (command_sender, command_receiver) = command_channel();

        let connection = startup_options.connection.take();

        let app = VeecleTelemetryApp {
            startup_options,

            store: Default::default(),

            state: AppState::new(command_sender),

            connection: None,
            connection_error: false,

            trace_timeline_panel: Default::default(),
            web_socket_modal: Default::default(),

            text_log_rx,
            toasts: Toasts::default().with_shadow(Shadow {
                offset: Default::default(),
                blur: 30,
                spread: 5,
                color: Color32::from_black_alpha(70),
            }),

            command_receiver,

            #[cfg(target_arch = "wasm32")]
            open_file_promise: Default::default(),
        };

        if let Some(connection) = connection {
            app.state.send_system(SystemCommand::Connect(connection));
        }

        app
    }

    /// Show recent text log messages to the user as toast notifications.
    fn show_text_logs_as_notifications(&mut self) {
        while let Ok(message) = self.text_log_rx.try_recv() {
            let toast_level = match message.level {
                Level::Error => ToastLevel::Error,
                Level::Warn => ToastLevel::Warning,
                Level::Info => ToastLevel::Info,
                Level::Debug | Level::Trace => {
                    // These don't get exposed to the user.
                    continue;
                }
            };

            let mut toast = Toast::custom(message.msg, toast_level);
            if message.level == Level::Error {
                toast.duration(None);
            }

            self.toasts.add(toast);
        }
    }

    fn run_pending_system_commands(&mut self, egui_ctx: &egui::Context) {
        while let Some(cmd) = self.command_receiver.recv_system() {
            self.run_system_command(egui_ctx, cmd);
        }
    }

    fn run_pending_ui_commands(&mut self, egui_ctx: &egui::Context) {
        while let Some(cmd) = self.command_receiver.recv_ui() {
            self.run_ui_command(egui_ctx, cmd);
        }
    }

    fn run_system_command(&mut self, egui_ctx: &egui::Context, command: SystemCommand) {
        match command {
            SystemCommand::Connect(connection) => {
                self.connection_error = false;
                self.store.clear();
                self.store.continuous = connection.is_continuous();

                self.connection = Some(connection);
                egui_ctx.request_repaint();
            }

            SystemCommand::ClearFilter => {
                self.state.filter_mut().clear();
            }
            SystemCommand::SetLevelFilter(level_filter) => {
                self.state.filter_mut().level.set(level_filter);
            }
            SystemCommand::SetTargetFilter(target_filter) => {
                self.state.filter_mut().target.set(target_filter);
            }
            SystemCommand::SetFileFilter(file_filter) => {
                self.state.filter_mut().file.set(file_filter);
            }
            SystemCommand::SetActorFilter(actor_filter) => {
                self.state.filter_mut().actor.set(actor_filter);
            }
            SystemCommand::SetMessageFilter(message_filter) => {
                self.state.filter_mut().message.set(message_filter);
            }
        }
    }

    fn run_ui_command(&mut self, egui_ctx: &egui::Context, command: UICommand) {
        match command {
            #[cfg(not(target_arch = "wasm32"))]
            UICommand::Open => {
                if let Some(path) = open_file_dialog_native() {
                    match FileConnection::new_boxed(path.display().to_string()) {
                        Ok(connection) => {
                            self.state.send_system(SystemCommand::Connect(connection));
                        }
                        Err(error) => {
                            log::error!("Failed to open file {:?}: {:?}", &path, error);
                        }
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            UICommand::Open => {
                let egui_ctx = egui_ctx.clone();

                let promise = poll_promise::Promise::spawn_local(async move {
                    let file = open_file_dialog_web().await;
                    egui_ctx.request_repaint(); // Wake ui thread
                    file
                });

                self.open_file_promise = Some(promise);
            }

            UICommand::Connect => {
                self.web_socket_modal.show();
            }

            #[cfg(not(target_arch = "wasm32"))]
            UICommand::Quit => {
                egui_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            UICommand::ToggleFilterPanel => {
                self.state.toggle_filter_panel();
            }

            UICommand::ToggleSelectionPanel => {
                self.state.toggle_selection_panel();
            }
        }
    }

    fn ui(&mut self, egui_ctx: &egui::Context, _frame: &eframe::Frame) {
        self.web_socket_modal.ui(egui_ctx, &self.state);

        egui::CentralPanel::default()
            .frame(egui::Frame::new())
            .show(egui_ctx, |ui| {
                self.top_panel_ui(ui);

                self.trace_timeline_panel.show(ui, &self.store, &self.state);

                filter_panel_ui(ui, &self.state, &self.store);
                selection_panel_ui(ui, &self.state, &self.store);

                log_ui(ui, &self.state, &self.store);
            });

        self.show_text_logs_as_notifications();
        self.toasts.show(egui_ctx);

        Self::preview_files_being_dropped(egui_ctx);
    }

    fn top_panel_ui(&self, ui: &mut egui::Ui) {
        if self.startup_options.hide_menu {
            return;
        }

        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::new().inner_margin(4))
            .show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            self.state.send_ui(UICommand::Open);
                        }

                        if ui.button("Connect").clicked() {
                            self.state.send_ui(UICommand::Connect);
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            ui.separator();

                            if ui.button("Quit").clicked() {
                                self.state.send_ui(UICommand::Quit);
                            }
                        }
                    });

                    ui.add_space(8.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if toggle_button_ui(
                            ui,
                            icons::LAYOUT_RIGHT_2_LINE,
                            icons::LAYOUT_RIGHT_LINE,
                            self.state.panel().selection_panel.is_expanded(),
                        )
                        .on_hover_text("Toggle selection panel")
                        .clicked()
                        {
                            self.state.send_ui(UICommand::ToggleSelectionPanel);
                        };

                        if toggle_button_ui(
                            ui,
                            icons::LAYOUT_LEFT_2_LINE,
                            icons::LAYOUT_LEFT_LINE,
                            self.state.panel().filter_panel.is_expanded(),
                        )
                        .on_hover_text("Toggle filter panel")
                        .clicked()
                        {
                            self.state.send_ui(UICommand::ToggleFilterPanel);
                        };

                        ui.add_space(8.0);

                        if let Some(connection) = &self.connection {
                            ui.horizontal(|ui| {
                                ui.monospace(connection.to_string());
                            });
                        }
                    });
                });
            });
    }

    /// Preview files dragged onto the app.
    fn preview_files_being_dropped(ctx: &egui::Context) {
        use std::fmt::Write as _;

        use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};

        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                let mut text = "Read file: ".to_owned();
                let file = &i.raw.hovered_files[0];
                if let Some(path) = &file.path {
                    write!(text, "{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "{}", file.mime).ok();
                } else {
                    text += "???";
                }
                text
            });

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let content_rect = ctx.content_rect();
            painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }
    }

    fn handle_dropping_files(egui_ctx: &egui::Context, state: &AppState) {
        Self::preview_files_being_dropped(egui_ctx);

        let mut dropped_files = egui_ctx.input_mut(|i| std::mem::take(&mut i.raw.dropped_files));

        let Some(file) = dropped_files.pop() else {
            return;
        };

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = file.path {
            match FileConnection::new_boxed(path.display().to_string()) {
                Ok(connection) => {
                    state.send_system(SystemCommand::Connect(connection));
                }
                Err(error) => {
                    log::error!("Failed to open file {:?}: {:?}", &path, error);
                }
            }

            return;
        }

        if let Some(bytes) = file.bytes {
            state.send_system(SystemCommand::Connect(FileContentsConnection::new_boxed(
                FileContents {
                    name: file.name,
                    bytes,
                },
            )));
        }
    }

    fn receive_data(&mut self, egui_ctx: &egui::Context) {
        let Some(connection) = &mut self.connection else {
            return;
        };

        let start = Instant::now();

        while let Some(message) = connection.try_recv() {
            match message {
                ConnectionMessage::Line(line) => {
                    // Don't process lines after an error.
                    if !self.connection_error
                        && let Err(error) = self.store.process_line(&line)
                    {
                        self.connection_error = true;
                        log::error!("Failed to process line: {error:?}");
                    }
                }
                ConnectionMessage::Error(error) => {
                    log::error!("Connection error: {error:?}");

                    self.connection_error = true;
                    self.store.continuous = false;
                }
                ConnectionMessage::Done => {
                    self.store.continuous = false;
                }
                ConnectionMessage::Restart => {
                    self.connection_error = false;
                    self.store.clear();
                    self.store.continuous = connection.is_continuous();
                }
            }

            if start.elapsed() > MAX_PROCESSING_DURATION {
                egui_ctx.request_repaint();
                break;
            }
        }

        if connection.is_done() {
            self.store.continuous = false;
        }
    }
}

impl eframe::App for VeecleTelemetryApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.state.on_frame_start();

        #[cfg(target_arch = "wasm32")]
        if let Some(promise) = self.open_file_promise.take() {
            match promise.try_take() {
                Ok(Some(file)) => {
                    self.state.send_system(SystemCommand::Connect(
                        FileContentsConnection::new_boxed(file),
                    ));
                }
                Ok(None) => {}
                Err(promise) => {
                    // put promise back if it's not done.
                    self.open_file_promise = Some(promise);
                }
            };
        }

        self.run_pending_ui_commands(ctx);
        self.run_pending_system_commands(ctx);

        self.receive_data(ctx);

        self.ui(ctx, frame);

        Self::handle_dropping_files(ctx, &self.state);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        let color = egui::lerp(
            egui::Rgba::from(visuals.panel_fill)..=egui::Rgba::from(visuals.extreme_bg_color),
            0.5,
        );
        let color = egui::Color32::from(color);
        color.to_normalized_gamma_f32()
    }
}

/// [This may only be called on the main thread](https://docs.rs/rfd/latest/rfd/#macos-non-windowed-applications-async-and-threading).
#[cfg(not(target_arch = "wasm32"))]
fn open_file_dialog_native() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Supported files", &["jsonl"])
        .pick_file()
}

#[cfg(target_arch = "wasm32")]
async fn open_file_dialog_web() -> Option<FileContents> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Supported files", &["jsonl"])
        .pick_file()
        .await?;

    let file_name = file.file_name();
    log::debug!("Reading {file_name}…");

    let bytes = file.read().await;
    log::debug!("{file_name} was {} bytes", bytes.len());

    Some(FileContents {
        name: file_name,
        bytes: bytes.into(),
    })
}
