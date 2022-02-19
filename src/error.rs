use tungstenite::Error;

pub enum ErrorKind {
    Msg(Error),
    Connection(usize),
}
