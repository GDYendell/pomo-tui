#[derive(Clone)]
pub struct Shortcut {
    pub key: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyHandleResult {
    #[allow(dead_code)] // Will be used in Phase 3
    Consumed,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    Timer,
    Tasks,
}
