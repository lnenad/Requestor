use crate::history_item::history_item::HistoryItem;
use crate::history_item::history_item_widget::history_item_widget;

pub fn ui_history(ui: &mut egui::Ui, items: &Vec<HistoryItem>) -> Option<HistoryItem> {
    let mut selected: Option<HistoryItem> = None;
    ui.set_width_range(100.0..=300.0);
    // For every item, show its name as a clickable label.
    egui::ScrollArea::both().show(ui, |ui| {
        for (id, item) in items.iter().enumerate() {
            let response = history_item_widget(ui, item.url.clone(), item.method.to_string());
            if response.clicked() {
                // Set this item to be the currently edited one
                selected = Some(HistoryItem {
                    id: id.to_string(),
                    url: item.url.clone(),
                    original_url: item.original_url.clone(),
                    method: item.method.clone(),
                    request_body: item.request_body.clone(),
                    request_header_keys: item.request_header_keys.clone(),
                    request_header_values: item.request_header_values.clone(),
                    query_param_keys: item.query_param_keys.clone(),
                    query_param_values: item.query_param_values.clone(),
                });
            };
            // Add some spacing to let it breathe
            ui.add_space(5.0);
        }
    });

    selected
}
