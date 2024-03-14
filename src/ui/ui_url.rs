use egui::Response;

use crate::app::request_method::RequestMethod;

pub fn ui_url(
    ui: &mut egui::Ui,
    url: &mut String,
    method: &mut RequestMethod,
) -> (Option<Response>, bool) {
    let mut trigger_fetch = false;

    ui.style_mut().text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, eframe::epaint::FontFamily::Proportional),
    );
    ui.style_mut().text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, eframe::epaint::FontFamily::Proportional),
    );

    let mut url_input = Option::None;

    ui.horizontal(|ui| {
        ui.label("Method: ");
        egui::ComboBox::from_id_source(1)
            .selected_text(method.to_string())
            .show_ui(ui, |ui| {
                for option in [
                    RequestMethod::GET,
                    RequestMethod::POST,
                    RequestMethod::PUT,
                    RequestMethod::PATCH,
                    RequestMethod::DELETE,
                ] {
                    ui.selectable_value(method, option, option.to_string());
                }
            });
        ui.label("URL: ");
        url_input = Some(
            ui.add(egui::TextEdit::singleline(url).desired_width(ui.available_width() - 60.0)),
        );
        if ui.button("Send").clicked() {
            trigger_fetch = true;
        }
    });

    (url_input, trigger_fetch)
}
