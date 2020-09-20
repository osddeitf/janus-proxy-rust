#[allow(dead_code)]
pub mod code;

use serde_json::error::Category;
use serde::Serialize;

#[derive(Serialize)]
pub struct JanusError {
    pub code: u32,
    pub reason: String
}

impl JanusError {
    pub fn new(code: u32, reason: String) -> JanusError {
        JanusError { code, reason }
    }

    pub fn from_json_parse_error(e: serde_json::Error) -> Self {
        match e.classify() {
            Category::Syntax => JanusError::new(code::JANUS_ERROR_INVALID_JSON_OBJECT, "Invalid json object".to_string()),
            Category::Io => JanusError::new(code::JANUS_ERROR_INVALID_JSON_OBJECT, "Invalid json object".to_string()),
            Category::Data => JanusError::new(code::JANUS_ERROR_MISSING_MANDATORY_ELEMENT, format!("Validation error: {}", e)),
            Category::Eof => JanusError::new(code::JANUS_ERROR_INVALID_JSON_OBJECT, "Invalid json object".to_string())
        }
    }

    pub fn from_json_stringify_error(e: serde_json::Error) -> Self {
        JanusError::new(code::JANUS_ERROR_UNKNOWN, e.to_string())
    }
}
