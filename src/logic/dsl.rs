use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Command {
    Start,
    Pause,
    Stop,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Data<T: ?Sized> {
    pub command: Command,    // command
    pub payload: T           // payload
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Request<T: ?Sized> {
    pub data: Data<T>        // data
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Response<R: ?Sized> {
    pub data: R              // data
}
