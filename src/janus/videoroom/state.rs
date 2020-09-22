use crate::janus::helper;
use std::collections::HashSet;
use std::sync::Mutex;

pub trait VideoRoomStateProvider: Send + Sync {
    fn new_room_id(&self) -> u64;
    fn has_room(&self, id: &u64) -> bool;
}

pub struct LocalVideoRoomState {
    rooms: Mutex<HashSet<u64>>
}

impl LocalVideoRoomState {
    pub fn new() -> LocalVideoRoomState {
        LocalVideoRoomState {
            rooms: Mutex::new(HashSet::new())
        }
    }
}

impl VideoRoomStateProvider for LocalVideoRoomState {
    fn new_room_id(&self) -> u64 {
        loop {
            let id = helper::rand_id();
            let mut rooms = self.rooms.lock().unwrap();
            if !rooms.insert(id) {
                return id
            }
        }
    }

    fn has_room(&self, id: &u64) -> bool {
        self.rooms.lock().unwrap().contains(id)
    }
}

// TODO: Redis implementation
pub struct _RedisVideoRoomState;
