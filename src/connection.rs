use std::{
    collections::HashMap, 
};
use futures_util::{
    sink::Send,
    stream::{
        Next, 
        SplitSink, 
        SplitStream
    }, 
    SinkExt, 
    StreamExt
};
use tokio::{
    sync::mpsc::UnboundedSender, 
    net::{
        TcpListener, 
        TcpStream
    }
};
use tokio_tungstenite::{
    WebSocketStream,
    accept_hdr_async,
};
use tungstenite::{
    Message,
    handshake::server::{Request, Response}
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

        Self {
            sender,
            receiver,
        }
    }

    pub fn listen(&mut self) -> Next<Receiver>{
        self.receiver.next()
    }

    pub fn send(&mut self, msg: tungstenite::Message) -> Send<Sender, tungstenite::Message>{
        self.sender.send(msg)
    }
}


pub struct SocketListener{
    pub socket: TcpListener,
    pub room_channels: HashMap<String, UnboundedSender<User>>
}

impl SocketListener{

    fn connect_user(&self, room: String, user: User){
        let _ = self.room_channels.get(&room).unwrap().send(user);
    }

    pub async fn listen(&self){

        loop{
            let mut room_to_conect = String::new();

            let mut callback = |req: &Request, resp: Response|{
    
                let query = req.uri().query().unwrap();
                room_to_conect = query.split('=').collect::<Vec<&str>>()[1].to_owned();
    
                Ok(resp)
            };

            let (stream, _) = self.socket.accept().await.unwrap();

            let ws = accept_hdr_async(stream, &mut callback).await.unwrap();
            let user = User::new(ws);

            self.connect_user(room_to_conect, user);
        } 
    }
}
