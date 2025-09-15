pub fn panel_content_ui<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 0))
        .show(ui, add_contents)
        .inner
}

pub fn collapsing_grid_ui(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let text = text.into();

    egui::CollapsingHeader::new(&text)
        .default_open(true)
        .show(ui, |ui| grid_ui(ui, text, add_contents));
}

pub fn grid_ui(
    ui: &mut egui::Ui,
    id_salt: impl std::hash::Hash,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    egui::Grid::new(id_salt)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, add_contents);
}
