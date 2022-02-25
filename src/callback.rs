use crate::room;
use room::RoomInfo;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::Mutex;

#[macro_export]
macro_rules! callback {

    ($room_name:ident, $state:tt, #[$parse_type:ty] $event_name:tt => $room:ident, $data:ident $event_block:block, $($tokens:tt)+) => {

            fn $event_name(room: Arc<Mutex<RoomInfo<$state>>>, data:String) -> CallbackFut  {
                let future = async move{
                    let mut $room = room.lock().await;
                    let $data = des::<$parse_type>(&data);
                    $event_block;
                };

                CallbackFut{fut: Box::pin(future)}
            }

            let event_wrapper = $event_name;
            $room_name.insert_event(stringify!($event_name), event_wrapper);


        callback!($room_name, $state, $($tokens)+);
    };

    ($room_name:ident, $state:tt, $event_name:tt => $room:ident, $data:ident $event_block:block, $($tokens:tt)+) => {

        fn $event_name(room: Arc<Mutex<RoomInfo<$state>>>, data:String) -> CallbackFut  {
            let future = async move{
                let mut $room = room.lock().await;
                let $data = data;
                $event_block;
            };

            CallbackFut{fut: Box::pin(future)}
        }

        let event_wrapper = $event_name;
        $room_name.insert_event(stringify!($event_name), event_wrapper);

        callback!($room_name, $state, $($tokens)+);
    };

    ($room_name:ident, $state:tt, #[$parse_type:ty] $event_name:tt => $room:ident, $data:ident $event_block:block) => {

        fn $event_name(room: Arc<Mutex<RoomInfo<$state>>>, data:String) -> CallbackFut  {
            let future = async move{
                    let mut $room = room.lock().await;
                    let $data = des::<$parse_type>(&data);
                    $event_block;
            };

            CallbackFut{fut: Box::pin(future)}
        }

        let event_wrapper = $event_name;
        $room_name.insert_event(stringify!($event_name), event_wrapper);
    };

    ($room_name:ident, $state:tt, $event_name:tt => $room:ident, $data:ident $event_block:block) => {

        fn $event_name(room: Arc<Mutex<RoomInfo<$state>>>, data:String) -> CallbackFut  {
            let future = async move{
                let mut $room = room.lock().await;
                let $data = data;
                $event_block;
            };

            CallbackFut{fut: Box::pin(future)}
        }

        let event_wrapper = $event_name;
        $room_name.insert_event(stringify!($event_name), event_wrapper);
    };
}

pub type Callback<T> = fn(Arc<Mutex<RoomInfo<T>>>, String) -> CallbackFut;

pub struct CallbackFut {
    pub fut: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Future for CallbackFut {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let event = self.get_mut();
        let poll_state = event.fut.as_mut().poll(cx);

        match poll_state {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => Poll::Ready(()),
        }
    }
}
