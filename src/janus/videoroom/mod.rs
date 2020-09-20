use super::plugin::Plugin;
use crate::janus::json::JSON_POSITIVE_INTEGER;

pub struct VideoRoomPlugin {
    handle_id: JSON_POSITIVE_INTEGER
}

impl VideoRoomPlugin {
    pub fn new(handle_id: JSON_POSITIVE_INTEGER) -> VideoRoomPlugin {
        VideoRoomPlugin { handle_id }
    }
}

impl Plugin for VideoRoomPlugin {
    fn set_opaque_id(&mut self, _opaque_id: &str) {}
}
