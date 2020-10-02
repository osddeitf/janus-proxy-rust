use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;
use super::apierror::*;

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct JanusIceTrickle {
    #[allow(non_snake_case)]
    sdpMid: Option<String>,

    #[allow(non_snake_case)]
    sdpMLineIndex: Option<u64>,

    candidate: Option<String>,

    // Trickle done
    completed: Option<bool>
}

impl JanusIceTrickle {
    pub fn validate(&self) -> Result<(), JanusError> {
        if self.completed.is_some() {
            return Ok(())
        }

        if self.sdpMid.is_none() && self.sdpMLineIndex.is_none() {
            return Err(JanusError::new(JANUS_ERROR_MISSING_MANDATORY_ELEMENT, "Trickle error: missing mandatory element (sdpMid or sdpMLineIndex)".to_string()))
        }

        if self.candidate.is_none() {
            return Err(JanusError::new(JANUS_ERROR_MISSING_MANDATORY_ELEMENT, "Trickle error: missing mandatory element (candidate)".to_string()))
        }

        Ok(())
    }
}
