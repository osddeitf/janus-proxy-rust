use crate::janus::error::JanusError;
use crate::janus::videoroom::VideoRoomPlugin;
use crate::janus::error::code::*;
use crate::janus::json::{self, *};

pub trait JanusPlugin: Send + Sync {
    fn get_name(&self) -> &'static str;
    fn handle_message(&self, body: String) -> Result<JanusPluginResult, JanusError> {
        let data = json::parse(&body)?;
        Ok(JanusPluginResult::new_ok(data))
    }
    fn set_opaque_id(&mut self, opaque_id: &str);
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
    pub fn new_ok(data: JSON_OBJECT) -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_OK,
            text: None, content: Some(data)
        }
    }

    pub fn new_ok_wait(text: String) -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_OK_WAIT,
            text: Some(text), content: None
        }
    }

    pub fn new_error() -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_ERROR,
            text: None, content: None
        }
    }
}

pub fn find_plugin(name: &str) -> Result<Box<dyn JanusPlugin>, JanusError> {
    match name {
        "janus.plugin.videoroom" => Ok(Box::new(VideoRoomPlugin::new())),
        _ => Err(JanusError::new(JANUS_ERROR_PLUGIN_NOT_FOUND, format!("No such plugin '{}'", name)))
    }
}
