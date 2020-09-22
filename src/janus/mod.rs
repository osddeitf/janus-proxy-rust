mod plugin;
mod error;
mod videoroom;
mod json;
mod request;
mod response;
pub mod state;
mod core;
mod connection;

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
use self::plugin::{find_plugin, JanusPluginResultType::*};
use self::error::{JanusError, code::*};
use self::state::SharedStateProvider;
use self::connection::accept_ws;

pub struct JanusProxy {
    _janus_server: String,
    state: Box<dyn SharedStateProvider>,
    sessions: RwLock<HashMap<u64, JanusSession>>     // TODO: switch to tokio::sync::Mutex?
}

impl JanusProxy {
    pub fn new(server: String, state_provider: Box<dyn SharedStateProvider>) -> JanusProxy {
        JanusProxy {
            _janus_server: server,
            state: state_provider,
            sessions: RwLock::new(HashMap::new())
        }
    }

    pub async fn listen(proxy: JanusProxy, mut listener: TcpListener) {
        let janus = Arc::new(proxy);

        while let Ok((stream, _addr)) = listener.accept().await {
            let ws = accept_ws(stream).await.unwrap();
            let (mut wtx, mut wrx) = ws.split();
            let (mut tx, mut rx) = mpsc::channel::<Message>(32);

            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    if let Err(e) = wtx.send(message).await {
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
                            let res = janus.handle_websocket(tx.clone(), message).await;
                            match tx.send(res).await {
                                Ok(_) => (),
                                Err(_) => break     // channel closed
                            }
                        },
                        Err(e) => eprintln!("Internal error: {}", e)
                    };
                }
            });
        }
    }

    async fn handle_websocket(&self, tx: JanusEventEmitter, item: Message) -> Message {
        if let Message::Text(data) = item {
            let response  = self.handle_request(tx, data).await;
            let text = response.stringify().ok().unwrap();
            Message::Text(text)
        }
        else {
            item
        }
    }

    async fn handle_request(&self, tx: JanusEventEmitter, text: String) -> JanusResponse {
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
            JanusResponse::new("error", session_id, response_transaction).err(e)
        };

        let response = async {
            if session_id == 0 && handle_id == 0 {
                let response = match &message_text[..] {
                    "ping" => JanusResponse::new("pong", 0, transaction),
                    "info" => JanusResponse::new("server_info", 0, transaction).data(json!({})), // TODO: response server info
                    "create" => {
                        let id = self.state.new_session();
                        let session = JanusSession::new(id);
                        {
                            self.sessions.write().await.insert(session.session_id, session);
                        }

                        let json = json!({ "id": id });
                        JanusResponse::new("success", 0, transaction).data(json)
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
                        let plugin = find_plugin(&params.plugin)?;
                        let handle = JanusHandle::new(id, session_id, tx, plugin);

                        // TODO: check existence first
                        self.sessions.write().await.get_mut(&session_id).unwrap().handles.insert(id, Arc::new(handle));

                        let json = json!({ "id": id });
                        JanusResponse::new("success", session_id, transaction).data(json)
                    },
                    "destroy" => {
                        // TODO: Clean-up, should close websocket connection?
                        self.state.remove_session(&session_id);
                        self.sessions.write().await.remove(&session_id);
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
        // if name.is_none() {
        //     return Err(JanusError::new(JANUS_ERROR_PLUGIN_MESSAGE, format!("No plugin to handle this message")))
        // }

        // TODO: handle jsep
        let result = handle.plugin.handle_message(serde_json::to_string(&body_params.body).unwrap())?;

        let response = match result.kind {
            // TODO: handle optional content
            JANUS_PLUGIN_OK => JanusResponse::new_result("success", transaction, handle, result.content.unwrap()),
            // TODO: add `hint`
            JANUS_PLUGIN_OK_WAIT => JanusResponse::new("ack", handle.session_id, transaction),
            JANUS_PLUGIN_ERROR => {
                let text = result.text.unwrap_or("Plugin returned a severe (unknown) error".to_string());
                return Err(JanusError::new(JANUS_ERROR_PLUGIN_MESSAGE, text))
            }
        };
        Ok(response)
    }
}
