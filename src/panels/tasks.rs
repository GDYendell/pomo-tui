use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::panel::{KeyHandleResult, Shortcut};
use crate::task::TaskSection;
use crate::task_manager::TaskManager;

pub struct TasksPanel;

impl Default for TasksPanel {
    fn default() -> Self {
        Self
    }
}

impl TasksPanel {
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, task_manager: &TaskManager) {
        let border_color = if focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Tasks ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into three equal sections: Backlog, Current, Completed
        let chunks = Layout::vertical([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(inner);

        self.render_section(
            frame,
            chunks[0],
            "Backlog",
            task_manager.backlog(),
            TaskSection::Backlog,
            &task_manager.focus,
            focused,
            true,
        );
        self.render_section(
            frame,
            chunks[1],
            "Current",
            task_manager.current(),
            TaskSection::Current,
            &task_manager.focus,
            focused,
            true,
        );
        self.render_section(
            frame,
            chunks[2],
            "Completed",
            task_manager.completed(),
            TaskSection::Completed,
            &task_manager.focus,
            focused,
            false,
        );
    }

    pub fn handle_key(&mut self, key: KeyEvent, task_manager: &mut TaskManager) -> KeyHandleResult {
        match key.code {
            KeyCode::Char('j') => {
                task_manager.move_down();
                KeyHandleResult::Consumed
            }
            KeyCode::Char('k') => {
                task_manager.move_up();
                KeyHandleResult::Consumed
            }
            KeyCode::Char('J') => {
                task_manager.reorder_down();
                KeyHandleResult::Consumed
            }
            KeyCode::Char('K') => {
                task_manager.reorder_up();
                KeyHandleResult::Consumed
            }
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    task_manager.prev_section();
                } else {
                    task_manager.next_section();
                }
                KeyHandleResult::Consumed
            }
            KeyCode::BackTab => {
                task_manager.prev_section();
                KeyHandleResult::Consumed
            }
            KeyCode::Enter => {
                task_manager.toggle_section();
                KeyHandleResult::Consumed
            }
            KeyCode::Char('x') => {
                task_manager.complete_focused();
                KeyHandleResult::Consumed
            }
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
                key: "s",
                description: "Sync",
            },
        ]
    }

    fn render_section(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        tasks: &[crate::task::Task],
        section: TaskSection,
        focus: &crate::task_manager::TasksFocus,
        panel_focused: bool,
        show_bottom_border: bool,
    ) {
        let is_focused_section = panel_focused && focus.section == section;

        let title_style = if is_focused_section {
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

        if tasks.is_empty() {
            let placeholder = Paragraph::new("(empty)")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(placeholder, inner);
            return;
        }

        let total_height = inner.height as usize;
        if total_height == 0 {
            return;
        }

        // Reserve last row for ellipsis indicator
        let visible_height = total_height.saturating_sub(1);
        let margin = 2usize;

        // Calculate scroll offset with margin
        let scroll_offset = if is_focused_section && visible_height > 0 {
            let max_offset = tasks.len().saturating_sub(visible_height);
            // Keep cursor at least `margin` from bottom when scrolling down
            let min_offset_for_cursor = focus.index.saturating_sub(visible_height.saturating_sub(margin).saturating_sub(1));
            // Keep cursor at least `margin` from top when scrolling up
            let max_offset_for_cursor = focus.index.saturating_sub(margin);

            // Clamp between the two constraints
            min_offset_for_cursor.min(max_offset).max(0).min(max_offset_for_cursor.max(0).min(max_offset))
        } else {
            0
        };

        let has_more_below = scroll_offset + visible_height < tasks.len();

        let items: Vec<ListItem> = tasks
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(i, task)| {
                let is_selected = is_focused_section && focus.index == i;

                let prefix = if section == TaskSection::Completed {
                    "[x] "
                } else {
                    "[ ] "
                };

                let content = if is_selected {
                    Line::from(vec![
                        Span::styled("> ", Style::default().fg(Color::Cyan)),
                        Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            &task.text,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                        Span::styled(&task.text, Style::default().fg(Color::Gray)),
                    ])
                };

                ListItem::new(content)
            })
            .collect();

        // Add ellipsis if there are more items below
        let indicator = if has_more_below {
            ListItem::new(Line::from(Span::styled(
                "  ...",
                Style::default().fg(Color::DarkGray),
            )))
        } else {
            ListItem::new(Line::from(""))
        };

        let mut all_items = items;
        all_items.push(indicator);

        let list = List::new(all_items);
        frame.render_widget(list, inner);
    }
}
