use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::panel::{Panel, PanelId};

pub struct AppLayout {
    pub title: Rect,
    pub timer: Rect,
    pub tasks: Option<Rect>,
    pub shortcuts_bar: Rect,
}

pub fn create_layout(area: Rect, tasks_visible: bool) -> AppLayout {
    // Main vertical split: title, main content, shortcuts
    let main_chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Min(10),   // Main content
        Constraint::Length(3), // Shortcuts bar
    ])
    .split(area);

    let (timer_area, tasks_area) = if tasks_visible {
        // Split horizontally: timer (left 50%) and tasks (right 50%)
        let content_chunks = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunks[1]);
        (content_chunks[0], Some(content_chunks[1]))
    } else {
        // Timer takes full width
        (main_chunks[1], None)
    };

    AppLayout {
        title: main_chunks[0],
        timer: timer_area,
        tasks: tasks_area,
        shortcuts_bar: main_chunks[2],
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let layout = create_layout(frame.area(), app.tasks_visible);

    render_title(frame, layout.title);

    // Render timer panel (always visible)
    app.panels
        .timer
        .render(frame, layout.timer, app.focused_panel == PanelId::Timer);

    // Render tasks panel if visible
    if let Some(tasks_area) = layout.tasks {
        app.panels
            .tasks
            .render(frame, tasks_area, app.focused_panel == PanelId::Tasks);
    }

    render_shortcuts_bar(frame, layout.shortcuts_bar, app);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("POMODORO TIMER")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_shortcuts_bar(frame: &mut Frame, area: Rect, app: &App) {
    let mut shortcuts: Vec<Span> = vec![];

    // Panel-specific shortcuts first
    let focused_panel = app.panels.get(app.focused_panel);
    for shortcut in focused_panel.shortcuts() {
        shortcuts.push(Span::styled(
            format!("[{}]", shortcut.key),
            Style::default().fg(Color::Yellow),
        ));
        shortcuts.push(Span::raw(format!(" {}  ", shortcut.description)));
    }

    // Global shortcuts on the right
    shortcuts.push(Span::styled("[Tab]", Style::default().fg(Color::Yellow)));
    shortcuts.push(Span::raw(" Next  "));
    shortcuts.push(Span::styled("[1-2]", Style::default().fg(Color::Yellow)));
    shortcuts.push(Span::raw(" Focus  "));
    shortcuts.push(Span::styled("[T]", Style::default().fg(Color::Yellow)));
    shortcuts.push(Span::raw(" Tasks  "));
    shortcuts.push(Span::styled("[q]", Style::default().fg(Color::Yellow)));
    shortcuts.push(Span::raw(" Quit"));

    let line = Line::from(shortcuts);
    let paragraph = Paragraph::new(line)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}
