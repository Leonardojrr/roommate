use crate::room;

use room::RoomInfo;
use std::{sync::Arc,future::Future,pin::Pin,task::{Context,Poll}};
use tokio::sync::Mutex;

pub type Callback<T> = fn(Arc<Mutex<RoomInfo<T>>>, String) -> CallbackFut;

pub struct CallbackFut{
    pub fut: Pin<Box<dyn Future<Output = ()> + Send>>
}

impl Future for CallbackFut{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>{

        let event = self.get_mut();
        let poll_state = event.fut.as_mut().poll(cx);

        match poll_state {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => Poll::Ready(())
        }
    }
}