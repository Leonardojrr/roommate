pub mod connection;
pub mod data;
pub mod event;
pub mod prelude;
pub mod protocol;
pub mod room;
mod room_builder;
mod user;

#[macro_export]
macro_rules! connect_rooms {
    ($($room: ident => [$($other_room: ident),+]),+) => {
        $(
            {
                $($room.connect_room($other_room.clone()).await;)+
            }
        )+
    };
}

#[macro_export]
macro_rules! router {
    ($($room: ident),+) => {{
        let mut rooms: HashMap<String, UnboundedSender<protocol::Room>> = HashMap::new();
        $(rooms.insert($room.name.clone(), $room.sender.clone());)+

        rooms
    }};
}
