use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::util::centered_rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncResolution {
    Incomplete,
    Complete,
    Remove,
}

#[derive(Debug, Clone)]
pub struct SyncItem {
    pub text: String,
    pub resolution: SyncResolution,
}

pub enum SyncAction {
    Consumed,
    Dismiss,
    Apply(Vec<SyncItem>),
}

pub struct SyncOverlay {
    pub items: Vec<SyncItem>,
    pub focused: usize,
}

impl SyncOverlay {
    pub const fn new(items: Vec<SyncItem>) -> Self {
        Self { items, focused: 0 }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SyncAction {
        match key.code {
            KeyCode::Esc => SyncAction::Dismiss,
            KeyCode::Char('j') => {
                if !self.items.is_empty() && self.focused + 1 < self.items.len() {
                    self.focused += 1;
                }
                SyncAction::Consumed
            }
            KeyCode::Char('k') => {
                if self.focused > 0 {
                    self.focused -= 1;
                }
                SyncAction::Consumed
            }
            KeyCode::Char(' ') => {
                if let Some(item) = self.items.get_mut(self.focused) {
                    item.resolution = SyncResolution::Incomplete;
                }
                SyncAction::Consumed
            }
            KeyCode::Char('x') => {
                if let Some(item) = self.items.get_mut(self.focused) {
                    item.resolution = SyncResolution::Complete;
                }
                SyncAction::Consumed
            }
            KeyCode::Char('d') => {
                if let Some(item) = self.items.get_mut(self.focused) {
                    item.resolution = SyncResolution::Remove;
                }
                SyncAction::Consumed
            }
            KeyCode::Enter => {
                let items = std::mem::take(&mut self.items);
                SyncAction::Apply(items)
            }
            _ => SyncAction::Consumed,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(""));

        if self.items.is_empty() {
            lines.push(Line::from(Span::styled(
                "  No changes",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (i, item) in self.items.iter().enumerate() {
                let is_focused = i == self.focused;

                let (checkbox, color) = match item.resolution {
                    SyncResolution::Incomplete => ("[ ]", Color::Blue),
                    SyncResolution::Complete => ("[x]", Color::Green),
                    SyncResolution::Remove => ("[~]", Color::Red),
                };

                let prefix = if is_focused { "> " } else { "  " };
                let prefix_style = if is_focused {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                let text_style = if is_focused {
                    Style::default().fg(color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(color)
                };

                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(prefix, prefix_style),
                    Span::styled(format!("{checkbox} "), Style::default().fg(color)),
                    Span::styled(&item.text, text_style),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("      "),
            Span::styled("[Space]", Style::default().fg(Color::Blue)),
            Span::raw(" "),
            Span::styled("[x]", Style::default().fg(Color::Green)),
            Span::raw(" "),
            Span::styled("[d]", Style::default().fg(Color::Red)),
            Span::raw(" Change State"),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("[j/k]", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Apply "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ]));
        lines.push(Line::from(""));

        let content_height = lines.len() as u16 + 2;
        let overlay_width = 50u16;
        let overlay_height = content_height.min(frame.area().height.saturating_sub(4));

        let overlay_area = centered_rect(frame.area(), overlay_width, overlay_height);
        frame.render_widget(Clear, overlay_area);

        let block = Block::default()
            .title(" Sync ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, overlay_area);
    }
}
