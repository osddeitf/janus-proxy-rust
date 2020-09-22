use std::collections::HashMap;
use crate::janus::plugin::JanusPlugin;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::Message;

pub struct JanusSession {
    pub session_id: u64,
    pub handles: HashMap<u64, Arc<JanusHandle>>
}

impl JanusSession {
    pub fn new(id: u64) -> JanusSession {
        JanusSession {
            session_id: id,
            handles: HashMap::new()
        }
    }
}

pub type JanusEventEmitter = Sender<Message>;
pub struct JanusHandle {
    pub plugin: Box<dyn JanusPlugin>,
    pub handle_id: u64,
    pub session_id: u64,
    pub event_emitter: JanusEventEmitter
}

impl JanusHandle {
    pub fn new(id: u64, session: u64, event_emitter: JanusEventEmitter, plugin: Box<dyn JanusPlugin>) -> JanusHandle {
        JanusHandle {
            plugin,
            event_emitter,
            session_id: session,
            handle_id: id
        }
    }
}
