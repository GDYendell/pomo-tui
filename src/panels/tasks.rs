use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::panel::{KeyHandleResult, Panel, PanelId, Shortcut};

pub struct TasksPanel;

impl Default for TasksPanel {
    fn default() -> Self {
        Self
    }
}

impl Panel for TasksPanel {
    fn id(&self) -> PanelId {
        PanelId::Tasks
    }

    fn title(&self) -> &str {
        "Tasks"
    }

    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
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

        // Split into three sections: Backlog, Current, Completed
        let chunks = Layout::vertical([
            Constraint::Percentage(40), // Backlog
            Constraint::Percentage(30), // Current
            Constraint::Percentage(30), // Completed
        ])
        .split(inner);

        self.render_backlog_section(frame, chunks[0]);
        self.render_current_section(frame, chunks[1]);
        self.render_completed_section(frame, chunks[2]);
    }

    fn handle_key(&mut self, _key: KeyEvent) -> KeyHandleResult {
        KeyHandleResult::Ignored
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![]
    }
}

impl TasksPanel {
    fn render_backlog_section(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Backlog ");

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
            .title(" Current ");

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
            .title(" Completed ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let placeholder = Paragraph::new("(Phase 3)")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, inner);
    }
}
