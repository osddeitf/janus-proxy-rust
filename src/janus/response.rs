use serde::Serialize;
use serde_with::skip_serializing_none;
use super::json::*;
use super::request::IncomingRequestParameters;

#[skip_serializing_none]
#[derive(Serialize)]
pub struct JanusResponse<'a> {
    janus: JSON_STRING_SLICE<'a>,
    transaction: JSON_STRING_SLICE<'a>,

    /** session_id (websocket) */
    #[serde(default, skip_serializing_if = "is_zero")]
    session_id: JSON_POSITIVE_INTEGER,

    /** handle_id (websocket) */
    #[serde(default, skip_serializing_if = "is_zero")]
    sender: JSON_POSITIVE_INTEGER,

    /** create, attach request */
    data: Option<JSON_OBJECT>,
    /** plugin request */
    plugin_data: Option<JSON_OBJECT>,

    /** JSEP SDP */
    jsep: Option<JSON_OBJECT>
}

impl<'a> JanusResponse<'a> {
    pub fn new_response_with_data(name: &'a str, request: &'a IncomingRequestParameters, data: JSON_OBJECT) -> JanusResponse<'a> {
        JanusResponse {
            janus: name,
            transaction: &request.transaction,
            session_id: 0,
            sender: 0,
            data: Some(data),
            plugin_data: None,
            jsep: None
        }
    }
}
