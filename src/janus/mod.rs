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

use self::videoroom::VideoRoom;
use self::request::*;
use self::response::*;
use self::error::{JanusError, JanusErrorCode::*};
use self::state::SharedStateProvider;
use self::connection::accept_ws;
use futures::{StreamExt, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Message, Error};
use serde_json::json;

pub struct Janus<'a> {
    _janus_server: &'a str,
    _videoroom: VideoRoom,
    store: Box<dyn SharedStateProvider>
}

impl<'a> Janus<'a> {
    pub fn new(server: &'a str, store: Box<dyn SharedStateProvider>) -> Janus<'a> {
        Janus {
            _janus_server: server,
            _videoroom: VideoRoom,
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

        let response = if session_id == 0 && handle_id == 0 {
            match message_text {
                "ping" => JanusResponse::new("pong", &request).stringify(),
                "info" => JanusResponse::new_with_data("server_info", &request, json!({})).stringify(), // TODO: response server info
                "create" => self.create_session(&request).await,
                x => Err(
                    JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request '{}' at this path", x))
                )
            }
        }
        else {
            match message_text {
                "keepalive" => JanusResponse::new("ack", &request).stringify(),
                // "attach" => (),
                // "destroy" => (),
                // "detach" => (),
                // "hangup" => (),
                // "claim" => (),
                // "message" => (),
                // "trickle" => (),
                x => Err(
                    JanusError::new(JANUS_ERROR_UNKNOWN_REQUEST, format!("Unknown quest '{}'", x))
                )
            }
        };

        // TODO: handle some unexpected error due to `unwrap()`
        match response {
            Err(e) => JanusResponse::new_with_error(&request, &e).stringify().ok().unwrap(),
            Ok(x) => x
        }
    }

    async fn create_session(&self, request: &IncomingRequestParameters) -> Result<String, JanusError> {
        // TODO: `apisecret`, `token` authentication?

        let id = self.store.new_session_id();
        let data = json!({ "id": id });
        JanusResponse::new_with_data("success", request, data).stringify()
    }
}
