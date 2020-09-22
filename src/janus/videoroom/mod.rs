use super::plugin::JanusPlugin;

pub struct VideoRoomPlugin {}

impl VideoRoomPlugin {
    pub fn new() -> VideoRoomPlugin {
        VideoRoomPlugin {}
    }
}

impl JanusPlugin for VideoRoomPlugin {
    fn get_name(&self) -> &'static str {
        "janus.plugin.videoroom"
    }
}
