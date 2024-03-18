use egui::{FontId, Pos2};

// Custom widget used in the history list display. Used
// to display the request method and the url.
pub fn history_item_widget(ui: &mut egui::Ui, method: String, url: String) -> egui::Response {
    let text_height = 19.0;
    let tile_width = ui.available_size().x;
    let font_size = 12.0;
    let font_color = catppuccin_egui::MOCHA.text;
    ui.spacing_mut().item_spacing = [0.0, 0.0].into();

    // Prepare all of the layouts
    let g2 = ui.painter().layout(
        url,
        FontId::monospace(font_size),
        font_color,
        ui.available_size().x,
    );
    let g1 = ui.painter().layout(
        method,
        FontId::monospace(font_size),
        font_color,
        ui.available_size().x,
    );
    let lines = g1.rows.len() as f32;
    let response = ui.allocate_response(
        egui::Vec2 {
            x: tile_width,
            y: (20.0 + lines * text_height) - (lines * 2.0),
        }
        .into(),
        egui::Sense::click(),
    );

    let color = if response.hovered {
        catppuccin_egui::MOCHA.overlay1
    } else {
        catppuccin_egui::MOCHA.crust
    };

    // Paint everything
    ui.painter().rect_filled(response.rect, 0.0, color);

    ui.painter().galley(
        Pos2 {
            x: response.rect.min.x + 5.0,
            y: response.rect.min.y + 20.0,
        },
        g1,
        font_color,
    );
    ui.painter().galley(
        Pos2 {
            x: response.rect.min.x + 5.0,
            y: response.rect.min.y + 5.0,
        },
        g2,
        font_color,
    );

    response
}
