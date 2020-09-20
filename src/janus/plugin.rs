use crate::janus::error::JanusError;
use crate::janus::videoroom::VideoRoomPlugin;
use crate::janus::error::JanusErrorCode::*;
use crate::janus::json::{JSON_OBJECT, JSON_POSITIVE_INTEGER};

pub trait Plugin {
    fn handle_message(&self, body: JSON_OBJECT) -> Result<PluginResult, JanusError> {
        Ok(PluginResult::new_ok(body))
    }
    fn set_opaque_id(&mut self, opaque_id: &str);
}

#[allow(non_camel_case_types)]
pub enum PluginResultType {
    JANUS_PLUGIN_ERROR,
    JANUS_PLUGIN_OK,
    JANUS_PLUGIN_OK_WAIT
}

pub struct PluginResult {
    pub kind: PluginResultType,     // 'type' is reserved
    pub text: Option<String>,
    pub content: Option<JSON_OBJECT>
}

impl PluginResult {
    pub fn new_ok(data: JSON_OBJECT) -> PluginResult {
        PluginResult {
            kind: PluginResultType::JANUS_PLUGIN_OK,
            text: None, content: Some(data)
        }
    }

    pub fn new_ok_wait(text: String) -> PluginResult {
        PluginResult {
            kind: PluginResultType::JANUS_PLUGIN_OK_WAIT,
            text: Some(text), content: None
        }
    }

    pub fn new_error() -> PluginResult {
        PluginResult {
            kind: PluginResultType::JANUS_PLUGIN_ERROR,
            text: None, content: None
        }
    }
}

pub fn find_plugin(name: &str, handle_id: JSON_POSITIVE_INTEGER) -> Result<Box<dyn Plugin>, JanusError> {
    match name {
        "janus.plugin.videoroom" => {
            let plugin = VideoRoomPlugin::new(handle_id);
            Ok(Box::new(plugin))
        },
        _ => Err(JanusError::new(JANUS_ERROR_PLUGIN_NOT_FOUND, format!("No such plugin '{}'", name)))
    }
}
