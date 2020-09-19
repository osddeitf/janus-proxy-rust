mod plugin;
mod error;
mod videoroom;
mod json;
mod request;
mod response;
pub mod state;
mod connection;

use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite;
use http::{Request};

/**
* Request types are ported from janus-gateway v0.10.5
*/

use self::videoroom::VideoRoom;
use self::request::*;
use self::response::*;
use self::error::{JanusError, JanusErrorCode::*};
use self::state::SharedStateProvider;
use self::connection::accept_ws;
use tokio::net::TcpStream;
use futures::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::{Message, Error};
use serde_json::json;

pub struct Janus<'a> {
    janus_server: &'a str,
    videoroom: VideoRoom,
    store: Box<dyn SharedStateProvider>
}

impl<'a> Janus<'a> {
    pub fn new(server: &'a str, store: Box<dyn SharedStateProvider>) -> Janus<'a> {
        Janus {
            janus_server: server,
            videoroom: VideoRoom,
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

    async fn handle_websocket(&self, item: Result<tungstenite::Message, tungstenite::Error>) -> Result<Message, Error> {
        if let Ok(tungstenite::Message::Text(data)) = item {
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

        let id: u64 = self.store.new_session_id();
        let data = json!({ "id": id });
        JanusResponse::new_with_data("success", request, data).stringify()
    }

    async fn new_janus_connection(&self) -> Result<WebSocketStream<TcpStream>, tungstenite::Error> {
        let janus_request = Request::builder()
            .uri(self.janus_server)
            .method("GET")
            .header("Sec-WebSocket-Protocol", "janus-protocol")
            .body(())
            .unwrap();

        let (janus_stream, _) = tokio_tungstenite::connect_async(janus_request).await?;
        return Ok(janus_stream);
    }
}
