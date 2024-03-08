use serde::{Deserialize, Serialize};
use std::fmt;

// Derive debut to conver to string and PartialEq to be comparable and allow the combobox to distinguish the selected option
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum RequestMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl fmt::Display for RequestMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
