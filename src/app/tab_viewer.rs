use poll_promise::Promise;
use serde::{Deserialize, Serialize};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Instant;

use crate::app::request_method::RequestMethod;
use crate::app::resource::Resource;
use crate::app::tab_state::TabState;
use crate::history_item::history_item::HistoryItem;
use crate::ui::{
    ui_body::ui_body, ui_headers::ui_headers, ui_query_params::ui_query_params,
    ui_response::ui_response, ui_url::ui_url,
};
use egui_dock::{NodeIndex, SurfaceIndex};
use egui_modal::Modal;
use url::Url;

pub type Tab = String;

#[derive(Serialize, Deserialize)]
pub struct TabViewer {
    pub open_requests: BTreeMap<String, TabState>,
    pub added_nodes: Vec<(usize, usize)>,
    pub history_items: Vec<HistoryItem>,
    pub active_tab: Option<String>,
    pub counter: usize,
    pub active_request: Option<HistoryItem>,
    #[serde(skip)]
    pub tab_name_modal: Option<Modal>,
    pub new_tab_name: String,
    pub new_tab_name_temp: String,
    pub tab_name_to_change: String,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = Tab;

    fn title(&mut self, title: &mut String) -> egui::WidgetText {
        egui::WidgetText::from(&*title)
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        self.added_nodes.push((surface.0, node.0));
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        if self.counter == 1 {
            return false;
        }
        self.counter -= 1;
        return true;
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            self.active_tab = Some(tab.clone());
        }

