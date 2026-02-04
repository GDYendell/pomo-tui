use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::digits::TIMER_MIN_WIDTH;
use crate::panel::{Panel, PanelId};

pub struct AppLayout {
    pub timer: Option<Rect>,
    pub tasks: Option<Rect>,
    pub shortcuts_bar: Option<Rect>,
}

pub fn create_layout(area: Rect, app: &App) -> AppLayout {
    // Main vertical split: main content + optional shortcuts
    let main_chunks = if app.shortcuts_visible {
        Layout::vertical([
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Shortcuts bar
        ])
        .split(area)
    } else {
        Layout::vertical([
            Constraint::Min(10), // Main content only
        ])
        .split(area)
    };

    let content_area = main_chunks[0];
    let shortcuts_area = if app.shortcuts_visible {
        Some(main_chunks[1])
    } else {
        None
    };

    let (timer_area, tasks_area) = if app.tasks_visible {
        // Split horizontally: timer (left 50%) and tasks (right 50%)
        let content_chunks = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(content_area);

        if content_chunks[0].width < TIMER_MIN_WIDTH {
            if app.focused_panel == PanelId::Timer {
                (Some(content_area), None)
            } else {
                (None, Some(content_area))
            } 
        } else {
            (Some(content_chunks[0]), Some(content_chunks[1]))
        }
    } else {
        // Timer takes full width
        (Some(content_area), None)
    };

    AppLayout {
        timer: timer_area,
        tasks: tasks_area,
        shortcuts_bar: shortcuts_area,
    }
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = create_layout(frame.area(), app);

    // Update app with whether two columns fit
    app.update_columns(layout.timer.is_some() && layout.tasks.is_some());

    // Render timer panel if visible
    if let Some(timer_area) = layout.timer {
        app.panels
            .timer
            .render(frame, timer_area, app.focused_panel == PanelId::Timer);
    }

    // Render tasks panel if visible
    if let Some(tasks_area) = layout.tasks {
        app.panels
            .tasks
            .render(frame, tasks_area, app.focused_panel == PanelId::Tasks);
    }

    // Render shortcuts bar if visible
    if let Some(shortcuts_area) = layout.shortcuts_bar {
        render_shortcuts_bar(frame, shortcuts_area, app);
    }
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
    shortcuts.push(Span::raw(" Next Panel  "));
    shortcuts.push(Span::styled("[T]", Style::default().fg(Color::Yellow)));
    shortcuts.push(Span::raw(" Tasks  "));
    shortcuts.push(Span::styled("[?]", Style::default().fg(Color::Yellow)));
    shortcuts.push(Span::raw(" Help  "));
    shortcuts.push(Span::styled("[Q]", Style::default().fg(Color::Yellow)));
    shortcuts.push(Span::raw(" Quit"));

    let line = Line::from(shortcuts);
    let paragraph = Paragraph::new(line)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}
