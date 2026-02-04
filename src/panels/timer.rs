use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::panel::{KeyHandleResult, Panel, PanelId, Shortcut};
use crate::timer::{SessionType, Timer, TimerState};

pub struct TimerPanel {
    pub timer: Timer,
}

impl Default for TimerPanel {
    fn default() -> Self {
        Self {
            timer: Timer::default(),
        }
    }
}

impl Panel for TimerPanel {
    fn id(&self) -> PanelId {
        PanelId::Timer
    }

    fn title(&self) -> &str {
        "Timer"
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
            .title(" Timer ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split inner area: timer display at top, current task at bottom
        let chunks = Layout::vertical([
            Constraint::Min(8),    // Timer display
            Constraint::Length(3), // Current task (single task)
        ])
        .split(inner);

        self.render_timer_display(frame, chunks[0]);
        self.render_current_task(frame, chunks[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyHandleResult {
        match key.code {
            KeyCode::Char(' ') => {
                self.timer.toggle();
                KeyHandleResult::Consumed
            }
            KeyCode::Char('r') => {
                self.timer.reset();
                KeyHandleResult::Consumed
            }
            _ => KeyHandleResult::Ignored,
        }
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "Space",
                description: "Start/Pause",
            },
            Shortcut {
                key: "r",
                description: "Reset",
            },
        ]
    }

    fn tick(&mut self) {
        self.timer.tick();
    }
}

impl TimerPanel {
    fn render_timer_display(&self, frame: &mut Frame, area: Rect) {
        let timer = &self.timer;

        let time_str = format!("{:02}:{:02}", timer.minutes(), timer.seconds());

        let session_str = match timer.session_type {
            SessionType::Work => "WORK SESSION",
            SessionType::ShortBreak => "SHORT BREAK",
            SessionType::LongBreak => "LONG BREAK",
        };

        let state_str = match timer.state {
            TimerState::Idle => "Press [Space] to start",
            TimerState::Running => "Running...",
            TimerState::Paused => "Paused",
        };

        let session_color = match timer.session_type {
            SessionType::Work => Color::Red,
            SessionType::ShortBreak => Color::Green,
            SessionType::LongBreak => Color::Blue,
        };

        let content = vec![
            Line::from(""),
            Line::from(Span::styled(
                time_str,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(session_str, Style::default().fg(session_color))),
            Line::from(""),
            Line::from(Span::styled(state_str, Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(format!("Sessions completed: {}", timer.sessions_completed)),
        ];

        let paragraph = Paragraph::new(content).alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_current_task(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Current Task ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Placeholder for single current task (Phase 3)
        let placeholder = Paragraph::new("No task selected")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        frame.render_widget(placeholder, inner);
    }
}
