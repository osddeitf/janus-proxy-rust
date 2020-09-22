use serde::Serialize;
use serde_with::skip_serializing_none;
use super::json::{self, *};
use super::error::JanusError;
use super::core::JanusHandle;

#[derive(Serialize)]
struct PluginResultWrapper {
    plugin: &'static str,
    data: JSON_OBJECT
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct JanusResponse {
    janus: &'static str,
    transaction: String,
    error: Option<JanusError>,

    /** session_id (websocket) */
    #[serde(default, skip_serializing_if = "is_zero")]
    session_id: JSON_POSITIVE_INTEGER,

    /** handle_id (websocket) */
    #[serde(default, skip_serializing_if = "is_zero")]
    sender: JSON_POSITIVE_INTEGER,

    /** create, attach request */
    data: Option<JSON_OBJECT>,

    /** plugin request */
    plugin_data: Option<PluginResultWrapper>,

    /** JSEP SDP */
    jsep: Option<JSON_OBJECT>
}

impl JanusResponse {
    pub fn data(mut self, data: JSON_OBJECT) -> JanusResponse {
        self.data = Some(data);
        self
    }

    pub fn err(mut self, error: JanusError) -> JanusResponse {
        self.error = Some(error);
        self
    }

    pub fn new(name: &'static str, session: u64, transaction: String) -> JanusResponse {
        // TODO: is `session_id` and `sender` returned from request
        JanusResponse {
            janus: name,
            transaction,
            error: None,
            session_id: session,
            sender: 0,
            data: None,
            plugin_data: None,
            jsep: None
        }
    }

    pub fn new_result(name: &'static str, transaction: String, handle: &JanusHandle, data: JSON_OBJECT) -> JanusResponse {
        let mut response = Self::new(name, handle.session_id, transaction);
        response.sender = handle.handle_id;
        response.plugin_data = Some(PluginResultWrapper { plugin: handle.plugin.get_name(), data });
        response
    }

    pub fn bad_request(error: JanusError) -> JanusResponse {
        JanusResponse {
            janus: "error",
            transaction: "".to_string(),
            error: Some(error),
            session_id: 0,
            sender: 0,
            data: None,
            plugin_data: None,
            jsep: None
        }
    }

    pub fn stringify(&self) -> Result<String, JanusError> {
        json::stringify(self)
    }
}
