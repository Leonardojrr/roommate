use crate::{data::Data, event::EventMap, protocol};
use serde_json::Value;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        Mutex, RwLock,
    },
    task::JoinHandle,
};
use uuid::Uuid;

pub struct Room {
    pub name: String,
    pub password: Option<String>,
    pub data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    pub events: EventMap,
    pub user_senders: RwLock<HashMap<Uuid, UnboundedSender<protocol::User>>>,
    pub room_senders: RwLock<HashMap<String, UnboundedSender<protocol::Room>>>,
    pub sender: UnboundedSender<protocol::Room>,
    pub receiver: Mutex<UnboundedReceiver<protocol::Room>>,
}

impl Room {
    pub async fn whisper(
        &self,
        emiter: protocol::Emiter,
        event: impl Into<String>,
        payload: Value,
    ) {
        match emiter {
            protocol::Emiter::Room(room_name) => {
                let room_senders = self.room_senders.read().await;

                let room_sender = room_senders.get(&room_name).unwrap();

                let new_emiter = protocol::Emiter::Room(self.name.clone());
                let _ = room_sender.send(protocol::Room::Event(event.into(), payload, new_emiter));
            }

            protocol::Emiter::User(user_id) => {
                let user_senders = self.user_senders.read().await;

                let user_sender = user_senders.get(&user_id).unwrap();

                let _ = user_sender.send(protocol::User::Event(event.into(), payload));
            }
        }
    }

    pub async fn emit_to_users(
        &self,
        emiter: protocol::Emiter,
        event: impl Into<String>,
        payload: Value,
    ) {
        let user_senders = self.user_senders.read().await;
        let event: String = event.into();

        match emiter {
            protocol::Emiter::User(user_id) => {
                for (id, sender) in user_senders.iter() {
                    if *id == user_id {
                        continue;
                    }

                    let _ = sender.send(protocol::User::Event(event.clone(), payload.clone()));
                }
            }

            protocol::Emiter::Room(_) => {
                for (_, sender) in user_senders.iter() {
                    let _ = sender.send(protocol::User::Event(event.clone(), payload.clone()));
                }
            }
        }
    }

    pub async fn broadcast_to_users(&self, event: impl Into<String>, payload: Value) {
        let user_senders = self.user_senders.read().await;
        let event: String = event.into();

        for (_, sender) in user_senders.iter() {
            let _ = sender.send(protocol::User::Event(event.clone(), payload.clone()));
        }
    }

    pub async fn emit_to_rooms(
        &self,
        emiter: protocol::Emiter,
        event: impl Into<String>,
        payload: Value,
    ) {
        let event: String = event.into();
        let room_senders = self.room_senders.read().await;

        let new_emiter = protocol::Emiter::Room(self.name.clone());
        let room_command = protocol::Room::Event(event, payload, new_emiter);

        match emiter {
            protocol::Emiter::User(_) => {
                for (_, sender) in room_senders.iter() {
                    let _ = sender.send(room_command.clone());
                }
            }

            protocol::Emiter::Room(room_name) => {
                for (room_id, sender) in room_senders.iter() {
                    if *room_id == room_name {
                        continue;
                    }
                    let _ = sender.send(room_command.clone());
                }
            }
        }
    }

    pub async fn broadcast_to_rooms(&self, event: impl Into<String>, payload: Value) {
        let event: String = event.into();
        let room_senders = self.room_senders.read().await;

        let new_emiter = protocol::Emiter::Room(self.name.clone());
        let room_command = protocol::Room::Event(event, payload, new_emiter);

        for (_, sender) in room_senders.iter() {
            let _ = sender.send(room_command.clone());
        }
    }

    pub async fn emit(&self, emiter: protocol::Emiter, event: impl Into<String>, payload: Value) {
        let event: String = event.into();
        let user_senders = self.user_senders.read().await;
        let room_senders = self.room_senders.read().await;

        match emiter {
            protocol::Emiter::User(user_id) => {
                for (id, sender) in user_senders.iter() {
                    if *id == user_id {
                        continue;
                    }

                    let _ = sender.send(protocol::User::Event(event.clone(), payload.clone()));
                }

                for (_, sender) in room_senders.iter() {
                    let room_command = protocol::Room::Event(
                        event.clone(),
                        payload.clone(),
                        protocol::Emiter::Room(self.name.clone()),
                    );
                    let _ = sender.send(room_command);
                }
            }

            protocol::Emiter::Room(room_name) => {
                for (room_id, sender) in room_senders.iter() {
                    if *room_id == room_name {
                        continue;
                    }

                    let room_command = protocol::Room::Event(
                        event.clone(),
                        payload.clone(),
                        protocol::Emiter::Room(self.name.clone()),
                    );
                    let _ = sender.send(room_command);
                }

                for (_, sender) in user_senders.iter() {
                    let _ = sender.send(protocol::User::Event(event.clone(), payload.clone()));
                }
            }
        }
    }

    pub async fn broadcast(&self, event: impl Into<String>, payload: Value) {
        let event: String = event.into();
        let user_senders = self.user_senders.read().await;
        let room_senders = self.room_senders.read().await;

        for (_, sender) in room_senders.iter() {
            let room_command = protocol::Room::Event(
                event.clone(),
                payload.clone(),
                protocol::Emiter::Room(self.name.clone()),
            );
            let _ = sender.send(room_command);
        }

        for (_, sender) in user_senders.iter() {
            let _ = sender.send(protocol::User::Event(event.clone(), payload.clone()));
        }
    }

    /////Data related functions////
    pub fn share_data<T: 'static>(&self) -> Data<T> {
        self.data
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref::<Data<T>>()
            .unwrap()
            .clone()
    }

    pub fn call(self: &Arc<Room>, event_name: String, payload: Value, emiter: protocol::Emiter) {
        let callback = match &emiter {
            protocol::Emiter::User(_) => self.events.get_default_event(&event_name),

            protocol::Emiter::Room(room_name) => match self.events.get(&Some(room_name.clone())) {
                Some(room_events_map) => match room_events_map.get(&event_name) {
                    None => self.events.get_default_event(&event_name),
                    some_callback => some_callback,
                },
                None => self.events.get_default_event(&event_name),
            },
        };

        match callback {
            Some(function) => {
                let future = function(self.clone(), payload, emiter);
                tokio::spawn(async move { future.await });
            }

            None => {}
        }
    }

    pub async fn connect_room(self: &Arc<Room>, room: Arc<Room>) {
        self.room_senders
            .write()
            .await
            .insert(room.name.clone(), room.sender.clone());
    }

    ///Runner////
    pub fn run(self: &Arc<Room>) -> JoinHandle<()> {
        let room = self.clone();

        tokio::spawn(async move {
            loop {
                let mut receiver = room.receiver.lock().await;

                let received_message = receiver.recv().await;

                match received_message {
                    Some(room_command) => match room_command {
                        protocol::Room::Event(event_name, payload, emiter) => {
                            room.call(event_name, payload, emiter);
                        }

                        protocol::Room::ConnectUser(id, user_sender) => {
                            let mut user_senders = room.user_senders.write().await;
                            user_senders.insert(id, user_sender);
                        }

                        protocol::Room::DisconnectUser(id) => {
                            let mut user_senders = room.user_senders.write().await;
                            user_senders.remove(&id);
                        }

                        protocol::Room::Close => break,
                    },
                    None => break,
                }
            }
        })
    }
}
