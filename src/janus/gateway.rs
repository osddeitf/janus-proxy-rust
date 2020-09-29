use futures::{StreamExt, SinkExt};
use tokio::sync::{mpsc, RwLock};
use tokio::sync::oneshot;
use tokio::time::Duration;
use tokio_tungstenite::tungstenite::{Message, Error};
use std::sync::Arc;
use std::collections::HashMap;
use super::core::json;
use super::core::request::IncomingRequestParameters;
use super::core::response::JanusResponse;
use super::connection::new_backend_connection;
use super::error::{JanusError, code::*};

struct JanusGatewayRequest {
    callback: oneshot::Sender<JanusResponse>,
    asynchronous: bool      // may or may not ignore "ack" response
}

pub struct JanusGateway {
    // url: String,     // TODO: should store this?
    queue: mpsc::Sender<Message>,
    requests: RwLock<HashMap<String, JanusGatewayRequest>>
}

impl JanusGateway {
    async fn on_websocket_message(&self, message: Message, event: &mut mpsc::Sender<Message>) {
        if let Message::Text(text) = &message {
            // TODO: handle unwrap - malformed response or struct definition error
            let response = json::parse::<JanusResponse>(text).unwrap();

            // TODO: how to handle "ack"?
            let mut lock = self.requests.write().await;
            if lock.contains_key(&response.transaction) {
                let asynchronous = lock.get(&response.transaction).unwrap().asynchronous;
                if asynchronous && response.janus == "ack" {
                    return
                }

                // Note: unwrap is safe here
                let requester = lock.remove(&response.transaction).unwrap();
                drop(lock);

                if let Err(_) = requester.callback.send(response) {
                    // TODO: do what??
                }
            } else {
                // TODO: should send "ack"?
                drop(lock);

                // TODO: unwrap?
                let message = Message::Text(response.stringify().unwrap());
                if let Err(_) = event.send(message).await {
                    // TODO: ignore or what?
                }
            }
        }
    }

    pub async fn connect(url: String, mut event: mpsc::Sender<Message>) -> Result<Arc<JanusGateway>, JanusError> {
        // TODO: try again with different url
        let ws = match new_backend_connection(&url).await {
            Ok(x) => x,
            // TODO: handle all error types
            Err(_) => return Err(JanusError::new(
                JANUS_ERROR_GATEWAY_CONNECTION_FAILED,
                format!("Could not connect to janus-gateway instance \"{}\"", url)
            ))
        };

        let (mut wtx, mut wrx) = ws.split();
        let (tx, mut rx) = mpsc::channel::<Message>(32);
        let instance = JanusGateway {
            // url,
            queue: tx,
            requests: RwLock::new(HashMap::new())
        };
        let instance = Arc::new(instance);
        let gateway = Arc::downgrade(&Arc::clone(&instance));

        tokio::spawn(async move {
            loop {
                let read_next = wrx.next();
                let queue_next = rx.recv();
                tokio::select! {
                    item = read_next => {
                        match item {
                            None => {
                                break;
                            },
                            Some(item) => {
                                let gateway = match gateway.upgrade() {
                                    None => break,
                                    Some(x) => x
                                };

                                match item {
                                    Ok(message) => gateway.on_websocket_message(message, &mut event).await,
                                    // TODO: handle socket error properly
                                    Err(e) => match e {
                                        Error::ConnectionClosed => {}
                                        Error::AlreadyClosed => {}
                                        Error::Io(_) => {}
                                        // Error::Tls(_) => {}
                                        Error::Capacity(_) => {}
                                        Error::Protocol(_) => {}
                                        Error::SendQueueFull(_) => {}
                                        Error::Utf8 => {}
                                        Error::Url(_) => {}
                                        Error::Http(_) => {}
                                        Error::HttpFormat(_) => {}
                                        // _ => {}
                                    }
                                }

                            }
                        };
                    },
                    item = queue_next => {
                        match item {
                            None => {
                                break
                            },
                            Some(item) => {
                                if let Err(_) = wtx.send(item).await {
                                    // TODO: let ignore for now
                                    break
                                }
                            }
                        }
                    }
                }
            }

            let gateway = match gateway.upgrade() {
                None => return,
                Some(x) => x
            };
            gateway.requests.write().await.clear();
            // NOTE: No need to close websocket manually
        });

        Ok(instance)
    }

    pub async fn send(&self, params: IncomingRequestParameters, is_asynchronous: bool) -> Result<JanusResponse, JanusError> {
        let (tx, rx) = oneshot::channel::<JanusResponse>();
        let request = JanusGatewayRequest {
            callback: tx,
            asynchronous: is_asynchronous
        };

        let json = json::stringify(&params)?;
        let transaction = params.transaction;

        self.requests.write().await.insert(transaction.clone(), request);    // TODO: avoid copy?
        if self.queue.clone().send(Message::Text(json)).await.is_err() {
            return Err(JanusError::new(JANUS_ERROR_GATEWAY_CONNECTION_CLOSED, String::from("connection to janus-gateway closed")))
        }

        match tokio::time::timeout(Duration::from_secs(5), rx).await {
            Ok(x) => match x {
                Ok(x) => Ok(x),     // NOTE: leave `response.error` for caller
                Err(_) => {
                    self.requests.write().await.remove(&transaction);
                    Err(JanusError::new(JANUS_ERROR_GATEWAY_INTERNAL_ERROR, String::from("janus-gateway send() oneshot channel is closed")))
                }
            },
            Err(_) => {
                self.requests.write().await.remove(&transaction);
                Err(JanusError::new(JANUS_ERROR_GATEWAY_TIMED_OUT, String::from("Request to janus-gateway backend timed out")))
            }
        }
    }
}
