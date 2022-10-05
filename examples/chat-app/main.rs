use roommate::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Msg {
    kind: String,
    data: Value,
}

#[derive(Serialize, Deserialize, Clone)]
struct MsgList {
    messages: Vec<Msg>,
}

impl MsgList {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    let (controller) = room! {
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
    };

    let handler = run_server!("192.168.1.118:8080", runner);

    let mut room = controller.ctx().await;

    room.broadcast("message", String::from("CULOO")).await;
}
