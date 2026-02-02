use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::timer::{SessionType, TimerState};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .split(frame.area());

    render_title(frame, chunks[0]);
    render_timer(frame, chunks[1], app);
    render_controls(frame, chunks[2]);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("POMODORO TIMER")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_timer(frame: &mut Frame, area: Rect, app: &App) {
    let timer = &app.timer;

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
        Line::from(Span::styled(
            session_str,
            Style::default().fg(session_color),
        )),
        Line::from(""),
        Line::from(Span::styled(
            state_str,
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(format!("Sessions completed: {}", timer.sessions_completed)),
    ];

    let paragraph = Paragraph::new(content)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let controls = Line::from(vec![
        Span::styled("[Space]", Style::default().fg(Color::Yellow)),
        Span::raw(" Start/Pause  "),
        Span::styled("[r]", Style::default().fg(Color::Yellow)),
        Span::raw(" Reset  "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit"),
    ]);

    let paragraph = Paragraph::new(controls)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}
