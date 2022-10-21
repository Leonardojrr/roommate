use crate::{room::Room, command};
use std::{future::Future, pin::Pin, sync::Arc, collections::HashMap};

pub type Callback = Box<dyn Fn(Arc<Room>, command::Emiter, String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct Events{
    events_map: HashMap<String, Callback>
}

impl Events{
    pub fn new() -> Self{
        Self { 
            events_map: HashMap::new()
        }
    }

    pub fn get(&self, event_name: &str) -> &Callback{
        self.events_map.get(event_name).unwrap()
    }

    pub fn insert_event(&mut self, event_name: String, callback: Callback){
        self.events_map.insert(event_name, callback);
    }
}


#[macro_export]
macro_rules! events {
    (
        $room:expr,
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => $room_ref:ident, $payload:ident $event_block:block, $($tokens:tt)+
    ) => {


    };

    (
        $room:expr,
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => $room_ref:ident, $payload:ident : $payload_type:ty $event_block:block, $($tokens:tt)+
    ) => {


        $room.insert_event($event_name.to_owned(), Box::new(|room: Arc<Room>|{

            $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+
     
             Box::pin(async move{$event_block})
     
         }));

        events!($room, $($tokens)+);
        
    };

    (
        $room:expr,
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => $room_ref:ident, $payload:ident $event_block:block
    ) => {

       
        
    };

    (
        $room:expr,
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => $room_ref:ident, $payload:ident:$payload_type:ty $event_block:block
    ) => {

        $room.insert_event($event_name.to_owned(), Box::new(|room: Arc<Room>|{
            
            $(let $data =  room.share_data::<$data_type>();)+
     
             Box::pin(async move{$event_block})
     
         }));            
    };

    (
        $room:expr,
        $event_name:expr => $room_ref:ident, $payload:ident $event_block:block, $($tokens:tt)+
    ) => {
    };

    (
        $room:expr,
        $event_name:expr => $room_ref:ident, $payload:ident:$payload_type:ty $event_block:block, $($tokens:tt)+
    ) => {

        events!($room, $($tokens)+);
        
    };




    (
        $room:expr,
        $event_name:expr => $room_ref:ident, $payload:ident $event_block:block
    ) => {
       
    };

    (
        $room:expr,
        $event_name:expr => $room_ref:ident, $payload:ident:$payload_type:ty $event_block:block
    ) => {

    };
}

