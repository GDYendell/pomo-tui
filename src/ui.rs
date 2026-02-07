use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, SyncDialogue, TaskInput};
use crate::digits::TIMER_MIN_WIDTH;
use crate::panel::PanelId;
use crate::task::TaskSection;

pub struct AppLayout {
    pub timer: Option<Rect>,
    pub tasks: Option<Rect>,
}

pub fn create_layout(area: Rect, app: &App) -> AppLayout {
    let (timer_area, tasks_area) = if app.tasks_visible {
        // Split horizontally: timer (left 50%) and tasks (right 50%)
        let content_chunks =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

        if content_chunks[0].width < TIMER_MIN_WIDTH {
            if app.focused_panel == PanelId::Timer {
                (Some(area), None)
            } else {
                (None, Some(area))
            }
        } else {
            (Some(content_chunks[0]), Some(content_chunks[1]))
        }
    } else {
        // Timer takes full width
        (Some(area), None)
    };

    AppLayout {
        timer: timer_area,
        tasks: tasks_area,
    }
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = create_layout(frame.area(), app);

    // Render timer panel if visible
    if let Some(timer_area) = layout.timer {
        app.timer_panel.render(
            frame,
            timer_area,
            app.focused_panel == PanelId::Timer,
            &app.timer,
            app.task_manager.active_task(),
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

    // Render overlays
    if let Some(ref message) = app.error_message {
        render_error_overlay(frame, message);
    } else if let Some(ref input) = app.task_input {
        render_task_input_dialogue(frame, input);
    } else if let Some(ref dialogue) = app.sync_dialogue {
        render_sync_dialogue(frame, dialogue);
    } else if app.shortcuts_visible {
        render_help_overlay(frame, app);
    }
}

fn render_error_overlay(frame: &mut Frame, message: &str) {
    use ratatui::widgets::Wrap;

    let dialogue_width = 40u16;
    // inner width = dialogue_width - 2 borders - 4 padding
    let inner_width = dialogue_width.saturating_sub(6) as usize;

    // Estimate wrapped line count for message
    let msg_lines = if inner_width > 0 {
        (message.len() + inner_width - 1) / inner_width
    } else {
        1
    };
    // Content: blank + message lines + blank + hint + blank
    let content_height = (1 + msg_lines + 1 + 1 + 1) as u16 + 2; // +2 for borders
    let dialogue_height = content_height.min(frame.area().height.saturating_sub(4));

    let area = frame.area();
    let x = area.x + (area.width.saturating_sub(dialogue_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialogue_height)) / 2;
    let dialogue_area = Rect::new(x, y, dialogue_width, dialogue_height);

    frame.render_widget(Clear, dialogue_area);

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(dialogue_area);
    frame.render_widget(block, dialogue_area);

    // Layout: pad, message, pad, hint, pad
    let rows = Layout::vertical([
        Constraint::Length(1),                // pad
        Constraint::Length(msg_lines as u16), // message
        Constraint::Length(1),                // pad
        Constraint::Length(1),                // hint
        Constraint::Min(0),                   // pad
    ])
    .split(inner);

    let msg_area = Rect {
        x: rows[1].x + 2,
        width: rows[1].width.saturating_sub(4),
        ..rows[1]
    };
    let msg = Paragraph::new(Span::styled(message, Style::default().fg(Color::Red)))
        .wrap(Wrap { trim: true });
    frame.render_widget(msg, msg_area);

    let hint_area = Rect {
        x: rows[3].x + 2,
        width: rows[3].width.saturating_sub(4),
        ..rows[3]
    };
    let hint = Paragraph::new(Span::styled(
        "Press any key to dismiss",
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(hint, hint_area);
}

fn render_help_overlay(frame: &mut Frame, app: &App) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from("")); // Empty line at top

    // Panel-specific shortcuts
    let panel_name = match app.focused_panel {
        PanelId::Timer => "Timer",
        PanelId::Tasks => "Tasks",
    };
    lines.push(Line::from(Span::styled(
        format!("  {} Panel", panel_name),
        Style::default().fg(Color::White),
    )));

    for shortcut in app.focused_shortcuts() {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                format!("[{}]", shortcut.key),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(format!(" {}", shortcut.description)),
        ]));
    }

    lines.push(Line::from(""));

    // Global shortcuts
    lines.push(Line::from(Span::styled(
        "  Global",
        Style::default().fg(Color::White),
    )));
    let global_shortcuts = [
        ("t", "Switch Panel Focus"),
        ("T", "Toggle Tasks Panel"),
        ("s", "Sync with File"),
        ("?", "Toggle Help"),
        ("Q", "Quit"),
    ];
    for (key, desc) in global_shortcuts {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("[{}]", key), Style::default().fg(Color::Yellow)),
            Span::raw(format!(" {}", desc)),
        ]));
    }

    lines.push(Line::from("")); // Empty line at bottom

    // Calculate dialogue dimensions
    let content_height = lines.len() as u16 + 2; // +2 for borders
    let dialogue_width = 30u16;
    let dialogue_height = content_height.min(frame.area().height.saturating_sub(4));

    // Center the dialogue
    let area = frame.area();
    let x = area.x + (area.width.saturating_sub(dialogue_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialogue_height)) / 2;
    let dialogue_area = Rect::new(x, y, dialogue_width, dialogue_height);

    // Clear background and render overlay
    frame.render_widget(Clear, dialogue_area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, dialogue_area);
}

