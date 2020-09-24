use crate::janus::json::*;
use std::sync::Weak;
use crate::janus::core::JanusHandle;

// Resemble `janus_videoroom_handle_message` function signature
pub struct JanusPluginMessage {
    pub handle: Weak<JanusHandle>,
    pub transaction: String,
    pub body: String,
    pub jsep: Option<JSON_OBJECT>
}

impl JanusPluginMessage {
    pub fn new(handle: Weak<JanusHandle>, transaction: String, body: String, jsep: Option<JSON_OBJECT>) -> JanusPluginMessage {
        JanusPluginMessage { handle, transaction, body, jsep }
    }
}

// * functions in traits cannot be declared `async`
// May `handle_message*` return JanusError? TODO
pub trait JanusPlugin: Send + Sync {
    fn get_name(&self) -> &'static str;
    fn handle_message(&self, message: JanusPluginMessage) -> JanusPluginResult;
    fn handle_async_message(&self, message: JanusPluginMessage) -> Option<JanusPluginResult>;
    // fn set_opaque_id(&mut self, opaque_id: &str);

    // TODO: wait for `async fn` in trait stable
    /** Functions allow memory-safety allocate/deallocate handle-scope state. */
    fn new_plugin_session(&self, handle_id: u64);
    fn drop_plugin_session(&self, handle_id: &u64);
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
    pub content: Option<JSON_OBJECT>
}

#[allow(dead_code)]
impl JanusPluginResult {
    pub fn ok(data: JSON_OBJECT) -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_OK,
            text: None, content: Some(data)
        }
    }

    pub fn wait(text: Option<String>) -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_OK_WAIT,
            text, content: None
        }
    }

    pub fn err() -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_ERROR,
            text: None, content: None
        }
    }
}
