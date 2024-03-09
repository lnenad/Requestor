pub mod request_method;
pub mod resource;

use poll_promise::Promise;
use std::cell::RefCell;
use std::time::Instant;

use crate::app::request_method::RequestMethod;
use crate::app::resource::Resource;
use crate::history_item::history_item::HistoryItem;
use crate::ui::{
    ui_body::ui_body, ui_headers::ui_headers, ui_history::ui_history,
    ui_query_params::ui_query_params, ui_response::ui_response, ui_url::ui_url,
};
use url::Url;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct HttpApp {
    url: String,
    method: RequestMethod,
    request_header_keys: Vec<String>,
    request_header_values: Vec<String>,
    query_param_keys: Vec<String>,
    query_param_values: Vec<String>,
    request_body: String,
    history_items: Vec<HistoryItem>,
    selected_item: Option<HistoryItem>,
    resource: Option<Resource>,
    #[cfg_attr(feature = "serde", serde(skip))]
    promise: Option<Promise<ehttp::Result<Resource>>>,
    show_headers: bool,
    show_body: bool,
}

impl Default for HttpApp {
    fn default() -> Self {
        Self {
            method: RequestMethod::GET,
            url: "".to_owned(),
            request_header_keys: vec!["".to_owned()],
            request_header_values: vec!["".to_owned()],
            query_param_keys: vec!["".to_owned()],
            query_param_values: vec!["".to_owned()],
            request_body: "".to_owned(),
            selected_item: None,
            history_items: vec![],
            resource: None,
            show_headers: true,
            show_body: false,
            promise: Default::default(),
        }
    }
}

impl HttpApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        let storage = cc.storage.unwrap();
        let history_string = storage.get_string("history").unwrap_or("[]".to_string());
        let history: Vec<HistoryItem> =
            serde_json::from_str::<Vec<HistoryItem>>(history_string.as_str()).unwrap();
        let mut default: Self = Default::default();
        // println!("Loading state: {:?}", history);
        match &history.last() {
            Some(item) => {
                default.url = item.url.clone();
                default.method = item.method
            }
            None => default.url = "https://httpbin.org/get".to_owned(),
        }
        default.history_items = history;

        default
    }
}

impl eframe::App for HttpApp {
    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(10)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // println!("Saving state {:?}", self.history_items);
        storage.set_string(
            "history",
            serde_json::to_value(&self.history_items)
                .unwrap()
                .to_string(),
        );
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
        let mut toasts = egui_toast::Toasts::new()
            .anchor(egui::Align2::LEFT_BOTTOM, (10.0, 10.0))
            .direction(egui::Direction::LeftToRight);

