pub mod request;
pub mod response;
pub mod json;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::stream::StreamExt;
use tokio_tungstenite::tungstenite::Message;
use super::plugin::{JanusPlugin, JanusPluginMessage};
use super::plugin::JanusPluginResultType::*;
use super::response::JanusResponse;
use super::error::JanusError;
use super::error::code::JANUS_ERROR_PLUGIN_MESSAGE;
use self::json::JSON_OBJECT;

pub(crate) struct JanusSession {
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

pub type JanusEventEmitter = mpsc::Sender<Message>;
pub struct JanusHandle {
    plugin: Arc<Box<dyn JanusPlugin>>,
    handle_id: u64,
    session_id: u64,

    /** Push event to underlying websocket connection */
    event_push: JanusEventEmitter,

    /** Push async message to processing queue (single for now) */
    handler_thread: mpsc::Sender<JanusPluginMessage>,
}

impl JanusHandle {
    pub fn new(id: u64, session: u64, event_push: JanusEventEmitter, plugin: Box<dyn JanusPlugin>) -> JanusHandle {
        let (tx, mut rx) = mpsc::channel::<JanusPluginMessage>(32);

        let plugin = Arc::new(plugin);
        let _plugin_ = Arc::clone(&plugin);
        let mut _event_push_ = mpsc::Sender::clone(&event_push);

        // Process async message one-by-one, mimic janus-gateway implementation
        tokio::spawn(async move {
            while let Some(message) = rx.next().await {
                // TODO: don't copy
                let transaction = message.transaction.clone();
                let result = match _plugin_.handle_async_message(message).await {
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

        JanusHandle {
            plugin,
            event_push,
            session_id: session,
            handle_id: id,
            handler_thread: tx
        }
    }

    pub async fn handle_message(handle: Arc<JanusHandle>, transaction: String, body: JSON_OBJECT, jsep: Option<JSON_OBJECT>) -> Result<JanusResponse, JanusError> {
        // Too many copy - TODO
        let result = handle.plugin.handle_message(JanusPluginMessage::new(
            Arc::downgrade(&handle),
            transaction.clone(),
            json::stringify(&body)?,
            jsep
        )).await;

        let response = match result.kind {
            // TODO: handle optional content
            JANUS_PLUGIN_OK => JanusResponse::new("success", handle.session_id, transaction)
                .with_plugindata(handle.handle_id, handle.plugin.get_name(), result.content.unwrap()),
            // TODO: add `hint`
            JANUS_PLUGIN_OK_WAIT => JanusResponse::new("ack", handle.session_id, transaction),
            JANUS_PLUGIN_ERROR => {
                let text = result.text.unwrap_or("Plugin returned a severe (unknown) error".to_string());
                return Err(JanusError::new(JANUS_ERROR_PLUGIN_MESSAGE, text))
            }
        };
        Ok(response)
    }

    pub async fn queue_push(&self, message: JanusPluginMessage) {
        if let Err(_) = self.handler_thread.clone().send(message).await {
            // let ignore "closed channel" error for now
        }
    }
}
