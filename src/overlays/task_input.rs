use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use ratatui_input_manager::{keymap, KeyMap};

use super::util::{centered_rect, render_overlay_frame};
use crate::task::TaskSection;

/// Overlay for adding new tasks
pub struct TaskInputOverlay {
    text: String,
    cursor: usize,
    section: TaskSection,
    dismissed: bool,
    submitted: bool,
}

impl TaskInputOverlay {
    pub fn new(section: TaskSection) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            section,
            dismissed: false,
            submitted: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.dismissed || self.submitted
    }

    /// Returns the submitted task text and section, or None if dismissed
    pub fn result(&self) -> Option<(String, TaskSection)> {
        self.submitted
            .then(|| (self.text.trim().to_string(), self.section))
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        KeyMap::handle(self, event);
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            self.insert_char(*c);
            true
        } else {
            false
        }
    }

    fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
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

#[keymap(backend = "crossterm")]
impl TaskInputOverlay {
    /// Cancel
    #[keybind(pressed(key=KeyCode::Esc))]
    fn dismiss(&mut self) {
        self.dismissed = true;
    }

    /// Add task
    #[keybind(pressed(key=KeyCode::Enter))]
    fn submit(&mut self) {
        if self.text.trim().is_empty() {
            self.dismissed = true;
        } else {
            self.submitted = true;
        }
    }

    /// Delete character
    #[keybind(pressed(key=KeyCode::Backspace))]
    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    /// Move cursor left
    #[keybind(pressed(key=KeyCode::Left))]
    fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    #[keybind(pressed(key=KeyCode::Right))]
    fn cursor_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }
}
