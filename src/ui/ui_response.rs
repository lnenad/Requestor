use crate::app::syntax_highlighting::{code_view_ui, get_type_from_mime, CodeTheme};

use crate::app::resource::Resource;

pub fn ui_response(
    ui: &mut egui::Ui,
    resource: &Resource,
    show_headers: &mut bool,
    show_body: &mut bool,
) {
    let Resource {
        response,
        timing: _,
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
        }

        ui.separator();
        if ui
            .add(egui::SelectableLabel::new(*show_body, "Body"))
            .clicked()
        {
            *show_body = true;
            *show_headers = false;
        }
        ui.separator();
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

    ui.add_space(5.0);
    let container = egui::ScrollArea::both()
        .auto_shrink(true)
        .max_width(ui.available_width())
        .id_source("1")
        .show(ui, |ui| {
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

                println!(
                    "Content-type: {:?} ",
                    get_type_from_mime(response.content_type().unwrap())
                );

                if let Some(image) = image {
                    ui.add(image.clone());
                } else if let Some(colored_text) = colored_text {
                    code_view_ui(
                        ui,
                        &CodeTheme::dark(),
                        text.as_ref().unwrap().as_str(),
                        get_type_from_mime(response.content_type().unwrap()),
                    );
                } else if let Some(text) = &text {
                    ui.add(egui::Label::new(text).selectable(true));
                }
            }
        });

    clipboard(
        ui.ctx(),
        "headers".to_owned(),
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
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            let tooltip = "Click to copy";
            if ui.button("📋").on_hover_text(tooltip).clicked() {
                ui.ctx().copy_text(text.clone());
            }
        });
}
