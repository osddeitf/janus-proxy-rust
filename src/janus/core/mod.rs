pub mod request;
pub mod response;
pub mod json;

use std::collections::HashMap;
use std::sync::{Arc, Weak};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::stream::StreamExt;
use tokio_tungstenite::tungstenite::Message;
use super::plugin::{JanusPlugin, JanusPluginMessage};
use super::response::JanusResponse;

pub struct JanusSession {
    pub id: u64,
    pub handles: RwLock<HashMap<u64, Arc<JanusHandle>>>,

    // When session actually created
    pub initialized: RwLock<bool>,

    /** Underlying pseudo websocket connection, impl by channel, send only */
    pub connection: mpsc::Sender<Message>
}

impl JanusSession {
    pub fn new(id: u64, connection: mpsc::Sender<Message>) -> JanusSession {
        JanusSession {
            id, connection,
            handles: RwLock::new(HashMap::new()),
            initialized: RwLock::new(false)
        }
    }
}

pub struct JanusHandle {
    pub id: u64,
    pub plugin: Box<dyn JanusPlugin>,
    session: Weak<JanusSession>,

    /** Push async message to processing queue (single for now) */
    handler_thread: mpsc::Sender<JanusPluginMessage>
}

impl JanusHandle {
    pub fn new(id: u64, session: Arc<JanusSession>, plugin: Box<dyn JanusPlugin>) -> Arc<JanusHandle> {
        let (tx, mut rx) = mpsc::channel::<JanusPluginMessage>(32);

        let session_id = session.id;
        let handle = Arc::new(JanusHandle {
            id, plugin,
            session: Arc::downgrade(&session),
            handler_thread: tx
        });

        // Process async message one-by-one, mimic janus-gateway implementation
        let handle_ref = Arc::clone(&handle);
        tokio::spawn(async move {
            while let Some(message) = rx.next().await {
                // TODO: don't copy
                let transaction = message.transaction.clone();

                // TODO: Optimization - Stop process requests if no result???
                let result = match handle_ref.plugin.handle_async_message(message).await {
                    Some(x) => x,
                    None => break
                };

                let response = JanusResponse::new("event", session_id, transaction)
                    .with_plugindata(handle_ref.id, handle_ref.plugin.get_name(), result.content.unwrap());

                // Stop process requests when session destroyed.
                let session = match handle_ref.session.upgrade() {
                    None => break,
                    Some(x) => x
                };

                // Stop process requests when websocket connection closed.
                if session.connection.clone().send(response.into()).await.is_err() {
                    break
                }
            }
        });

        handle
    }

    pub async fn queue_push(&self, message: JanusPluginMessage) {
        if let Err(_) = self.handler_thread.clone().send(message).await {
            // TODO: let ignore "closed channel" error for now
        }
    }

    pub async fn transport_gone(&self) -> bool {
        self.session.upgrade().is_none()
    }
}
