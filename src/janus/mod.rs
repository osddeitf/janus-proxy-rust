mod plugin;
mod error;
mod videoroom;
mod json;
mod request;
mod response;
pub mod state;
mod connection;

/**
* Request types are ported from janus-gateway v0.10.5
*/

// use self::videoroom::VideoRoom;
use self::request::*;
use self::response::*;
use self::plugin::find_plugin;
use self::error::{JanusError, JanusErrorCode::*};
use self::state::SharedStateProvider;
use self::connection::accept_ws;
use futures::{StreamExt, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Message, Error};
use serde_json::json;

pub struct Janus<'a> {
    _janus_server: &'a str,
    // _videoroom: VideoRoom,
    store: Box<dyn SharedStateProvider>
}

impl<'a> Janus<'a> {
    pub fn new(server: &'a str, store: Box<dyn SharedStateProvider>) -> Janus<'a> {
        Janus {
            _janus_server: server,
            // _videoroom: VideoRoom,
            store
        }
    }

    pub async fn accept(&self, stream: TcpStream) {
        let ws = accept_ws(stream).await.unwrap();
        let (mut tx, mut rx) = ws.split();
        // rx
        //     .map(|item| self.handle_websocket(item))
        //     .flat_map(|future| future.into_stream())
        //     .forward(tx)
        //     .await;
        while let Some(item) = rx.next().await {
            let response = self.handle_websocket(item).await;
            tx.send(response.unwrap()).await.unwrap();      //TODO: handle `tx` closed
        }
    }

    async fn handle_websocket(&self, item: Result<Message, Error>) -> Result<Message, Error> {
        if let Ok(Message::Text(data)) = item {
            let message = self.handle_request(data).await;
            Ok(Message::Text(message))
        }
        else {
            item
        }
    }

    async fn handle_request(&self, text: String) -> String {
        let request: IncomingRequestParameters = match json::parse(&text) {
            Ok(x) => x,
            Err(e) => return JanusResponse::bad_request(&e).stringify().ok().unwrap()
        };

        let message_text = &request.janus[..];
        let session_id = request.session_id;
        let handle_id = request.handle_id;

        let response = async {
            if session_id == 0 && handle_id == 0 {
                return match message_text {
                    "ping" => JanusResponse::new("pong", &request).stringify(),
                    "info" => JanusResponse::new_with_data("server_info", &request, json!({})).stringify(), // TODO: response server info
                    "create" => self.create_session(&request).await,
                    x => Err(
                        JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request '{}' at this path", x))
                    )
                }
            }

            if session_id == 0 {
                return Err(JanusError::new(JANUS_ERROR_SESSION_NOT_FOUND, format!("Invalid session")))
            }

            let session = self.store.find_session(&session_id);
            if !session {
                return Err(JanusError::new(JANUS_ERROR_SESSION_NOT_FOUND, format!("No such session \"{}\"", session_id)))
            }

            /* Both session-level and handle-level request */
            if message_text == "keepalive" {
                return JanusResponse::new("ack", &request).stringify()
            }
            if message_text == "claim" {    //TODO: implement later
                return JanusResponse::new("success", &request).stringify()
            }

            /* Session-level request */
            if handle_id == 0 {
                match message_text {
                    "attach" => self.create_handle(&request, &json::parse(&text)?).await,
                    "destroy" => self.destroy_session(&request).await,
                    "detach" | "hangup" | "message" | "trickle" => Err(
                        JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request '{}' at this path", message_text))
                    ),
                    x => Err(JanusError::new(JANUS_ERROR_UNKNOWN_REQUEST, format!("Unknown request '{}'", x)))
                }
            }
            else {
                /* Handle-level request */
                let handle = self.store.find_handle(&handle_id);
                if !handle {
                    return Err(JanusError::new(JANUS_ERROR_HANDLE_NOT_FOUND, format!("No such handle \"{}\" in session \"{}\"", handle_id, session_id)))
                }

                match message_text {
                    // "detach" => (),
                    // "hangup" => (),
                    // "message" => (),
                    // "trickle" => (),
                    "attach" | "destroy" => Err(
                        JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request '{}' at this path", message_text))
                    ),
                    x => Err(JanusError::new(JANUS_ERROR_UNKNOWN_REQUEST, format!("Unknown request '{}'", x)))
                }
            }
        };

        // TODO: handle some unexpected error due to `unwrap()`
        response.await.unwrap_or_else(|e| {
            JanusResponse::new_with_error(&request, &e).stringify().ok().unwrap()
        })
    }

    async fn create_session(&self, request: &IncomingRequestParameters) -> Result<String, JanusError> {
        // TODO: `apisecret`, `token` authentication?

        let id = self.store.new_session_id();
        let data = json!({ "id": id });
        JanusResponse::new_with_data("success", request, data).stringify()
    }

    async fn destroy_session(&self, request: &IncomingRequestParameters) -> Result<String, JanusError> {
        //TODO: Clean-up, should close websocket connection?
        self.store.destroy_session(&request.session_id);
        //TODO: notify event handlers. Btw, what is 'event handler'
        JanusResponse::new("success", &request).stringify()
    }

    async fn create_handle(&self, request: &IncomingRequestParameters, attach_params: &AttachParameters) -> Result<String, JanusError>{
        // TODO: verify `token`
        let mut plugin = find_plugin(&attach_params.plugin)?;
        if let Some(opaque_id) = &attach_params.opaque_id {
            plugin.set_opaque_id(opaque_id);
        }

        // TODO: Initalize plugin (ice?,...)

        let id = self.store.new_handle_id();
        JanusResponse::new_with_data("success", &request, json!({ "id": id })).stringify()
    }
}
