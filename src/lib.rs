mod room;
mod callback;
mod connection;
pub mod prelude;

#[macro_export]
macro_rules! run_server{
    ($port:expr, $($tokens:tt)+)=>{
            let room_channels = HashMap::new();
            let mut socket = SocketListener::new($port, room_channels);

            let mut handlers_list = router!(socket, $($tokens)+);
            let handler = tokio::spawn(async move {socket.listen().await});

            handlers_list.push(handler);

            for handler in handlers_list{
                handler.await;
            }
    };
}


#[macro_export]
macro_rules! room {
    (   
        $room_name:ident<$state:tt>{
            $state_init:expr,
            $($event_name:tt => $room:ident, $data:ident $event_block:block), *
        }
    ) =>{
            
                let room_name = stringify!($room_name);
                let room_instance: Arc<Mutex<RoomInfo<$state>>> = RoomInfo::new(String::from(room_name), $state_init);
                let mut $room_name = Room::new(&room_instance);
            {
                $(
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
                )*
            }
    };

    (   
        $room_name:ident{
            $($event_name:tt => $room:ident, $data:ident $event_block:block), *
        }
    ) =>{

                let room_name = stringify!($room_name);
                let room_instance: Arc<Mutex<RoomInfo<EmptyState>>> = RoomInfo::new(String::from(room_name), EmptyState);
                let mut $room_name = Room::new(&room_instance);

                {
                    $(
                        fn $event_name(room: Arc<Mutex<RoomInfo<EmptyState>>>, data:String) -> CallbackFut  {
                            let future = async move{
                                let mut $room = room.lock().await;
                                let $data = data;
                                $event_block;
                            };
    
                            CallbackFut{fut: Box::pin(future)}
                        }
    
                        let event_wrapper = $event_name;
                        $room_name.insert_event(stringify!($event_name), event_wrapper);
                    )*
                }
    }
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