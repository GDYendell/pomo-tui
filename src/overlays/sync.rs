use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use ratatui_input_manager::keymap;

use super::util::centered_rect;
use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncResolution {
    Incomplete,
    Complete,
    Remove,
}

/// A task item to sync with desired resolution (incomplete/complete/remove)
#[derive(Debug, Clone)]
pub struct SyncItem {
    pub text: String,
    pub resolution: SyncResolution,
}

/// Overlay for reviewing and applying task file sync changes
pub struct SyncOverlay {
    items: Vec<SyncItem>,
    focused: usize,
    dismissed: bool,
    applied: bool,
}

impl SyncOverlay {
    pub fn new(items: Vec<SyncItem>) -> Self {
        Self {
            items,
            focused: 0,
            dismissed: false,
            applied: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.dismissed || self.applied
    }

    /// Returns the sync items to apply, or None if dismissed
    pub fn result(&self) -> Option<&[SyncItem]> {
        self.applied.then_some(&self.items)
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

#[keymap(backend = "crossterm")]
impl SyncOverlay {
    /// Cancel
    #[keybind(pressed(key=KeyCode::Esc))]
    fn dismiss(&mut self) {
        self.dismissed = true;
    }

    /// Apply changes
    #[keybind(pressed(key=KeyCode::Enter))]
    fn apply(&mut self) {
        self.applied = true;
    }

    /// Move focus down
    #[keybind(pressed(key=KeyCode::Char('j')))]
    fn move_down(&mut self) {
        if !self.items.is_empty() && self.focused + 1 < self.items.len() {
            self.focused += 1;
        }
    }

    /// Move focus up
    #[keybind(pressed(key=KeyCode::Char('k')))]
    fn move_up(&mut self) {
        if self.focused > 0 {
            self.focused -= 1;
        }
    }

    /// Mark as incomplete
    #[keybind(pressed(key=KeyCode::Char(' ')))]
    fn mark_incomplete(&mut self) {
        if let Some(item) = self.items.get_mut(self.focused) {
            item.resolution = SyncResolution::Incomplete;
        }
    }

    /// Mark as complete
    #[keybind(pressed(key=KeyCode::Char('x')))]
    fn mark_complete(&mut self) {
        if let Some(item) = self.items.get_mut(self.focused) {
            item.resolution = SyncResolution::Complete;
        }
    }

    /// Mark for removal
    #[keybind(pressed(key=KeyCode::Char('d')))]
    fn mark_remove(&mut self) {
        if let Some(item) = self.items.get_mut(self.focused) {
            item.resolution = SyncResolution::Remove;
        }
    }
}
