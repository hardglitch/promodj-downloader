use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Command {
    Start,
    Pause,
    Stop,
    Message,
    Search,
}
#[derive(Debug)]
pub struct Data {
    command: Command,          // command
    payload: Option<Vec<u8>>   // payload
}
impl Data {
    pub fn new<'a, T: Serialize + Deserialize<'a>>(command: Command, payload: T) -> Self {
        let payload = serde_json::to_vec(&payload).ok();
        Self { command, payload }
    }
}
