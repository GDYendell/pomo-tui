#[derive(Clone)]
pub struct Shortcut {
    pub key: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyHandleResult {
    Consumed,
    Ignored,
    AddTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    Timer,
    Tasks,
}
