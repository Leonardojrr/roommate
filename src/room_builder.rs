use crate::{
    event::{BoxFut, EventMap},
    protocol,
    room::Room,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    marker::{Send, Sync},
    sync::Arc,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    Mutex, RwLock,
};

pub struct RoomBuilder {
    name: Option<String>,
    password: Option<String>,
    events: EventMap,
    room_senders: RwLock<HashMap<String, UnboundedSender<protocol::Room>>>,
}

impl RoomBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            password: None,
            events: EventMap::new(),
            room_senders: RwLock::new(HashMap::new()),
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

    pub fn on(
        mut self,
        event_name: &str,
        event: impl Fn(Arc<Room>, Value, protocol::Emiter) -> BoxFut + Send + Sync + 'static,
    ) -> RoomBuilder {
        self.events.insert(event_name.to_string(), Box::new(event));
        self
    }

    pub async fn connect_room(self, room: Arc<Room>) -> RoomBuilder {
        self.room_senders
            .write()
            .await
            .insert(room.name.clone(), room.sender.clone());

        self
    }

    pub fn build(self) -> Arc<Room> {
        let name = self.name.unwrap_or_default();
        let password = self.password;
        let events = self.events;
        let room_senders = self.room_senders;
        let user_senders = RwLock::new(HashMap::new());

        let (sender, receiver) = unbounded_channel::<protocol::Room>();
        let receiver = Mutex::new(receiver);

        Arc::new(Room {
            name,
            password,
            events,
            sender,
            receiver,
            user_senders,
            room_senders,
        })
    }
}
