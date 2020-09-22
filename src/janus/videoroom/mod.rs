use super::plugin::{JanusPlugin, JanusPluginResult, JanusPluginFactory, BoxedPlugin};

pub struct VideoRoomPluginFactory;

impl JanusPluginFactory for VideoRoomPluginFactory {
    fn new(&self) -> BoxedPlugin {
        Box::new(VideoRoomPlugin::new())
    }
}


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
