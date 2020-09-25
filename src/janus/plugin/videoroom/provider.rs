use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use crate::janus::helper;
use super::request::CreateParameters;

pub trait VideoRoomStateProvider: Send + Sync {
    fn new_room_id(&self) -> u64;
    fn has_room(&self, id: &u64) -> bool;

    fn save_room_parameters(&self, room: CreateParameters);
    fn get_room_parameters(&self, room: &u64) -> String;
}

pub struct MemoryVideoRoomState {
    rooms: Mutex<HashSet<u64>>,
    params: Mutex<HashMap<u64, String>>
}

impl MemoryVideoRoomState {
    pub fn new() -> MemoryVideoRoomState {
        MemoryVideoRoomState {
            rooms: Mutex::new(HashSet::new()),
            params: Mutex::new(HashMap::new())
        }
    }
}

impl VideoRoomStateProvider for MemoryVideoRoomState {
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

    fn save_room_parameters(&self, room: CreateParameters) {
        // TODO: json stringify error handling
        // TODO: more efficient storing method
        self.params.lock().unwrap().insert(room.room.unwrap(), serde_json::to_string(&room).unwrap());
    }

    fn get_room_parameters(&self, room: &u64) -> String {
        // TODO: do NOT copy
        self.params.lock().unwrap().get(room).unwrap().clone()
    }
}

// TODO: Redis implementation
pub struct _RedisVideoRoomState;
