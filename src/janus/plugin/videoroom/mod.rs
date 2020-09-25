#[allow(dead_code)]
mod error;

#[allow(dead_code)]
mod request;

#[allow(dead_code)]
mod request_mixin;
mod helper;
mod provider;
mod constant;

use std::sync::{Arc, Weak};
use serde_json::json;
use async_trait::async_trait;
use tokio::sync::RwLock;
use self::constant::*;
use self::error::*;
use self::request::{CreateParameters, JoinParameters, SubscriberParameters, PublishParameters, ConfigureParameters};
use self::request_mixin::*;
use self::provider::{VideoRoomStateProvider, MemoryVideoRoomState};
use crate::janus::plugin::{JanusPlugin, JanusPluginResult, JanusPluginMessage};
use crate::janus::core::json::JSON_OBJECT;
use crate::janus::provider::{JanusPluginFactory, BoxedPlugin};

pub struct VideoRoomPluginFactory;

impl JanusPluginFactory for VideoRoomPluginFactory {
    fn new(&self) -> BoxedPlugin {
        let provider = Box::new(MemoryVideoRoomState::new());
        Box::new(VideoRoomPlugin::new(Arc::new(provider)))
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
                    handle.queue_push(message).await
                }
                Ok(JanusPluginResult::wait(None))
            }
            _ => Err(
                VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_REQUEST, format!("Unknown request '{}'", request_text))
            )
        }
    }

    async fn process_message_async(&self, message: JanusPluginMessage) -> Result<JanusPluginResult, VideoroomError> {
        let request: RequestParameters = helper::parse_json(&message.body)?;
        let request_text = request.request;
        let participant_type = self.session.read().await.participant_type;

        if participant_type == JANUS_VIDEOROOM_P_TYPE_NONE {
            if request_text != "join" && request_text != "joinandconfigure" {
                return Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_JOIN_FIRST, format!("Invalid request on unconfigured participant")))
            }

            let join_params: JoinParameters = helper::parse_json(&message.body)?;
            return match &join_params.ptype[..] {
                "publisher" => {
                    // TODO: check room access
                    // TODO: set display name
                    // TODO: set user id (or random?)

                    self.session.write().await.participant_type = JANUS_VIDEOROOM_P_TYPE_PUBLISHER;

                    // TODO: return list of available publishers
                    let data = json!({
                        "videoroom": "joined",
                        "room": 0, //
                        "description": "",
                        "id": 0,    // feed
                        "private_id": 0,
                        "publishers": []
                        // ...omitted... TODO
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                // "listener" is deprecated
                "subscriber" | "listener" => {
                    let _params: SubscriberParameters = helper::parse_json(&message.body)?;
                    // TODO: verify `spatial_layer`, `substream`
                    // TODO: verify `temporal`, `temporal_layer`

                    // TODO: verify `feed` (publisher) existence
                    // TODO: mutex...
                    // sessions.participant_type = JANUS_VIDEOROOM_P_TYPE_SUBSCRIBER
                    let data = json!({
                        "videoroom": "attached",
                        "room": 0, //
                        "id": 0, // feed
                        // ...omitted... TODO
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                _ => Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_ELEMENT, String::from("Invalid element (ptype)")))
            }
        }
        else if participant_type == JANUS_VIDEOROOM_P_TYPE_PUBLISHER {
            if request_text == "join" || request_text == "joinandconfigure" {
                return Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_ALREADY_JOINED, String::from("Already in as a publisher on this handle")))
            }

            return match &request_text[..] {
                "configure" | "publish" => {
                    // TODO: "publish" -> check already published
                    // TODO: check kicked
                    let _params: PublishParameters = helper::parse_json(&message.body)?;
                    // TODO: should verify audiocodec, videocodec?
                    let data = json!({
                        "videoroom": "event",
                        "room": 0,
                        "configured": "ok"
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                "unpublish" => {
                    let data = json!({
                        "videoroom": "event",
                        "room": 0,
                        "unpublished": "ok"
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                "leave" => {
                    let data = json!({
                        "videoroom": "event",
                        "room": 0,
                        "leaving": "ok"
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                _ => Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_REQUEST, format!("Unknown request '{}'", request_text)))
            }
        }
        else if participant_type == JANUS_VIDEOROOM_P_TYPE_SUBSCRIBER {
            return match &request_text[..] {
                "join" => Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_ALREADY_JOINED, String::from("Already in as a subscriber on this handle"))),
                "start" => {
                    let data = json!({
                        "videoroom": "event",
                        "room": 0,
                        "started": "ok"
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                "configure" => {
                    let _params: ConfigureParameters = helper::parse_json(&message.body)?;
                    let data = json!({
                        "videoroom": "event",
                        "room": 0,
                        "configured": "ok"
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                "pause" => {
                    let data = json!({
                        "videoroom": "event",
                        "room": 0,
                        "paused": "ok"
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                "switch" => {
                    let _params: SubscriberParameters = helper::parse_json(&message.body)?;
                    let data = json!({
                        "videoroom": "event",
                        "room": 0,
                        "id": 0,
                        "switched": "ok"
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                "leave" => {
                    let data = json!({
                        "videoroom": "event",
                        "room": 0,
                        "left": "ok"
                    });
                    Ok(JanusPluginResult::ok(data))
                },
                _ => Err(VideoroomError::new(JANUS_VIDEOROOM_ERROR_INVALID_REQUEST, format!("Unknown request '{}'", request_text)))
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

    fn list_room(&self) -> Result<JanusPluginResult, VideoroomError> {
        // TODO: do real listing
        let data = json!({
            "videoroom": "success",
            "list": Vec::<JSON_OBJECT>::new()
        });
        Ok(JanusPluginResult::ok(data))
    }
}
