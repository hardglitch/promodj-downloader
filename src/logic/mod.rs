pub mod search;
pub mod file;
pub mod proxy;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub enum Command {
    Start,
    Pause,
    Stop,
    Message(String),
    Progress(f32, usize, usize),
    Success,
    Started,
}
