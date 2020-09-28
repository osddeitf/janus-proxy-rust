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
use super::gateway::JanusGateway;
use super::error::JanusError;
use super::error::code::*;
use super::helper;
use super::core::json::JSON_ANY;
use super::core::request::IncomingRequestParameters;
use super::JanusProxy;

pub struct Gateway {
    instance: Arc<JanusGateway>,
    session: u64,
    handle: u64
}

pub struct JanusSession {
    pub id: u64,
    pub handles: RwLock<HashMap<u64, Arc<JanusHandle>>>,

    /** State of the session, true when session actually created */
    pub initialized: RwLock<bool>,

    /** App: for global state access */
    pub app: Arc<JanusProxy>,

    /** Underlying pseudo websocket connection, impl by channel, send only */
    pub connection: mpsc::Sender<Message>,

    /** Janus gateway connection, only initialize once, so it's more like an `Arc` */
    pub gateway: RwLock<Option<Gateway>>
}

impl JanusSession {
    pub fn new(app: Arc<JanusProxy>, id: u64, connection: mpsc::Sender<Message>) -> JanusSession {
        JanusSession {
            id, connection, app,
            handles: RwLock::new(HashMap::new()),
            initialized: RwLock::new(false),
            gateway: RwLock::new(None)
        }
    }

    pub async fn init_gateway(&self, plugin: &str) -> Result<(), JanusError> {
        // Should be run only once
        if self.gateway.read().await.is_none() {
            let url = match self.app.backend.get_backend() {
                None => return Err(JanusError::new(JANUS_ERROR_GATEWAY_UNAVAILABLE, String::from("No janus-gateway instance available"))),
                Some(x) => x
            };

            // TODO: modify session_id, sender
            let gateway = JanusGateway::connect(url, self.connection.clone()).await?;
            let (session, handle) = Self::get_plugin_handle(&gateway, plugin).await?;

            *self.gateway.write().await = Some(Gateway {
                instance: Arc::clone( &gateway),
                session, handle
            });
        }
        Ok(())
    }

    async fn get_plugin_handle(gateway: &Arc<JanusGateway>, plugin: &str) -> Result<(u64, u64), JanusError> {
        let session = {
            let data = Self::prepare("create".to_string(), None, None);
            let response = gateway.send(data, false).await?;
            let session = match response.data {
                None => 0,
                Some(x) => x["id"].as_u64().unwrap_or(0)
            };

            if session == 0 {
                // TODO: may be print `response.error` if present
                return Err(JanusError::new(JANUS_ERROR_GATEWAY_INTERNAL_ERROR, String::from("Could not obtain janus-gateway session_id")))
            }
            session
        };

        let handle = {
            let mut data = Self::prepare("attach".to_string(), None, None);
            data.session_id = session;
            data.plugin = Some(plugin.to_string());

            let response = gateway.send(data, false).await?;
            let handle = match response.data {
                None => 0,
                Some(x) => x["id"].as_u64().unwrap_or(0)
            };

            if handle == 0 {
                // TODO: may be print `response.error` if present
                return Err(JanusError::new(JANUS_ERROR_GATEWAY_INTERNAL_ERROR, String::from("Could not obtain janus-gateway handle_id")))
            }
            handle
        };

        Ok((session, handle))
    }

    fn prepare(request: String, body: Option<JSON_ANY>, jsep: Option<JSON_ANY>) -> IncomingRequestParameters {
        IncomingRequestParameters {
            transaction: helper::rand_id().to_string(),     // TODO: conflict resolution
            janus: request,
            id: 0,
            session_id: 0,
            handle_id: 0,
            plugin: None,
            body, jsep
        }
    }

    // TODO: request &'static str
    pub async fn forward(&self, request: String, body: Option<JSON_ANY>, jsep: Option<JSON_ANY>, is_async: bool) -> Result<JanusResponse, JanusError> {
        match &*self.gateway.read().await {
            Some(x) => {
                let mut request = Self::prepare(request, body, jsep);
                request.session_id = x.session;
                request.handle_id = x.handle;

                x.instance.send(request, is_async).await
            },
            None => return Err(JanusError::new(JANUS_ERROR_GATEWAY_INTERNAL_ERROR, "janus-gateway connection hasn't been initialized".to_string()))
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

    pub async fn forward_message(&self, body: JSON_ANY, jsep: Option<JSON_ANY>, is_async: bool) -> Result<JSON_ANY, JanusError> {
        // TODO: session close -> return nothing?
        let session = self.session.upgrade().unwrap();
        session.init_gateway(self.plugin.get_name()).await?;

        let response = session.forward("message".to_string(), Some(body), jsep, is_async).await?;
        if let Some(e) = response.error {
            return Err(JanusError::new(JANUS_ERROR_GATEWAY_INTERNAL_ERROR, format!("janus-gateway error: {:?}", e)))
        }

        match response.plugindata {
            None => Err(JanusError::new(JANUS_ERROR_GATEWAY_INTERNAL_ERROR, format!("Empty plugindata response data from janus-gateway: {:?}", response))),
            Some(x) => {
                if x.plugin != self.plugin.get_name() {
                    Err(JanusError::new(JANUS_ERROR_GATEWAY_INTERNAL_ERROR, "Mismatch plugindata returned from janus-gateway".to_string()))
                }
                else {
                    Ok(x.data)
                }
            }
        }
    }
}
