use tokio::net::{TcpStream};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::handshake::server::{Response, Request, ErrorResponse, Callback};
use tokio_tungstenite::tungstenite::Error;

struct WithProtocolHeader;

impl<'a> Callback for WithProtocolHeader {
    fn on_request(self, _request: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
        response.headers_mut()
            .append("Sec-WebSocket-Protocol", "janus-protocol".parse().unwrap());
        return Ok(response);
    }
}

pub(crate) async fn accept_ws<'a>(stream: TcpStream) -> Result<WebSocketStream<TcpStream>, Error> {
    tokio_tungstenite::accept_hdr_async(stream, WithProtocolHeader).await
}

pub(crate) async fn new_backend_connection(janus_server: &str) -> Result<WebSocketStream<TcpStream>, Error> {
    let janus_request = Request::builder()
        .uri(janus_server)
        .method("GET")
        .header("Sec-WebSocket-Protocol", "janus-protocol")
        .body(())
        .unwrap();

    let (janus_stream, _) = tokio_tungstenite::connect_async(janus_request).await?;
    return Ok(janus_stream);
}
