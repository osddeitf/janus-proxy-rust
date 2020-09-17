use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{Response, Request, ErrorResponse, Callback};
use janus_proxy::janus;

struct WithProtocolHeader<'a> {
    protocol: &'a str
}

impl<'a> WithProtocolHeader<'a> {
    pub fn new(protocol: &str) -> WithProtocolHeader {
        WithProtocolHeader { protocol }
    }
}

impl<'a> Callback for WithProtocolHeader<'a> {
    fn on_request(self, _request: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
        response.headers_mut()
            .append("Sec-WebSocket-Protocol", self.protocol.parse().unwrap());
        return Ok(response);
    }
}

async fn handle_connection(stream: TcpStream, _addr: SocketAddr) {

    let ws_stream = match accept_hdr_async(stream, WithProtocolHeader::new("janus-protocol")).await {
        Ok(x) => x,
        Err(err) => {
            println!("Websocket connection failed: {}", err);
            return
        }
    };

    let server = "ws://localhost:8188"
    let janus = janus::Janus::new(server);
    janus.handle(ws_stream).await;
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
