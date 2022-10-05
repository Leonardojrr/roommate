use crate::room;
use room::Context;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

pub type Callback =
    Box<dyn Fn(Arc<RwLock<i32>>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[macro_export]
macro_rules! callback {

    ($room_name:ident, $state:tt, #[$parse_type:ty] $event_name:tt => $room:ident, $data:ident $event_block:block, $($tokens:tt)+) => {

            fn $event_name(room: Arc<Mutex<RwLock<$state>>>, data:String) -> CallbackFut  {
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

        fn $event_name(room: Arc<Mutex<RwLock<$state>>>, data:String) -> CallbackFut  {
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

        fn $event_name(room: Arc<RwLock<Context<$state>>>, data:String) -> CallbackFut  {
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

        fn $event_name(room: Arc<RwLock<Context<$state>>>, data:String) -> CallbackFut  {
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
