use serde_json::{from_str, json, Value};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Clone)]
pub enum Emiter {
    User(Uuid),
    Room(String),
}

pub enum Error {
    CommandNotfound(String),
    CommandIsNotAString,
    NeedMoreArguments,
    NotAJson,
    NoCommandIncluded,
}

impl Into<String> for Error {
    fn into(self) -> String {
        match self {
            Self::CommandNotfound(unknown_command) => {
                format!("The command {unknown_command} doesn't exist")
            }

            Self::NeedMoreArguments => "This command needs more argument to work".to_string(),

            Self::CommandIsNotAString => "The command property has to be a string".to_string(),

            Self::NoCommandIncluded => "The data doesn't include a command property".to_string(),

            Self::NotAJson => "The data sended is not in a json format".to_string(),
        }
    }
}

impl Into<Value> for Error {
    fn into(self) -> Value {
        json!({"event" : "error", "message": Into::<String>::into(self)})
    }
}

#[derive(Clone)]
pub enum User {
    //Event of User
    Event(String, Value),

    //Connect to room
    ConnectRoom(String),

    //Disconnect to room
    DisconnectRoom(String),

    //Close User stream
    Close,
}

impl TryFrom<String> for User {
    type Error = crate::protocol::Error;

    fn try_from(value: String) -> Result<Self, Error> {
        let result = from_str::<Value>(&value);

        let json = match result {
            Ok(json) => json,
            Err(_) => return Err(Error::NotAJson),
        };

        let command = match &json["command"] {
            Value::String(command) => command,

            Value::Null => return Err(Error::NoCommandIncluded),
            _ => return Err(Error::CommandIsNotAString),
        };

        let user_protocol = match command.as_str() {
            "message" => {
                let event = match &json["event"] {
                    Value::String(event) => event.clone(),
                    _ => return Err(Error::NeedMoreArguments),
                };

                let data = match json.get("data") {
                    Some(data) => data,
                    None => return Err(Error::NeedMoreArguments),
                };

                User::Event(event, data.clone())
            }

            "connect" | "disconnect" => {
                let room = match &json["room"] {
                    Value::String(room) => room.clone(),
                    _ => return Err(Error::NeedMoreArguments),
                };

                match command.as_str() {
                    "connect" => User::ConnectRoom(room),
                    "disconnect" => User::DisconnectRoom(room),

                    _ => panic!(),
                }
            }

            "close" => User::Close,

            _ => return Err(Error::CommandNotfound(command.clone())),
        };

        return Ok(user_protocol);
    }
}

impl Into<Value> for User {
    fn into(self) -> Value {
        match self {
            User::Event(event_name, data) => {
                json!({"event": event_name, "data": data})
            }

            User::Close => json!({"event": "close"}),

            _ => json!(null),
        }
    }
}

#[derive(Clone)]
pub enum Room {
    //Event of Room
    Event(String, Value, Emiter),

    //Connect User
    ConnectUser(Uuid, UnboundedSender<User>),

    //Disconnect User
    DisconnectUser(Uuid),

    Close,
}
