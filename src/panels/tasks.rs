use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::util::{panel_block, KeyHandleResult};
use crate::task::TaskSection;
use crate::task_manager::TaskManager;
use crate::util::Shortcut;

const SECTIONS: [(TaskSection, &str, &str, bool); 3] = [
    (TaskSection::Backlog, "Backlog", "[ ]", true),
    (TaskSection::Current, "Current", "[ ]", true),
    (TaskSection::Completed, "Completed", "[x]", false),
];

#[derive(Debug, Clone)]
struct TaskFocus {
    section: TaskSection,
    index: usize,
}

impl Default for TaskFocus {
    fn default() -> Self {
        Self {
            section: TaskSection::Backlog,
            index: 0,
        }
    }
}

pub struct TasksPanel {
    focus: TaskFocus,
    /// Visible task rows per section (updated during render)
    section_page_size: usize,
}

impl Default for TasksPanel {
    fn default() -> Self {
        Self {
            focus: TaskFocus::default(),
            section_page_size: 10,
        }
    }
}

impl TasksPanel {
    pub fn focused_section(&self) -> TaskSection {
        self.focus.section
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
        task_manager: &TaskManager,
    ) {
        let block = panel_block(" Tasks ", focused);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into three equal sections manually to avoid rounding issues
        let h = inner.height;
        let third = h / 3;
        let remainder = h % 3;
        // Distribute remainder: first section gets +1 if remainder >= 1, second if >= 2
        let h0 = third + if remainder >= 1 { 1 } else { 0 };
        let h1 = third + if remainder >= 2 { 1 } else { 0 };
        let h2 = h - h0 - h1;
        let chunks = Layout::vertical([
            Constraint::Length(h0),
            Constraint::Length(h1),
            Constraint::Length(h2),
        ])
        .split(inner);

        // Store page size for page up/down
        // Section inner height = chunk height - border (1) - ellipsis row (1)
        self.section_page_size = (third as usize).saturating_sub(3).max(1);

        for (i, ((section, title, checkbox, bottom_border), tasks)) in SECTIONS
            .iter()
            .zip([
                task_manager.backlog(),
                task_manager.current(),
                task_manager.completed(),
            ])
            .enumerate()
        {
            let section_focused = focused && self.focus.section == *section;
            let cursor = if section_focused {
                Some(self.focus.index)
            } else {
                None
            };
            let inner = Self::render_section_frame(
                frame,
                chunks[i],
                title,
                section_focused,
                *bottom_border,
            );
            Self::render_task_list(frame, inner, tasks, checkbox, cursor);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, task_manager: &mut TaskManager) -> KeyHandleResult {
        match key.code {
            KeyCode::Char('j') => {
                self.move_down(task_manager);
                KeyHandleResult::Consumed
            }
            KeyCode::Char('k') => {
                self.move_up();
                KeyHandleResult::Consumed
            }
            KeyCode::Char('J') => {
                task_manager.reorder_down(self.focus.section, self.focus.index);
                let len = task_manager.section_len(self.focus.section);
                if self.focus.index + 1 < len {
                    self.focus.index += 1;
                }
                KeyHandleResult::Consumed
            }
            KeyCode::Char('K') => {
                task_manager.reorder_up(self.focus.section, self.focus.index);
                if self.focus.index > 0 {
                    self.focus.index -= 1;
                }
                KeyHandleResult::Consumed
            }
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_section(task_manager);
                } else {
                    self.next_section(task_manager);
                }
                KeyHandleResult::Consumed
            }
            KeyCode::BackTab => {
                self.prev_section(task_manager);
                KeyHandleResult::Consumed
            }
            KeyCode::Enter => {
                task_manager.toggle_section(self.focus.section, self.focus.index);
                self.clamp_focus(task_manager);
                KeyHandleResult::Consumed
            }
            KeyCode::Char('x') => {
                task_manager.complete_focused(self.focus.section, self.focus.index);
                self.clamp_focus(task_manager);
                KeyHandleResult::Consumed
            }
            KeyCode::Char(',') => {
                self.page_down(task_manager);
                KeyHandleResult::Consumed
            }
            KeyCode::Char('.') => {
                self.page_up();
                KeyHandleResult::Consumed
            }
            KeyCode::Char('a') => KeyHandleResult::AddTask,
            _ => KeyHandleResult::Ignored,
        }
    }

    pub fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "Tab",
                description: "Section",
            },
            Shortcut {
                key: "j/k",
                description: "Navigate",
            },
            Shortcut {
                key: "J/K",
                description: "Reorder",
            },
            Shortcut {
                key: "Enter",
                description: "Move",
            },
            Shortcut {
                key: "x",
                description: "Complete",
            },
            Shortcut {
                key: "a",
                description: "Add",
            },
            Shortcut {
                key: "s",
                description: "Sync",
            },
        ]
    }

    // -- Focus/navigation methods --

    pub fn clamp_focus(&mut self, task_manager: &TaskManager) {
        let len = task_manager.section_len(self.focus.section);
        if self.focus.index >= len {
            self.focus.index = len.saturating_sub(1);
        }
    }

    fn move_down(&mut self, task_manager: &TaskManager) {
        let len = task_manager.section_len(self.focus.section);
        if len > 0 && self.focus.index + 1 < len {
            self.focus.index += 1;
        }
    }

    fn move_up(&mut self) {
        if self.focus.index > 0 {
            self.focus.index -= 1;
        }
    }

    fn page_down(&mut self, task_manager: &TaskManager) {
        let len = task_manager.section_len(self.focus.section);
        if len > 0 {
            self.focus.index = (self.focus.index + self.section_page_size).min(len - 1);
        }
    }

    fn page_up(&mut self) {
        self.focus.index = self.focus.index.saturating_sub(self.section_page_size);
    }

    fn next_section(&mut self, task_manager: &TaskManager) {
        self.focus.section = match self.focus.section {
            TaskSection::Backlog => TaskSection::Current,
            TaskSection::Current => TaskSection::Completed,
            TaskSection::Completed => TaskSection::Backlog,
        };
        self.clamp_focus(task_manager);
    }

    fn prev_section(&mut self, task_manager: &TaskManager) {
        self.focus.section = match self.focus.section {
            TaskSection::Backlog => TaskSection::Completed,
            TaskSection::Current => TaskSection::Backlog,
            TaskSection::Completed => TaskSection::Current,
        };
        self.clamp_focus(task_manager);
    }

    // -- Rendering helpers --

    fn render_section_frame(
        frame: &mut Frame,
        area: Rect,
        title: &str,
        focused: bool,
        show_bottom_border: bool,
    ) -> Rect {
        let title_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let borders = if show_bottom_border {
            Borders::BOTTOM
        } else {
            Borders::NONE
        };

        let block = Block::default()
            .borders(borders)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" {} ", title))
            .title_style(title_style)
            .title_alignment(Alignment::Right);

        let inner = block.inner(area);
        frame.render_widget(block, area);
        inner
    }

    fn render_task_list(
        frame: &mut Frame,
        area: Rect,
        tasks: &[crate::task::Task],
        checkbox: &str,
        focused_index: Option<usize>,
    ) {
        if tasks.is_empty() {
            let shrunk = Rect {
                height: area.height.saturating_sub(2),
                ..area
            };
            let centered = Layout::vertical([Constraint::Length(1)])
                .flex(ratatui::layout::Flex::Center)
                .split(shrunk)[0];
            let placeholder = Paragraph::new("(empty)")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(placeholder, centered);
            return;
        }

        let total_height = area.height as usize;
        if total_height == 0 {
            return;
        }

        // Reserve last row for ellipsis indicator
        let visible_height = total_height.saturating_sub(1);

        let scroll_offset = scroll_offset(tasks.len(), visible_height, focused_index);
        let has_more_below = scroll_offset + visible_height < tasks.len();

        let prefix_width = 6; // "> [x] " or "  [x] "
        let trailing_space = 10;
        let max_text_width = (area.width as usize)
            .saturating_sub(prefix_width)
            .saturating_sub(trailing_space);

        let prefix = format!("{} ", checkbox);

        let mut items: Vec<ListItem> = tasks
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(i, task)| {
                let is_selected = focused_index == Some(i);
                let display_text = truncate_with_ellipsis(&task.text, max_text_width);

                let content = if is_selected {
                    Line::from(vec![
                        Span::styled("> ", Style::default().fg(Color::Cyan)),
                        Span::styled(&prefix, Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            display_text,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(&prefix, Style::default().fg(Color::DarkGray)),
                        Span::styled(display_text, Style::default().fg(Color::Gray)),
                    ])
                };

                ListItem::new(content)
            })
            .collect();

        // Add ellipsis if there are more items below
        items.push(if has_more_below {
            ListItem::new(Line::from(Span::styled(
                "  ...",
                Style::default().fg(Color::DarkGray),
            )))
        } else {
            ListItem::new(Line::from(""))
        });

        frame.render_widget(List::new(items), area);
    }
}

fn scroll_offset(total: usize, visible: usize, focused: Option<usize>) -> usize {
    let Some(cursor) = focused else { return 0 };
    if visible == 0 {
        return 0;
    }
    let max_offset = total.saturating_sub(visible);
    let margin = 2usize;
    // Keep cursor at least `margin` from bottom when scrolling down
    let min_offset_for_cursor =
        cursor.saturating_sub(visible.saturating_sub(margin).saturating_sub(1));
    // Keep cursor at least `margin` from top when scrolling up
    let max_offset_for_cursor = cursor.saturating_sub(margin);
    // Clamp between the two constraints
    min_offset_for_cursor
        .min(max_offset)
        .max(0)
        .min(max_offset_for_cursor.max(0).min(max_offset))
}

fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        return text.to_string();
    }
    if max_width < 3 {
        return ".".repeat(max_width);
    }
    let limit = max_width - 3; // room for "..."
    let mut result = String::new();
    for word in text.split_whitespace() {
        if result.is_empty() {
            if word.len() > limit {
                return "...".to_string();
            }
            result = word.to_string();
        } else if result.len() + 1 + word.len() <= limit {
            result.push(' ');
            result.push_str(word);
        } else {
            break;
        }
    }
    format!("{}...", result)
}
