use serde::{Serialize, Deserialize};
use crate::janus::error::JanusError;
use super::json;

pub trait Plugin {
    fn handle(&self, message: &PluginMessage) -> Result<String, JanusError> {
        println!("Data: {}", message.body);
        json::stringify(&message.body)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginMessage {
    pub janus: String,      //should be "message"
    pub session_id: u64,
    pub handle_id: u64,
    pub transaction: String,
    pub body: serde_json::Value
}
