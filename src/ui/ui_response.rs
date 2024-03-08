use crate::app::resource::Resource;

pub fn ui_response(ui: &mut egui::Ui, resource: &Resource) {
    let Resource {
        response,
        timing: _,
        text,
        image,
        colored_text,
    } = resource;
    ui.style_mut().text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, eframe::epaint::FontFamily::Proportional),
    );
    ui.style_mut().text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, eframe::epaint::FontFamily::Proportional),
    );
    ui.with_layout(
        egui::Layout::left_to_right(egui::Align::Min).with_cross_justify(true),
        |ui| {
            egui::ScrollArea::horizontal()
                .auto_shrink(true)
                .max_width(ui.available_width() / 2.0)
                .id_source("1")
                .show(ui, |ui| {
                    egui::CollapsingHeader::new("Headers")
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::Grid::new("response_headers")
                                .spacing(egui::vec2(ui.spacing().item_spacing.x * 4.0, 4.0))
                                .show(ui, |ui| {
                                    for (k, v) in &response.headers {
                                        ui.label(k);
                                        ui.label(v);
                                        ui.end_row();
                                    }
                                });
                            ui.add_space(10.0);
                        });
                });

            let body_response = egui::ScrollArea::vertical()
                .auto_shrink(true)
                .id_source("2")
                .show(ui, |ui| {
                    ui.separator();
                    if let Some(image) = image {
                        Some(ui.add(image.clone()));
                    } else if let Some(colored_text) = colored_text {
                        Some(colored_text.ui(ui));
                    } else if let Some(text) = &text {
                        Some(ui.add(egui::Label::new(text).selectable(true)));
                    } else {
                        Some(ui.monospace("[binary]"));
                    }
                });

            if let Some(text) = &text {
                egui::Area::new("clipboard")
                    .current_pos(egui::Pos2 {
                        x: body_response.inner_rect.min.x + body_response.inner_rect.width() - 22.0,
                        y: body_response.inner_rect.min.y,
                    })
                    .order(egui::Order::Foreground)
                    .interactable(true)
                    .show(ui.ctx(), |ui| {
                        let tooltip = "Click to copy the response body";
                        if ui.button("📋").on_hover_text(tooltip).clicked() {
                            ui.ctx().copy_text(text.clone());
                        }
                    });
            }
        },
    );
}
