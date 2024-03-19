pub mod environment_injector;
pub mod request_method;
pub mod request_sender;
pub mod resource;
pub mod syntax_highlighting;
pub mod tab_state;
pub mod tab_viewer;

use std::collections::BTreeMap;

use crate::app::tab_state::TabState;
use crate::app::tab_viewer::{Tab, TabViewer};

use crate::ui::ui_history::ui_history;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    active_tab: String,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct HttpApp {
    open_requests: TabViewer,
    tree: DockState<Tab>,
    loaded_initial: bool,
}

impl Default for HttpApp {
    fn default() -> Self {
        let mut tab_states = BTreeMap::default();
        tab_states.insert("Test".to_owned(), TabState::default());
        let history_items = vec![];
        Self {
            loaded_initial: false,
            open_requests: TabViewer {
                counter: 1 as usize,
                open_requests: tab_states,
                added_nodes: vec![],
                history_items,
                active_tab: None,
                active_request: None,
                tab_name_modal: None,
                new_tab_name: "".to_owned(),
                new_tab_name_temp: "".to_owned(),
                tab_name_to_change: "".to_owned(),
                env_modal_opened: false,
            },
            tree: DockState::new(vec!["Test".to_owned()]),
        }
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
            let tree: DockState<Tab> =
                serde_json::from_str::<DockState<Tab>>(tree_str.unwrap().as_str()).unwrap();

            default.open_requests.counter = tree.iter_all_tabs().count();
            default.tree = tree
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
        storage.set_string(
            "tree",
            serde_json::to_value(&self.tree).unwrap().to_string(),
        );
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);

        self.open_requests.tab_name_modal = Some(self.open_requests.prompt_modal(ctx));

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
                                    state.url = item.original_url.clone();
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
                let new_tab = format!("Tab {}", self.open_requests.counter.to_string());
                self.open_requests
                    .open_requests
                    .insert(new_tab.clone(), TabState::default());
                self.tree.push_to_focused_leaf(new_tab.clone());
                self.open_requests.active_tab = Some(new_tab);
                self.open_requests.counter += 1;
            });

        if !self.loaded_initial {
            let active_tab = self.open_requests.active_tab.as_ref();
            match active_tab {
                Some(active_tab_name) => {
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
