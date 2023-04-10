use crate::{
    data::Data,
    event::{Callback, EventMap},
    protocol,
    room::Room,
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::{Send, Sync},
    sync::Arc,
};
use tokio::sync::{mpsc::unbounded_channel, Mutex, RwLock};

pub struct RoomBuilder {
    name: Option<String>,
    password: Option<String>,
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    events: EventMap,
}

impl RoomBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            password: None,
            data: HashMap::new(),
            events: EventMap::new(),
        }
    }

    pub fn name(mut self, name: &str) -> RoomBuilder {
        self.name = Some(String::from(name));
        self
    }

    pub fn password(mut self, password: &str) -> RoomBuilder {
        self.password = Some(String::from(password));
        self
    }

    pub fn data<T: 'static>(mut self, data: &Data<T>) -> RoomBuilder
    where
        T: Sync + Send,
    {
        self.data.insert(data.data_type_id, Box::new(data.clone()));
        self
    }

    pub fn event(mut self, event: (String, Callback)) -> RoomBuilder {
        self.events.insert(event.0, event.1);
        self
    }

    pub fn events(mut self, eventmap: EventMap) -> RoomBuilder {
        self.events.insert_eventmap(eventmap);
        self
    }

    pub fn build(self) -> Arc<Room> {
        let name = self.name.unwrap_or_default();

        let password = self.password;

        let data = self.data;

        let events = self.events;

        let user_senders = RwLock::new(HashMap::new());
        let room_senders = RwLock::new(HashMap::new());

        let (sender, receiver) = unbounded_channel::<protocol::Room>();
        let receiver = Mutex::new(receiver);

        Arc::new(Room {
            name,
            password,
            data,
            events,
            sender,
            receiver,
            user_senders,
            room_senders,
        })
    }
}