fn render_task_input_dialogue(frame: &mut Frame, input: &TaskInput) {
    let title = match input.section {
        TaskSection::Backlog => " Add to Backlog ",
        TaskSection::Current => " Add to Current ",
        TaskSection::Completed => " Add Task ",
    };

    // Dialogue: border + 1 pad + input row + 1 pad + hints row + 1 pad + border = 7 rows
    let dialogue_width = 40u16;
    let dialogue_height = 7u16;

    // Center the dialogue
    let area = frame.area();
    let x = area.x + (area.width.saturating_sub(dialogue_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialogue_height)) / 2;
    let dialogue_area = Rect::new(x, y, dialogue_width, dialogue_height);

    frame.render_widget(Clear, dialogue_area);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(dialogue_area);
    frame.render_widget(block, dialogue_area);

    // Layout: pad, input, pad, hints, pad
    let rows = Layout::vertical([
        Constraint::Length(1), // pad
        Constraint::Length(1), // input
        Constraint::Length(1), // pad
        Constraint::Length(1), // hints
        Constraint::Min(0),    // pad
    ])
    .split(inner);

    // Render input text with cursor
    let input_area = Rect {
        x: rows[1].x + 1,
        width: rows[1].width.saturating_sub(2),
        ..rows[1]
    };
    let available_width = input_area.width as usize;

    let scroll = if input.cursor > available_width {
        input.cursor - available_width
    } else {
        0
    };
    let visible_text: String = input
        .text
        .chars()
        .skip(scroll)
        .take(available_width)
        .collect();
    let cursor_pos = input.cursor - scroll;

    let input_line = Line::from(Span::styled(
        &visible_text,
        Style::default().fg(Color::White),
    ));
    frame.render_widget(Paragraph::new(input_line), input_area);

    // Place cursor
    let cursor_x = input_area.x + cursor_pos as u16;
    if cursor_x < input_area.x + input_area.width {
        frame.set_cursor_position((cursor_x, input_area.y));
    }

    // Render hints
    let hints = Line::from(vec![
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(" Add "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" Cancel"),
    ]);
    let hints_area = Rect {
        x: rows[3].x + 1,
        width: rows[3].width.saturating_sub(2),
        ..rows[3]
    };
    frame.render_widget(
        Paragraph::new(hints).alignment(ratatui::layout::Alignment::Center),
        hints_area,
    );
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
