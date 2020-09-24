pub mod request;
pub mod response;
pub mod json;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::stream::StreamExt;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use super::plugin::{JanusPlugin, JanusPluginMessage};
use super::response::JanusResponse;

pub struct JanusSession {
    pub session_id: u64,
    pub handles: HashMap<u64, Arc<JanusHandle>>     // TODO: consider using std::sync::Weak
}

impl JanusSession {
    pub fn new(id: u64) -> JanusSession {
        JanusSession {
            session_id: id,
            handles: HashMap::new()
        }
    }
}

pub type JanusEventEmitter = mpsc::Sender<Message>;
pub struct JanusHandle {
    pub plugin: Arc<Box<dyn JanusPlugin>>,
    pub handle_id: u64,
    pub session_id: u64,

    /** Push event to underlying websocket connection */
    pub event_push: JanusEventEmitter,

    /** Push async message to processing queue (single for now) */
    pub handler_thread: mpsc::Sender<JanusPluginMessage>,

    /** Internal join handler, get drop with handle. TODO: verify this statement is correct */
    worker: JoinHandle<()>
}

impl JanusHandle {
    pub fn new(id: u64, session: u64, event_push: JanusEventEmitter, plugin: Arc<Box<dyn JanusPlugin>>) -> JanusHandle {
        let (tx, mut rx) = mpsc::channel::<JanusPluginMessage>(32);

        let _plugin_ = Arc::clone(&plugin);
        let mut _event_push_ = mpsc::Sender::clone(&event_push);

        let join_handle = tokio::spawn(async move {
            while let Some(message) = rx.next().await {
                // TODO: don't copy
                let transaction = message.transaction.clone();
                let result = match _plugin_.handle_async_message(message) {
                    Some(x) => x,
                    None => break
                };

                let response = JanusResponse::new("event", session, transaction)
                    .with_plugindata(id, _plugin_.get_name(), result.content.unwrap());

                if _event_push_.send(response.into()).await.is_err() {
                    break
                }
            }
        });

        plugin.new_plugin_session(id);
        JanusHandle {
            plugin,
            event_push,
            session_id: session,
            handle_id: id,
            handler_thread: tx,
            worker: join_handle
        }
    }
}

impl Drop for JanusHandle {
    fn drop(&mut self) {
        println!("Handle dropped");
        self.plugin.drop_plugin_session(&self.handle_id);
    }
}
