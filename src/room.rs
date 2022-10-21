use crate::{command, event::{Callback, Events}, data::Data};
use std::{collections::HashMap, sync::Arc, any::{TypeId, Any}};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        RwLock,
        Mutex
    },
    task::JoinHandle,
};
use uuid::Uuid;

#[macro_export]
macro_rules! room {
    (
        name:$room_name:expr,
        data:[$($data:expr),+],
        password:$password:expr,
        events:{$($events:tt)+}
    ) => {
        {
            let mut room = Room::new($room_name.to_owned(), Some($password.to_owned()));
            $(room.insert_data($data.clone());)+

            events!(room, $($events)+);

            room
        }
    };

    (
        name:$room_name:expr,
        password:$password:expr,
        events:{$($events:tt)+}
    ) => {
        {
            let mut room = Room::new($room_name.to_owned(), Some($password.to_owned()));

            events!(room, $($events)+);

            room
        }
        
    };

    (
        name:$room_name:expr,
        data:[$($data:expr),+],
        events:{$($events:tt)+}
    ) => {

        {
            let mut room = Room::new($room_name.to_owned(), None);
            $(room.insert_data($data.clone());)+
            
            events!(room, $($events)+);

            room
        }
        
    };
    
    (
        name:$room_name:expr,
        events:{$($events:tt)+}
    ) => {

        {
            let mut room = Room::new($room_name.to_owned(), None);
            
            events!(room, $($events)+);

            room
        }
        
    };

}

pub struct Room{
    name: String,
    password: Option<String>,
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    events: Events,
    user_senders: RwLock<HashMap<Uuid, UnboundedSender<command::User>>>,
    room_senders: RwLock<HashMap<String, UnboundedSender<command::Room>>>,
    sender:UnboundedSender<command::Room>,
    receiver: Mutex<UnboundedReceiver<command::Room>>
}

impl Room  {

    pub fn new(name: String, password: Option<String>) -> Self{
        let (sender, receiver) = unbounded_channel::<command::Room>();

        Self {
            name,
            password, 
            data: HashMap::new(), 
            events: Events::new(),
            user_senders:RwLock::new(HashMap::new()) ,
            room_senders: RwLock::new(HashMap::new()) ,
            sender,
            receiver: Mutex::new(receiver)
        }
    }

    pub async fn whisper(&self, emiter: command::Emiter, event:String, payload: String) {
        match emiter {
            command::Emiter::Room(room_name) =>{
                let room_senders = self.room_senders.read().await;

                let room_sender = room_senders.get(&room_name).unwrap();
                
                let new_emiter = command::Emiter::Room(self.name.clone());
                room_sender.send(command::Room::Event(event, payload, new_emiter));
            }

            command::Emiter::User(user_id) =>{
                let user_senders = self.user_senders.read().await;

                let user_sender = user_senders.get(&user_id).unwrap();

                user_sender.send(command::User::Event(event, payload));
            }
        }
    }

    pub async fn emit_to_users(&self, emiter: command::Emiter, event:String, payload: String) {

        let user_senders = self.user_senders.read().await;

        match  emiter {
            command::Emiter::User(user_id) =>{
                for (id, sender) in user_senders.iter(){
                    if *id == user_id {continue;}

                    sender.send(command::User::Event(event.clone(), payload.clone()));
                }
            }

            command::Emiter::Room(_) =>{
                for (_, sender) in user_senders.iter(){
                    sender.send(command::User::Event(event.clone(), payload.clone()));
                }
            }
            
        }
    }

    pub async fn broadcast_to_users(&self, event:String, payload: String) {
        let user_senders = self.user_senders.read().await;

        for (_, sender) in user_senders.iter(){
            sender.send(command::User::Event(event.clone(), payload.clone()));
        }
    }

