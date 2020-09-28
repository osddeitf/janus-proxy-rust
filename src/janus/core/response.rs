use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;
use tokio_tungstenite::tungstenite::Message;
use crate::janus::json::{self, *};
use crate::janus::error::JanusError;

#[derive(Serialize, Deserialize)]
pub struct PluginResultWrapper {
    plugin: String,
    data: JSON_ANY
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct JanusResponse {
    pub janus: String,
    pub transaction: String,
    pub error: Option<JanusError>,

    /** session_id (websocket) */
    #[serde(default, skip_serializing_if = "is_zero")]
    pub session_id: JSON_POSITIVE_INTEGER,

    /** handle_id (websocket) */
    #[serde(default, skip_serializing_if = "is_zero")]
    pub sender: JSON_POSITIVE_INTEGER,

    /** create, attach request */
    pub data: Option<JSON_ANY>,

    /** plugin request */
    pub plugindata: Option<PluginResultWrapper>,

    /** JSEP SDP */
    pub jsep: Option<JSON_ANY>
}

impl JanusResponse {
    pub fn with_data(mut self, data: JSON_ANY) -> JanusResponse {
        self.data = Some(data);
        self
    }

    pub fn with_err(mut self, error: JanusError) -> JanusResponse {
        self.error = Some(error);
        self
    }

    pub fn with_plugindata(mut self, handle_id: u64, plugin: &'static str, data: JSON_ANY) -> JanusResponse {
        self.sender = handle_id;
        self.plugindata = Some(PluginResultWrapper { plugin: plugin.to_string(), data });
        self
    }

    pub fn new(name: &'static str, session: u64, transaction: String) -> JanusResponse {
        // TODO: is `session_id` and `sender` returned from request
        JanusResponse {
            janus: name.to_string(),
            transaction,
            error: None,
            session_id: session,
            sender: 0,
            data: None,
            plugindata: None,
            jsep: None
        }
    }

    pub fn bad_request(error: JanusError) -> JanusResponse {
        JanusResponse {
            janus: "error".to_string(),
            transaction: "".to_string(),
            error: Some(error),
            session_id: 0,
            sender: 0,
            data: None,
            plugindata: None,
            jsep: None
        }
    }

    pub fn stringify(&self) -> Result<String, JanusError> {
        json::stringify(self)
    }
}

impl From<JanusResponse> for Message {
    fn from(response: JanusResponse) -> Self {
        let text = response.stringify().ok().unwrap();
        Message::Text(text)
    }
}
