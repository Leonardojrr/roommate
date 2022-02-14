pub mod room;
pub mod callback;
pub mod connection;

#[macro_export]
macro_rules! run_server{
    ($port:expr, $($room:ident =>[$($room_to_connect:ident),*]),*)=>{

        {
            let socket = tokio::net::TcpListener::bind($port).await.unwrap();
            let mut room_channels = HashMap::new();

            $(
               let room_ref = $room.room.lock().await;
               room_channels.insert(room_ref.name(), room_ref.conn_channel());
               drop(room_ref);
            )*

            let ws_socket = SocketListener{socket, room_channels};

            $(
                let mut room_ref = $room.room.lock().await;
                $(room_ref.connect_room($room_to_connect.room.clone()).await;)*
                drop(room_ref);
            )*

            (
                tokio::spawn(async move {ws_socket.listen().await})
                $(,tokio::spawn(async move {$room.run().await}))*
            )
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
                let room_instance: Arc<Mutex<RoomInfo<$state>>> = RoomInfo::new(String::from(room_name), Some($state_init));
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
                let room_instance: Arc<Mutex<RoomInfo<EmptyState>>> = RoomInfo::new(String::from(room_name), None);
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