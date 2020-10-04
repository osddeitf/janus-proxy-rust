use serde::{Serialize, Deserialize};
use crate::janus::core::json::*;
use super::error::VideoroomError;

#[derive(Serialize, Deserialize)]
pub struct VideoroomResponse<T = JSON_ANY>
where T: Serialize
{
    pub videoroom: String,

    #[serde(flatten)]
    pub error: Option<VideoroomError>,

    #[serde(flatten)]
    pub data: T
}