        egui::TopBottomPanel::bottom("http_bottom").show(ctx, |ui| {
            let layout = egui::Layout::right_to_left(egui::Align::Center);
            ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
                // ui.hyperlink_to("Author's website", "https://nenadspp.com");
                match &self.resource {
                    Some(resource) => {
                        ui.monospace(format!("url: {}", resource.response.url));
                        ui.monospace(format!(
                            "status: {} ({})",
                            resource.response.status, resource.response.status_text
                        ));
                        ui.monospace(format!(
                            "content-type: {}",
                            resource.response.content_type().unwrap_or_default()
                        ));
                        ui.monospace(format!(
                            "size: {:.1} kB",
                            resource.response.bytes.len() as f32 / 1000.0
                        ));
                        ui.monospace(format!("timing: {:.1}ms", resource.timing.as_millis()));
                    }
                    None => {
                        ui.monospace("Status line");
                    }
                }
            })
        });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.set_width_range(80.0..=400.0);
                ui.vertical(|ui| {
                    ui.set_width_range(80.0..=400.0);

                    let selected_item = ui_history(ui, &self.history_items);

                    match selected_item {
                        Some(item) => {
                            self.url = item.url.clone();
                            self.method = item.method.clone();
                            self.request_body = item.request_body.clone();
                            self.request_header_keys = item.request_header_keys.clone();
                            self.request_header_values = item.request_header_values.clone();
                            self.query_param_keys = item.query_param_keys.clone();
                            self.query_param_values = item.query_param_values.clone();
                            self.selected_item = Some(item);
                        }
                        None => (),
                    }

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        if ui
                            .add_sized(ui.available_size(), egui::Button::new("Clear History"))
                            .clicked()
                        {
                            self.history_items = vec![];
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let prev_url = self.url.clone();

            ui.style_mut().text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(18.0, eframe::epaint::FontFamily::Proportional),
            );
            ui.label("Request");
            ui.separator();
            let (url_input, trigger_fetch) = ui_url(ui, frame, &mut self.url, &mut self.method);
            if url_input.unwrap().changed() {
                let violations = RefCell::new(Vec::new());
                let parsed_url = Url::options()
                    .syntax_violation_callback(Some(&|v| violations.borrow_mut().push(v)))
                    .parse(&self.url);

                match parsed_url {
                    Ok(result_url) => {
                        let mut hash_query = result_url.query_pairs();
                        let mut x = 0;
                        loop {
                            let val = match hash_query.next() {
                                Some(v) => v,
                                None => break,
                            };
                            if self.query_param_keys.len() == x {
                                self.query_param_keys.insert(x, val.0.to_string())
                            } else {
                                self.query_param_keys[x] = val.0.to_string();
                            }
                            if self.query_param_values.len() == x {
                                self.query_param_values.insert(x, val.1.to_string())
                            } else {
                                self.query_param_values[x] = val.1.to_string();
                            }
                            x += 1;
                        }
                    }
                    Err(err) => {
                        let mut err_text: String = "Error parsing URL: ".to_string();
                        err_text.push_str(&err.to_string());
                        toasts.add(egui_toast::Toast {
                            text: err_text.into(),
                            kind: egui_toast::ToastKind::Error,
                            options: egui_toast::ToastOptions::default()
                                .duration_in_seconds(3.0)
                                .show_progress(true)
                                .show_icon(true),
                        });
                        for idx in 0..self.query_param_keys.len() {
                            if self.query_param_keys.len() == idx {
                                continue;
                            }
                            self.query_param_values[idx] = "Invalid URL".to_owned();
                            self.query_param_keys[idx] = "Invalid URL".to_owned();
                        }
                    }
                }
            }

            ui_query_params(
                ui,
                frame,
                &mut self.url,
                &mut self.query_param_keys,
                &mut self.query_param_values,
            );

            ui_headers(
                ui,
                frame,
                &mut self.request_header_keys,
                &mut self.request_header_values,
            );

            ui_body(ui, frame, &mut self.request_body);

            if trigger_fetch {
                let ctx = ctx.clone();
                let (sender, promise) = Promise::new();

                let mut request = match self.method {
                    RequestMethod::GET => ehttp::Request::get(&self.url),
                    RequestMethod::POST => ehttp::Request::post(&self.url, Vec::new()),
                    RequestMethod::PUT => ehttp::Request::post(&self.url, Vec::new()),
                    RequestMethod::PATCH => ehttp::Request::post(&self.url, Vec::new()),
                    RequestMethod::DELETE => ehttp::Request::post(&self.url, Vec::new()),
                };
                for idx in 0..self.request_header_keys.len() {
                    if self.request_header_keys[idx].len() == 0 {
                        continue;
                    }
                    request.headers.insert(
                        &self.request_header_keys[idx],
                        &self.request_header_values[idx],
                    );
                }

                if (request.method != RequestMethod::GET.to_string()
                    || request.method != RequestMethod::DELETE.to_string())
                    && self.request_body.len() > 0
                {
                    request.body = Vec::from(self.request_body.clone());
                }

                let start = Instant::now();
                ehttp::fetch(request, move |response| {
                    let elapsed = start.elapsed();
                    ctx.forget_image(&prev_url);
                    ctx.request_repaint(); // wake up UI thread
                    let resource =
                        response.map(|response| Resource::from_response(&ctx, response, elapsed));
                    sender.send(resource);
                });
                self.history_items.insert(
                    0,
                    HistoryItem {
                        id: (self.history_items.len()).to_string(),
                        url: self.url.clone(),
                        method: self.method.clone(),
                        request_body: self.request_body.clone(),
                        request_header_keys: self.request_header_keys.clone(),
                        request_header_values: self.request_header_values.clone(),
                        query_param_keys: self.query_param_keys.clone(),
                        query_param_values: self.query_param_values.clone(),
                    },
                );
                self.promise = Some(promise);
            }

            ui.separator();

            if let Some(promise) = &self.promise {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(resource) => {
                            ui.style_mut().text_styles.insert(
                                egui::TextStyle::Body,
                                egui::FontId::new(18.0, eframe::epaint::FontFamily::Proportional),
                            );
                            ui.add_space(20.0);
                            ui.label("Response");
                            ui.separator();
                            ui_response(ui, resource, &mut self.show_headers, &mut self.show_body);
                            self.resource = Some(resource.clone());
                        }
                        Err(error) => {
                            // This should only happen if the fetch API isn't available or something similar.
                            ui.colored_label(
                                ui.visuals().error_fg_color,
                                if error.is_empty() { "Error" } else { error },
                            );
                        }
                    }
                } else {
                    ui.spinner();
                }
            }
        });

        toasts.show(ctx);
    } // Update impl end
}
