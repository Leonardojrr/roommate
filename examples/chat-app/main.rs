use serde::{Serialize, Deserialize};
use serde_json::Value;
use roommate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Msg{
    kind: String,
    data: Value
}

#[derive(Serialize, Deserialize, Clone)]
struct MsgList{
    messages: Vec<Msg>
}

impl MsgList{
    pub fn new() -> Self{
        Self{messages:Vec::new()}
    }
}

#[tokio::main]
async fn main(){

    room!{
        chat<MsgList>{
            MsgList::new(),
        
            get_messages => room, _data{
                let messages = room.get_state().messages.clone();
                room.whisper("messages", messages).await;
            },


            #[Msg]
            message => room, data{
                let state = room.get_mut_state();
                let message = data.unwrap();

                state.messages.push(message.clone());
                room.emit("message", message).await;
            }
        }
    }

    run_server!("192.168.1.118:8080", chat);
}
