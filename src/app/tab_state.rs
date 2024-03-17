use crate::app::request_method::RequestMethod;
use crate::app::resource::Resource;

use poll_promise::Promise;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TabState {
    pub url: String,
    pub method: RequestMethod,
    pub request_header_keys: Vec<String>,
    pub request_header_values: Vec<String>,
    pub query_param_keys: Vec<String>,
    pub query_param_values: Vec<String>,
    pub request_body: String,
    pub show_headers: bool,
    pub show_body: bool,
    pub show_info: bool,
    #[serde(skip)]
    pub resource: Option<Resource>,
    #[serde(skip)]
    pub promise: Option<Promise<ehttp::Result<Resource>>>,
}

impl Clone for TabState {
    fn clone(&self) -> Self {
        Self {
            method: self.method,
            url: self.url.clone(),
            request_header_keys: self.request_header_keys.clone(),
            request_header_values: self.request_header_values.clone(),
            query_param_keys: self.query_param_keys.clone(),
            query_param_values: self.query_param_values.clone(),
            request_body: self.request_body.clone(),
            resource: self.resource.clone(),
            show_headers: self.show_headers,
            show_body: self.show_body,
            show_info: self.show_info,
            promise: Default::default(),
        }
    }
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
