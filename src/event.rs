use crate::{protocol, room::Room};
use serde_json::Value;
use std::{future::Future, pin::Pin, sync::Arc};

pub type Callback = Box<
    dyn Fn(Arc<Room>, protocol::Emiter, Value) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident) $event_block:block, $($tokens:tt)+
    ) => {{
        let mut events_map: HashMap<String, Callback> = HashMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident : $payload_type:ty) $event_block:block, $($tokens:tt)+
    ) => {{
        let mut events_map: HashMap<String, Callback> = HashMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident) $event_block:block
    ) => {
        $events_map.insert(String::from(
            $event_name,
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
                    $(let $data: Data<$data_type> =  room.share_data::<$data_type>();)+

                    let $room_ref = room;
                    let $emiter = emiter;
                    let $payload = payload;

                    Box::pin(async move { $event_block })
                },
            ),
        ));
    };

    ($events_map:ident,
        #[$($data:ident : $data_type:ty),+]
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident : $payload_type:ty) $event_block:block
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident) $event_block:block
    ) => {{
        let mut events_map: HashMap<String, Callback> = HashMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident : $payload_type:ty) $event_block:block
    ) => {{
        let mut events_map: HashMap<String, Callback> = HashMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident) $event_block:block, $($tokens:tt)+
    ) => {{
        let mut events_map: HashMap<String, Callback> = HashMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident : $payload_type:ty) $event_block:block, $($tokens:tt)+
    ) => {{
        let mut events_map: HashMap<String, Callback> = HashMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident) $event_block:block
    ) => {{
        let mut events_map: HashMap<String, Callback> = HashMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident : $payload_type:ty) $event_block:block
    ) => {{
        let mut events_map: HashMap<String, Callback> = HashMap::new();

        events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident) $event_block:block, $($tokens:tt)+
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident : $payload_type:ty) $event_block:block, $($tokens:tt)+
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident) $event_block:block, $($tokens:tt)+
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
        $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident:$payload_type:ty) $event_block:block, $($tokens:tt)+
    ) => {
        $events_map.insert(String::from($event_name),
            Box::new(
                |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
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
            $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident) $event_block:block
        ) => {
            $events_map.insert(String::from($event_name),
                Box::new(
                    |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
                        let $room_ref = room;
                        let $emiter = emiter;
                        let $payload = payload;

                        Box::pin(async move { $event_block })
                    },
                ),
            );
        };

    ($events_map:ident,
            $event_name:expr => ($room_ref:ident, $emiter:ident, $payload:ident:$payload_type:ty) $event_block:block
        ) => {
            $events_map.insert(String::from($event_name),
                Box::new(
                    |room: Arc<Room>, emiter: protocol::Emiter, payload: Value| {
                        let $room_ref = room;
                        let $emiter = emiter;
                        let $payload : Result<$payload_type, _> = TryFrom::<Value>::try_from(payload);

                        Box::pin(async move { $event_block })
                    },
                ),
            );
        };
}
