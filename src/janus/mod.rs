mod core;
mod connection;
mod helper;
mod gateway;
pub mod plugin;
pub mod provider;

/**
* Request types are ported from janus-gateway v0.10.5
*/
use futures::{StreamExt, SinkExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::{Message, Error};
use serde_json::json;
use std::sync::Arc;
use self::core::*;
use self::core::apierror::*;
use self::request::*;
use self::response::*;
use self::provider::{ProxyStateProvider, JanusBackendProvider};
use self::connection::accept_ws;
use self::plugin::{JanusPluginProvider, JanusPluginResultType::*, JanusPluginMessage};

// TODO: add gracefully shutdown
pub struct JanusProxy {
    // /** Local mapping from connection_id -> session_id. TODO: Is there any better way? */
    // connections: RwLock<HashMap<u64, Option<u64>>>,
    // /** Local sessions (managed by this instance, corresponding to a websocket connection) store */
    // sessions: RwLock<HashMap<u64, JanusSession>>,
    /** Shared state between proxy instances: include "session_ids" and "handle_ids" */
    state: Box<dyn ProxyStateProvider>,
    /** Stored backend, like `state` above */
    backend: Arc<Box<dyn JanusBackendProvider>>,
    /** Plugin resolver */
    plugins: JanusPluginProvider
}

impl JanusProxy {
    pub fn new(
        state_provider: Box<dyn ProxyStateProvider>,
        plugin_provider: JanusPluginProvider,
        backend_provider: Arc<Box<dyn JanusBackendProvider>>
    ) -> JanusProxy {
        JanusProxy {
            // connections: RwLock::new(HashMap::new()),
            // sessions: RwLock::new(HashMap::new()),
            state: state_provider,
            backend: backend_provider,
            plugins: plugin_provider
        }
    }

    pub async fn listen(proxy: JanusProxy, mut listener: TcpListener) {
        let janus = Arc::new(proxy);

        while let Ok((stream, _addr)) = listener.accept().await {
            let ws = accept_ws(stream).await.unwrap();
            let (mut wtx, mut wrx) = ws.split();
            let (mut tx, mut rx) = mpsc::channel::<Message>(32);

            println!("New connection");
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

            // Each connection correspond to a session, which may or may not have an id.
            // Each process websocket message in synchronous fashion, response before next request.
            // And, may emit event back to websocket connection.
            let janus = Arc::clone(&janus);
            tokio::spawn(async move {
                // Assign session id beforehand
                let id = janus.state.new_session();
                let session = JanusSession::new(Arc::clone(&janus), id, tx.clone());
                let session = Arc::new(session);  // for WeakRef from handle

                while let Some(item) = wrx.next().await {
                    match item {
                        Ok(message) => {
                            let res = janus.handle_websocket(&session, message).await;
                            if tx.send(res).await.is_err() {
                                break     // channel closed
                            }
                        },
                        Err(e) => eprintln!("Internal error: {}", e)
                    };
                }

                // This clean up session (if present) and any resources associated (owned) with it
                janus.state.remove_session(&id);
            });
        }
    }

    async fn handle_websocket(&self, session: &Arc<JanusSession>, item: Message) -> Message {
        if let Message::Text(data) = item {
            self.handle_request(session, data).await.into()
        }
        else {
            item
        }
    }

    async fn handle_request(&self, session: &Arc<JanusSession>, text: String) -> JanusResponse {
        let request: IncomingRequestParameters = match json::parse(&text) {
            Ok(x) => x,
            Err(e) => return JanusResponse::bad_request(e)
        };

        let IncomingRequestParameters {
            transaction,
            janus: message_text,
            session_id,
            handle_id,
            body, jsep,
            rest,
            ..
        } = request;

        // Some trade-off occur here, I don't wanna add lifecycle to JanusResponse.
        // TODO: prevent memory copy as soon as possible: verify `transaction` length.
        let response_transaction = transaction.clone();
        let response_error = |e: JanusError| {
            JanusResponse::new("error", session_id, response_transaction).with_err(e)
        };

        let response = async {
            if !*session.initialized.read().await {
                let response = match &message_text[..] {
                    "ping" => JanusResponse::new("pong", 0, transaction),
                    "info" => JanusResponse::new("server_info", 0, transaction).with_data(json!({})), // TODO: response server info
                    "create" => {
                        *session.initialized.write().await = true;
                        let json = json!({ "id": session.id });
                        JanusResponse::new("success", 0, transaction).with_data(json)
                    }
                    x => return Err(
                        JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request '{}' at this path", x))
                    )
                };
                return Ok(response)
            }

            // TODO: Currently not support multiple session per websocket connection, reasonable?
            if session.id != session_id {
                // return Err(JanusError::new(JANUS_ERROR_SESSION_NOT_FOUND, format!("Invalid session")))
                return Err(JanusError::new(JANUS_ERROR_TRANSPORT_SPECIFIC, format!("Invalid session, support only one per websocket connection")))
            }

            /* Both session-level and handle-level request */
            if message_text == "keepalive" {
                return Ok(JanusResponse::new("ack", session_id, transaction))
            }
            if message_text == "claim" {    //TODO: implement later
                return Ok(JanusResponse::new("success", session_id, transaction))
            }

            /* Session-level request */
            return if handle_id == 0 {
                let response = match &message_text[..] {
                    "attach" => {
                        // TODO: verify `token`, `opaque_id`
                        let params: AttachParameters = json::from_object(rest)?;
                        let id = self.state.new_handle();
                        let plugin = self.plugins.resolve(params.plugin)?;

                        let session_ref = Arc::clone(&session);
                        let handle = JanusHandle::new(id, session_ref, plugin);

                        session.handles.write().await.insert(id, handle);

                        let json = json!({ "id": id });
                        JanusResponse::new("success", session_id, transaction).with_data(json)
                    },
                    "destroy" => {
                        // TODO: should reset session id?
                        *session.initialized.write().await = false;
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
                Ok(response)
            } else {
                /* Handle-level request */
                if !session.handles.read().await.contains_key(&handle_id) {
                    return Err(
                        JanusError::new(JANUS_ERROR_HANDLE_NOT_FOUND, format!("No such handle \"{}\" in session \"{}\"", handle_id, session_id))
                    )
                }

                let response = match &message_text[..] {
                    "detach" => {
                        // TODO: clean-up?
                        self.state.remove_handle(&handle_id);
                        session.handles.write().await.remove(&handle_id);
                        JanusResponse::new("success", session_id, transaction)
                    },
                    "message" => {
                        let body = match body {
                            Some(x) => x,
                            None => return Err(JanusError::new(JANUS_ERROR_MISSING_MANDATORY_ELEMENT, "missing 'body'".to_string()))
                        };

                        let handle = Arc::clone(session.handles.read().await.get(&handle_id).unwrap());
                        let result = handle.plugin.handle_message(JanusPluginMessage::new(
                            Arc::clone(&handle),
                            transaction.clone(),        // TODO: Don't copy
                            body,
                            jsep
                        )).await;

                        let response = match result.kind {
                            // TODO: handle optional content
                            JANUS_PLUGIN_OK => JanusResponse::new("success", session.id, transaction)
                                .with_plugindata(&handle, result.content.unwrap(), result.jsep),
                            // TODO: add `hint`
                            JANUS_PLUGIN_OK_WAIT => JanusResponse::new("ack", session.id, transaction),
                            JANUS_PLUGIN_ERROR => {
                                let text = result.text.unwrap_or("Plugin returned a severe (unknown) error".to_string());
                                return Err(JanusError::new(JANUS_ERROR_PLUGIN_MESSAGE, text))
                            }
                        };
                        return Ok(response)
                    },
                    // TODO: do real hangup.. Should forward to plugin?
                    "hangup" => JanusResponse::new("success", session_id, transaction),
                    // TODO: forward to plugin?
                    "trickle" => {
                        let params: TrickleParameters = json::from_object(rest)?;
                        if params.candidate.is_some() && params.candidates.is_some() {
                            return Err(JanusError::new(JANUS_ERROR_MISSING_MANDATORY_ELEMENT, "Missing mandatory element (candidate|candidates)".to_string()))
                        }

                        if let Some(candidate) = params.candidate {
                            candidate.validate()?;
                            let handle = Arc::clone(session.handles.read().await.get(&handle_id).unwrap());
                            handle.trickle(candidate).await?;
                        }
                        else if let Some(candidates) = params.candidates {
                            let err = candidates.iter().find_map(|x| x.validate().err());
                            if let Some(e) = err {
                                return Err(e)
                            }

                            let handle = Arc::clone(session.handles.read().await.get(&handle_id).unwrap());
                            for x in candidates.into_iter() {
                                handle.trickle(x).await?
                            }
                        }
                        else {
                            return Err(JanusError::new(JANUS_ERROR_INVALID_JSON, "Can't have both candidate and candidates".to_string()))
                        }

                        JanusResponse::new("ack", session.id, transaction)
                    },
                    "attach" | "destroy" => return Err(
                        JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request '{}' at this path", message_text))
                    ),
                    x => return Err(
                        JanusError::new(JANUS_ERROR_UNKNOWN_REQUEST, format!("Unknown request '{}'", x))
                    )
                };
                Ok(response)
            }
        };

        response.await.unwrap_or_else(response_error)
    }
}
