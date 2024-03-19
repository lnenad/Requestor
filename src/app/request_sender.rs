use std::cell::RefCell;
use std::time::Instant;

use egui_toast::Toasts;
use poll_promise::Promise;
use url::Url;

use crate::history_item::history_item::HistoryItem;

use super::environment_injector::inject_environment;
use super::request_method::RequestMethod;
use super::resource::Resource;
use super::tab_state::TabState;

pub fn send_request(
    ui: &mut egui::Ui,
    state: &mut TabState,
    toasts: &mut Toasts,
    active_request: &mut Option<HistoryItem>,
    next_id: usize,
) {
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
        let (h_k, _err) = inject_environment(&state.request_header_keys[idx], &state.environment);
        let (h_v, _err) = inject_environment(&state.request_header_values[idx], &state.environment);
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
        //ctx.forget_image(&prev_url);
        ctx.request_repaint(); // wake up UI thread
        let resource = response.map(|response| Resource::from_response(&ctx, response, elapsed));
        sender.send(resource);
    });

    *active_request = Some(HistoryItem {
        id: next_id.to_string(),
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
