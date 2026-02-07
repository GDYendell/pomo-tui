#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSection {
    Backlog,
    Current,
    Completed,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub text: String,
}

impl Task {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}
