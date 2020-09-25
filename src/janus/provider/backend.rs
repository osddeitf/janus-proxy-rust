use std::collections::BTreeSet;
use std::sync::Mutex;

// Status: preview. TODO: refine the apis
pub trait JanusBackendProvider: Send + Sync {
    fn update_backend(&self, url: String, up: bool);
    fn get_backend(&self) -> Option<String>;
}

pub struct MemoryBackendProvider {
    // TODO: set id for server replacement, use HashMap?
    alive: Mutex<BTreeSet<String>>
}

impl MemoryBackendProvider {
    pub fn new() -> MemoryBackendProvider {
        MemoryBackendProvider {
            alive: Mutex::new(BTreeSet::new())
        }
    }
}

impl JanusBackendProvider for MemoryBackendProvider {
    fn update_backend(&self, url: String, up: bool) {
        if up {
            self.alive.lock().unwrap().insert(url);
        }
        else {
            self.alive.lock().unwrap().remove(&url);
        }
    }

    fn get_backend(&self) -> Option<String> {
        // TODO: round-robin,... fashion
        let x = self.alive.lock().unwrap().iter().next()?.clone();
        Some(x)
    }
}

// TODO: Redis version
struct _RedisBackendProvider;
