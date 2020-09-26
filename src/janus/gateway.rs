use futures::{StreamExt, SinkExt};
use tokio::sync::{mpsc, RwLock};
use tokio::sync::oneshot;
use tokio::time::Duration;
use tokio_tungstenite::tungstenite::{Message, Error};
use std::sync::Arc;
use std::collections::HashMap;
use super::core::json;
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
    pub async fn connect(url: String, mut event: mpsc::Sender<JanusResponse>) -> Result<Arc<JanusGateway>, JanusError> {
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
        let gateway = Arc::clone(&instance);

        tokio::spawn(async move {
            while let Some(item) = wrx.next().await {
                match item {
                    Ok(message) => if let Message::Text(text) = &message {
                        // TODO: handle unwrap - malformed response or struct definition error
                        let response = json::parse::<JanusResponse>(text).unwrap();

                        // TODO: how to handle "ack"?
                        let mut lock = gateway.requests.write().await;
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
                            drop(lock);
                            if let Err(_) = event.send(response).await {
                                // TODO: ignore or what?
                            }
                        }
                    },
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
                };
            }
        });

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(item) = rx.recv() => {
                        if let Err(_) = wtx.send(item).await {
                            // TODO: let ignore for now
                        }
                    },
                    _ = tokio::time::delay_for(Duration::from_secs(25)) => {
                        // wtx.send()
                        // TODO: send "keepalive" request, filter "ack" from response
                    }
                }
            }
        });

        Ok(instance)
    }

    pub async fn send(&self, transaction: String, json: String, is_asynchronous: bool) -> Result<JanusResponse, JanusError>{
        let (tx, rx) = oneshot::channel::<JanusResponse>();
        let request = JanusGatewayRequest {
            callback: tx,
            asynchronous: is_asynchronous
        };

        self.requests.write().await.insert(transaction.clone(), request);    // TODO: avoid copy?
        if let Err(_) = self.queue.clone().send(Message::Text(json)).await {
            // TODO: handle send too fast? Ignore channel closed by now
        }

        match tokio::time::timeout(Duration::from_secs(5), rx).await {
            Ok(x) => match x {
                Ok(x) => match x.error {
                    None => Ok(x),
                    Some(_) => Err(JanusError::new(JANUS_ERROR_GATEWAY_INTERNAL_ERROR, String::from("Request to janus-gateway got an error")))
                },
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
