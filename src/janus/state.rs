use super::json::*;
use std::collections::{HashSet, HashMap};
use rand::prelude::*;
use std::sync::Mutex;

type ID = JSON_POSITIVE_INTEGER;

pub trait SharedStateProvider: Send + Sync {
    fn new_session(&self) -> ID;
    fn new_handle(&self, plugin_name: String) -> ID;
    // TODO: return handle/session object?
    fn has_session(&self, id: &ID) -> bool;
    fn has_handle(&self, id: &ID) -> bool;

    fn get_handle(&self, id: &ID) -> Option<String>;

    fn destroy_session(&self, id: &ID) -> bool;
    fn destroy_handle(&self, id: &ID) -> bool;
}

pub struct HashSetStateProvider {
    sessions: Mutex<HashSet<ID>>,
    // Must be unique within a session, using global index for simplicity
    handles: Mutex<HashMap<ID, String>>
}

impl HashSetStateProvider {
    pub fn new() -> HashSetStateProvider {
        HashSetStateProvider {
            sessions: Mutex::new(HashSet::new()),
            handles: Mutex::new(HashMap::new())
        }
    }

    // TODO: u64 not fit Javascript Number (i53)
    fn rand(&self) -> ID {
        let mut rng = thread_rng();     //TODO: this may affect performances?
        loop {
            let n: ID = rng.next_u32() as ID;
            if n != 0 {
                return n;
            }
        }
    }
}

impl SharedStateProvider for HashSetStateProvider {
    fn new_session(&self) -> ID {
        loop {
            let id = self.rand();
            let mut sessions = self.sessions.lock().unwrap();
            if sessions.insert(id) {
                return id
            }
        }
    }

    fn new_handle(&self, plugin_name: String) -> ID {
        loop {
            let id = self.rand();
            let mut handles = self.handles.lock().unwrap();
            if !handles.contains_key(&id) {
                handles.insert(id, plugin_name);
                return id
            }
        }
    }

    fn has_session(&self, id: &ID) -> bool {
        self.sessions.lock().unwrap().contains(id)
    }

    fn has_handle(&self, id: &ID) -> bool {
        self.handles.lock().unwrap().contains_key(id)
    }

    fn get_handle(&self, id: &ID) -> Option<String> {
        self.handles.lock().unwrap().get(id).map(|s| s.clone())
    }

    fn destroy_session(&self, id: &ID) -> bool {
        self.sessions.lock().unwrap().remove(id)
    }

    fn destroy_handle(&self, id: &ID) -> bool {
        self.handles.lock().unwrap().remove(id).is_none()
    }
}

/* TODO: Implement redis for scale */
pub struct _RedisStateProvider;
