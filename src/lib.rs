pub mod connection;
pub mod data;
pub mod event;
pub mod prelude;
pub mod protocol;
pub mod room;
mod room_builder;
mod user;

// #[macro_export]
// macro_rules! run_server{
//     ($port:expr, $($tokens:tt)+)=>{
//         {
//             let room_channels = HashMap::new();
//             let mut socket = SocketListener::new($port, room_channels);

//             let mut handlers_list = router!(socket, $($tokens)+);
//             let handler = tokio::spawn(async move {socket.listen().await});

//             handlers_list.push(handler);

//             tokio::spawn(async move{
//                 for handler in handlers_list{
//                     let _ = handler.await;
//                  }
//             })
//         }
//     };
// }

#[macro_export]
macro_rules! router {
    ($($room: ident => [$($other_room: ident),+]),+) => {{
        let mut rooms: HashMap<String, UnboundedSender<protocol::Room>> = HashMap::new();

        $(rooms.insert($room.name.clone(), $room.sender.clone());)+
        $(
            {
                $($room.connect_room($other_room.clone()).await;)+
            }
        )+

        rooms
    }};
}
