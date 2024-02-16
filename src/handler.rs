use serde::{Deserialize, Serialize};
use std::time;
use std::fmt;
use std::sync;
use std::net;
use std::collections::HashMap;

#[derive(Serialize)]
pub enum EventType {
    READ,
    WRITE
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EventType::READ => write!(f, "READ"),
            EventType::WRITE => write!(f, "WRITE"),
        }
    }}

#[derive(Serialize, Deserialize)]
pub struct Sender {
    pub addr : Option<net::IpAddr>
}

#[derive(Serialize)]
pub struct Event {
    uuid : u64,
    sender : Sender,
    timestamp : time::SystemTime,
    event_type : EventType,
    key : String,
    value : Option<String>
}

impl Event {
    pub fn new(event_sender: Sender, etype: EventType, event_key: String, event_value: Option<String>) -> Self {
        return Self {
            uuid: 1,
            sender: event_sender,
            timestamp: time::SystemTime::now(),
            event_type: etype,
            key: event_key,
            value: event_value 
        }
    }
}

pub enum StoreError {
    NO_VALUE_IN_EVENT,
    NO_KEY_IN_EVENT
}


pub enum ReadError {
    NO_KEY_FOUND
}

#[derive(Clone)]
pub struct Database {
    pub map: sync::Arc<sync::Mutex<HashMap<String, String>>>
}

impl Database {
    pub fn store(&self, event : &Event) -> Result<(), StoreError> {
        if let Some(value) = &event.value {
            self.map.lock().unwrap().insert(event.key.to_string(), value.to_string());
            return Ok(());
        } else {
            return Err(StoreError::NO_VALUE_IN_EVENT);
        }
    }

    pub fn read(&self, key: String) -> Result<Option<String>, ReadError> {
        if let Some(value) = self.map.lock().unwrap().get(&key).cloned() {
            return Ok(Some(value))
        } else {
            return Err(ReadError::NO_KEY_FOUND);
        }
    }
}

