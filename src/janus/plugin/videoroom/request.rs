use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;
use super::request_mixin::*;
use crate::janus::core::json::*;

// mixins: RoomParameters (optional), AdminKeyParameters (if enabled)
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct CreateParameters {
	request: String,
	pub room: Option<Identity>,
    pub description: Option<JSON_STRING>,
    pub is_private: Option<JSON_BOOL>,
    pub allowed: Option<JSON_STRING_ARRAY>,
    pub secret: Option<JSON_STRING>,
    pub pin: Option<JSON_STRING>,
    pub require_pvtid: Option<JSON_BOOL>,
    pub bitrate: Option<JSON_POSITIVE_INTEGER>,
    pub bitrate_cap: Option<JSON_BOOL>,
    pub fir_freq: Option<JSON_POSITIVE_INTEGER>,
    pub publishers: Option<JSON_POSITIVE_INTEGER>,
    pub audiocodec: Option<JSON_STRING>,
    pub videocodec: Option<JSON_STRING>,
    pub vp9_profile: Option<JSON_STRING>,
    pub h264_profile: Option<JSON_STRING>,
    pub opus_fec: Option<JSON_BOOL>,
    pub video_svc: Option<JSON_BOOL>,
    pub audiolevel_ext: Option<JSON_BOOL>,
    pub audiolevel_event: Option<JSON_BOOL>,
    pub audio_active_packets: Option<JSON_POSITIVE_INTEGER>,
    pub audio_level_average: Option<JSON_POSITIVE_INTEGER>,
    pub videoorient_ext: Option<JSON_BOOL>,
    pub playoutdelay_ext: Option<JSON_BOOL>,
    pub transport_wide_cc_ext: Option<JSON_BOOL>,
    pub record: Option<JSON_BOOL>,
    pub rec_dir: Option<JSON_STRING>,
    pub lock_record: Option<JSON_BOOL>,
    pub permanent: Option<JSON_BOOL>,
    pub notify_joining: Option<JSON_BOOL>,
    pub require_e2ee: Option<JSON_BOOL>
}

// mixins: RoomParameters
// missing new_lock_record in parameters definition of janus_videoroom.c
#[derive(Deserialize)]
pub struct EditParameters {
    pub secret: Option<JSON_STRING>,    // janus_videoroom_access_room(check_modify=TRUE)
    pub new_description: Option<JSON_STRING>,
    pub new_is_private: Option<JSON_BOOL>,
    pub new_secret: Option<JSON_STRING>,
    pub new_pin: Option<JSON_STRING>,
    pub new_require_pvtid: Option<JSON_BOOL>,
    pub new_bitrate: Option<JSON_POSITIVE_INTEGER>,
    pub new_fir_freq: Option<JSON_POSITIVE_INTEGER>,
    pub new_publishers: Option<JSON_POSITIVE_INTEGER>,
    pub new_lock_record: Option<JSON_BOOL>,
    pub permanent: Option<JSON_BOOL>
}

// mixins: RoomParameters
// missing secret - janus_videoroom_access_room(check_modify=TRUE)
#[derive(Deserialize)]
pub struct DestroyParameters {
    pub secret: Option<JSON_STRING>,
    pub permanent: Option<JSON_BOOL>
}

// mixins: AdminKeyParameters (if enabled)
#[derive(Deserialize)]
pub struct ListParameters {}

// mixins: RoomParameters, PidParameters, AdminKeyParameters (if lock_rtp_forward, admin_key enabled)
// missing secret - janus_videoroom_access_room(check_modify=TRUE)
#[derive(Deserialize)]
pub struct RtpForwardParameters {
	pub secret: Option<JSON_STRING>,
	pub video_port: Option<JSON_POSITIVE_INTEGER>,
	pub video_rtcp_port: Option<JSON_POSITIVE_INTEGER>,
	pub video_ssrc: Option<JSON_POSITIVE_INTEGER>,
	pub video_pt: Option<JSON_POSITIVE_INTEGER>,
	pub video_port_2: Option<JSON_POSITIVE_INTEGER>,
	pub video_ssrc_2: Option<JSON_POSITIVE_INTEGER>,
	pub video_pt_2: Option<JSON_POSITIVE_INTEGER>,
	pub video_port_3: Option<JSON_POSITIVE_INTEGER>,
	pub video_ssrc_3: Option<JSON_POSITIVE_INTEGER>,
	pub video_pt_3: Option<JSON_POSITIVE_INTEGER>,
	pub audio_port: Option<JSON_POSITIVE_INTEGER>,
	pub audio_rtcp_port: Option<JSON_POSITIVE_INTEGER>,
	pub audio_ssrc: Option<JSON_POSITIVE_INTEGER>,
	pub audio_pt: Option<JSON_POSITIVE_INTEGER>,
	pub data_port: Option<JSON_POSITIVE_INTEGER>,
	pub host: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub host_family: Option<JSON_STRING>,
	pub simulcast: Option<JSON_BOOL>,
	pub srtp_suite: Option<JSON_POSITIVE_INTEGER>,
	pub srtp_crypto: Option<JSON_STRING>
}

