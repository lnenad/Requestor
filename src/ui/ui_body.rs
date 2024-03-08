pub fn ui_body(ui: &mut egui::Ui, _frame: &mut eframe::Frame, body: &mut String) {
    //println!("{:?}", &request_header_keys);
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
