pub mod search;
pub mod tools;
pub mod dsl;
pub mod file;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub enum Command {
    Start,
    Pause,
    Stop,
    Message(String),
    Progress(f32, usize, usize),
}