    pub async fn emit_to_rooms(&self, emiter: command::Emiter, event:String, payload: String) {
        let room_senders = self.room_senders.read().await;

        let new_emiter = command::Emiter::Room(self.name.clone());
        let room_command = command::Room::Event(event, payload, new_emiter);

        match  emiter {
            command::Emiter::User(_) =>{
                for (_, sender) in room_senders.iter(){
                    sender.send(room_command.clone());
                }
            }

            command::Emiter::Room(room_name) =>{
                for (room_id, sender) in room_senders.iter(){
                    if *room_id == room_name {continue;}
                    sender.send(room_command.clone());
                }
            }
        }
    }

    pub async fn broadcast_to_rooms(&self, event:String, payload: String) {
        let room_senders = self.room_senders.read().await;

        let new_emiter = command::Emiter::Room(self.name.clone());
        let room_command = command::Room::Event(event, payload, new_emiter);

        for (_, sender) in room_senders.iter(){
            sender.send(room_command.clone());
        }
    }
    
    pub async fn emit(&self, emiter: command::Emiter, event:String, payload: String) {
        let user_senders = self.user_senders.read().await;
        let room_senders = self.room_senders.read().await;

        match  emiter {
            command::Emiter::User(user_id) =>{
                for (id, sender) in user_senders.iter(){
                    if *id == user_id {continue;}

                    sender.send(command::User::Event(event.clone(), payload.clone()));
                }

                for (_, sender) in room_senders.iter(){
                    let room_command = command::Room::Event(event.clone(), payload.clone(), command::Emiter::Room(self.name.clone()));
                    sender.send(room_command);
                }
            }

            command::Emiter::Room(room_name) =>{
                for (room_id, sender) in room_senders.iter(){
                    if *room_id == room_name {continue;}

                    let room_command = command::Room::Event(event.clone(), payload.clone(), command::Emiter::Room(self.name.clone()));
                    sender.send(room_command);
                }

                for (_, sender) in user_senders.iter(){
                    sender.send(command::User::Event(event.clone(), payload.clone()));
                }
            }
        }
    }

    pub async fn broadcast(&self, event: String, payload: String) {
        let user_senders = self.user_senders.read().await;
        let room_senders = self.room_senders.read().await;

        for (_, sender) in room_senders.iter(){
            let room_command = command::Room::Event(event.clone(), payload.clone(), command::Emiter::Room(self.name.clone()));
            sender.send(room_command);
        }

        for (_, sender) in user_senders.iter(){
            sender.send(command::User::Event(event.clone(), payload.clone()));
        }

    }

    /////Data related functions////
    pub fn share_data<T:'static>(&self) -> Data<T>{
        self.data.get(&TypeId::of::<T>()).unwrap().downcast_ref::<Data<T>>().unwrap().clone()
    }
    
    pub fn insert_data<T: Sync + Send + 'static>(&mut self, data:Data<T>){
        self.data.insert(data.data_type_id,Box::new(data));
    }

    ////Event related functions////
    pub fn on(&mut self, event_name: String, callback: Callback){
        self.events.insert_event(event_name, callback);
    }

    pub fn call(self: &Arc<Room>, event_name:String, emiter: command::Emiter, payload: String) -> JoinHandle<()>{
        let future = self.events.get(&event_name)(self.clone(), emiter, payload);
        tokio::spawn(async move{future.await})
    }


    ///Runner////
    pub fn run(self: &Arc<Room>) -> JoinHandle<()>{

        let room  = self.clone();

        tokio::spawn(async move {
            loop{
                let mut receiver  = room.receiver.lock().await;

                let received_message = receiver.recv().await;

                match received_message {
                    Some(room_command) =>{
                        match room_command {

                            command::Room::Event(event_name, payload, emiter) =>{

                                room.call(event_name, emiter, payload);

                            }

                            command::Room::ConnectUser(id, user_sender ) =>{

                                let mut user_senders = room.user_senders.write().await;
                                user_senders.insert(id, user_sender);


                            }

                            command::Room::DisconnectUser(id) =>{

                                let mut user_senders = room.user_senders.write().await;
                                user_senders.remove(&id);

                            }

                            command::Room::Close => break

                        }
                    }
                    None => break
                }
            }
        })
    }
}