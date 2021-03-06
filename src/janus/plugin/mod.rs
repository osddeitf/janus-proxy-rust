mod videoroom;
mod provider;

pub use self::provider::*;
use std::sync::Arc;
use async_trait::async_trait;
use crate::janus::core::json::*;
use crate::janus::core::JanusHandle;

// Resemble `janus_videoroom_handle_message` function signature
pub struct JanusPluginMessage {
    pub handle: Arc<JanusHandle>,
    pub transaction: String,
    pub body: JSON_ANY,
    pub jsep: Option<JSON_ANY>
}

impl JanusPluginMessage {
    pub fn new(handle: Arc<JanusHandle>, transaction: String, body: JSON_ANY, jsep: Option<JSON_ANY>) -> JanusPluginMessage {
        JanusPluginMessage { handle, transaction, body, jsep }
    }
}

// * functions in traits cannot be declared `async`
// May `handle_message*` return JanusError? TODO
#[async_trait]
pub trait JanusPlugin: Send + Sync {
    fn get_name(&self) -> &'static str;
    async fn handle_message(&self, message: JanusPluginMessage) -> JanusPluginResult;
    async fn handle_async_message(&self, message: JanusPluginMessage) -> Option<JanusPluginResult>;
    // fn set_opaque_id(&mut self, opaque_id: &str);
}

#[allow(non_camel_case_types, dead_code)]
pub enum JanusPluginResultType {
    // 'Shutting down' or 'Plugin not initialized'
    JANUS_PLUGIN_ERROR,
    JANUS_PLUGIN_OK,
    JANUS_PLUGIN_OK_WAIT
}

pub struct JanusPluginResult {
    pub kind: JanusPluginResultType,     // 'type' is reserved
    pub text: Option<String>,
    pub content: Option<JSON_ANY>,
    pub jsep: Option<JSON_ANY>
}

#[allow(dead_code)]
impl JanusPluginResult {
    pub fn ok(data: JSON_ANY) -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_OK,
            text: None, content: Some(data), jsep: None
        }
    }

    pub fn wait(text: Option<String>) -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_OK_WAIT,
            text, content: None, jsep: None
        }
    }

    pub fn err() -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_ERROR,
            text: None, content: None, jsep: None
        }
    }

    pub fn with_jsep(mut self, jsep: Option<JSON_ANY>) -> Self {
        self.jsep = jsep;
        self
    }
}
