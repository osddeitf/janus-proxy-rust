use std::sync::Arc;
use tokio::net::{TcpListener};
use janus_proxy::janus::JanusProxy;
use janus_proxy::janus::plugin::JanusPluginProvider;
use janus_proxy::janus::provider::{MemoryStateProvider, MemoryBackendProvider, JanusBackendProvider};

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let socket = TcpListener::bind(addr).await;
    let listener = socket.expect("Failed to bind");

    let server = String::from("ws://localhost:8188");
    let backend = MemoryBackendProvider::new();
    backend.update_backend(server, true);

    let janus = JanusProxy::new(
        Box::new(MemoryStateProvider::new()),
        JanusPluginProvider::default(),
        Arc::new(Box::new(backend))
    );

    // TODO: enable http server for managing janus-gateway instances, token...

    // TODO: Check whether .await yield task back to scheduler
    JanusProxy::listen(janus, listener).await;
}
