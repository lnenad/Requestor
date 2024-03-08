use crate::app::request_method::RequestMethod;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryItem {
    pub id: String,
    pub url: String,
    pub method: RequestMethod,
    pub request_header_keys: Vec<String>,
    pub request_header_values: Vec<String>,
    pub query_param_keys: Vec<String>,
    pub query_param_values: Vec<String>,
    pub request_body: String,
}
