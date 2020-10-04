#[allow(dead_code)]
mod error;

#[allow(dead_code)]
mod request;

#[allow(dead_code)]
mod request_mixin;
mod provider;
mod constant;
mod response;

use std::sync::Arc;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use async_trait::async_trait;
use tokio::sync::RwLock;
use self::constant::*;
use self::error::*;
use self::request::{CreateParameters, JoinParameters, ExistsParameters};
use self::response::VideoroomResponse;
use self::provider::{VideoRoomStateProvider, MemoryVideoRoomState};
use super::{JanusPluginFactory, BoxedPlugin};
use crate::janus::plugin::{JanusPlugin, JanusPluginResult, JanusPluginMessage};
use crate::janus::core::json::*;
use crate::janus::core::JanusHandle;

pub struct VideoRoomPluginFactory {
    provider: Arc<Box<dyn VideoRoomStateProvider>>
}

impl VideoRoomPluginFactory {
    pub fn new() -> VideoRoomPluginFactory {
        VideoRoomPluginFactory {
            provider: Arc::new(Box::new(MemoryVideoRoomState::new()))
        }
    }
}

impl JanusPluginFactory for VideoRoomPluginFactory {
    fn new(&self) -> BoxedPlugin {
        Box::new(VideoRoomPlugin::new(Arc::clone(&self.provider)))
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
    state: Arc<Box<dyn VideoRoomStateProvider>>,
    session: RwLock<VideoRoomSession>,     // must use std::sync?
}

impl VideoRoomPlugin {
    pub fn new(state_provider: Arc<Box<dyn VideoRoomStateProvider>>) -> VideoRoomPlugin {
        VideoRoomPlugin {
            state: state_provider,
            session: RwLock::new(VideoRoomSession::new())
        }
    }
}

#[async_trait]
impl JanusPlugin for VideoRoomPlugin {
    fn get_name(&self) -> &'static str {
        "janus.plugin.videoroom"
    }

    async fn handle_message(&self, message: JanusPluginMessage) -> JanusPluginResult {
        match self.process_message(message).await {
            Ok(x) => x,
            Err(e) => JanusPluginResult::ok(e.into())
        }
    }

    async fn handle_async_message(&self, message: JanusPluginMessage) -> Option<JanusPluginResult> {
        match self.process_message_async(message).await {
            Ok(x) => Some(x),
            Err(e) => Some(JanusPluginResult::ok(e.into()))
        }
    }
}

