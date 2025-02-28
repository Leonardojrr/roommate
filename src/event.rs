use crate::{protocol, room::Room};
use serde_json::Value;
use std::{
    collections::HashMap,
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
};

pub type BoxFut = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type Event = Box<dyn Fn(Arc<Room>, Value, protocol::Emiter) -> BoxFut + Send + Sync>;

pub struct EventMap {
    events: HashMap<String, Event>,
}

impl EventMap {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }
}

impl Deref for EventMap {
    type Target = HashMap<String, Event>;

    fn deref(&self) -> &Self::Target {
        &self.events
    }
}

impl DerefMut for EventMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.events
    }
}

#[macro_export]
macro_rules! event {
    (
        $event_name:expr,
        #[$($data:ident : $data_type:ty),+]
        ($room_ref:ident, $payload:ident)
        $event_block:block
    ) => {{
        let event: (String, Callback) = (String::from($event_name),  Box::new(|room: Arc<Room>, emiter: protocol::Emiter, payload:Value|{

            $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+
            let $room_ref = room;
            let $payload = payload;

             Box::pin(async move{$event_block})

         }));

         event
    }};

    (
        $event_name:expr,
        #[$($data:ident : $data_type:ty),+]
        ($room_ref:ident, $payload:ident : $payload_type:ty)
        $event_block:block
    ) => {

        (String::from($event_name),  Box::new(|room: Arc<Room>|{

            $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+
            $room_ref = room;

             Box::pin(async move{$event_block})
        }))
    };

    (
        $event_name:expr,
        ($room_ref:ident, $payload:ident)
        $event_block:block
    ) => {

        {
            let event : (String, Callback) = (String::from($event_name), Box::new(|room: Arc<Room>, emiter: protocol::Emiter, payload:Value|{
                let $room_ref = room;
                let $payload = payload;

                 Box::pin(async move{$event_block})
            }));

            event
        }
    };
}

#[macro_export]
macro_rules! events {
    (
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident ) $event_block:block, $($tokens:tt)+
    ) => {{
        let mut events_map = EventMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+

                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload = payload;

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events!(events_map, $($tokens)+);

        events_map
    }};

    (
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => ($room_ref:ident, $payload:ident : $payload_type:ty, $emiter:ident ) $event_block:block, $($tokens:tt)+
    ) => {{
        let mut events_map = EventMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    $(let $data: Data<$data_type> = room.share_data::<$data_type>();)+

                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload : Result<$payload_type, _> = TryFrom::<Value>::try_from(payload);

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events!(events_map, $($tokens)+);

        events_map
    }};

    ($events_map:ident,
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident ) $event_block:block
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+

                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload = payload;

                    Box::pin(async move { $event_block })
                },
            ),
        );
    };

    ($events_map:ident,
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => ($room_ref:ident, $payload:ident : $payload_type:ty, $emiter:ident ) $event_block:block
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+

                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload : Result<$payload_type, _> = TryFrom::<Value>::try_from(payload);

                    Box::pin(async move { $event_block })
                },
            ),
        );
    };

    (
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident ) $event_block:block
    ) => {{
        let mut events_map = EventMap::new();


        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+

                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload = payload;

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events_map
    }};

    (
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => ($room_ref:ident, $payload:ident : $payload_type:ty, $emiter:ident ) $event_block:block
    ) => {{
        let mut events_map = EventMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+

                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload : Result<$payload_type, _> = TryFrom::<Value>::try_from(payload);

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events_map
    }};

    (
        $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident ) $event_block:block, $($tokens:tt)+
    ) => {{
        let mut events_map = EventMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload = payload;

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events!(events_map, $($tokens)+);

        events_map
    }};

    (
        $event_name:expr => ($room_ref:ident, $payload:ident : $payload_type:ty, $emiter:ident ) $event_block:block, $($tokens:tt)+
    ) => {{
        let mut events_map = EventMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    let $room_ref = room;
                    let $emiter = emiter;

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events!(events_map, $($tokens)+);

        events_map
    }};
////////////////////////////////////////////////////////////////////


    (
        $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident ) $event_block:block
    ) => {{
        let mut events_map = EventMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload = payload;

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events_map
    }};

    (
        $event_name:expr => ($room_ref:ident, $payload:ident : $payload_type:ty, $emiter:ident ) $event_block:block
    ) => {{
        let mut events_map = EventMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload: Result<$payload_type, _> = TryFrom::<Value>::try_from(payload);

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events_map
    }};

    //////////////////////////////////////////////////////////////
    ($events_map:ident,
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident ) $event_block:block, $($tokens:tt)+
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+

                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload = payload;

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events!($events_map, $($tokens)+);

    };

    ($events_map:ident,
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => ($room_ref:ident, $payload:ident : $payload_type:ty, $emiter:ident ) $event_block:block, $($tokens:tt)+
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+

                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload : Result<$payload_type, _> = TryFrom::<Value>::try_from(payload);

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events!($events_map, $($tokens)+);

    };

    ($events_map:ident,
        $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident ) $event_block:block, $($tokens:tt)+
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload = payload;

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events!($events_map, $($tokens)+);

    };

    ($events_map:ident,
        $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident :$payload_type:ty) $event_block:block, $($tokens:tt)+
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload : Result<$payload_type, _> = TryFrom::<Value>::try_from(payload);

                    Box::pin(async move { $event_block })
                },
            ),
        );

        events!($events_map, $($tokens)+);

    };

    //////////////////////////////////////////////////////////////


    ($events_map:ident,
            $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident ) $event_block:block
        ) => {
            $events_map.insert(String::from($event_name),
                Box::new(
                    |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                        let $room_ref = room;
                        let $emiter = emiter;
                        let $payload = payload;

                        Box::pin(async move { $event_block })
                    },
                ),
            );
        };

    ($events_map:ident,
            $event_name:expr => ($room_ref:ident, $payload:ident, $emiter:ident :$payload_type:ty) $event_block:block
        ) => {
            $events_map.insert(String::from($event_name),
                Box::new(
                    |room: Arc<Room>, payload: Value, emiter: protocol::Emiter| {
                        let $room_ref = room;
                        let $emiter = emiter;
                        let $payload : Result<$payload_type, _> = TryFrom::<Value>::try_from(payload);

                        Box::pin(async move { $event_block })
                    },
                ),
            );
        };
}
