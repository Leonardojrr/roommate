use crate::connection::User;
use crate::callback::Callback;
use futures_util::{select, 
    future::{
        join_all, 
        select_all
    }, 
    FutureExt
};
use serde::{Serialize};
use tokio::sync::{
    Mutex,
    mpsc::{
    unbounded_channel, 
    UnboundedReceiver, 
    UnboundedSender
}};
use tungstenite::Message;
use std::{
    collections::{HashMap}, 
    sync::Arc
};
use serde_json::{
    self, 
    json, 
    Value
};

pub enum EmisionKind{
    Room(String),
    User(usize),
    Connection(User)
}

pub struct EmptyState;

pub struct RoomInfo<T>{
    name: String,
    connection_channel: (UnboundedSender<User>, UnboundedReceiver<User>),
    channel: (UnboundedSender<(String, Message)>, UnboundedReceiver<(String, Message)>),
    senders: HashMap<String, UnboundedSender<(String, Message)>>,
    users: Vec<User>,
    last_emiter: Option<EmisionKind>,
    state: T,
}

impl<T> RoomInfo< T>{
    pub fn new(name: String, state: T) -> Arc<Mutex<Self>>{
        Arc::new(Mutex::new(Self{
            name,
            connection_channel:unbounded_channel(),
            channel: unbounded_channel(),
            senders: HashMap::new(),
            users: Vec::new(),
            last_emiter: None,
            state
        }))
    }

    /////////////////////////////////////////////////////

    pub fn conn_channel(&self) -> UnboundedSender<User>{
        self.connection_channel.0.clone()
    }

    /////////////////////////////////////////////////////

    pub fn room_channel(&self) -> UnboundedSender<(String, Message)>{
        self.channel.0.clone()
    }

    /////////////////////////////////////////////////////

    pub async fn connect_room<State>(&mut self, room: Arc<Mutex<RoomInfo<State>>>){

        let room_ref = room.lock().await;

        let sender = room_ref.room_channel();
        let room_name = room_ref.name();

        self.senders.insert(room_name, sender);
    }

    /////////////////////////////////////////////////////

    pub fn name(&self) -> String{
        self.name.clone()
    }

    /////////////////////////////////////////////////////

    fn last_emiter(&mut self) -> Option<EmisionKind>{
        self.last_emiter.take()
    }

    /////////////////////////////////////////////////////

    fn connect_user(&mut self, user: User){
        self.users.push(user);
    }

    /////////////////////////////////////////////////////

    fn change_emiter(&mut self, emiter: EmisionKind){
        self.last_emiter = Some(emiter);
    }

    /////////////////////////////////////////////////////

    pub fn get_state(&self) -> &T{
         &self.state
    }

    /////////////////////////////////////////////////////

    pub fn get_mut_state(&mut self) -> &mut T{
        &mut self.state
    }

    /////////////////////////////////////////////////////
    
    pub async fn broadcast<S: Serialize>(&mut self, event_name: &str, data: S) {

        let json = json!({
            "event": event_name,
            "data": data
        }).to_string();

        let msg = tungstenite::Message::Text(json);
        let room_name = self.name();


        for sender in self.senders.values_mut(){
           let _ = sender.send((room_name.clone(), msg.clone()));
        }

        let mut user_senders_futures = vec![];

        for user in &mut self.users{
            user_senders_futures.push(user.send(msg.clone()));
        }

        join_all(user_senders_futures).await;
    }

    /////////////////////////////////////////////////////

    pub async fn emit<S: Serialize>(&mut self, event_name: &str, data: S) {

        let json = json!({
            "event": event_name,
            "data": data
        }).to_string();

        let msg = tungstenite::Message::Text(json);
        let room_name = self.name();

        match self.last_emiter(){
            Some(EmisionKind::Room(emiter_room_name)) =>{

                for (map_room_name, sender) in &mut self.senders{
                    if &emiter_room_name == map_room_name{
                        continue;
                    }

                    let _ = sender.send((room_name.clone(), msg.clone()));
                 }

                 let mut user_senders_futures = vec![];

                 for user in &mut self.users{
                     user_senders_futures.push(user.send(msg.clone()));
                 }
         
                 join_all(user_senders_futures).await;
            },

            Some(EmisionKind::User(emiter_user_index)) =>{

                for sender in self.senders.values_mut(){
                    let _ = sender.send((room_name.clone(), msg.clone()));
                 }

                 let mut user_senders_futures = vec![];

                 for (index, user ) in self.users.iter_mut().enumerate(){

                     if index == emiter_user_index{
                         continue;
                     }
                     user_senders_futures.push(user.send(msg.clone()));
                 }
         
                 join_all(user_senders_futures).await;
            },

            _ =>{}
        }
    }