impl VideoRoomPlugin {
    async fn process_message(&self, message: JanusPluginMessage) -> Result<JanusPluginResult, VideoroomError> {
        let request_text = match message.body["request"].as_str() {
            Some(x) => x,
            None => return Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_MISSING_ELEMENT, "'request' is required".to_string()))
        };

        match request_text {
            "create" => self.create_room(serde_json::from_value(message.body)?),
            // "edit" => (),
            // "destroy" => (),
            "list" => {
                let rooms = self.state.list_rooms().into_iter()
                    .map(|x| json!({ "room": x }))
                    .collect::<Vec<JSON_ANY>>();

                let data = json!({
                    "videoroom": "success",
                    "list": rooms
                });
                Ok(JanusPluginResult::ok(data))
            },
            // "rtp_forward" => (),
            // "stop_rtp_forward" => (),
            "exists" => {
                let params: ExistsParameters = serde_json::from_value(message.body)?;
                let exists = self.state.has_room(&params.room);
                let data = json!({
                    "videoroom": "success",
                    "room": params.room,
                    "exists": exists
                });
                Ok(JanusPluginResult::ok(data))
            },
            // "allowed" => (),
            // "kick" => (),
            // "listparticipants" => (),
            // "listforwarders" => (),
            // "enable_recording" => (),
            x if ["join", "joinandconfigure", "configure", "publish", "unpublish", "start", "pause", "switch", "leave"].contains(&x) => {
                Arc::clone(&message.handle).queue_push(message).await;
                Ok(JanusPluginResult::wait(None))
            }
            _ => Err(
                VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_REQUEST, format!("Unknown request '{}'", request_text))
            )
        }
    }

    async fn gateway_forward(handle: &Arc<JanusHandle>, body: JSON_ANY, jsep: Option<JSON_ANY>, is_async: bool) -> Result<JanusPluginResult, VideoroomError> {
        let (res, jsep) = handle.forward_message(body, jsep, is_async).await?;
        Ok(JanusPluginResult::ok(res).with_jsep(jsep))
    }

    async fn gateway_request<T: DeserializeOwned + Serialize>(handle: &Arc<JanusHandle>, body: JSON_ANY, jsep: Option<JSON_ANY>, is_async: bool) -> Result<(VideoroomResponse<T>, Option<JSON_ANY>), VideoroomError>{
        let (response, jsep) = handle.forward_message(body, jsep, is_async).await?;

        // Parse with JSON_ANY to check "error" first (T may have required field)
        let response: VideoroomResponse = match serde_json::from_value(response) {
            Ok(x) => x,
            Err(e) => return Err(
                VideoroomError::new(JANUS_VIDEOROOM_ERROR_INTERNAL, format!("Error parsing janus-gateway response: {}", e))
            )
        };

        if let Some(e) = response.error {
            return Err(e)
        }

        let data: T = match serde_json::from_value(response.data.into()) {
            Ok(x) => x,
            Err(e) => return Err(
                VideoroomError::new(JANUS_VIDEOROOM_ERROR_INTERNAL, format!("Error parsing janus-gateway response data: {}", e))
            )
        };

        let response = VideoroomResponse {
            videoroom: response.videoroom,
            error: None,
            data
        };

        Ok((response, jsep))
    }

    async fn process_message_async(&self, message: JanusPluginMessage) -> Result<JanusPluginResult, VideoroomError> {
        let request_text = match message.body["request"].as_str() {
            Some(x) => x,
            None => return Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_MISSING_ELEMENT, "'request' is required".to_string()))
        };
        let participant_type = self.session.read().await.participant_type;

        if participant_type == JANUS_VIDEOROOM_P_TYPE_NONE {
            if request_text != "join" && request_text != "joinandconfigure" {
                return Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_JOIN_FIRST, format!("Invalid request on unconfigured participant")))
            }

            let params: JoinParameters = serde_json::from_value(message.body)?;
            return match &params.ptype[..] {
                "publisher" => {
                    // TODO: check room access
                    // TODO: set display name
                    // TODO: set user id (or random?)
                    let handle = message.handle;

                    // Create room. TODO: check created.
                    let room_params = self.state.get_room_parameters(&params.room);
                    Self::gateway_request::<JSON_ANY>(&handle, serde_json::from_str(&room_params)?, None, false).await?;

                    // Actually join
                    let params = serde_json::to_value(params)?;
                    let (response, jsep) = Self::gateway_request::<JSON_ANY>(&handle, params, None, true).await?;

                    self.session.write().await.participant_type = JANUS_VIDEOROOM_P_TYPE_PUBLISHER;

                    // TODO: return list of available publishers
                    Ok(JanusPluginResult::ok(serde_json::to_value(response)?).with_jsep(jsep))
                },
                // "listener" is deprecated
                "subscriber" | "listener" => {
                    let params = serde_json::to_value(params)?;
                    let (response, jsep) = Self::gateway_request::<JSON_ANY>(&message.handle, params, None, true).await?;

                    self.session.write().await.participant_type = JANUS_VIDEOROOM_P_TYPE_SUBSCRIBER;

                    Ok(JanusPluginResult::ok(serde_json::to_value(response)?).with_jsep(jsep))
                },
                _ => {
                    Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_ELEMENT, String::from("Invalid element (ptype)")))
                }
            }
        }
        else if participant_type == JANUS_VIDEOROOM_P_TYPE_PUBLISHER {
            return match &request_text[..] {
                "join" | "joinandconfigure" => {
                    Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_ALREADY_JOINED, String::from("Already in as a publisher on this handle")))
                }
                "configure" | "publish" => {
                    Self::gateway_forward(&message.handle, message.body, message.jsep, true).await
                },
                "unpublish" => {
                    Self::gateway_forward(&message.handle, message.body, message.jsep, true).await
                },
                "leave" => {
                    Self::gateway_forward(&message.handle, message.body, message.jsep, true).await
                },
                _ => {
                    Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_REQUEST, format!("Unknown request '{}'", request_text)))
                }
            }
        }
        else if participant_type == JANUS_VIDEOROOM_P_TYPE_SUBSCRIBER {
            return match &request_text[..] {
                "join" => {
                    Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_ALREADY_JOINED, String::from("Already in as a subscriber on this handle")))
                },
                "start" => {
                    Self::gateway_forward(&message.handle, message.body, message.jsep, true).await
                },
                "configure" => {
                    Self::gateway_forward(&message.handle, message.body, message.jsep, true).await
                },
                "pause" => {
                    Self::gateway_forward(&message.handle, message.body, message.jsep, true).await
                },
                "switch" => {
                    Self::gateway_forward(&message.handle, message.body, message.jsep, true).await
                },
                "leave" => {
                    Self::gateway_forward(&message.handle, message.body, message.jsep, true).await
                },
                _ => {
                    Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_REQUEST, format!("Unknown request '{}'", request_text)))
                }
            }
        }
        Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_UNKNOWN_ERROR, String::from("Unexpected server error, plugin state malformed")))
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
}
