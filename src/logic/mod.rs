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
    Message(&'static str),
    Progress(f32),
    Search(usize),
}
