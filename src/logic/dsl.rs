use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Command {
    Start,
    Pause,
    Stop,
    Message,
    Search,
    Temp,
}
#[derive(Debug)]
pub struct Data {
    command: Command,          // command
    payload: Option<Vec<u8>>   // payload
}
impl Data {
    pub fn new<T: Serialize>(command: Command, payload: T) -> Self {
        let payload = serde_json::to_vec(&payload).ok();
        Self { command, payload }
    }
    #[inline]
    pub fn command(&self) -> Command {
        self.command
    }
    #[inline]
    pub fn payload<T: DeserializeOwned>(self) -> Option<T> {
        let pl = self.payload?;
        let value = serde_json::from_slice::<T>(&pl).ok()?;
        Some(value)
    }
}
