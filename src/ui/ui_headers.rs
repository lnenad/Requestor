pub fn ui_headers(
    ui: &mut egui::Ui,
    _frame: &mut eframe::Frame,
    request_header_keys: &mut Vec<String>,
    request_header_values: &mut Vec<String>,
) {
    ui.separator();
    egui::CollapsingHeader::new("Request headers")
        .default_open(true)
        .show(ui, |ui| {
            for idx in 0..request_header_keys.len() {
                if request_header_keys.len() == idx {
                    continue;
                }
                ui.horizontal(|ui| {
                    ui.label("Key:");
                    ui.add(egui::TextEdit::singleline(&mut request_header_keys[idx]));
                    ui.label("Value:");
                    ui.add(egui::TextEdit::singleline(&mut request_header_values[idx]));
                    if ui.button("Remove").clicked() {
                        request_header_keys.remove(idx);
                    }
                });
            }
            ui.horizontal(|ui| {
                if ui.button("Add header").clicked() {
                    request_header_keys.push("".to_owned());
                    request_header_values.push("".to_owned());
                }
            });
        });
}
