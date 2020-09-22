use super::helper;
use super::json::*;
use std::collections::HashSet;
use std::sync::Mutex;

type ID = JSON_POSITIVE_INTEGER;

pub trait SharedStateProvider: Send + Sync {
    fn new_session(&self) -> ID;
    fn new_handle(&self) -> ID;
    // TODO: return handle/session object?
    fn has_session(&self, id: &ID) -> bool;
    fn has_handle(&self, id: &ID) -> bool;

    fn remove_session(&self, id: &ID) -> bool;
    fn remove_handle(&self, id: &ID) -> bool;
}

pub struct HashSetStateProvider {
    sessions: Mutex<HashSet<ID>>,
    // Must be unique within a session, using global unique for simplicity
    handles: Mutex<HashSet<ID>>
}

impl HashSetStateProvider {
    pub fn new() -> HashSetStateProvider {
        HashSetStateProvider {
            sessions: Mutex::new(HashSet::new()),
            handles: Mutex::new(HashSet::new())
        }
    }
}

impl SharedStateProvider for HashSetStateProvider {
    fn new_session(&self) -> ID {
        loop {
            let id = helper::rand_id();
            let mut sessions = self.sessions.lock().unwrap();
            if sessions.insert(id) {
                return id
            }
        }
    }

    fn new_handle(&self) -> ID {
        loop {
            let id = helper::rand_id();
            let mut handles = self.handles.lock().unwrap();
            if !handles.insert(id) {
                return id
            }
        }
    }

    fn has_session(&self, id: &ID) -> bool {
        self.sessions.lock().unwrap().contains(id)
    }

    fn has_handle(&self, id: &ID) -> bool {
        self.handles.lock().unwrap().contains(id)
    }

    fn remove_session(&self, id: &ID) -> bool {
        self.sessions.lock().unwrap().remove(id)
    }

    fn remove_handle(&self, id: &ID) -> bool {
        self.handles.lock().unwrap().remove(id)
    }
}

/* TODO: Implement redis for scale */
pub struct _RedisStateProvider;
