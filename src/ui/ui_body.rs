pub fn ui_body(ui: &mut egui::Ui, body: &mut String) {
    ui.separator();
    egui::CollapsingHeader::new("Request body")
        .default_open(true)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(body)
                    .code_editor()
                    .desired_width(ui.available_width() - 15.0),
            )
        });
}
