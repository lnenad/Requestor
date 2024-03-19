use egui_toast::Toasts;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::app::tab_state::TabState;
use crate::history_item::history_item::HistoryItem;
use crate::ui::{
    ui_body::ui_body, ui_headers::ui_headers, ui_query_params::ui_query_params,
    ui_response::ui_response, ui_url::ui_url,
};
use egui_dock::{NodeIndex, SurfaceIndex};
use egui_modal::Modal;

use super::request_sender::send_request;

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

        environment_status_icons(
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

        let _prev_url = state.url.clone();

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
                    send_request(
                        ui,
                        state,
                        &mut toasts,
                        &mut self.active_request,
                        self.history_items.len(),
                    );
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
                                    &mut state.wrap_text,
                                    &mut state.stx_hgl,
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

fn environment_status_icons(
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
