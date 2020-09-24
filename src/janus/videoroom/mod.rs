#[allow(dead_code)]
mod error;

#[allow(dead_code)]
mod request;

#[allow(dead_code)]
mod request_mixin;
mod helper;
mod room;
mod state;
mod constant;

use std::collections::HashMap;
use std::sync::{Mutex, Arc, Weak};
use serde_json::json;
use self::constant::*;
use self::error::*;
use self::request::CreateParameters;
use self::request_mixin::*;
use self::state::{VideoRoomStateProvider, LocalVideoRoomState};
use super::core::JanusHandle;
use super::plugin::{JanusPlugin, JanusPluginResult, JanusPluginFactory, BoxedPlugin, JanusPluginMessage};
use super::json::JSON_OBJECT;

pub struct VideoRoomPluginFactory;

impl JanusPluginFactory for VideoRoomPluginFactory {
    fn new(&self) -> BoxedPlugin {
        let provider = Box::new(LocalVideoRoomState::new());
        Box::new(VideoRoomPlugin::new(provider))
    }
}


struct VideoRoomSession {
    participant_type: u8,
    // participant: Option<?>
    // gateway: Websocket connection to janus-gateway
}

impl VideoRoomSession {
    pub fn new() -> VideoRoomSession {
        VideoRoomSession {
            participant_type: JANUS_VIDEOROOM_P_TYPE_NONE
        }
    }
}


pub struct VideoRoomPlugin {
    state: Box<dyn VideoRoomStateProvider>,
    sessions: Mutex<HashMap<u64, Arc<VideoRoomSession>>>     // must use std::sync?
}

impl VideoRoomPlugin {
    pub fn new(state_provider: Box<dyn VideoRoomStateProvider>) -> VideoRoomPlugin {
        VideoRoomPlugin {
            state: state_provider,
            sessions: Mutex::new(HashMap::new())
        }
    }
}

impl JanusPlugin for VideoRoomPlugin {
    fn get_name(&self) -> &'static str {
        "janus.plugin.videoroom"
    }

    fn handle_message(&self, message: JanusPluginMessage) -> JanusPluginResult {
        match self.process_message(message) {
            Ok(x) => x,
            Err(e) => JanusPluginResult::ok(e.into())
        }
    }

    fn handle_async_message(&self, message: JanusPluginMessage) -> Option<JanusPluginResult> {
        let handle = match Weak::upgrade(&message.handle) {
            Some(x) => x,
            None => return None
        };
        match self.process_message_async(handle, message) {
            Ok(x) => Some(x),
            Err(e) => Some(JanusPluginResult::ok(e.into()))
        }
    }

    fn new_plugin_session(&self, handle_id: u64) {
        self.sessions.lock().unwrap().insert(handle_id, Arc::new(VideoRoomSession::new()));
    }

    fn drop_plugin_session(&self, handle_id: &u64) {
        self.sessions.lock().unwrap().remove(handle_id);
    }
}

impl VideoRoomPlugin {
    fn process_message(&self, message: JanusPluginMessage) -> Result<JanusPluginResult, VideoroomError> {
        let request: RequestParameters = helper::parse_json(&message.body)?;
        let request_text = request.request;
        match &request_text[..] {
            "create" => self.create_room(helper::parse_json(&message.body)?),
            // "edit" => (),
            // "destroy" => (),
            "list" => self.list_room(),
            // "rtp_forward" => (),
            // "stop_rtp_forward" => (),
            // "exists" => (),
            // "allowed" => (),
            // "kick" => (),
            // "listparticipants" => (),
            // "listforwarders" => (),
            // "enable_recording" => (),
            x if ["join", "joinandconfigure", "configure", "publish", "unpublish", "start", "pause", "switch", "leave"].contains(&x) => {
                if let Some(handle) = Weak::upgrade(&message.handle) {
                    tokio::spawn(async move {
                        if let Err(_) = handle.handler_thread.clone().send(message).await {
                            // let ignore "closed channel" error for now
                        }
                    });
                }
                Ok(JanusPluginResult::wait(None))
            }
            _ => Err(
                VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_REQUEST, format!("Unknown request '{}'", request_text))
            )
        }
    }

    fn process_message_async(&self, handle: Arc<JanusHandle>, message: JanusPluginMessage) -> Result<JanusPluginResult, VideoroomError> {
        Ok(JanusPluginResult::ok(message.body.into()))
    }

    /** This function only validate and store the room for later creation */
    fn create_room(&self, mut params: CreateParameters) -> Result<JanusPluginResult, VideoroomError>{
        if let Some(audiocodec) = &params.audiocodec {
            let supported = ["opus", "multiopus", "isac32", "isac16", "pcmu", "pcma", "g722"];
            if !audiocodec.split(",").take(4).all(|x| supported.contains(&x)) {
                let reason = format!("Invalid element (audiocodec can only be or contain opus, isac32, isac16, pcmu, pcma or g722)");
                return Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_ELEMENT, reason))
            }
        }
        if let Some(videocodec) = &params.videocodec {
            let supported = ["vp8", "vp9", "h264", "av1", "h265"];
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

        let result = JanusPluginResult::ok(json!({
            "videoroom": "created",
            "room": room,
            "permanent": params.permanent.is_some()
        }));

        // TODO: store params to send to backend later
        self.state.save_room_parameters(params);

        Ok(result)
    }

    fn list_room(&self) -> Result<JanusPluginResult, VideoroomError> {
        // TODO: do real listing
        let data = json!({
            "videoroom": "success",
            "list": Vec::<JSON_OBJECT>::new()
        });
        Ok(JanusPluginResult::ok(data))
    }
}
