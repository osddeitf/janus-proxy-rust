use serde::Deserialize;
use crate::janus::core::json::*;

#[derive(Deserialize)]
pub struct RequestParameters {
    pub request: JSON_STRING // JANUS_JSON_PARAM_REQUIRED
}

#[derive(Deserialize)]
pub struct AdminKeyParameters {
    pub admin_key: JSON_STRING // JANUS_JSON_PARAM_REQUIRED
}

// Configurable string_id,...
#[derive(Deserialize)]
pub enum Identity { Integer, str }

#[derive(Deserialize)]
pub struct RoomParameters {
    pub room: Identity // JANUS_JSON_PARAM_REQUIRED, or not, JANUS_JSON_PARAM_POSITIVE
}

#[derive(Deserialize)]
pub struct IdParameters {
    pub id: Identity // JANUS_JSON_PARAM_REQUIRED, or not, JANUS_JSON_PARAM_POSITIVE
}

#[derive(Deserialize)]
pub struct PidParameters {
    pub publisher_id: Identity // JANUS_JSON_PARAM_REQUIRED, JANUS_JSON_PARAM_POSITIVE
}

#[derive(Deserialize)]
pub struct FeedParameters {
    pub feed: Identity // JANUS_JSON_PARAM_REQUIRED, JANUS_JSON_PARAM_POSITIVE
}

/** Not officially declared in janus_videoroom.c */
#[derive(Deserialize)]
pub struct SecretParameters {
    pub secret: JSON_STRING
}

// enum Type {
//     /* Synchronous request */
//     "create",
//     "edit",
//     "destroy",
//     "list",
//     "rtp_forward",
//     "stop_rtp_forward",
//     "exists",
//     "allowed",
//     "kick",
//     "listparticipants",
//     "listforwarders",
//     "enable_recording",
//     /* Asynchronous message request */
//     "join", "joinandconfigure",
//     "configure", "publish", "unpublish",
//     "start", "pause", "switch",
//     "leave"
// }
