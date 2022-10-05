use crate::{command::Room, user::User};

use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    sync::mpsc::UnboundedSender,
    task::JoinHandle,
};
use tokio_tungstenite::accept_async;

pub struct SocketListener<A: ToSocketAddrs> {
    pub addr: A,
    pub room_channels: Arc<HashMap<String, UnboundedSender<Room>>>,
}

impl<A: ToSocketAddrs> SocketListener<A> {
    pub fn new(addr: A, room_channels: HashMap<String, UnboundedSender<Room>>) -> Self {
        let room_channels = Arc::new(room_channels);

        Self {
            addr,
            room_channels,
        }
    }

    //For later, also for this we I need to put a RWLock inside the room_channels Arc

    // pub fn connect_room(&mut self, room: String, room_channel: UnboundedSender<Room>) {
    //     self.room_channels.insert(room, room_channel);
    // }

    // pub fn disconnect_room(&mut self, room: String, room_channel: UnboundedSender<Room>) {
    //     self.room_channels.remove(room, room_channel);
    // }

    pub async fn listen(&self) {
        let connection_listener = TcpListener::bind(&self.addr).await.unwrap();
        let mut user_task_handlers: Vec<JoinHandle<()>> = vec![];

        loop {
            let (stream, _) = connection_listener.accept().await.unwrap();

            let result = accept_async(stream).await;

            match result {
                Ok(ws) => {
                    let user = User::new(ws, Arc::downgrade(&self.room_channels));
                    &user_task_handlers.push(user.run());
                }

                Err(_) => {}
            }
        }
    }
}
