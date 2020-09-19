use serde::{Serialize, Deserialize};
use crate::janus::error::JanusError;
use super::json;
use crate::janus::videoroom::VideoRoomPlugin;
use crate::janus::error::JanusErrorCode::*;

pub trait Plugin {
    fn handle(&self, message: &PluginMessage) -> Result<String, JanusError> {
        println!("Data: {}", message.body);
        json::stringify(&message.body)
    }
    fn set_opaque_id(&mut self, opaque_id: &str);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginMessage {
    pub janus: String,      //should be "message"
    pub session_id: u64,
    pub handle_id: u64,
    pub transaction: String,
    pub body: serde_json::Value
}

pub fn find_plugin(name: &str) -> Result<Box<dyn Plugin>, JanusError> {
    match name {
        "janus.plugin.videoroom" => Ok(Box::new(VideoRoomPlugin) as Box<dyn Plugin>),
        _ => Err(JanusError::new(JANUS_ERROR_PLUGIN_NOT_FOUND, format!("No such plugin '{}'", name)))
    }
}
