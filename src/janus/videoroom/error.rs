use serde_json::json;
use crate::janus::json::JSON_OBJECT;
use serde_json::error::Category;

pub static JANUS_VIDEOROOM_ERROR_UNKNOWN_ERROR     : u32 = 499;
pub static JANUS_VIDEOROOM_ERROR_NO_MESSAGE        : u32 = 421;
pub static JANUS_VIDEOROOM_ERROR_INVALID_JSON      : u32 = 422;
pub static JANUS_VIDEOROOM_ERROR_INVALID_REQUEST   : u32 = 423;
pub static JANUS_VIDEOROOM_ERROR_JOIN_FIRST        : u32 = 424;
pub static JANUS_VIDEOROOM_ERROR_ALREADY_JOINED    : u32 = 425;
pub static JANUS_VIDEOROOM_ERROR_NO_SUCH_ROOM      : u32 = 426;
pub static JANUS_VIDEOROOM_ERROR_ROOM_EXISTS       : u32 = 427;
pub static JANUS_VIDEOROOM_ERROR_NO_SUCH_FEED      : u32 = 428;
pub static JANUS_VIDEOROOM_ERROR_MISSING_ELEMENT   : u32 = 429;
pub static JANUS_VIDEOROOM_ERROR_INVALID_ELEMENT   : u32 = 430;
pub static JANUS_VIDEOROOM_ERROR_INVALID_SDP_TYPE  : u32 = 431;
pub static JANUS_VIDEOROOM_ERROR_PUBLISHERS_FULL   : u32 = 432;
pub static JANUS_VIDEOROOM_ERROR_UNAUTHORIZED      : u32 = 433;
pub static JANUS_VIDEOROOM_ERROR_ALREADY_PUBLISHED : u32 = 434;
pub static JANUS_VIDEOROOM_ERROR_NOT_PUBLISHED     : u32 = 435;
pub static JANUS_VIDEOROOM_ERROR_ID_EXISTS         : u32 = 436;
pub static JANUS_VIDEOROOM_ERROR_INVALID_SDP       : u32 = 437;

pub struct VideoroomError {
    error_code: u32,
    error: String
}

impl Into<JSON_OBJECT> for VideoroomError {
    fn into(self) -> JSON_OBJECT {
        json!({
            "videoroom": "event",
            "error_code": self.error_code,
            "error": self.error
        })
    }
}

impl VideoroomError {
    pub fn new(code: u32, reason: String) -> VideoroomError {
        VideoroomError {
            error_code: code,
            error: reason
        }
    }

    pub fn from_json_parse_error(e: serde_json::Error) -> Self {
        match e.classify() {
            Category::Syntax => VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_JSON, "Invalid json object".to_string()),
            Category::Io => VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_JSON, "Invalid json object".to_string()),
            Category::Data => VideoroomError::new(JANUS_VIDEOROOM_ERROR_MISSING_ELEMENT, format!("Validation error: {}", e)),
            Category::Eof => VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_JSON, "Invalid json object".to_string())
        }
    }
}
