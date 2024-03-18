use serde_json::{Map, Value};

pub fn inject_environment(
    str: &String,
    environment: &Map<String, Value>,
) -> (String, Option<String>) {
    let mut new_str = str.clone();
    let mut err: Option<String> = None;
    for (k, v) in environment {
        let val = v.as_str();
        match val {
            Some(value) => {
                new_str = new_str.replace(&format!("{{{}}}", k), value);
            }
            None => err = Some("Error parsing value from json".to_owned()),
        }
    }
    (new_str, err)
}
