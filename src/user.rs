use crate::command::{self, clasify_message, Emiter};
use futures_util::{
    select,
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt,
};
use serde_json::{self, json};
use std::{
    collections::{HashMap, HashSet},
    sync::Weak,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message::{self, Binary, Close, Ping, Pong, Text};
use uuid::Uuid;

type Sender = SplitSink<WebSocketStream<TcpStream>, Message>;
type Receiver = SplitStream<WebSocketStream<TcpStream>>;

pub struct User {
    id: Uuid,
    //Map of all the rooms in the websocket server with their channel sender
    rooms: Weak<HashMap<String, UnboundedSender<command::Room>>>,
    //Set of rooms which this user is currently subscribe to
    connected_rooms: HashSet<String>,

    //This channel exist to connect other entities to this user
    channel_receiver: UnboundedReceiver<command::User>,
    channel_sender: UnboundedSender<command::User>,

    //Client sender
    sender: Sender,
    //Client receiver
    receiver: Receiver,
}

impl User {
    pub fn new(
        stream: WebSocketStream<TcpStream>,
        rooms: Weak<HashMap<String, UnboundedSender<command::Room>>>,
    ) -> Self {
        let id = Uuid::new_v4();
        let (sender, receiver) = stream.split();
        let (channel_sender, channel_receiver) = unbounded_channel::<command::User>();
        let connected_rooms = HashSet::new();

        Self {
            id,
            rooms,
            connected_rooms,
            channel_receiver,
            channel_sender,
            sender,
            receiver,
        }
    }

    pub fn run(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                let receiver_fut = self.receiver.next();
                let room_input_fut = self.channel_receiver.recv();

                select! {
                    //This is the message that come from the client
                     user_input = receiver_fut.fuse() =>{

                        //Get the message from user
                        let message  = match user_input{
                            Some(msg_result) => {
                                match msg_result{
                                    //If the message is ok return it to use it in message variable
                                    Ok(msg) => msg,

                                    //If not, break the listening loop causing to close the connection with user
                                    Err(_) => break // I have to change this to be able to disconnect the room connection
                                }
                            },
                            None => continue
                        };


                        let result = Self::clasify_user_input(&mut self, message);

                        if let Ok(command) = result{

                            match command{

                                command::User::Event(event_name, data) =>{
                                    Self::send_to_rooms(&self, command::Room::Event(event_name, data, Emiter::User(self.id)));
                                }

                                command::User::ConnectRoom(room_name) =>{
                                    Self::connect_room(&mut self, room_name)
                                }

                                command::User::DisconnectRoom(room_name) =>{
                                    Self::disconnect_room(&mut self, room_name)
                                }

                                command::User::Close =>{break}
                            }

                        }
                        else{
                            //Send error message
                            Self::send_to_user(&mut self, result);
                        }
                     }

                     //This is the message that come from other sources that are connected to this task with the input channel
                     room_input = room_input_fut.fuse() =>{




                     }
                }
            }
        })
    }

    fn send_to_user(&mut self, command_result: Result<command::User, String>) {
        let sender = &mut self.sender;
        let mut message = json!({});

        match command_result {
            Ok(command) => {}

            Err(error) => {
                message = json!({"event": "error", "msg": error});
            }
        }

        let serialize_message = serde_json::to_string(&message).unwrap();
        sender.send(Message::text(serialize_message));
    }

    fn send_to_rooms(&self, command: command::Room) {
        let mut room_senders = vec![];

        let rooms = self.rooms.upgrade().expect("Server closed");
        for room_name in &self.connected_rooms {
            room_senders.push(rooms.get(room_name).unwrap());
        }

        match command {
            command::Room::Event(event, data, emiter) => {
                for sender in room_senders {
                    let _ = sender.send(command::Room::Event(event.clone(), data.clone(), emiter.clone()));
                }
            }
            _ => {}
        }
    }

    fn clasify_user_input(&mut self, input: Message) -> Result<command::User, String> {
        match input {
            //Handle message if it is text
            Text(message) => Ok(clasify_message(message)?),

            //Handle message if it is binary data
            Binary(bytes) => {
                unimplemented!();
            }

            //Handle message if it is a ping
            Ping(bytes) => {
                unimplemented!();
            }

            //Handle message if it is a pong
            Pong(bytes) => {
                unimplemented!();
            }

            //Handle close message
            Close(close_frame) => {
                unimplemented!();
            }
        }
    }

    fn connect_room(&mut self, room_name: String) {
        let rooms_ref = self
            .rooms
            .upgrade()
            .expect("The server is no longer running");

        match rooms_ref.get(&room_name) {
            Some(room_channel) => {
                self.connected_rooms.insert(room_name);
                room_channel.send(command::Room::ConnectUser(self.id, self.channel_sender.clone()));
            }

            None => {
                unimplemented!();
            }
        }
    }

    fn disconnect_room(&mut self, room_name: String) {
        let rooms_ref = self
            .rooms
            .upgrade()
            .expect("The server is no longer running");

        match rooms_ref.get(&room_name) {
            Some(room_channel) => {
                self.connected_rooms.remove(&room_name);
                room_channel.send(command::Room::DisconnectUser(self.id));
            }

            None => {
                unimplemented!();
            }
        }
    }
}
