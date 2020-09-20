use serde::Serialize;
use serde_with::skip_serializing_none;
use super::json::{self, *};
use super::error::JanusError;
use super::request::IncomingRequestParameters;

#[derive(Serialize)]
struct PluginResultWrapper<'a> {
    plugin: &'a str,
    data: JSON_OBJECT
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct JanusResponse<'a> {
    janus: JSON_STRING_SLICE<'a>,
    transaction: JSON_STRING_SLICE<'a>,
    error: Option<&'a JanusError>,

    /** session_id (websocket) */
    #[serde(default, skip_serializing_if = "is_zero")]
    session_id: JSON_POSITIVE_INTEGER,

    /** handle_id (websocket) */
    #[serde(default, skip_serializing_if = "is_zero")]
    sender: JSON_POSITIVE_INTEGER,

    /** create, attach request */
    data: Option<JSON_OBJECT>,
    /** plugin request */
    plugin_data: Option<PluginResultWrapper<'a>>,

    /** JSEP SDP */
    jsep: Option<JSON_OBJECT>
}

type JanusRequest = IncomingRequestParameters;

impl<'a> JanusResponse<'a> {
    pub fn new_with_data(name: &'a str, request: &'a JanusRequest, data: JSON_OBJECT) -> JanusResponse<'a> {
        let mut response = Self::new(name, request);
        response.data = Some(data);
        response
    }

    pub fn new(name: &'a str, request: &'a JanusRequest) -> JanusResponse<'a> {
        // TODO: is `session_id` and `sender` returned from request
        JanusResponse {
            janus: name,
            transaction: &request.transaction,
            error: None,
            session_id: request.session_id,
            sender: 0,
            data: None,
            plugin_data: None,
            jsep: None
        }
    }

    pub fn new_plugin_result(name: &'a str, request: &'a JanusRequest, plugin: &'a str, data: JSON_OBJECT) -> JanusResponse<'a> {
        let mut response = Self::new(name, request);
        response.sender = request.handle_id;
        response.plugin_data = Some(PluginResultWrapper { plugin, data });
        response
    }

    // TODO: whether `JanusRequest` and `JanusError` or not should have the same lifetime parameter
    pub fn new_with_error(request: &'a JanusRequest, error: &'a JanusError) -> JanusResponse<'a> {
        let mut response = Self::new("error", request);
        response.error = Some(error);
        response
    }

    pub fn bad_request(error: &JanusError) -> JanusResponse {
        JanusResponse {
            janus: "error",
            transaction: "",
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
