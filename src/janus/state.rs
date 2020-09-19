use super::json::*;
use std::collections::HashSet;
use rand::prelude::*;
use std::sync::Mutex;

pub trait SharedStateProvider: Send + Sync {
    fn new_session_id(&self) -> JSON_POSITIVE_INTEGER;
    fn new_handle_id(&self) -> JSON_POSITIVE_INTEGER;
}

pub struct HashSetStateProvider {
    sessions: Mutex<HashSet<JSON_POSITIVE_INTEGER>>,
    // Must be unique within a session, using global index for simplicity
    handles: Mutex<HashSet<JSON_POSITIVE_INTEGER>>
}

impl HashSetStateProvider {
    pub fn new() -> HashSetStateProvider {
        HashSetStateProvider {
            sessions: Mutex::new(HashSet::new()),
            handles: Mutex::new(HashSet::new())
        }
    }

    // TODO: u64 not fit Javascript Number
    fn rand() -> JSON_POSITIVE_INTEGER {
        loop {
            let n: u64 = random();
            if n != 0 {
                return n;
            }
        }
    }
}

impl SharedStateProvider for HashSetStateProvider {
    fn new_session_id(&self) -> u64 {
        loop {
            let id = Self::rand();
            let mut sessions = self.sessions.lock().unwrap();
            if sessions.insert(id) {
                return id
            }
        }
    }

    fn new_handle_id(&self) -> u64 {
        loop {
            let id = Self::rand();
            let mut handles = self.handles.lock().unwrap();
            if handles.insert(id) {
                return id
            }
        }
    }
}

/* TODO: Implement redis for scale */
pub struct _RedisStateProvider;