// same as RtpForwardParameters
#[derive(Deserialize)]
pub struct StopRtpForwardParameters {
	pub secret: Option<JSON_STRING>,
	pub stream_id: JSON_POSITIVE_INTEGER, // JANUS_JSON_PARAM_REQUIRED
}

// Mixins: RoomParameters,
#[derive(Deserialize)]
pub struct ExistsParameters {}

// mixins: RoomParameters
#[derive(Deserialize)]
pub struct AllowedParameters {
	pub secret: Option<JSON_STRING>,	// janus_videoroom_access_room(check_modify=TRUE)
	pub action: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub allowed: Option<JSON_STRING_ARRAY>
}

// mixins: RoomParameters, IdParameters
#[derive(Deserialize)]
pub struct KickParameters {
	pub secret: Option<JSON_STRING>	// janus_videoroom_access_room(check_modify=TRUE)
}

// mixins: RoomParameters 
#[derive(Deserialize)]
pub struct ListParticipantsParameters {}

// mixins: RoomParameters
// missing secret - janus_videoroom_access_room(check_modify=TRUE)
#[derive(Deserialize)]
pub struct ListForwardersParameters {
	pub secret: Option<JSON_STRING>
}

// mixins: RoomParameters, janus_videoroom_access_room=TRUE
#[derive(Deserialize)]
pub struct RecordParameters {
	pub record: JSON_BOOL, // JANUS_JSON_PARAM_REQUIRED
}

pub type EnableRecordingParameters = RecordParameters;

/** Asynchronous request type definitions */
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct JoinParameters {
	request: String,
	pub room: Identity,
	pub feed: Option<JSON_POSITIVE_INTEGER>,

    pub ptype: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub audio: Option<JSON_BOOL>,
	pub video: Option<JSON_BOOL>,
	pub data: Option<JSON_BOOL>,
	pub bitrate: Option<JSON_POSITIVE_INTEGER>,
	pub record: Option<JSON_BOOL>,
	pub filename: Option<JSON_STRING>,
	pub token: Option<JSON_STRING>
}

#[derive(Deserialize)]
pub struct PublishParameters {
    pub audio: Option<JSON_BOOL>,
	pub audiocodec: Option<JSON_STRING>,
	pub video: Option<JSON_BOOL>,
	pub videocodec: Option<JSON_STRING>,
	pub data: Option<JSON_BOOL>,
	pub bitrate: Option<JSON_POSITIVE_INTEGER>,
	pub keyframe: Option<JSON_BOOL>,
	pub record: Option<JSON_BOOL>,
	pub filename: Option<JSON_STRING>,
	pub display: Option<JSON_STRING>,
	pub secret: Option<JSON_STRING>,
	pub audio_level_averge: Option<JSON_POSITIVE_INTEGER>,
	pub audio_active_packets: Option<JSON_POSITIVE_INTEGER>,
	/* The following are just to force a renegotiation and/or an ICE restart */
	pub update: Option<JSON_BOOL>,
	pub restart: Option<JSON_BOOL>
}

#[derive(Deserialize)]
pub struct PublisherParameters {
    pub display: Option<JSON_STRING>
}

#[derive(Deserialize)]
pub struct ConfigureParameters {
    pub audio: Option<JSON_BOOL>,
	pub video: Option<JSON_BOOL>,
	pub data: Option<JSON_BOOL>,
	/* For talk detection */
	pub audio_level_averge: Option<JSON_POSITIVE_INTEGER>,
	pub audio_active_packets: Option<JSON_POSITIVE_INTEGER>,
	/* For VP8 (or H.264) simulcast */
	pub substream: Option<JSON_POSITIVE_INTEGER>,
	pub temporal: Option<JSON_POSITIVE_INTEGER>,
	pub fallback: Option<JSON_POSITIVE_INTEGER>,
	/* For VP9 SVC */
	pub spatial_layer: Option<JSON_POSITIVE_INTEGER>,
	pub temporal_layer: Option<JSON_POSITIVE_INTEGER>,
	/* The following is to handle a renegotiation */
	pub update: Option<JSON_BOOL>
}

#[derive(Deserialize)]
pub struct SubscriberParameters {
	pub private_id: Option<JSON_POSITIVE_INTEGER>,
	pub close_pc: Option<JSON_BOOL>,
	pub audio: Option<JSON_BOOL>,
	pub video: Option<JSON_BOOL>,
	pub data: Option<JSON_BOOL>,
	pub offer_audio: Option<JSON_BOOL>,
	pub offer_video: Option<JSON_BOOL>,
	pub offer_data: Option<JSON_BOOL>,
	/* For VP8 (or H.264) simulcast */
	pub substream: Option<JSON_POSITIVE_INTEGER>,
	pub temporal: Option<JSON_POSITIVE_INTEGER>,
	pub fallback: Option<JSON_POSITIVE_INTEGER>,
	/* For VP9 SVC */
	pub spatial_layer: Option<JSON_POSITIVE_INTEGER>,
	pub temporal_layer: Option<JSON_POSITIVE_INTEGER>,
}
