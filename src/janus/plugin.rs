use serde::{Serialize, Deserialize};
use super::error::JsonError;

pub trait Plugin {
    fn handle(&self, message: &PluginMessage) -> Result<String, JsonError> {
        println!("Data: {}", message.body);
        return serde_json::to_string(message).map_err(JsonError::SerialError);
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
