#[allow(dead_code)]
mod error;

#[allow(dead_code)]
mod request;

#[allow(dead_code)]
mod request_mixin;
mod helper;
mod state;

use serde_json::json;
use self::error::*;
use self::request::CreateParameters;
use self::request_mixin::*;
use self::state::VideoRoomStateProvider;
use self::state::LocalVideoRoomState;
use super::error::JanusError;
use super::plugin::{JanusPlugin, JanusPluginResult, JanusPluginFactory, BoxedPlugin};

pub struct VideoRoomPluginFactory;

impl JanusPluginFactory for VideoRoomPluginFactory {
    fn new(&self) -> BoxedPlugin {
        let provider = Box::new(LocalVideoRoomState::new());
        Box::new(VideoRoomPlugin::new(provider))
    }
}


pub struct VideoRoomPlugin {
    state: Box<dyn VideoRoomStateProvider>
}

impl VideoRoomPlugin {
    pub fn new(state_provider: Box<dyn VideoRoomStateProvider>) -> VideoRoomPlugin {
        VideoRoomPlugin {
            state: state_provider
        }
    }
}

impl JanusPlugin for VideoRoomPlugin {
    fn get_name(&self) -> &'static str {
        "janus.plugin.videoroom"
    }

    fn handle_message(&self, body: String) -> Result<JanusPluginResult, JanusError> {
        match self.process_message(body) {
            Ok(x) => Ok(x),
            Err(e) => Ok(JanusPluginResult::new_ok(e.into()))
        }
    }

    fn set_opaque_id(&mut self, _opaque_id: &str) {}
}

impl VideoRoomPlugin {
    fn process_message(&self, body: String) -> Result<JanusPluginResult, VideoroomError> {
        let request: RequestParameters = helper::parse_json(&body)?;
        let request_text = request.request;
        match &request_text[..] {
            "create" => self.create_room(helper::parse_json(&body)?),
            _ => Err(
                VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_REQUEST, format!("Unknown request '{}'", request_text))
            )
        }
    }

    fn create_room(&self, mut params: CreateParameters) -> Result<JanusPluginResult, VideoroomError>{
        if let Some(audiocodec) = params.audiocodec {
            let supported = vec!["opus", "multiopus", "isac32", "isac16", "pcmu", "pcma", "g722"];
            if !audiocodec.split(",").take(4).all(|x| supported.contains(&x)) {
                let reason = format!("Invalid element (audiocodec can only be or contain opus, isac32, isac16, pcmu, pcma or g722)");
                return Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_ELEMENT, reason))
            }
        }
        if let Some(videocodec) = params.videocodec {
            let supported = vec!["vp8", "vp9", "h264", "av1", "h265"];
            if !videocodec.split(",").take(4).all(|x| supported.contains(&x)) {
                let reason = format!("Invalid element (videocodec can only be or contain vp8, vp9, av1, h264 or h265)");
                return Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_ELEMENT, reason))
            }
        }

        // TODO: permanent check, for now, ignore it
        params.permanent = None;

        // TODO: support string id, for now, only integer are supported
        let room = match params.room {
            Some(room) => {
                if self.state.has_room(&room) {
                    return Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_ROOM_EXISTS, format!("Room {} already exists", room)))
                }
                room
            },
            None => self.state.new_room_id()
        };
        params.room = Some(room);

        let result = JanusPluginResult::new_ok(json!({
            "videoroom": "created",
            "room": room,
            "permanent": params.permanent.is_some()
        }));

        Ok(result)
    }
}
