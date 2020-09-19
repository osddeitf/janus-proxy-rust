use super::json::*;
use std::collections::HashSet;
use rand::prelude::*;
use std::sync::Mutex;

type ID = JSON_POSITIVE_INTEGER;

pub trait SharedStateProvider: Send + Sync {
    fn new_session_id(&self) -> ID;
    fn new_handle_id(&self) -> ID;
    // TODO: return handle/session object?
    fn find_session(&self, id: &ID) -> bool;
    fn find_handle(&self, id: &ID) -> bool;
}

pub struct HashSetStateProvider {
    sessions: Mutex<HashSet<ID>>,
    // Must be unique within a session, using global index for simplicity
    handles: Mutex<HashSet<ID>>
}

impl HashSetStateProvider {
    pub fn new() -> HashSetStateProvider {
        HashSetStateProvider {
            sessions: Mutex::new(HashSet::new()),
            handles: Mutex::new(HashSet::new())
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
    fn new_session_id(&self) -> ID {
        loop {
            let id = self.rand();
            let mut sessions = self.sessions.lock().unwrap();
            if sessions.insert(id) {
                return id
            }
        }
    }

    fn new_handle_id(&self) -> ID {
        loop {
            let id = self.rand();
            let mut handles = self.handles.lock().unwrap();
            if handles.insert(id) {
                return id
            }
        }
    }

    fn find_session(&self, id: &ID) -> bool {
        self.sessions.lock().unwrap().contains(id)
    }

    fn find_handle(&self, id: &ID) -> bool {
        self.handles.lock().unwrap().contains(id)
    }
}

/* TODO: Implement redis for scale */
pub struct _RedisStateProvider;
