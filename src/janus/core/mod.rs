pub mod request;
pub mod response;
pub mod json;
pub mod ice;
#[allow(dead_code)]
pub mod apierror;

use std::collections::HashMap;
use std::sync::{Arc, Weak};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::stream::StreamExt;
use tokio::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use super::plugin::{JanusPlugin, JanusPluginMessage};
use super::response::JanusResponse;
use super::gateway::JanusGateway;
use super::JanusProxy;
use self::apierror::*;
use self::json::*;
use self::request::IncomingRequestParameters;
use crate::janus::core::ice::JanusIceTrickle;

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

    pub async fn init_gateway(&self, plugin: &str, handle_id: u64) -> Result<(), JanusError> {
        // TODO: support multiple janus-gateway instances, one per handle
        if self.gateway.read().await.is_none() {
            let url = match self.app.backend.get_backend() {
                None => return Err(JanusError::new(JANUS_ERROR_GATEWAY_UNAVAILABLE, String::from("No janus-gateway instance available"))),
                Some(x) => x
            };

            // TODO: is unbounded safe?
            let (tx, mut rx) = mpsc::unbounded_channel::<JanusResponse>();
            let mut wtx = self.connection.clone();

            let session_id = self.id;
            tokio::spawn(async move {
                while let Some(mut x) = rx.recv().await {
                    if x.session_id != 0 {
                        x.session_id = session_id;
                    }
                    if x.sender != 0 {
                        x.sender = handle_id;
                    }

                    let text = Message::Text(x.stringify().unwrap());
                    if wtx.send(text).await.is_err() {
                        break;
                    }
                }
            });

            // TODO: modify session_id, sender
            let backend = JanusGateway::connect(url, tx).await?;
            let (session, handle) = Self::get_plugin_handle(&backend, plugin).await?;

            // TODO: This may block the above? YES!!!
            let gateway = Arc::downgrade(&Arc::clone(&backend));
            tokio::spawn(async move {
                loop {
                    tokio::time::delay_for(Duration::from_secs(15)).await;
                    let gateway = match gateway.upgrade() {
                        None => break,
                        Some(x) => x
                    };

                    let mut request = IncomingRequestParameters::prepare("keepalive".to_string(), None, None);
                    request.session_id = session;

                    let response = gateway.send(request, false).await;

                    // Stop ping
                    if let Err(e) = response {
                        if e.code == JANUS_ERROR_GATEWAY_CONNECTION_CLOSED {
                            println!("Connection to janus-gateway closed, stop ping");
                            break
                        }
                    }
                }
            });

            *self.gateway.write().await = Some(Gateway {
                instance: Arc::clone( &backend),
                session, handle
            });
        }
        Ok(())
    }

    async fn get_plugin_handle(gateway: &Arc<JanusGateway>, plugin: &str) -> Result<(u64, u64), JanusError> {
        let session = {
            let data = IncomingRequestParameters::prepare("create".to_string(), None, None);
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
            let mut data = IncomingRequestParameters::prepare("attach".to_string(), None, None);
            data.session_id = session;
            data.rest = JSON_OBJECT::new();
            data.rest.insert("plugin".to_string(), plugin.to_string().into());

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

    // TODO: request &'static str
    pub async fn forward(&self, mut request: IncomingRequestParameters, is_async: bool) -> Result<JanusResponse, JanusError> {
        match &*self.gateway.read().await {
            Some(x) => {
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
        let handle_ref = Arc::downgrade(&handle);
        tokio::spawn(async move {
            while let Some(message) = rx.next().await {
                let handle = match handle_ref.upgrade() {
                    None => break,
                    Some(x) => x
                };

                // TODO: don't copy
                let transaction = message.transaction.clone();

                // TODO: Optimization - Stop process requests if no result???
                let result = match handle.plugin.handle_async_message(message).await {
                    Some(x) => x,
                    None => break
                };

                let response = JanusResponse::new("event", session_id, transaction)
                    .with_plugindata(&handle, result.content.unwrap(), result.jsep);

                // Stop process requests when session destroyed.
                let session = match handle.session.upgrade() {
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

    pub async fn forward_message(&self, body: JSON_ANY, jsep: Option<JSON_ANY>, is_async: bool) -> Result<(JSON_ANY, Option<JSON_ANY>), JanusError> {
        let session = match self.session.upgrade() {
            Some(x) => x,
            None => return Err(JanusError::new(JANUS_ERROR_SESSION_NOT_FOUND, format!("Session closed")))
        };
        session.init_gateway(self.plugin.get_name(), self.id).await?;

        let request = IncomingRequestParameters::prepare("message".to_string(), Some(body), jsep);
        let response = session.forward(request, is_async).await?;
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
                    Ok((x.data, response.jsep))
                }
            }
        }
    }

    pub async fn trickle(&self, item: JanusIceTrickle) -> Result<(), JanusError> {
        let session = match self.session.upgrade() {
            Some(x) => x,
            None => return Err(JanusError::new(JANUS_ERROR_SESSION_NOT_FOUND, format!("Session closed")))
        };

        // TODO: store trickle if janus-gateway not connected yet?
        let mut request = IncomingRequestParameters::prepare("trickle".to_string(), None, None);
        request.rest.insert("candidate".to_string(), match serde_json::to_value(item) {
            Ok(x) => x,
            Err(_) => return Err(
                JanusError::new(JANUS_ERROR_GATEWAY_INTERNAL_ERROR, format!("Cannot serialize janus-ice-trickle",))
            )
        });

        session.forward(request, false).await?;

        Ok(())
    }
}
