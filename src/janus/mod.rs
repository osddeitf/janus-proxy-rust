use tokio_tungstenite::tungstenite;
use http::StatusCode;

/**
* Request types are ported from janus-gateway v0.10.5
*/
mod plugin;
mod error;
mod videoroom;

use self::error::JsonError;
use self::plugin::PluginMessage;
use self::videoroom::VideoRoom;
use crate::janus::plugin::Plugin;

pub struct Janus {
    videoroom: VideoRoom
}

impl Janus {
    pub fn new() -> Janus {
        Janus {
            videoroom: VideoRoom {}
        }
    }

    pub fn handle_incoming(&self, data: &String) -> Result<tungstenite::Message, tungstenite::Error> {
        let data: serde_json::Value = serde_json::from_str(data).map_err(JsonError::ParseError)?; //parseJson(data)?;
        if data["janus"].eq("message") {
            let request: PluginMessage = serde_json::from_value(data).map_err(JsonError::SerialError)?;
            let result = self.videoroom.handle(&request)?;
            return Ok(tungstenite::Message::Text(result));
        }
        return Ok(tungstenite::Message::Text(data.to_string()));
    }
}
