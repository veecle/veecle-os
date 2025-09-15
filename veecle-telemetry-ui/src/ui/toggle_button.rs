pub fn toggle_button_ui(
    ui: &mut egui::Ui,
    icon_off: &str,
    icon_on: &str,
    selected: bool,
) -> egui::Response {
    let text = egui::RichText::new(if selected { icon_on } else { icon_off }).size(16.0);

    let mut response = ui.add(egui::Button::new(text));
    if response.clicked() {
        response.mark_changed();
    }

    response
}
