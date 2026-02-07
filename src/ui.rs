use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, SyncDialogue};
use crate::digits::TIMER_MIN_WIDTH;
use crate::panel::PanelId;

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
        let content_chunks =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
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

pub fn render(frame: &mut Frame, app: &App) {
    let layout = create_layout(frame.area(), app);

    // Render timer panel if visible
    if let Some(timer_area) = layout.timer {
        let active_task = if app.timer.session_type == crate::timer::SessionType::Work {
            app.task_manager.active_task()
        } else {
            None
        };
        app.timer_panel.render(
            frame,
            timer_area,
            app.focused_panel == PanelId::Timer,
            &app.timer,
            active_task,
        );
    }

    // Render tasks panel if visible
    if let Some(tasks_area) = layout.tasks {
        app.tasks_panel.render(
            frame,
            tasks_area,
            app.focused_panel == PanelId::Tasks,
            &app.task_manager,
        );
    }

    // Render shortcuts bar if visible
    if let Some(shortcuts_area) = layout.shortcuts_bar {
        render_shortcuts_bar(frame, shortcuts_area, app);
    }

    // Render sync dialogue overlay if active
    if let Some(ref dialogue) = app.sync_dialogue {
        render_sync_dialogue(frame, dialogue);
    }
}

fn render_shortcuts_bar(frame: &mut Frame, area: Rect, app: &App) {
    let mut shortcuts: Vec<Span> = vec![];

    // Panel-specific shortcuts first
    for shortcut in app.focused_shortcuts() {
        shortcuts.push(Span::styled(
            format!("[{}]", shortcut.key),
            Style::default().fg(Color::Yellow),
        ));
        shortcuts.push(Span::raw(format!(" {}  ", shortcut.description)));
    }

    // Global shortcuts on the right
    shortcuts.push(Span::styled("[t]", Style::default().fg(Color::Yellow)));
    shortcuts.push(Span::raw(" Focus  "));
    shortcuts.push(Span::styled("[T]", Style::default().fg(Color::Yellow)));
    shortcuts.push(Span::raw(" Tasks "));
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

fn render_sync_dialogue(frame: &mut Frame, dialogue: &SyncDialogue) {
    use crate::task_manager::SyncResolution;
    use ratatui::style::Modifier;

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from("")); // Empty line at top

    if dialogue.items.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No changes",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, item) in dialogue.items.iter().enumerate() {
            let is_focused = i == dialogue.focused;

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
                Span::styled(format!("{} ", checkbox), Style::default().fg(color)),
                Span::styled(&item.text, text_style),
            ]));
        }
    }

    // Add menu options
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
    lines.push(Line::from("")); // Empty line at bottom

    // Calculate dialogue dimensions
    let content_height = lines.len() as u16 + 2; // +2 for borders
    let dialogue_width = 50u16;
    let dialogue_height = content_height.min(frame.area().height.saturating_sub(4));

    // Center the dialogue
    let area = frame.area();
    let x = area.x + (area.width.saturating_sub(dialogue_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialogue_height)) / 2;
    let dialogue_area = Rect::new(x, y, dialogue_width, dialogue_height);

    // Clear background and render dialogue
    frame.render_widget(Clear, dialogue_area);

    let block = Block::default()
        .title(" Sync ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, dialogue_area);
}
