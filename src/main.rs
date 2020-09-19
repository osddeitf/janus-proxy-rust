use tokio::net::{TcpListener};
use janus_proxy::janus::state::{HashSetStateProvider};
use janus_proxy::janus::Janus;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let socket = TcpListener::bind(addr).await;
    let mut listener = socket.expect("Failed to bind");

    let server = "ws://localhost:8188"
    let janus = Janus::new(server, Box::new(HashSetStateProvider::new()));
    let janus = Arc::new(janus);

    while let Ok((stream, _addr)) = listener.accept().await {
        let janus = Arc::clone(&janus);
        tokio::spawn(async move {
            janus.accept(stream).await;
        });
    }
}
