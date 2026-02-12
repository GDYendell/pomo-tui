#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSection {
    Backlog,
    Current,
    Completed,
}

/// A single task with a text description
#[derive(Debug, Clone)]
pub struct Task {
    pub text: String,
}

impl Task {
    pub const fn new(text: String) -> Self {
        Self { text }
    }
}
