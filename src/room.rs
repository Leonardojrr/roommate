use crate::{command, event::Callback};
use futures_util::future::join_all;
use serde::Serialize;
use serde_json::{self, json};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        RwLock,
    },
    task::JoinHandle,
};
use uuid::Uuid;

#[macro_export]
macro_rules! room {
    (

        $room_name:ident <$state:tt>{
            $state_init:expr,
            $($tokens:tt)+
        }
    ) =>{

        {
            let room_name = stringify!($room_name);
            let ctx: Arc<Mutex<Context<$state>>> = Context::new(String::from(room_name), $state_init);
            let (signal_sender, signal_receiver) = tokio::sync::mpsc::channel::<bool>(1);


            let mut room = Room::new(ctx.clone(), signal_receiver);

            callback!(room, $state, $($tokens)+);
        }
    };

    (
        $room_name:ident
        {
            $($tokens:tt)+
        }
    ) =>{

        {
            let room_name = stringify!($room_name);
            let ctx: Arc<Mutex<Context<EmptyState>>> = Context::new(String::from(room_name), EmptyState);
            let (signal_sender, signal_receiver) = tokio::sync::mpsc::channel::<bool>(1);

            let mut room = Room::new(ctx, signal_receiver);

            callback!(room, EmptyState, $($tokens)+);
        }
    }
}

pub struct Context<T> {
    name: String,
    state: Option<Arc<RwLock<T>>>,
    emiter: Option<command::Emiter>,
    user_senders: HashMap<Uuid, UnboundedSender<command::User>>,
    room_senders: HashMap<String, UnboundedSender<command::Room>>,
    sender: UnboundedSender<command::Room>,
}

impl<T> Context<T> {
    pub fn new(
        name: String,
        state: T,
        sender: UnboundedSender<command::Room>,
    ) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            state,
            name,
            emiter: None,
            user_senders: HashMap::new(),
            room_senders: HashMap::new(),
            sender,
        }))
    }

    /////////////////////////////////////////////////////

    pub fn name(&self) -> String {
        self.name.clone()
    }

    /////////////////////////////////////////////////////

    pub fn room_channel(&self) -> UnboundedSender<command::Room> {
        self.sender.clone()
    }

    /////////////////////////////////////////////////////

    pub async fn connect_room<State>(&mut self, room: Arc<RwLock<Context<State>>>) {
        let room_ref = room.read().await;

        let sender = room_ref.room_channel();
        let room_name = room_ref.name();

        self.room_senders.insert(room_name, sender);
    }

    /////////////////////////////////////////////////////

    fn last_emiter(&mut self) -> Option<EmisionKind> {
        self.last_emiter.take()
    }

    /////////////////////////////////////////////////////

    fn change_emiter(&mut self, emiter: EmisionKind) {
        self.last_emiter = Some(emiter);
    }

    /////////////////////////////////////////////////////

    fn connect_user(&mut self, user_id: Uuid, sender: UnboundedSender<command::User>) {
        self.user_senders.insert(user_id, sender);
    }

    /////////////////////////////////////////////////////

    async fn disconnect_user(&mut self, user_id: Uuid) {
        self.user_senders.remove(&user_id);
    }

    /////////////////////////////////////////////////////

    pub fn get_state(&self) -> &T {
        &self.state
    }

    /////////////////////////////////////////////////////

    pub fn get_mut_state(&mut self) -> &mut T {
        &mut self.state
    }

    /////////////////////////////////////////////////////

    pub async fn broadcast<S: Serialize>(&mut self, event_name: &str, data: S) {
        let json = json!({
            "event": event_name,
            "data": data
        })
        .to_string();

        let msg = tungstenite::Message::Text(json);
        let room_name = self.name();

        for sender in self.senders.values_mut() {
            let _ = sender.send((room_name.clone(), msg.clone()));
        }

        let mut user_senders_futures = vec![];

        for user in &mut self.users {
            user_senders_futures.push(user.send(msg.clone()));
        }

        join_all(user_senders_futures).await;
    }

    /////////////////////////////////////////////////////

    pub async fn emit<S: Serialize>(&mut self, event_name: &str, data: S) {
        let json = json!({
            "event": event_name,
            "data": data
        })
        .to_string();

        let msg = tungstenite::Message::Text(json);
        let room_name = self.name();

        match self.last_emiter() {
            Some(EmisionKind::Room(emiter_room_name)) => {
                for (map_room_name, sender) in &mut self.senders {
                    if &emiter_room_name == map_room_name {
                        continue;
                    }

                    let _ = sender.send((room_name.clone(), msg.clone()));
                }

                let mut user_senders_futures = vec![];

                for user in &mut self.users {
                    user_senders_futures.push(user.send(msg.clone()));
                }

                join_all(user_senders_futures).await;
            }

            Some(EmisionKind::User(emiter_user_index)) => {
                for sender in self.senders.values_mut() {
                    let _ = sender.send((room_name.clone(), msg.clone()));
                }

                let mut user_senders_futures = vec![];

                for (index, user) in self.users.iter_mut().enumerate() {
                    if index == emiter_user_index {
                        continue;
                    }
                    user_senders_futures.push(user.send(msg.clone()));
                }

                join_all(user_senders_futures).await;
            }

            _ => {}
        }
    }

    /////////////////////////////////////////////////////

    pub async fn whisper<S: Serialize>(&mut self, event_name: &str, data: S) {
        let json = json!({
            "event": event_name,
            "data": data
        })
        .to_string();

        let msg = tungstenite::Message::Text(json);
        let room_name = self.name();

        match self.last_emiter() {
            Some(EmisionKind::Room(emiter_room_name)) => {
                match self.senders.get_mut(emiter_room_name.as_str()) {
                    Some(sender) => {
                        let _ = sender.send((room_name, msg));
                    }

                    None => {}
                }
            }

            Some(EmisionKind::User(emiter_user_index)) => {
                let _ = self.users[emiter_user_index].send(msg).await;
            }

            _ => {}
        }
    }
}

pub struct Room<'a, T: Send + Sync> {
    ctx: Arc<RwLock<Context<T>>>,
    event_list: HashMap<&'a str, Callback<T>>,
    receiver: UnboundedReceiver<command::Room>,
}

impl<'a, T: Send + Sync> Room<'a, T> {
    pub fn new(ctx: Arc<RwLock<Context<T>>>, receiver: UnboundedReceiver<command::Room>) -> Self {
        Self {
            ctx,
            event_list: HashMap::new(),
            receiver,
        }
    }

    pub fn insert_event(&mut self, event_name: &'a str, event: Callback<T>) {
        self.event_list.insert(event_name, event);
    }

    async fn call(&self, event_name: String, data: String) {
        let event_option = self.event_list.get(event_name.as_str());

        match event_option {
            Some(event_function) => {
                event_function(Arc::clone(&self.ctx), data).await;
            }

            None => { /*panic!("The Event({}) doesn't exist",event_name)*/ }
        }
    }

    pub fn run(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                let room_command = self
                    .receiver
                    .recv()
                    .await
                    .expect("There is no more connections to this room");

                match room_command {
                    command::Room::Event(event_name, data, emiter) => {}

                    command::Room::ConnectUser(id, sender) => {
                        let mut room_ref = self.ctx.write().await;
                        room_ref.connect_user(id, sender);
                    }

                    command::Room::DisconnectUser(id) => {
                        let mut room_ref = self.ctx.write().await;
                        room_ref.disconnect_user(id);
                    }

                    //Break the loop, Finishing this room
                    command::Room::Close => break,
                }
            }
        })
    }
}
