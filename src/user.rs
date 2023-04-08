use crate::protocol::{self, Emiter};
use futures_util::{
    select,
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt,
};
use serde_json::{self, Value};
use std::{collections::HashMap, sync::Weak};
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
    rooms: Weak<HashMap<String, UnboundedSender<protocol::Room>>>,

    //Set of rooms which this user is currently subscribe to
    connected_rooms: HashMap<String, UnboundedSender<protocol::Room>>,

    //This channel exist to connect other entities to this user
    channel_receiver: UnboundedReceiver<protocol::User>,
    channel_sender: UnboundedSender<protocol::User>,

    //Client sender
    sender: Sender,
    //Client receiver
    receiver: Receiver,
}

impl User {
    pub fn new(
        stream: WebSocketStream<TcpStream>,
        rooms: Weak<HashMap<String, UnboundedSender<protocol::Room>>>,
    ) -> Self {
        let id = Uuid::new_v4();
        let (sender, receiver) = stream.split();
        let (channel_sender, channel_receiver) = unbounded_channel::<protocol::User>();
        let connected_rooms = HashMap::new();

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


                        let result = self.classify_user_input( message);

                        match result {
                            Ok(user_protocol) =>{

                                match user_protocol{
                                    protocol::User::Event(event_name, data) =>{
                                        Self::send_to_rooms(&self, protocol::Room::Event(event_name, data, Emiter::User(self.id)));
                                    }

                                    protocol::User::ConnectRoom(room_name) =>{
                                        Self::connect_room(&mut self, room_name)
                                    }

                                    protocol::User::DisconnectRoom(room_name) =>{
                                        Self::disconnect_room(&mut self, room_name)
                                    }

                                    protocol::User::Close =>{break}
                                }
                            },

                            Err(user_protocol_error) => self.send_to_user(Err(user_protocol_error)).await,
                        }
                     }

                     //This is the message that come from other sources that are connected to this task with the input channel
                     room_input = room_input_fut.fuse() =>{
                        match room_input{
                            Some(user_protocol) =>{
                                match user_protocol{
                                    protocol::User::Event(event_name, data) =>{
                                        let _ = Self::send_to_user(&mut self, Ok(protocol::User::Event(event_name, data))).await;
                                    }

                                    protocol::User::ConnectRoom(room_name) =>{
                                        Self::connect_room(&mut self, room_name)
                                    }

                                    protocol::User::DisconnectRoom(room_name) =>{
                                        Self::disconnect_room(&mut self, room_name)
                                    }

                                    protocol::User::Close =>{break}
                                }
                            }
                            None => unimplemented!()
                        }
                     }
                }
            }
        })
    }

    async fn send_to_user(&mut self, command_result: Result<protocol::User, protocol::Error>) {
        let sender = &mut self.sender;

        let message: Value = match command_result {
            Ok(command) => command.into(),

            Err(error) => error.into(),
        };

        let serialize_message = serde_json::to_string(&message).unwrap();
        let _ = sender.send(Message::text(serialize_message)).await;
    }

    fn send_to_rooms(&self, command: protocol::Room) {
        for (_, room_sender) in &self.connected_rooms {
            let _ = room_sender.send(command.clone());
        }
    }

    fn classify_user_input(&mut self, input: Message) -> Result<protocol::User, protocol::Error> {
        match input {
            //Handle message if it is text
            Text(message) => Ok(protocol::User::try_from(message)?),

            //Handle message if it is binary data
            Binary(_) => {
                unimplemented!();
            }

            //Handle message if it is a ping
            Ping(_) => {
                unimplemented!();
            }

            //Handle message if it is a pong
            Pong(_) => {
                unimplemented!();
            }

            //Handle close message
            Close(_) => {
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
                self.connected_rooms.insert(room_name, room_channel.clone());

                let _ = room_channel.send(protocol::Room::ConnectUser(
                    self.id,
                    self.channel_sender.clone(),
                ));
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
                let _ = room_channel.send(protocol::Room::DisconnectUser(self.id));
            }

            None => {
                unimplemented!();
            }
        }
    }
}
