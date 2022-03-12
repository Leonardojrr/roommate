use futures_util::{
    sink::Send,
    stream::{Next, SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::collections::HashMap;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::UnboundedSender,
};
use tokio_tungstenite::{accept_hdr_async, WebSocketStream};
use tungstenite::{
    handshake::server::{Request, Response},
    Message,
};

type Sender = SplitSink<WebSocketStream<TcpStream>, Message>;
type Receiver = SplitStream<WebSocketStream<TcpStream>>;

pub struct User {
    sender: Sender,
    receiver: Receiver,
}

impl User {
    pub fn new(ws: WebSocketStream<TcpStream>) -> Self {
        let (sender, receiver) = ws.split();

        Self { sender, receiver }
    }

    pub fn listen(&mut self) -> Next<Receiver> {
        self.receiver.next()
    }

    pub fn send(&mut self, msg: tungstenite::Message) -> Send<Sender, tungstenite::Message> {
        self.sender.send(msg)
    }

    pub async fn close_connection(self) {
        let (receiver, sender) = (self.receiver, self.sender);
        let mut ws = receiver.reunite(sender).unwrap();

        match ws.close(None).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}

pub struct SocketListener<'a> {
    pub addr: &'a str,
    pub room_channels: HashMap<String, UnboundedSender<User>>,
}

impl<'a> SocketListener<'a> {
    pub fn new(addr: &'a str, room_channels: HashMap<String, UnboundedSender<User>>) -> Self {
        Self {
            addr,
            room_channels,
        }
    }

    pub fn connect_room(&mut self, room: String, room_channel: UnboundedSender<User>) {
        self.room_channels.insert(room, room_channel);
    }

    fn send_user(&self, room: String, user: User) {
        match self.room_channels.get(&room) {
            Some(room_channel) => {
                let _ = room_channel.send(user);
            }

            None => {}
        }
    }

    pub async fn listen(&self) {
        let connection_listener = TcpListener::bind(self.addr).await.unwrap();

        loop {
            let mut room_to_connect = String::new();

            let mut callback = |req: &Request, resp: Response| {
                let query = req.uri().query().unwrap();
                room_to_connect = query.split('=').collect::<Vec<&str>>()[1].to_owned();

                Ok(resp)
            };

            let (stream, _) = connection_listener.accept().await.unwrap();

         
            let result = accept_hdr_async(stream, &mut callback).await;

            match result{
                Ok(ws) =>{
                    let user = User::new(ws);
                    self.send_user(room_to_connect, user);
                }

                Err(_)=>{

                }
            }
        }
    }
}
