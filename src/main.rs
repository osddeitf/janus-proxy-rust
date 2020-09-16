use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use futures::{StreamExt, SinkExt};
use std::io::Bytes;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::handshake::server::{Response, Request, ErrorResponse, create_response};
use std::borrow::BorrowMut;
use janus_proxy::janus;
use tokio_tungstenite::tungstenite::Message;
use http::StatusCode;

struct WithProtocolHeader<'a> {
    protocol: &'a str
}

impl<'a> WithProtocolHeader<'a> {
    pub fn new(protocol: &str) -> WithProtocolHeader {
        WithProtocolHeader { protocol }
    }
}

impl<'a> tungstenite::handshake::server::Callback for WithProtocolHeader<'a> {
    fn on_request(self, request: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
        response.headers_mut()
            .append("Sec-WebSocket-Protocol", self.protocol.parse().unwrap());
        return Ok(response);
    }
}

async fn handle_connection(stream: TcpStream, _addr: SocketAddr) {

    let ws_stream = match tokio_tungstenite::accept_hdr_async(stream, WithProtocolHeader::new("janus-protocol")).await {
        Ok(x) => x,
        Err(err) => {
            println!("Websocket connection failed: {}", err);
            return
        }
    };

    let (mut tx, rx) = ws_stream.split();

    let janus_request = Request::builder()
        .uri("ws://localhost:8188")
        .method("GET")
        .header("Sec-WebSocket-Protocol", "janus-protocol")
        .body(())
        .unwrap();

    let (janus_stream, _) = match tokio_tungstenite::connect_async(janus_request).await {
        Ok(x) => x,
        Err(err) => {
            println!("Cannot connect to Janus server: {}", err);
            return
        }
    };


    let (jtx, jrx) = janus_stream.split();

    let janus = janus::Janus::new();
    futures::future::select(
        // rx.forward(jtx),
        rx.map(|item| {
            match item {
                Ok(message) => match &message {
                    Message::Text(data) => janus.handle_incoming(data),
                    _ => Ok(message)
                }
                x => x
            }
        })
            .forward(jtx),
        jrx.forward(tx)
    ).await;
    println!("Websocket connection closed");
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let socket = TcpListener::bind(addr).await;
    let mut listener = socket.expect("Failed to bind");

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}
