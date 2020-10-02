use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;
use tokio_tungstenite::tungstenite::Message;
use super::json::{self, *};
use super::apierror::JanusError;
use crate::janus::core::JanusHandle;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginResultWrapper {
    pub plugin: String,
    pub data: JSON_ANY
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct JanusResponse {
    pub janus: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transaction: String,        // TODO: ice event like "webrtcup" not having a transaction string
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

    pub fn with_plugindata(mut self, handle: &Arc<JanusHandle>, data: JSON_ANY, jsep: Option<JSON_ANY>) -> JanusResponse {
        self.sender = handle.id;
        self.plugindata = Some(PluginResultWrapper {
            plugin: handle.plugin.get_name().to_string(),
            data
        });
        self.jsep = jsep;
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
