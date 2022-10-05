# Motivation
I started this project to create a library in rust to make bidirectional servers using webscokets. Keep in mind this library I made it in a way that would be very easy to start and create a server with the help of rust macros. My main goal was productivity and the easynest to work with this library. A popular library made in javascript I took as a reference is [socket.io](https://socket.io/).

***Note:*** I do not recommend to use this library for a production product until the performance of this one would be tested properly by side non critical projects.

# Example

```rust
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
```

***Note:*** To see this example more in depth you can go to [this repository](https://github.com/Leonardojrr/Chat-app).

---


### If you want to collaborate to this project, please feel free to write a message or to send me a pull request