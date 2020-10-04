use serde::{Serialize, Deserialize};
use crate::janus::core::json::*;
use super::error::VideoroomError;

#[derive(Serialize, Deserialize)]
pub struct VideoroomResponse<T = JSON_OBJECT>
where T: Serialize
{
    pub videoroom: String,

    #[serde(flatten)]
    pub error: Option<VideoroomError>,

    #[serde(flatten)]
    pub data: Option<T>
}

impl<T> VideoroomResponse<T>
where T: Serialize
{
    pub fn new(text: String, error: Option<VideoroomError>, data: Option<T>) -> VideoroomResponse<T> {
        VideoroomResponse {
            videoroom: text,
            error, data
        }
    }
}
