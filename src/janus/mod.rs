mod core;
mod connection;
mod error;
mod videoroom;
mod json;
mod request;
mod response;
pub mod state;
pub mod plugin;
mod helper;

/**
* Request types are ported from janus-gateway v0.10.5
*/
use futures::{StreamExt, SinkExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::{Message, Error};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use self::core::*;
use self::request::*;
use self::response::*;
use self::plugin::{JanusPluginProvider, JanusPluginResultType::*};
use self::error::{JanusError, code::*};
use self::state::SharedStateProvider;
use self::connection::accept_ws;
use crate::janus::plugin::JanusPluginMessage;

// TODO: add gracefully shutdown
pub struct JanusProxy {
    _janus_server: String,
    /** Local mapping from connection_id -> session_id. TODO: Is there any better way? */
    connections: RwLock<HashMap<u64, Option<u64>>>,
    /** Shared state between instances: include "session_ids" and "handle_ids" */
    state: Box<dyn SharedStateProvider>,
    /** Plugin resolver */
    plugins: JanusPluginProvider,
    /** Local sessions (managed by this instance, corresponding to a websocket connection) store */
    sessions: RwLock<HashMap<u64, JanusSession>>
}

impl JanusProxy {
    pub fn new(server: String, state_provider: Box<dyn SharedStateProvider>, plugin_provider: JanusPluginProvider) -> JanusProxy {
        JanusProxy {
            _janus_server: server,
            state: state_provider,
            plugins: plugin_provider,
            connections: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new())
        }
    }

    pub async fn listen(proxy: JanusProxy, mut listener: TcpListener) {
        let janus = Arc::new(proxy);

        while let Ok((stream, _addr)) = listener.accept().await {
            let ws = accept_ws(stream).await.unwrap();
            let (mut wtx, mut wrx) = ws.split();
            let (mut tx, mut rx) = mpsc::channel::<Message>(32);

            // Assign this websocket a unique connection_id
            let connection_id = loop {
                let id = helper::rand_id();
                let mut connections = janus.connections.write().await;
                if !connections.contains_key(&id) {
                    connections.insert(id, None);
                    break id
                }
            };
            println!("New connection {}", connection_id);

            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    if let Err(e) = wtx.send(message).await {
                        // TODO: more properly error handling
                        match e {
                            Error::ConnectionClosed => println!("Connection closed"),
                            Error::AlreadyClosed => eprintln!("Internal error: connection already closed"),
                            // Error::Io(_) => {}
                            // Error::Tls(_) => {}
                            // Error::Capacity(_) => {}
                            // Error::Protocol(_) => {}
                            // Error::SendQueueFull(_) => {}
                            // Error::Utf8 => {}
                            // Error::Url(_) => {}
                            // Error::Http(_) => {}
                            // Error::HttpFormat(_) => {}
                            _ => continue
                        }
                        break
                    }
                }
            });

            let janus = Arc::clone(&janus);
            tokio::spawn(async move {
                while let Some(item) = wrx.next().await {
                    match item {
                        Ok(message) => {
                            let res = janus.handle_websocket(connection_id, tx.clone(), message).await;
                            if tx.send(res).await.is_err() {
                                break     // channel closed
                            }
                        },
                        Err(e) => eprintln!("Internal error: {}", e)
                    };
                }

                // This clean up session (if present) and any resources associated (owned) with it
                let mut connections = janus.connections.write().await;
                if let Some(Some(session_id)) = connections.get(&connection_id) {
                    janus.destroy_session(session_id).await;
                    connections.remove(&connection_id);
                }
            });
        }
    }

    async fn handle_websocket(&self, connection_id: u64, tx: JanusEventEmitter, item: Message) -> Message {
        if let Message::Text(data) = item {
            self.handle_request(connection_id, tx, data).await.into()
        }
        else {
            item
        }
    }

    async fn handle_request(&self, connection_id: u64, tx: JanusEventEmitter, text: String) -> JanusResponse {
        let request: IncomingRequestParameters = match json::parse(&text) {
            Ok(x) => x,
            Err(e) => return JanusResponse::bad_request(e)
        };

        let IncomingRequestParameters {
            transaction,
            janus: message_text,
            session_id,
            handle_id,
            ..
        } = request;

        // Some trade-off occur here, I don't wanna add lifecycle to JanusResponse.
        // TODO: prevent memory copy as soon as possible: verify `transaction` length.
        let response_transaction = transaction.clone();
        let response_error = |e: JanusError| {
            JanusResponse::new("error", session_id, response_transaction).with_err(e)
        };

        let response = async {
            if session_id == 0 && handle_id == 0 {
                let response = match &message_text[..] {
                    "ping" => JanusResponse::new("pong", 0, transaction),
                    "info" => JanusResponse::new("server_info", 0, transaction).with_data(json!({})), // TODO: response server info
                    "create" => {
                        let id = self.create_session(connection_id).await;
                        let json = json!({ "id": id });
                        JanusResponse::new("success", 0, transaction).with_data(json)
                    }
                    x => return Err(
                        JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request '{}' at this path", x))
                    )
                };
                return Ok(response)
            }

            if session_id == 0 {
                return Err(JanusError::new(JANUS_ERROR_SESSION_NOT_FOUND, format!("Invalid session")))
            }

            if !self.sessions.read().await.contains_key(&session_id) {
                return Err(JanusError::new(JANUS_ERROR_SESSION_NOT_FOUND, format!("No such session \"{}\"", session_id)))
            }

            /* Both session-level and handle-level request */
            if message_text == "keepalive" {
                return Ok(JanusResponse::new("ack", session_id, transaction))
            }
            if message_text == "claim" {    //TODO: implement later
                return Ok(JanusResponse::new("success", session_id, transaction))
            }

            /* Session-level request */
            if handle_id == 0 {
                let response = match &message_text[..] {
                    "attach" => {
                        // TODO: verify `token`, `opaque_id`
                        let params: AttachParameters = json::parse(&text)?;
                        let id = self.state.new_handle();
                        let plugin = self.plugins.resolve(params.plugin)?;
                        let handle = JanusHandle::new(id, session_id, tx, plugin);

                        // TODO: check existence first
                        self.sessions.write().await.get_mut(&session_id).unwrap().handles.insert(id, Arc::new(handle));

                        let json = json!({ "id": id });
                        JanusResponse::new("success", session_id, transaction).with_data(json)
                    },
                    "destroy" => {
                        self.destroy_session(&session_id).await;
                        // TODO: notify event handlers. Btw, what is 'event handler'
                        JanusResponse::new("success", session_id, transaction)
                    },
                    "detach" | "hangup" | "message" | "trickle" => return Err(
                        JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request '{}' at this path", message_text))
                    ),
                    x => return Err(
                        JanusError::new(JANUS_ERROR_UNKNOWN_REQUEST, format!("Unknown request '{}'", x))
                    )
                };
                return Ok(response)
            } else {
                /* Handle-level request */
                // TODO: check session existence first
                if !self.sessions.read().await.get(&session_id).unwrap().handles.contains_key(&handle_id) {
                    return Err(
                        JanusError::new(JANUS_ERROR_HANDLE_NOT_FOUND, format!("No such handle \"{}\" in session \"{}\"", handle_id, session_id))
                    )
                }

                let response = match &message_text[..] {
                    "detach" => {
                        // TODO: clean-up, check session existence first
                        self.state.remove_handle(&handle_id);
                        self.sessions.write().await.get_mut(&session_id).unwrap().handles.remove(&handle_id);
                        JanusResponse::new("success", session_id, transaction)
                    },
                    "message" => {
                        // TODO: check session existence first
                        let handle = self.sessions.read().await.get(&session_id).unwrap().handles.get(&handle_id).unwrap().clone();
                        return Self::handle_plugin_message(transaction, &handle, json::parse(&text)?).await
                    },
                    // TODO: do real hangup.. Should forward to plugin?
                    "hangup" => JanusResponse::new("success", session_id, transaction),
                    // TODO: forward to plugin?
                    // "trickle" => (),
                    "attach" | "destroy" => return Err(
                        JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request '{}' at this path", message_text))
                    ),
                    x => return Err(
                        JanusError::new(JANUS_ERROR_UNKNOWN_REQUEST, format!("Unknown request '{}'", x))
                    )
                };
                return Ok(response)
            }
        };

        response.await.unwrap_or_else(response_error)
    }

    async fn handle_plugin_message(transaction: String, handle: &Arc<JanusHandle>, body_params: BodyParameters) -> Result<JanusResponse, JanusError> {
        // Too many copy - TODO
        let result = handle.plugin.handle_message(JanusPluginMessage::new(
            Arc::clone(handle),
            transaction.clone(),
            serde_json::to_string(&body_params.body).unwrap(),
            body_params.jsep
        ));

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

    async fn create_session(&self, connection_id: u64) -> u64 {
        let id = self.state.new_session();
        let session = JanusSession::new(id);
        self.connections.write().await.insert(connection_id, Some(id));
        self.sessions.write().await.insert(id, session);
        id
    }

    async fn destroy_session(&self, session_id: &u64) {
        // TODO: Clean-up handles, as Arc wrapped
        self.state.remove_session(session_id);
        self.sessions.write().await.remove(session_id);
    }
}
