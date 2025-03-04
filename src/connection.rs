use crate::{protocol::Room, user::User};

use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    sync::mpsc::UnboundedSender,
    task::JoinHandle,
};
use tokio_tungstenite::accept_async;

pub struct SocketListener<A: ToSocketAddrs + Send + Sync> {
    pub addr: A,
    pub room_channels: Arc<HashMap<String, UnboundedSender<Room>>>,
}

impl<A: ToSocketAddrs + Send + Sync + 'static> SocketListener<A> {
    pub fn new(addr: A, room_channels: HashMap<String, UnboundedSender<Room>>) -> Self {
        let room_channels = Arc::new(room_channels);

        Self {
            addr,
            room_channels,
        }
    }

    pub fn listen(self) -> JoinHandle<()> {
        tokio::task::spawn(async move {
            let connection_listener = TcpListener::bind(&self.addr)
                .await
                .expect("The address of the socket is not valid");

            let mut user_task_handlers: Vec<JoinHandle<()>> = vec![];

            loop {
                let (stream, _) = connection_listener.accept().await.unwrap();
                let result = accept_async(stream).await;

                match result {
                    Ok(ws) => {
                        let user = User::new(ws, Arc::downgrade(&self.room_channels));
                        user_task_handlers.push(user.run());
                    }

                    Err(_) => {}
                }
            }
        })
    }
}
