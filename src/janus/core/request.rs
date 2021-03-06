use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;
use super::json::*;
use crate::janus::helper;
use crate::janus::core::ice::JanusIceTrickle;

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct IncomingRequestParameters {
	pub transaction: JSON_STRING,		// JANUS_JSON_PARAM_REQUIRED
	pub janus: JSON_STRING,				// JANUS_JSON_PARAM_REQUIRED
	#[serde(default, skip_serializing_if = "is_zero")]
	pub id: JSON_POSITIVE_INTEGER,

	/** Additional (unofficial) parameters */
	#[serde(default, skip_serializing_if = "is_zero")]
	pub session_id: JSON_POSITIVE_INTEGER,
	#[serde(default, skip_serializing_if = "is_zero")]
	pub handle_id: JSON_POSITIVE_INTEGER,

	/** Plugin message, should not be null if `janus == "message"` */
	pub body: Option<JSON_ANY>,
	pub jsep: Option<JSON_ANY>,

	#[serde(flatten)]
	pub rest: JSON_OBJECT
}

impl IncomingRequestParameters {
	pub fn prepare(request: String, body: Option<JSON_ANY>, jsep: Option<JSON_ANY>) -> IncomingRequestParameters {
		IncomingRequestParameters {
			transaction: helper::rand_id().to_string(),     // TODO: conflict resolution
			janus: request,
			id: 0,
			session_id: 0,
			handle_id: 0,
			body,
			jsep,
			rest: Default::default()
		}
	}
}

/** janus=trickle, unofficial */
#[derive(Deserialize)]
pub struct TrickleParameters {
	pub candidate: Option<JanusIceTrickle>,
	pub candidates: Option<Vec<JanusIceTrickle>>
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct AttachParameters {
	pub plugin: JSON_STRING,		// JANUS_JSON_PARAM_REQUIRED
	pub opaque_id: Option<JSON_STRING>
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct BodyParameters {
	pub body: JSON_ANY,
	/** Unofficial property */
	pub jsep: Option<JSON_ANY>
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct JsepParameters {
	#[serde(rename = "type")]
	pub _type: JSON_STRING, 		// JANUS_JSON_PARAM_REQUIRED
	pub sdp: JSON_STRING, 			// JANUS_JSON_PARAM_REQUIRED
	pub trickle: JSON_BOOL,
	pub e2ee: JSON_BOOL
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct AddTokenParameters {
	pub token: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub plugins: JSON_ARRAY<JSON_STRING>
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct TokenParameters {
	pub token: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct AdminParameters {
	pub transaction: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub janus: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct DebugParameters {
	pub debug: JSON_BOOL, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct TimeoutParameters {
	pub timeout: JSON_POSITIVE_INTEGER, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct LevelParameters {
	pub level: JSON_POSITIVE_INTEGER, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct TimestampsParameters {
	pub timestamps: JSON_BOOL, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct ColorsParameters {
	pub colors: JSON_BOOL, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct MnqParameters {
	pub min_nack_queue: JSON_POSITIVE_INTEGER, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct NmtParameters {
	pub no_media_timer: JSON_POSITIVE_INTEGER, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct StParameters {
	pub slowlink_threshold: JSON_POSITIVE_INTEGER, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct AnsParameters {
	pub accept: JSON_BOOL, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct QueryHandlerParameters {
	pub handler: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub request: JSON_ANY
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct QueryLoggerParameters {
	pub logger: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub request: JSON_ANY
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct MessagePluginParameters {
	pub plugin: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub request: JSON_ANY
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct CustomEventParamaters {
	pub schema: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub data: JSON_ANY, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct CustomLoglineParameters {
	pub line: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub level: JSON_POSITIVE_INTEGER
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct Text2pcapParameters {
	pub folder: JSON_STRING,
	pub filename: JSON_STRING,
	pub truncate: JSON_POSITIVE_INTEGER
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct HandleInfoParameters {
	pub plugin_only: JSON_BOOL
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct ResAddrParameters {
	pub address: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
}

#[skip_serializing_none]
#[derive(Deserialize)]
pub struct TestStunParameters {
	pub address: JSON_STRING, // JANUS_JSON_PARAM_REQUIRED
	pub port: JSON_POSITIVE_INTEGER, // JANUS_JSON_PARAM_REQUIRED
	pub localport: JSON_POSITIVE_INTEGER,
}
