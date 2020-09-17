mod plugin;
mod error;
mod videoroom;
mod json;
mod request;
mod response;

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
use tokio::net::TcpStream;
use futures::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::{Message, Error};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct Janus<'a> {
    janus_server: &'a str,
    videoroom: VideoRoom
}

impl<'a> Janus<'a> {
    pub fn new(server: &str) -> Janus {
        Janus {
            janus_server: server,
            videoroom: VideoRoom,
        }
    }

    pub async fn handle(&self, ws: WebSocketStream<TcpStream>) {
        let (mut tx, mut rx) = ws.split();
        // rx
        //     .map(|item| self.handle_websocket(item))
        //     .flat_map(|future| future.into_stream())
        //     .forward(tx)
        //     .await;
        while let Some(item) = rx.next().await {
            let response = self.handle_websocket(item).await;
            tx.send(response.unwrap()).await.unwrap();
        }
    }

    async fn handle_websocket(&self, item: Result<tungstenite::Message, tungstenite::Error>) -> Result<Message, Error> {
        if let Ok(tungstenite::Message::Text(data)) = item {
            let message = match self.handle_request(data).await {
                Ok(response) => response,
                Err(janus_error) => json::stringify(&janus_error).ok().unwrap(),
            };
            Ok(Message::Text(message))
        }
        else {
            item
        }
    }

    async fn handle_request(&self, text: String) -> Result<String, JanusError> {
        let request: IncomingRequestParameters = json::parse(&text)?;
        let session_id = json::parse::<WebsocketSessionId>(&text).map(|x| x.session_id)?;
        let handle_id = json::parse::<WebsocketHandleId>(&text).map(|x| x.handle_id)?;

        if session_id == 0 && handle_id == 0 {
            return match &request.janus[..] {
                "ping" => Ok("".to_string()),
                "info" => Ok("".to_string()),
                "create" => self.create_session(&request).await,
                x => Err(
                    JanusError::new(JANUS_ERROR_INVALID_REQUEST_PATH, format!("Unhandled request {}", x))
                )
            }
        }

        println!("{} {}", handle_id, session_id);

        json::stringify(&request)
    }

    async fn create_session(&self, request: &IncomingRequestParameters) -> Result<String, JanusError> {
        // TODO: `apisecret`, `token` authentication?

        let id: u64 = 19213907;     // mock
        let data = json!({ "id": id });
        let response = JanusResponse::new_response_with_data("success", request, data);
        json::stringify(&response)
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
