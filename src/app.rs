pub mod request_method;
pub mod resource;
pub mod syntax_highlighting;

use egui::emath::Numeric;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Instant;

use crate::app::request_method::RequestMethod;
use crate::app::resource::Resource;
use crate::history_item::history_item::HistoryItem;
use crate::ui::{
    ui_body::ui_body, ui_headers::ui_headers, ui_history::ui_history,
    ui_query_params::ui_query_params, ui_response::ui_response, ui_url::ui_url,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabIndex};
use url::Url;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    active_tab: String,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct HttpApp {
    open_requests: TabViewer,
    tree: DockState<String>,
    counter: usize,
    loaded_initial: bool,
}

impl Default for HttpApp {
    fn default() -> Self {
        let mut tab_states = BTreeMap::default();
        tab_states.insert("Test".to_owned(), TabState::default());
        let history_items = vec![];
        Self {
            counter: 1 as usize,
            loaded_initial: false,
            open_requests: TabViewer {
                open_requests: tab_states,
                added_nodes: vec![],
                history_items,
                active_tab: None,
                active_request: None,
            },
            tree: DockState::new(vec!["Test".to_owned()]),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TabViewer {
    open_requests: BTreeMap<String, TabState>,
    added_nodes: Vec<(usize, usize)>,
    history_items: Vec<HistoryItem>,
    active_tab: Option<String>,
    active_request: Option<HistoryItem>,
}

#[derive(Serialize, Deserialize)]
struct TabState {
    url: String,
    method: RequestMethod,
    request_header_keys: Vec<String>,
    request_header_values: Vec<String>,
    query_param_keys: Vec<String>,
    query_param_values: Vec<String>,
    request_body: String,
    show_headers: bool,
    show_body: bool,
    show_info: bool,
    #[serde(skip)]
    resource: Option<Resource>,
    #[serde(skip)]
    promise: Option<Promise<ehttp::Result<Resource>>>,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            method: RequestMethod::GET,
            url: "".to_owned(),
            request_header_keys: vec!["".to_owned()],
            request_header_values: vec!["".to_owned()],
            query_param_keys: vec!["".to_owned()],
            query_param_values: vec!["".to_owned()],
            request_body: "".to_owned(),
            resource: None,
            show_headers: true,
            show_body: false,
            show_info: false,
            promise: Default::default(),
        }
    }
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;

    fn title(&mut self, title: &mut String) -> egui::WidgetText {
        egui::WidgetText::from(&*title)
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        self.open_requests
            .insert("test".to_owned(), TabState::default());
        self.added_nodes.push((surface.0, node.0));
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            self.active_tab = Some(tab.clone());
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let state = self.open_requests.entry(tab.clone()).or_default();

        let mut toasts = egui_toast::Toasts::new()
            .anchor(egui::Align2::LEFT_BOTTOM, (10.0, 10.0))
            .direction(egui::Direction::BottomUp);

        let prev_url = state.url.clone();

        ui.style_mut().text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(18.0, eframe::epaint::FontFamily::Proportional),
        );
        ui.label("Request");
        ui.separator();
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
                let resource =
                    response.map(|response| Resource::from_response(&ctx, response, elapsed));
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

        ui.separator();

        if let Some(promise) = &state.promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(resource) => {
                        self.history_items
                            .insert(0, self.active_request.as_ref().unwrap().clone());
                        self.active_request = None;

                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::new(18.0, eframe::epaint::FontFamily::Proportional),
                        );
                        ui.add_space(20.0);
                        ui.label("Response");
                        ui.separator();
                        ui_response(
                            ui,
                            resource,
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
        }

        toasts.show(ui.ctx());
    }
}

impl HttpApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        let storage = cc.storage.unwrap();
        let mut default: Self = Default::default();

        // let config_string = storage
        //     .get_string("config")
        //     .unwrap_or(r#"{"active_tab": "tab2"}"#.to_string());
        // let config: Config = serde_json::from_str::<Config>(config_string.as_str()).unwrap();
        let open_requests_str = storage.get_string("open_requests");
        if open_requests_str.is_some() {
            let open_requests: TabViewer =
                serde_json::from_str::<TabViewer>(open_requests_str.unwrap().as_str()).unwrap();
            default.open_requests = open_requests;
        }
        let tree_str = storage.get_string("tree");

        if tree_str.is_some() {
            let tree: Vec<String> =
                serde_json::from_str::<Vec<String>>(tree_str.unwrap().as_str()).unwrap();

            default.counter = tree.len();
            default.tree = DockState::new(tree);
        }
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
            "open_requests",
            serde_json::to_value(&self.open_requests)
                .unwrap()
                .to_string(),
        );
        // println!("Saving state {:?}", self.history_items);
        let mut tabs: Vec<String> = vec![];
        for (_, tab) in self.tree.iter_all_tabs() {
            tabs.push(tab.to_string());
        }
        storage.set_string("tree", serde_json::to_value(tabs).unwrap().to_string());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.set_width_range(80.0..=400.0);
                ui.vertical(|ui| {
                    ui.set_width_range(80.0..=400.0);

                    let selected_item = ui_history(ui, &self.open_requests.history_items);

                    match selected_item {
                        Some(item) => {
                            let active_tab = self.tree.find_active_focused();
                            if active_tab.is_some() {
                                let state_opt = self
                                    .open_requests
                                    .open_requests
                                    .get_mut(active_tab.unwrap().1);

                                if state_opt.is_some() {
                                    let state = state_opt.unwrap();
                                    println!("STate: {:?}", state.url);
                                    state.url = item.url.clone();
                                    state.method = item.method.clone();
                                    state.request_body = item.request_body.clone();
                                    state.request_header_keys = item.request_header_keys.clone();
                                    state.request_header_values =
                                        item.request_header_values.clone();
                                    state.query_param_keys = item.query_param_keys.clone();
                                    state.query_param_values = item.query_param_values.clone();
                                }
                            }
                        }
                        None => (),
                    }

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.style_mut().spacing.button_padding = (40.0, 3.0).into();
                        if ui.add(egui::Button::new("Clear History")).clicked() {
                            self.open_requests.history_items.clear();
                        }
                    });
                });
            });

        DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.open_requests);

        self.open_requests
            .added_nodes
            .drain(..)
            .for_each(|(surface, node)| {
                self.tree.set_focused_node_and_surface((
                    SurfaceIndex::from(surface),
                    NodeIndex::from(node),
                ));
                let new_tab = format!("Tab {}", self.counter.to_string());
                self.tree.push_to_focused_leaf(new_tab.clone());
                self.open_requests.active_tab = Some(new_tab);
                self.counter += 1;
            });

        if !self.loaded_initial {
            let active_tab = self.open_requests.active_tab.as_ref();
            match active_tab {
                Some(active_tab_name) => {
                    println!("Active tab: {:?}", active_tab_name);

                    let tab = self.tree.find_tab(&active_tab_name);
                    if tab.is_some() {
                        let (surface_idx, node_idx, tab_idx) = tab.unwrap();
                        self.tree.set_active_tab((surface_idx, node_idx, tab_idx));
                    }
                }
                None => {
                    self.tree
                        .set_focused_node_and_surface((SurfaceIndex::from(0), NodeIndex::from(0)));
                }
            }

            self.loaded_initial = true;
        }
    } // Update impl end
}