    /////////////////////////////////////////////////////

    pub async fn whisper<S: Serialize>(&mut self, event_name: &str, data: S) {

        let json = json!({
            "event": event_name,
            "data": data
        }).to_string();

        let msg = tungstenite::Message::Text(json);
        let room_name = self.name();

        match self.last_emiter(){
            Some(EmisionKind::Room(emiter_room_name)) =>{

                match self.senders.get_mut(emiter_room_name.as_str()){
                    Some(sender)=>{
                        let _ = sender.send((room_name, msg));
                    },

                    None =>{}
                }

            }

            Some(EmisionKind::User(emiter_user_index)) =>{
                let _ = self.users[emiter_user_index].send(msg).await;
            }

            _ =>{}
        }
    }

    /////////////////////////////////////////////////////

    async fn listen_events(&mut self) -> Result<(Option<tungstenite::Message>, EmisionKind), tungstenite::Error>{

       let mut users_listener = vec![];
       for user in &mut self.users{
            users_listener.push(user.listen());
       }

       let rooms_channel_listener = self.channel.1.recv();
       let connection_channel_listener = self.connection_channel.1.recv();

       if users_listener.len() > 0{
            select!{
                // Listen for user messages
            (msg, user_index, _) = select_all(users_listener).fuse() =>{
    
                let message = msg.unwrap()?;
                return Ok((Some(message) ,EmisionKind::User(user_index)))
            },
    
                // Listen for others rooms messages
            room_msg = rooms_channel_listener.fuse() =>{
    
                let (room_name, msg) = room_msg.unwrap();
    
                return Ok((Some(msg), EmisionKind::Room(room_name)))
            }
    
                // Listen for connections
            user = connection_channel_listener.fuse() =>{
                return  Ok((None, EmisionKind::Connection(user.unwrap())))
            }
            };
       }else{

        select!{
            // Listen for others rooms messages
        room_msg = rooms_channel_listener.fuse() =>{

            let (room_name, msg) = room_msg.unwrap();

            return Ok((Some(msg), EmisionKind::Room(room_name)))
        }

            // Listen for connections
        user = connection_channel_listener.fuse() =>{
            return  Ok((None, EmisionKind::Connection(user.unwrap())))
        }
        };

       }
     
    }
}

pub struct Room<'a,T>{
    pub room: Arc<Mutex<RoomInfo<T>>>,
    event_list: HashMap<&'a str, Callback<T>>
}


impl <'a, T> Room <'a, T>{
    pub fn new(room_ref: &Arc<Mutex<RoomInfo<T>>> ) -> Self{
        let room = Arc::clone(room_ref);

        Self{
            room,
            event_list: HashMap::new()
        }
    }

    pub fn insert_event(&mut self, event_name:&'a str,event: Callback<T>){
        self.event_list.insert(event_name, event);
    }

    async fn call(&mut self, event_name: String ,data: String){
        let event_option = self.event_list.get_mut(event_name.as_str());

        match event_option{
            Some(event_function) => {
                event_function(Arc::clone(&self.room), data).await;
            }

            None => {/*panic!("The Event({}) doesn't exist",event_name)*/ }
        }

    }

    pub async fn run(&mut self){

        loop{
            let mut room_ref = self.room.lock().await;
            let received_emission = room_ref.listen_events().await;
            drop(room_ref);

            match received_emission{
                Ok(emision) =>{
                    
                    match emision{
                        ( _, EmisionKind::Connection(user)) =>{
                            let mut room_ref = self.room.lock().await;
                            room_ref.connect_user(user);
                            drop(room_ref);
                        },

                        (Some(msg), emision_type) =>{
                            let mut room_ref = self.room.lock().await;
                            room_ref.change_emiter(emision_type);
                            drop(room_ref);

                            match msg {
                                Message::Text(string) =>{

                                    let ser_result =  serde_json::from_str::<Value>(string.as_str());

                                    if let Ok(ser_msg) = ser_result{

                                       let (event_name, data) = {

                                            let mut event = String::new();

                                            if let Value::String(event_name)  = &ser_msg["event"]{
                                                event = event_name.clone();
                                            }

                                            let data = (&ser_msg["data"].to_string()).clone();

                                            (event, data)
                                       };

                                       self.call(event_name, data).await;
                                    }
                                },

                                // unimplemented
                                Message::Binary(_) =>{
                                    
                                },

                                // unimplemented
                                Message::Ping(_) =>{

                                }

                                // unimplemented
                                Message::Pong(_) =>{

                                }

                                // unimplemented
                                Message::Close(_) =>{

                                }
                            }
                        },

                        _ =>{}
                    }
                },

                Err(_) =>{}
            }

        }
    }
}