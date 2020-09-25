use tokio::net::{TcpListener};
use janus_proxy::janus::provider::{MemoryStateProvider, JanusPluginProvider};
use janus_proxy::janus::JanusProxy;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let socket = TcpListener::bind(addr).await;
    let listener = socket.expect("Failed to bind");

    let server = String::from("ws://localhost:8188");
    let janus = JanusProxy::new(
        Box::new(MemoryStateProvider::new()),
        JanusPluginProvider::default()
    );

    JanusProxy::listen(janus, listener).await;
}
