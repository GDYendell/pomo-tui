use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Paragraph, Wrap},
    Frame,
};

use super::util::{centered_rect, render_overlay_frame};

pub fn render_error_overlay(frame: &mut Frame, message: &str) {
    let overlay_width = 40u16;
    let inner_width = overlay_width.saturating_sub(6) as usize;

    let msg_lines = if inner_width > 0 {
        message.len().div_ceil(inner_width)
    } else {
        1
    };
    let content_height = (1 + msg_lines + 1 + 1 + 1) as u16 + 2;
    let overlay_height = content_height.min(frame.area().height.saturating_sub(4));

    let overlay_area = centered_rect(frame.area(), overlay_width, overlay_height);
    let inner = render_overlay_frame(frame, overlay_area, " Error ", Color::Red);

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
