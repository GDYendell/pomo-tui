use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

#[derive(Clone)]
pub struct Shortcut {
    pub key: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyHandleResult {
    Consumed,
    Ignored,
}

pub trait Panel {
    fn id(&self) -> PanelId;
    fn title(&self) -> &str;
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool);
    fn handle_key(&mut self, key: KeyEvent) -> KeyHandleResult;
    fn shortcuts(&self) -> Vec<Shortcut>;
    fn tick(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    Timer,
    Tasks,
}

impl PanelId {
    pub fn all() -> &'static [PanelId] {
        &[PanelId::Timer, PanelId::Tasks]
    }

    pub fn from_number(n: u8) -> Option<PanelId> {
        match n {
            1 => Some(PanelId::Timer),
            2 => Some(PanelId::Tasks),
            _ => None,
        }
    }
}
