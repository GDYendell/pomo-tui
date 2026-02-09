use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::util::{centered_rect, shortcut_line};
use crate::util::Shortcut;

pub fn render_help_overlay(frame: &mut Frame, panel_name: &str, shortcuts: &[Shortcut]) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        format!("  {} Panel", panel_name),
        Style::default().fg(Color::White),
    )));

    for shortcut in shortcuts {
        lines.push(shortcut_line(shortcut.key, shortcut.description));
    }

    lines.push(Line::from(""));

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
        lines.push(shortcut_line(key, desc));
    }

    lines.push(Line::from(""));

    let content_height = lines.len() as u16 + 2;
    let overlay_width = 30u16;
    let overlay_height = content_height.min(frame.area().height.saturating_sub(4));

    let overlay_area = centered_rect(frame.area(), overlay_width, overlay_height);
    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}
