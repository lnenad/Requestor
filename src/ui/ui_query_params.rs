pub fn ui_query_params(
    ui: &mut egui::Ui,
    url: &mut String,
    query_param_keys: &mut Vec<String>,
    query_param_values: &mut Vec<String>,
) {
    ui.separator();
    egui::CollapsingHeader::new("Query Parameters")
        .default_open(true)
        .show(ui, |ui| {
            for idx in 0..query_param_keys.len() {
                if query_param_keys.len() == idx {
                    continue;
                }
                ui.horizontal(|ui| {
                    ui.label("Key:");
                    ui.add_enabled(
                        false,
                        egui::TextEdit::singleline(&mut query_param_keys[idx]),
                    );
                    ui.label("Value:");
                    ui.add_enabled(
                        false,
                        egui::TextEdit::singleline(&mut query_param_values[idx]),
                    );

                    // if ui.button("Remove").clicked() {
                    //     query_param_keys.remove(idx);
                    // }
                });
            }
            ui.horizontal(|ui| {
                if ui.button("Add query parameter").clicked() {
                    if query_param_keys.len() >= 1 && query_param_keys[0].len() > 0 {
                        url.push_str("&");
                        query_param_keys.push("key".to_owned());
                        query_param_values.push("value".to_owned());
                    } else {
                        url.push_str("?");
                        query_param_keys[0] = "key".to_owned();
                        query_param_values[0] = "value".to_owned();
                    }
                    url.push_str("key=value");
                }
            });
        });
}