        if response.double_clicked() {
            self.tab_name_to_change = tab.to_string();
            self.tab_name_modal.as_ref().unwrap().open();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let state = self.open_requests.entry(tab.clone()).or_default();
        if self.new_tab_name != ""
            && self.tab_name_to_change != ""
            && self.tab_name_to_change == tab.clone()
        {
            let state_clone = state.clone();
            self.open_requests
                .insert(self.new_tab_name.clone(), state_clone);
            self.open_requests.remove(&tab.clone());
            tab.clear();
            tab.insert_str(0, &self.new_tab_name);
            self.new_tab_name = "".to_owned();
            self.tab_name_to_change = "".to_owned();
            return;
        }

        let mut toasts = egui_toast::Toasts::new()
            .anchor(egui::Align2::LEFT_BOTTOM, (10.0, 10.0))
            .direction(egui::Direction::BottomUp);

        let prev_url = state.url.clone();

        ui.style_mut().text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(20.0, eframe::epaint::FontFamily::Proportional),
        );
        egui::CollapsingHeader::new("Request")
            .default_open(true)
            .show(ui, |ui| {
                let (url_input, trigger_fetch) = ui_url(ui, &mut state.url, &mut state.method);
                if url_input.unwrap().changed() {
                    let violations = RefCell::new(Vec::new());
                    let parsed_url = Url::options()
                        .syntax_violation_callback(Some(&|v| violations.borrow_mut().push(v)))
                        .parse(&state.url);

                    match parsed_url {
                        Ok(result_url) => {
                            let mut hash_query = result_url.query_pairs();
                            let mut x = 0;
                            loop {
                                let val = match hash_query.next() {
                                    Some(v) => v,
                                    None => break,
                                };
                                if state.query_param_keys.len() == x {
                                    state.query_param_keys.insert(x, val.0.to_string())
                                } else {
                                    state.query_param_keys[x] = val.0.to_string();
                                }
                                if state.query_param_values.len() == x {
                                    state.query_param_values.insert(x, val.1.to_string())
                                } else {
                                    state.query_param_values[x] = val.1.to_string();
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
                            for idx in 0..state.query_param_keys.len() {
                                if state.query_param_keys.len() == idx {
                                    continue;
                                }
                                state.query_param_values[idx] = "Invalid URL".to_owned();
                                state.query_param_keys[idx] = "Invalid URL".to_owned();
                            }
                        }
                    }
                }

                ui_query_params(
                    ui,
                    &mut state.url,
                    &mut state.query_param_keys,
                    &mut state.query_param_values,
                );

                ui_headers(
                    ui,
                    &mut state.request_header_keys,
                    &mut state.request_header_values,
                );

                ui_body(ui, &mut state.request_body);

                if trigger_fetch {
                    let (sender, promise) = Promise::new();

                    let ctx = ui.ctx().clone();

                    let mut request = match state.method {
                        RequestMethod::GET => ehttp::Request::get(&state.url),
                        RequestMethod::POST => ehttp::Request::post(&state.url, Vec::new()),
                        RequestMethod::PUT => ehttp::Request::post(&state.url, Vec::new()),
                        RequestMethod::PATCH => ehttp::Request::post(&state.url, Vec::new()),
                        RequestMethod::DELETE => ehttp::Request::post(&state.url, Vec::new()),
                    };
                    for idx in 0..state.request_header_keys.len() {
                        if state.request_header_keys[idx].len() == 0 {
                            continue;
                        }
                        request.headers.insert(
                            &state.request_header_keys[idx],
                            &state.request_header_values[idx],
                        );
                    }

                    if (request.method != RequestMethod::GET.to_string()
                        || request.method != RequestMethod::DELETE.to_string())
                        && state.request_body.len() > 0
                    {
                        request.body = Vec::from(state.request_body.clone());
                    }

                    let start = Instant::now();
                    ehttp::fetch(request, move |response| {
                        let elapsed = start.elapsed();
                        ctx.forget_image(&prev_url);
                        ctx.request_repaint(); // wake up UI thread
                        let resource = response
                            .map(|response| Resource::from_response(&ctx, response, elapsed));
                        sender.send(resource);
                    });

                    self.active_request = Some(HistoryItem {
                        id: (self.history_items.len()).to_string(),
                        url: state.url.clone(),
                        method: state.method.clone(),
                        request_body: state.request_body.clone(),
                        request_header_keys: state.request_header_keys.clone(),
                        request_header_values: state.request_header_values.clone(),
                        query_param_keys: state.query_param_keys.clone(),
                        query_param_values: state.query_param_values.clone(),
                    });

                    state.promise = Some(promise);
                }
            });

        ui.separator();

        if let Some(promise) = &mut state.promise {
            egui::CollapsingHeader::new("Response")
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok(resource) => {
                                if self.active_request.is_some() {
                                    self.history_items
                                        .insert(0, self.active_request.as_ref().unwrap().clone());
                                    self.active_request = None;
                                }

                                ui.style_mut().text_styles.insert(
                                    egui::TextStyle::Body,
                                    egui::FontId::new(
                                        18.0,
                                        eframe::epaint::FontFamily::Proportional,
                                    ),
                                );
                                ui_response(
                                    ui,
                                    resource,
                                    tab,
                                    &mut state.show_headers,
                                    &mut state.show_body,
                                    &mut state.show_info,
                                );
                                state.resource = Some(resource.clone());
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
                });
        }

        toasts.show(ui.ctx());
    }
}

impl TabViewer {
    pub fn prompt_modal(self: &mut TabViewer, ctx: &egui::Context) -> Modal {
        let modal = Modal::new(ctx, "my_modal");

        // What goes inside the modal
        modal.show(|ui| {
            // these helper functions help set the ui based on the modal's
            // set style, but they are not required and you can put whatever
            // ui you want inside [`.show()`]
            modal.title(ui, "Tab name");
            modal.frame(ui, |ui| {
                ui.label("New name: ");
                ui.add(egui::TextEdit::singleline(&mut self.new_tab_name_temp));
            });
            modal.buttons(ui, |ui| {
                // After clicking, the modal is automatically closed
                if modal.button(ui, "Close").clicked() {
                    self.new_tab_name_temp = "".to_owned();
                };
                // After clicking, the modal is automatically closed
                if modal.button(ui, "Ok").clicked() && self.new_tab_name_temp != "" {
                    self.new_tab_name = self.new_tab_name_temp.clone();
                    self.new_tab_name_temp = "".to_owned();
                };
            });
        });

        modal
    }
}
