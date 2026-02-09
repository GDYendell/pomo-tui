use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};

pub fn panel_block(title: &str, focused: bool) -> Block<'_> {
    let color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title)
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
