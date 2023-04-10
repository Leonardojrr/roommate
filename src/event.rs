use crate::{protocol, room::Room};
use serde_json::Value;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

pub type Callback = Box<
    dyn Fn(Arc<Room>, Value, protocol::Emiter) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

pub struct EventMap {
    hashmap: HashMap<Option<String>, HashMap<String, Callback>>,
}

impl EventMap {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
        }
    }

    pub fn get(&self, value: &Option<String>) -> Option<&HashMap<String, Callback>> {
        self.hashmap.get(value)
    }

    pub fn get_default_event(&self, event_name: &str) -> Option<&Callback> {
        match self.get(&None) {
            Some(default_events_map) => default_events_map.get(event_name),
            None => None,
        }
    }

    pub fn insert(&mut self, event: String, callback: Callback) {
        let event_name: Vec<&str> = event.split(':').map(|string| string.trim()).collect();

        println!("{}", event_name.len());

        match event_name.len() {
            2 => {
                self.hashmap.insert(
                    Some(event_name[0].into()),
                    HashMap::from([(event_name[1].into(), callback)]),
                );
            }

            1 => {
                self.hashmap
                    .insert(None, HashMap::from([(event, callback)]));
            }

            _ => panic!("The event '{event}' is not in the right format"),
        }
    }

    pub fn insert_eventmap(&mut self, eventmap: EventMap) {
        for (k, v) in eventmap.hashmap.into_iter() {
            self.hashmap.insert(k, v);
        }
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
