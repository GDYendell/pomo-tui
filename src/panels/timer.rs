use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::digits::{render_time, render_wave, wave_position};
use crate::panel::Shortcut;
use crate::timer::{SessionType, Timer, TimerState};

pub struct TimerPanel {
    tick_count: u32,
}

impl Default for TimerPanel {
    fn default() -> Self {
        Self { tick_count: 0 }
    }
}

impl TimerPanel {
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, timer: &Timer) {
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
            Constraint::Min(10),   // Timer display (block digits need more space)
            Constraint::Length(3), // Current task (single task)
        ])
        .split(inner);

        self.render_timer_display(frame, chunks[0], timer);
        self.render_current_task(frame, chunks[1]);
    }

    pub fn shortcuts(&self, timer: &Timer) -> Vec<Shortcut> {
        let mut shortcuts = vec![
            Shortcut {
                key: "Space",
                description: "Start/Pause",
            },
            Shortcut {
                key: "R",
                description: "Reset",
            },
        ];

        // Show mode switching shortcuts only when idle
        if timer.state == TimerState::Idle {
            shortcuts.push(Shortcut {
                key: "W/S/L",
                description: "Mode",
            });
        }

        shortcuts
    }

    /// Update animation tick counter without ticking the timer
    pub fn tick_animation(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    fn render_timer_display(&self, frame: &mut Frame, area: Rect, timer: &Timer) {
        // Render block digits
        let time_lines = render_time(timer.minutes(), timer.seconds());

        let session_str = match timer.session_type {
            SessionType::Work => "WORK",
            SessionType::ShortBreak => "SHORT BREAK",
            SessionType::LongBreak => "LONG BREAK",
        };

        let session_color = match timer.session_type {
            SessionType::Work => Color::Red,
            SessionType::ShortBreak => Color::Green,
            SessionType::LongBreak => Color::Blue,
        };

        // Wave animation - only animate when running
        let wave = if timer.state == TimerState::Running {
            render_wave(Some(wave_position(self.tick_count)))
        } else {
            render_wave(None)
        };

        let mut content: Vec<Line> = vec![Line::from("")];

        // Add block digit lines
        for line in time_lines {
            content.push(Line::from(Span::styled(
                line,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            wave,
            Style::default().fg(session_color),
        )));
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            session_str,
            Style::default().fg(session_color),
        )));
        content.push(Line::from(format!(
            "Sessions: {}",
            timer.sessions_completed
        )));

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
