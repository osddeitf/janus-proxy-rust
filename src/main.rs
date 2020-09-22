use tokio::net::{TcpListener};
use janus_proxy::janus::state::{HashSetStateProvider};
use janus_proxy::janus::JanusProxy;
use janus_proxy::janus::plugin::JanusPluginProvider;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let socket = TcpListener::bind(addr).await;
    let listener = socket.expect("Failed to bind");

    let server = String::from("ws://localhost:8188");
    let janus = JanusProxy::new(
        server,
        Box::new(HashSetStateProvider::new()),
        JanusPluginProvider::default()
    );

    JanusProxy::listen(janus, listener).await;
}
