use egui_toast::Toasts;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
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

use super::environment_injector::inject_environment;

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
    pub env_modal_opened: bool,
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
        let mut toasts = egui_toast::Toasts::new()
            .anchor(egui::Align2::CENTER_TOP, (10.0, 10.0))
            .direction(egui::Direction::TopDown);

        let state = self.open_requests.entry(tab.clone()).or_default();

        let menu_response = egui::menu::bar(ui, |ui| {
            ui.menu_button("Environment", |ui| {
                if ui.button("Load").clicked() {
                    let file = rfd::FileDialog::new()
                        .add_filter("text", &["txt"])
                        .add_filter("json", &["json"])
                        .pick_file();

                    match file {
                        Some(file_path) => {
                            println!("File: {:?}", file_path);
                            state.environment_path = file_path.clone();
                            load_environment(file_path, state);
                            toasts.add(egui_toast::Toast {
                                text: "Environment loaded".into(),
                                kind: egui_toast::ToastKind::Success,
                                options: egui_toast::ToastOptions::default()
                                    .duration_in_seconds(3.0)
                                    .show_progress(true)
                                    .show_icon(true),
                            });
                        }
                        None => (),
                    }
                    ui.close_menu();
                }
                if ui.button("Clear").clicked() {
                    state.environment = Default::default();
                    ui.close_menu();
                }
            });
        });

        environment_status(
            ui.ctx(),
            tab,
            &mut self.env_modal_opened,
            state.environment.len() > 0,
            menu_response.response.rect,
            state,
            &mut toasts,
        );

        if state.environment.len() > 0 {
            // Env values modal window
            let mut modal_title = "Environment variables for ".to_owned();
            modal_title.push_str(tab.as_str());
            egui::Window::new(modal_title)
                .open(&mut self.env_modal_opened)
                .show(ui.ctx(), |ui| {
                    egui::Grid::new("env_values")
                        .spacing(egui::vec2(ui.spacing().item_spacing.x * 4.0, 4.0))
                        .show(ui, |ui| {
                            for (k, v) in &state.environment {
                                ui.label(k);
                                ui.label(v.as_str().unwrap_or("Invalid value"));
                                ui.end_row();
                            }
                        });
                });
        }

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

        let prev_url = state.url.clone();

        ui.style_mut().text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(20.0, eframe::epaint::FontFamily::Proportional),
        );
        egui::CollapsingHeader::new("Request")
            .default_open(true)
            .show(ui, |ui| {
                let trigger_fetch = ui_url(ui, &mut state.url, &mut state.method);

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
                    let (url, error) = inject_environment(&state.url, &state.environment);
                    if error.is_some() {
                        toasts.add(egui_toast::Toast {
                            text: error.unwrap().into(),
                            kind: egui_toast::ToastKind::Error,
                            options: egui_toast::ToastOptions::default()
                                .duration_in_seconds(3.0)
                                .show_progress(true)
                                .show_icon(true),
                        });
                        return;
                    }

                    // Check if URL is valid
                    let violations = RefCell::new(Vec::new());
                    let parsed_url = Url::options()
                        .syntax_violation_callback(Some(&|v| violations.borrow_mut().push(v)))
                        .parse(&url);

                    match parsed_url {
                        Ok(result_url) => {
                            let mut hash_query = result_url.query_pairs();
                            let mut x = 0;
                            loop {
                                let val = match hash_query.next() {
                                    Some(v) => v,
                                    None => break,
                                };
                                let (injected_key, _err) =
                                    inject_environment(&val.0.to_string(), &state.environment);
                                if state.query_param_keys.len() == x {
                                    state.query_param_keys.insert(x, injected_key)
                                } else {
                                    state.query_param_keys[x] = injected_key;
                                }
                                let (injected_val, _err) =
                                    inject_environment(&val.0.to_string(), &state.environment);
                                if state.query_param_values.len() == x {
                                    state.query_param_values.insert(x, injected_val)
                                } else {
                                    state.query_param_values[x] = injected_val;
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

                            return;
                        }
                    }

                    let (sender, promise) = Promise::new();

                    let ctx = ui.ctx().clone();

                    let mut request = match state.method {
                        RequestMethod::GET => ehttp::Request::get(&url),
                        RequestMethod::POST => ehttp::Request::post(&url, Vec::new()),
                        RequestMethod::PUT => ehttp::Request::post(&url, Vec::new()),
                        RequestMethod::PATCH => ehttp::Request::post(&url, Vec::new()),
                        RequestMethod::DELETE => ehttp::Request::post(&url, Vec::new()),
                    };
                    for idx in 0..state.request_header_keys.len() {
                        if state.request_header_keys[idx].len() == 0 {
                            continue;
                        }
                        let (h_k, _err) =
                            inject_environment(&state.request_header_keys[idx], &state.environment);
                        let (h_v, _err) = inject_environment(
                            &state.request_header_values[idx],
                            &state.environment,
                        );
                        request.headers.insert(&h_k, &h_v);
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
                        url: url.clone(),
                        original_url: state.url.clone(),
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

fn environment_status(
    ctx: &egui::Context,
    tab: &String,
    env_modal_opened: &mut bool,
    loaded: bool,
    rect: egui::Rect,
    state: &mut TabState,
    toasts: &mut Toasts,
) {
    let mut name = "env_status".to_owned();
    let pos_sub = if loaded { 51.0 } else { 23.0 };
    name.push_str(tab.as_str());
    egui::Area::new(name)
        .current_pos(egui::Pos2 {
            x: rect.min.x + rect.width() - pos_sub,
            y: rect.min.y,
        })
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            if loaded {
                ui.horizontal(|ui| {
                    if ui
                        .button("🔁")
                        .on_hover_text("Reload environment values.")
                        .clicked()
                    {
                        load_environment(state.environment_path.clone(), state);
                        toasts.add(egui_toast::Toast {
                            text: "Environment loaded".into(),
                            kind: egui_toast::ToastKind::Success,
                            options: egui_toast::ToastOptions::default()
                                .duration_in_seconds(3.0)
                                .show_progress(true)
                                .show_icon(true),
                        });
                    }
                    if ui
                        .button("✅")
                        .on_hover_text("Environment loaded. Click to preview values")
                        .clicked()
                    {
                        *env_modal_opened = true;
                    }
                });
            } else {
                let tooltip = "Environment not loaded.";
                if ui.button("❎").on_hover_text(tooltip).clicked() {}
            }
        });
}

fn load_environment(file_path: PathBuf, state: &mut TabState) {
    match fs::read_to_string(file_path) {
        Ok(contents) => {
            let parsed: Value = serde_json::from_str(&contents).unwrap();
            let obj: Map<String, Value> = parsed.as_object().unwrap().clone();
            println!("Parsed: {:?} ", obj);
            state.environment = obj;
        }
        Err(error) => {
            println!("{:?}", error);
        }
    }
}
