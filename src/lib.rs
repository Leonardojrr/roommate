mod callback;
mod connection;
mod error;
pub mod prelude;
mod room;

#[macro_export]
macro_rules! run_server{
    ($port:expr, $($tokens:tt)+)=>{
            let room_channels = HashMap::new();
            let mut socket = SocketListener::new($port, room_channels);

            let mut handlers_list = router!(socket, $($tokens)+);
            let handler = tokio::spawn(async move {socket.listen().await});

            handlers_list.push(handler);

            for handler in handlers_list{
               let _ = handler.await;
            }
    };
}

#[macro_export]
macro_rules! router{

    ($socket_listener:ident, $room_ident:ident => [$($room_to_connect:ident),*], $($tokens:tt)+)=>{

        {
            let mut room_ref = $room_ident.room.lock().await;

            $socket_listener.connect_room(room_ref.name(), room_ref.conn_channel());
            $(room_ref.connect_room($room_to_connect.room.clone()).await;)*

            drop(room_ref);

            let mut handlers_list = router!($socket_listener, $($tokens)+);
            let handler = tokio::spawn(async move {$room_ident.run().await});

            handlers_list.push(handler);

            handlers_list
        }
    };

    ($socket_listener:ident, $room_ident:ident, $($tokens:tt)+)=>{

        {
            let mut room_ref = $room_ident.room.lock().await;

            $socket_listener.connect_room(room_ref.name(), room_ref.conn_channel());

            drop(room_ref);

            let mut handlers_list = router!($socket_listener, $($tokens)+);
            let handler = tokio::spawn(async move {$room_ident.run().await});

            handlers_list.push(handler);

            handlers_list
        }
    };

    ($socket_listener:ident, $room_ident:ident => [$($room_to_connect:ident),*]) =>{

        {
            let mut room_ref = $room_ident.room.lock().await;

            $socket_listener.connect_room(room_ref.name(), room_ref.conn_channel());
            $(room_ref.connect_room($room_to_connect.room.clone()).await;)*

            drop(room_ref);

            vec![tokio::spawn(async move {$room_ident.run().await})]
        }
    };

    ($socket_listener:ident, $room_ident:ident)=>{

        {
            let mut room_ref = $room_ident.room.lock().await;

            $socket_listener.connect_room(room_ref.name(), room_ref.conn_channel());

            drop(room_ref);

            vec![tokio::spawn(async move {$room_ident.run().await})]
        }
    };
}
