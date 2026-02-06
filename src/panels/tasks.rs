use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::panel::{KeyHandleResult, Shortcut};

pub struct TasksPanel;

impl Default for TasksPanel {
    fn default() -> Self {
        Self
    }
}

impl TasksPanel {
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_color = if focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Tasks ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into three equal sections: Backlog, Current, Completed
        let chunks = Layout::vertical([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(inner);

        self.render_backlog_section(frame, chunks[0]);
        self.render_current_section(frame, chunks[1]);
        self.render_completed_section(frame, chunks[2]);
    }

    pub fn handle_key(&mut self, _key: KeyEvent) -> KeyHandleResult {
        KeyHandleResult::Ignored
    }

    pub fn shortcuts(&self) -> Vec<Shortcut> {
        vec![]
    }

    fn render_backlog_section(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Backlog ")
            .title_style(Style::default().fg(Color::DarkGray))
            .title_alignment(Alignment::Right);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let placeholder = Paragraph::new("(Phase 3)")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, inner);
    }

    fn render_current_section(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Current ")
            .title_style(Style::default().fg(Color::DarkGray))
            .title_alignment(Alignment::Right);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let placeholder = Paragraph::new("(Phase 3)")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, inner);
    }

    fn render_completed_section(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::NONE)
            .title(" Completed ")
            .title_style(Style::default().fg(Color::DarkGray))
            .title_alignment(Alignment::Right);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let placeholder = Paragraph::new("(Phase 3)")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, inner);
    }
}
