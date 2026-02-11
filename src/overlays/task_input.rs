use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::util::{centered_rect, render_overlay_frame};
use crate::task::TaskSection;

pub enum TaskInputAction {
    Consumed,
    Dismiss,
    Submit { text: String, section: TaskSection },
}

pub struct TaskInputOverlay {
    pub text: String,
    pub cursor: usize,
    pub section: TaskSection,
}

impl TaskInputOverlay {
    pub const fn new(section: TaskSection) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            section,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> TaskInputAction {
        match key.code {
            KeyCode::Esc => TaskInputAction::Dismiss,
            KeyCode::Enter => {
                let text = self.text.trim().to_string();
                if text.is_empty() {
                    TaskInputAction::Dismiss
                } else {
                    TaskInputAction::Submit {
                        text,
                        section: self.section,
                    }
                }
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.text.remove(self.cursor - 1);
                    self.cursor -= 1;
                }
                TaskInputAction::Consumed
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                TaskInputAction::Consumed
            }
            KeyCode::Right => {
                if self.cursor < self.text.len() {
                    self.cursor += 1;
                }
                TaskInputAction::Consumed
            }
            KeyCode::Char(c) => {
                self.text.insert(self.cursor, c);
                self.cursor += 1;
                TaskInputAction::Consumed
            }
            _ => TaskInputAction::Consumed,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let title = match self.section {
            TaskSection::Backlog => " Add to Backlog ",
            TaskSection::Current => " Add to Current ",
            TaskSection::Completed => " Add Task ",
        };

        let overlay_width = 40u16;
        let overlay_height = 7u16;

        let overlay_area = centered_rect(frame.area(), overlay_width, overlay_height);
        let inner = render_overlay_frame(frame, overlay_area, title, Color::Cyan);

        let rows = Layout::vertical([
            Constraint::Length(1), // pad
            Constraint::Length(1), // input
            Constraint::Length(1), // pad
            Constraint::Length(1), // hints
            Constraint::Min(0),    // pad
        ])
        .split(inner);

        let input_area = Rect {
            x: rows[1].x + 1,
            width: rows[1].width.saturating_sub(2),
            ..rows[1]
        };
        let available_width = input_area.width as usize;

        let scroll = self.cursor.saturating_sub(available_width);
        let visible_text: String = self
            .text
            .chars()
            .skip(scroll)
            .take(available_width)
            .collect();
        let cursor_pos = self.cursor - scroll;

        let input_line = Line::from(Span::styled(
            &visible_text,
            Style::default().fg(Color::White),
        ));
        frame.render_widget(Paragraph::new(input_line), input_area);

        let cursor_x = input_area.x + cursor_pos as u16;
        if cursor_x < input_area.x + input_area.width {
            frame.set_cursor_position((cursor_x, input_area.y));
        }

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
            Paragraph::new(hints).alignment(Alignment::Center),
            hints_area,
        );
    }
}
