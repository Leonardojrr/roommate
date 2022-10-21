use serde_json::{from_str, json, Value};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub enum User {
    //Event of User
    Event(String, String),

    //Connect to room
    ConnectRoom(String),

    //Disconnect to room
    DisconnectRoom(String),

    //Close User stream
    Close,
}

#[derive(Clone)]
pub enum Room {
    //Event of Room
    Event(String, String, Emiter),

    //Connect User
    ConnectUser(Uuid, UnboundedSender<User>),

    //Disconnect User
    DisconnectUser(Uuid),

    Close,
}

#[derive(Clone)]
pub enum Emiter {
    User(Uuid),
    Room(String),
}

pub fn clasify_message(message: String) -> Result<User, String> {
    let object_from_user = match from_str::<Value>(&message) {
        Ok(object_from_user) => object_from_user,

        Err(_) => return Err("there was an error serializing the passed message".to_owned()),
    };

    let command = match object_from_user.get("command") {
        Some(command) => command.as_str().unwrap_or(""),

        None => return Err("There is no command field on the passed message".to_owned()),
    };

    match command {
        "message" => {
            let event = match object_from_user.get("event") {
                Some(event) => event.as_str().unwrap_or(""),

                None => "",
            };

            let data = match object_from_user.get("data") {
                Some(data) => data.as_str().unwrap_or("").to_owned(),

                None => String::from(""),
            };

            Ok(User::Event(event.to_owned(), data))
        }

        "connect" => {
            let room = match object_from_user.get("room") {
                Some(room) => room.as_str().unwrap_or(""),

                None => "",
            };

            Ok(User::ConnectRoom(room.to_owned()))
        }

        "disconnect" => {
            let room = match object_from_user.get("room") {
                Some(room) => room.as_str().unwrap_or(""),

                None => "",
            };

            Ok(User::DisconnectRoom(room.to_owned()))
        }

        "close" => Ok(User::Close),

        _ => return Err("The command: {command} doesn't exist".to_owned()),
    }
}
