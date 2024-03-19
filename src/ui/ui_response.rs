use crate::app::syntax_highlighting::{code_view_ui, get_type_from_mime, CodeTheme};

use crate::app::resource::Resource;

pub fn ui_response(
    ui: &mut egui::Ui,
    resource: &Resource,
    tab: &mut String,
    show_headers: &mut bool,
    show_body: &mut bool,
    show_info: &mut bool,
    wrap_text: &mut bool,
    stx_hgl: &mut bool,
) {
    let Resource {
        response,
        timing,
        raw_text,
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
    ui.horizontal(|ui| {
        if ui
            .add(egui::SelectableLabel::new(*show_headers, "Headers"))
            .clicked()
        {
            *show_headers = true;
            *show_body = false;
            *show_info = false;
        }

        ui.separator();
        if ui
            .add(egui::SelectableLabel::new(*show_body, "Body"))
            .clicked()
        {
            *show_body = true;
            *show_headers = false;
            *show_info = false;
        }
        ui.separator();
        if ui
            .add(egui::SelectableLabel::new(*show_info, "Info"))
            .clicked()
        {
            *show_body = false;
            *show_headers = false;
            *show_info = true;
        }
        ui.separator();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            ui.separator();
            if ui
                .add(egui::SelectableLabel::new(*wrap_text, "Wrap text"))
                .clicked()
            {
                *wrap_text = !*wrap_text;
            }
            ui.separator();
            if ui
                .add(egui::SelectableLabel::new(*stx_hgl, "Syntax highlighting"))
                .clicked()
            {
                *stx_hgl = !*stx_hgl;
            }
            ui.separator();
        });
    });

    ui.separator();

    let mut text_to_copy = "".to_owned();

    if *show_headers {
        // Can't use serde as headers aren't serializable to k:v
        text_to_copy = "{".to_owned();
        for idx in 0..response.headers.headers.len() {
            text_to_copy.push_str(format!("\"{}\":", &response.headers.headers[idx].0).as_str());
            text_to_copy.push_str(format!("\"{}\"", &response.headers.headers[idx].1).as_str());
            if idx != response.headers.headers.len() - 1 {
                text_to_copy.push_str(",");
            }
        }
        text_to_copy.push_str("}");
    }
    if *show_body {
        if let Some(text) = &text {
            text_to_copy = text.clone();
        }
        if let Some(raw_text) = &raw_text {
            text_to_copy = raw_text.clone();
        }
    }

    ui.add_space(5.0); // Top margin

    let container = egui::ScrollArea::both()
        .auto_shrink(false)
        .max_width(ui.available_width())
        .id_source("1")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                if *show_headers {
                    egui::Grid::new("response_headers")
                        .spacing(egui::vec2(ui.spacing().item_spacing.x * 4.0, 4.0))
                        .show(ui, |ui| {
                            for (k, v) in &response.headers {
                                ui.label(k);
                                ui.label(v);
                                ui.end_row();
                            }
                        });
                }

                if *show_body {
                    if response.content_type().is_none() {
                        if let Some(raw_text) = &raw_text {
                            ui.add(egui::Label::new(raw_text).selectable(true));
                        } else {
                            ui.monospace("[binary]");
                        }
                    }

                    if let Some(image) = image {
                        ui.add(image.clone());
                    } else if *stx_hgl {
                        if let Some(_colored_text) = colored_text {
                            code_view_ui(
                                ui,
                                &CodeTheme::dark(),
                                text.as_ref().unwrap().as_str(),
                                get_type_from_mime(response.content_type().unwrap()),
                                wrap_text,
                            );
                        } else {
                            ui.add(
                                egui::Label::new("Unable to perform syntax highligting")
                                    .selectable(true),
                            );
                        }
                    } else if let Some(text) = &text {
                        ui.add(egui::Label::new(text).wrap(*wrap_text).selectable(true));
                    }
                }

                if *show_info {
                    egui::Grid::new("response_headers")
                        .spacing(egui::vec2(ui.spacing().item_spacing.x * 4.0, 4.0))
                        .show(ui, |ui| {
                            ui.monospace(format!("url: {}", response.url));
                            ui.end_row();
                            ui.monospace(format!(
                                "status: {} ({})",
                                response.status, response.status_text
                            ));
                            ui.end_row();
                            ui.monospace(format!(
                                "content-type: {}",
                                response.content_type().unwrap_or_default()
                            ));
                            ui.end_row();
                            ui.monospace(format!(
                                "size: {:.1} kB",
                                response.bytes.len() as f32 / 1000.0
                            ));
                            ui.end_row();
                            ui.monospace(format!("timing: {:.1}ms", timing.as_millis()));
                            ui.end_row();
                        });
                }
            });
        });

    let mut ui_name = "response".to_owned();
    ui_name.push_str(tab.as_str());
    clipboard(
        ui.ctx(),
        ui_name,
        container.inner_rect,
        &text_to_copy.to_owned(),
    );
}

fn clipboard(ctx: &egui::Context, name: String, rect: egui::Rect, text: &String) {
    egui::Area::new(name)
        .current_pos(egui::Pos2 {
            x: rect.min.x + rect.width() - 22.0,
            y: rect.min.y,
        })
        .interactable(true)
        .show(ctx, |ui| {
            let tooltip = "Click to copy";
            if ui.button("📋").on_hover_text(tooltip).clicked() {
                ui.ctx().copy_text(text.clone());
            }
        });
}
