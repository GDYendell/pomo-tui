use std::io;
use std::path::PathBuf;

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use ratatui_input_manager::{keymap, KeyMap};

use super::util::panel_block;
use crate::overlays::{SyncItem, SyncOverlay, TaskInputOverlay};
use crate::task::{Task, TaskSection};
use crate::task_manager::TaskManager;

const SECTIONS: [(TaskSection, &str, &str, bool); 3] = [
    (TaskSection::Backlog, "Backlog", "[ ]", true),
    (TaskSection::Current, "Current", "[ ]", true),
    (TaskSection::Completed, "Completed", "[x]", false),
];

/// Current focus position within the tasks panel (section and index)
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

/// Tasks panel displaying backlog, current, and completed task sections
pub struct TasksPanel {
    focus: TaskFocus,
    /// Visible task rows per section (updated during render)
    section_page_size: usize,
    task_manager: TaskManager,
    task_input_overlay: Option<TaskInputOverlay>,
    sync_overlay: Option<SyncOverlay>,
    pending_error: Option<String>,
}

impl Default for TasksPanel {
    fn default() -> Self {
        Self::new(TaskManager::new())
    }
}

impl TasksPanel {
    pub fn from_file(path: Option<PathBuf>) -> (Self, Option<String>) {
        let Some(path) = path else {
            return (Self::default(), None);
        };
        match TaskManager::load(path) {
            Ok(tm) => (Self::new(tm), None),
            Err(e) => (Self::default(), Some(format!("Failed to load tasks: {e}"))),
        }
    }

    fn new(task_manager: TaskManager) -> Self {
        Self {
            focus: TaskFocus::default(),
            section_page_size: 10,
            task_manager,
            task_input_overlay: None,
            sync_overlay: None,
            pending_error: None,
        }
    }

    /// Route the event to the active overlay if one is open, otherwise dispatch keybindings
    pub fn handle_event(&mut self, event: &Event) {
        if let Some(ref mut overlay) = self.task_input_overlay {
            overlay.handle_event(event);
        } else if let Some(ref mut overlay) = self.sync_overlay {
            overlay.handle(event);
        } else {
            self.handle(event);
        }
    }

    pub fn task_input_overlay(&self) -> Option<&TaskInputOverlay> {
        self.task_input_overlay.as_ref()
    }

    pub fn sync_overlay(&self) -> Option<&SyncOverlay> {
        self.sync_overlay.as_ref()
    }

    pub fn take_error(&mut self) -> Option<String> {
        self.pending_error.take()
    }

    fn process_overlay(&mut self) {
        if let Some(overlay) = self.task_input_overlay.take_if(|o| o.is_done()) {
            if let Some((text, section)) = overlay.result() {
                self.task_manager.add_task(text, section);
            }
        }

        if let Some(overlay) = self.sync_overlay.take_if(|o| o.is_done()) {
            if let Some(items) = overlay.result() {
                if let Err(e) = self.apply_sync(items) {
                    self.pending_error = Some(format!("Sync failed: {e}"));
                }
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = panel_block(" Tasks ", focused);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into three equal sections manually to avoid rounding issues
        let h = inner.height;
        let third = h / 3;
        let remainder = h % 3;
        // Distribute remainder: first section gets +1 if remainder >= 1, second if >= 2
        let h0 = third + u16::from(remainder >= 1);
        let h1 = third + u16::from(remainder >= 2);
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
                self.task_manager.backlog(),
                self.task_manager.current(),
                self.task_manager.completed(),
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

    pub fn active_task(&self) -> Option<&Task> {
        self.task_manager.active_task()
    }

    fn apply_sync(&mut self, items: &[SyncItem]) -> Result<(), io::Error> {
        self.task_manager.apply_sync(items)?;
        self.clamp_focus();
        Ok(())
    }

    pub fn complete_current_task(&mut self) {
        self.task_manager.complete_current_task();
    }

    // -- Focus/navigation methods --

    /// Prepare a SyncOverlay by computing sync items from the task manager
    fn sync_tasks(&mut self) -> Result<SyncOverlay, String> {
        if !self.task_manager.has_file_path() {
            if let Err(e) = self.task_manager.create_default_file() {
                return Err(format!("Failed to create default task file: {e}"));
            }
        }
        self.task_manager
            .compute_sync_items()
            .map(SyncOverlay::new)
            .map_err(|e| format!("Sync failed: {e}"))
    }

    fn clamp_focus(&mut self) {
        let len = self.task_manager.section_len(self.focus.section);
        if self.focus.index >= len {
            self.focus.index = len.saturating_sub(1);
        }
    }

    fn move_down(&mut self) {
        let len = self.task_manager.section_len(self.focus.section);
        if len > 0 && self.focus.index + 1 < len {
            self.focus.index += 1;
        }
    }

    fn move_up(&mut self) {
        if self.focus.index > 0 {
            self.focus.index -= 1;
        }
    }

    fn reorder_down(&mut self) {
        self.task_manager
            .reorder_down(self.focus.section, self.focus.index);
        let len = self.task_manager.section_len(self.focus.section);
        if self.focus.index + 1 < len {
            self.focus.index += 1;
        }
    }

    fn reorder_up(&mut self) {
        self.task_manager
            .reorder_up(self.focus.section, self.focus.index);
        if self.focus.index > 0 {
            self.focus.index -= 1;
        }
    }

    fn page_down(&mut self) {
        let len = self.task_manager.section_len(self.focus.section);
        if len > 0 {
            self.focus.index = (self.focus.index + self.section_page_size).min(len - 1);
        }
    }

    fn page_up(&mut self) {
        self.focus.index = self.focus.index.saturating_sub(self.section_page_size);
    }

    fn next_section(&mut self) {
        self.focus.section = match self.focus.section {
            TaskSection::Backlog => TaskSection::Current,
            TaskSection::Current => TaskSection::Completed,
            TaskSection::Completed => TaskSection::Backlog,
        };
        self.clamp_focus();
    }

    fn prev_section(&mut self) {
        self.focus.section = match self.focus.section {
            TaskSection::Backlog => TaskSection::Completed,
            TaskSection::Current => TaskSection::Backlog,
            TaskSection::Completed => TaskSection::Current,
        };
        self.clamp_focus();
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
            .title(format!(" {title} "))
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

        let scroll_offset = calculate_scroll_offset(tasks.len(), visible_height, focused_index);
        let has_more_below = scroll_offset + visible_height < tasks.len();

        let prefix_width = 6; // "> [x] " or "  [x] "
        let trailing_space = 10;
        let max_text_width = (area.width as usize)
            .saturating_sub(prefix_width)
            .saturating_sub(trailing_space);

        let prefix = format!("{checkbox} ");

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

#[keymap(backend = "crossterm")]
impl TasksPanel {
    /// Move focus down
    #[keybind(pressed(key=KeyCode::Char('j')))]
    fn key_move_down(&mut self) {
        self.move_down();
    }

    /// Move focus up
    #[keybind(pressed(key=KeyCode::Char('k')))]
    fn key_move_up(&mut self) {
        self.move_up();
    }

    /// Reorder task down
    #[keybind(pressed(key=KeyCode::Char('J')))]
    fn key_reorder_down(&mut self) {
        self.reorder_down();
    }

    /// Reorder task up
    #[keybind(pressed(key=KeyCode::Char('K')))]
    fn key_reorder_up(&mut self) {
        self.reorder_up();
    }

    /// Next section
    #[keybind(pressed(key=KeyCode::Tab))]
    fn key_next_section(&mut self) {
        self.next_section();
    }

    /// Previous section
    #[keybind(pressed(key=KeyCode::BackTab))]
    fn key_prev_section(&mut self) {
        self.prev_section();
    }

    /// Move task to next section
    #[keybind(pressed(key=KeyCode::Enter))]
    fn key_cycle_task(&mut self) {
        self.task_manager
            .cycle_task_section(self.focus.section, self.focus.index);
        self.clamp_focus();
    }

    /// Toggle task completion
    #[keybind(pressed(key=KeyCode::Char('x')))]
    fn key_toggle_completion(&mut self) {
        self.task_manager
            .toggle_completion(self.focus.section, self.focus.index);
        self.clamp_focus();
    }

    /// Page down
    #[keybind(pressed(key=KeyCode::Char(',')))]
    fn key_page_down(&mut self) {
        self.page_down();
    }

    /// Page up
    #[keybind(pressed(key=KeyCode::Char('.')))]
    fn key_page_up(&mut self) {
        self.page_up();
    }

    /// Add new task
    #[keybind(pressed(key=KeyCode::Char('a')))]
    fn key_add_task(&mut self) {
        if self.focus.section != TaskSection::Completed {
            self.task_input_overlay = Some(TaskInputOverlay::new(self.focus.section));
        }
    }

    /// Sync tasks with file
    #[keybind(pressed(key=KeyCode::Char('s')))]
    #[keybind(pressed(key=KeyCode::Char('S')))]
    fn key_sync(&mut self) {
        match self.sync_tasks() {
            Ok(overlay) => self.sync_overlay = Some(overlay),
            Err(e) => self.pending_error = Some(e),
        }
    }

    /// Delete focused task
    #[keybind(pressed(key=KeyCode::Char('d')))]
    fn key_delete_task(&mut self) {
        self.task_manager
            .delete_task(self.focus.section, self.focus.index);
        self.clamp_focus();
    }
}

/// Calculates scroll offset to keep focused item within margin from edges
fn calculate_scroll_offset(total: usize, visible: usize, focused: Option<usize>) -> usize {
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
    format!("{result}...")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_with_ellipsis() {
        // No truncation needed
        assert_eq!(truncate_with_ellipsis("Hello", 10), "Hello");
        assert_eq!(truncate_with_ellipsis("Hello world", 11), "Hello world");
        assert_eq!(truncate_with_ellipsis("Hello", 5), "Hello");

        // Basic truncation
        assert_eq!(truncate_with_ellipsis("Hello world", 8), "Hello...");
        assert_eq!(truncate_with_ellipsis("This is a test", 10), "This is...");

        // Word boundary (should not split words)
        assert_eq!(truncate_with_ellipsis("Hello world test", 12), "Hello...");
        assert_eq!(
            truncate_with_ellipsis("One two three four five", 15),
            "One two..."
        );

        // Long single word (too long to fit)
        assert_eq!(
            truncate_with_ellipsis("Supercalifragilisticexpialidocious", 10),
            "..."
        );

        // Very small widths
        assert_eq!(truncate_with_ellipsis("Hello", 1), ".");
        assert_eq!(truncate_with_ellipsis("Hello", 2), "..");
        assert_eq!(truncate_with_ellipsis("Hello", 3), "...");
    }

    #[test]
    fn test_scroll_offset() {
        // No focus returns 0
        assert_eq!(calculate_scroll_offset(10, 5, None), 0);

        // All items fit in view, no scroll needed
        assert_eq!(calculate_scroll_offset(5, 10, Some(0)), 0);
        assert_eq!(calculate_scroll_offset(5, 10, Some(4)), 0);

        // Cursor at top stays at offset 0
        assert_eq!(calculate_scroll_offset(20, 10, Some(0)), 0);

        // Cursor near top with margin=2, still at offset 0
        assert_eq!(calculate_scroll_offset(20, 10, Some(2)), 0);

        // Cursor moves down: index 10, visible=10, margin=2
        // min_offset_for_cursor = 10 - (10 - 2 - 1) = 3
        assert_eq!(calculate_scroll_offset(20, 10, Some(10)), 3);

        // Cursor at bottom (index 19), visible=10
        assert_eq!(calculate_scroll_offset(20, 10, Some(19)), 10);

        // Respects max offset
        assert_eq!(calculate_scroll_offset(10, 10, Some(15)), 0);
        assert_eq!(calculate_scroll_offset(15, 10, Some(20)), 5);

        // Zero visible height
        assert_eq!(calculate_scroll_offset(10, 0, Some(5)), 0);
    }

    #[test]
    fn test_tasks_panel_default_focus() {
        let panel = TasksPanel::default();
        assert_eq!(panel.focus.section, TaskSection::Backlog);
        assert_eq!(panel.focus.index, 0);
    }

    #[test]
    fn test_move_up_down_navigation() {
        let mut panel = TasksPanel::default();
        panel
            .task_manager
            .add_task("Task 1".to_string(), TaskSection::Backlog);
        panel
            .task_manager
            .add_task("Task 2".to_string(), TaskSection::Backlog);
        panel
            .task_manager
            .add_task("Task 3".to_string(), TaskSection::Backlog);

        // Move down
        assert_eq!(panel.focus.index, 0);
        panel.move_down();
        assert_eq!(panel.focus.index, 1);
        panel.move_down();
        assert_eq!(panel.focus.index, 2);

        // Should not go beyond last item
        panel.move_down();
        assert_eq!(panel.focus.index, 2);

        // Move up
        panel.move_up();
        assert_eq!(panel.focus.index, 1);
        panel.move_up();
        assert_eq!(panel.focus.index, 0);

        // Should not go below 0
        panel.move_up();
        assert_eq!(panel.focus.index, 0);
    }

    #[test]
    fn test_page_up_down_navigation() {
        let mut panel = TasksPanel {
            section_page_size: 5,
            ..Default::default()
        };
        for i in 0..20 {
            panel
                .task_manager
                .add_task(format!("Task {i}"), TaskSection::Backlog);
        }

        // Page down
        assert_eq!(panel.focus.index, 0);
        panel.page_down();
        assert_eq!(panel.focus.index, 5);
        panel.page_down();
        assert_eq!(panel.focus.index, 10);

        // Should clamp to last item
        panel.page_down();
        panel.page_down();
        assert_eq!(panel.focus.index, 19);

        // Page up
        panel.page_up();
        assert_eq!(panel.focus.index, 14);
        panel.page_up();
        assert_eq!(panel.focus.index, 9);
        panel.page_up();
        assert_eq!(panel.focus.index, 4);

        // Jump to near beginning and page up to boundary
        panel.focus.index = 2;
        panel.page_up();
        assert_eq!(panel.focus.index, 0);

        // Should not go negative
        panel.page_up();
        assert_eq!(panel.focus.index, 0);
    }

    #[test]
    fn test_section_navigation() {
        let mut panel = TasksPanel::default();

        // Next section cycling
        assert_eq!(panel.focus.section, TaskSection::Backlog);
        panel.next_section();
        assert_eq!(panel.focus.section, TaskSection::Current);
        panel.next_section();
        assert_eq!(panel.focus.section, TaskSection::Completed);
        panel.next_section();
        assert_eq!(panel.focus.section, TaskSection::Backlog);

        // Previous section cycling
        panel.prev_section();
        assert_eq!(panel.focus.section, TaskSection::Completed);
        panel.prev_section();
        assert_eq!(panel.focus.section, TaskSection::Current);
        panel.prev_section();
        assert_eq!(panel.focus.section, TaskSection::Backlog);
    }

    #[test]
    fn test_focus_clamping() {
        let mut panel = TasksPanel::default();

        // Clamp in empty section
        panel.focus.index = 5;
        panel.clamp_focus();
        assert_eq!(panel.focus.index, 0); // Clamped to 0 when section is empty

        // Clamp with items present
        panel
            .task_manager
            .add_task("Task 1".to_string(), TaskSection::Backlog);
        panel
            .task_manager
            .add_task("Task 2".to_string(), TaskSection::Backlog);
        panel.focus.index = 5;
        panel.clamp_focus();
        assert_eq!(panel.focus.index, 1); // Clamped to last item (index 1)

        // No clamp when already valid
        panel.focus.index = 1;
        panel.clamp_focus();
        assert_eq!(panel.focus.index, 1); // No change

        // Section switching auto-clamps focus
        for i in 0..5 {
            panel
                .task_manager
                .add_task(format!("Backlog {i}"), TaskSection::Backlog);
        }
        panel
            .task_manager
            .add_task("Current 1".to_string(), TaskSection::Current);
        panel
            .task_manager
            .add_task("Current 2".to_string(), TaskSection::Current);

        panel.focus.index = 4; // Last item in backlog (now has 7 items)
        panel.next_section(); // Switch to Current

        // Index should be clamped to 1 (last item in Current)
        assert_eq!(panel.focus.section, TaskSection::Current);
        assert_eq!(panel.focus.index, 1);
    }

    #[test]
    fn test_delete_task_from_any_section() {
        let mut panel = TasksPanel::default();
        panel
            .task_manager
            .add_task("Backlog 1".to_string(), TaskSection::Backlog);
        panel
            .task_manager
            .add_task("Backlog 2".to_string(), TaskSection::Backlog);
        panel
            .task_manager
            .add_task("Current 1".to_string(), TaskSection::Current);
        panel
            .task_manager
            .add_task("Current 2".to_string(), TaskSection::Current);
        panel
            .task_manager
            .add_task("Completed 1".to_string(), TaskSection::Completed);
        panel
            .task_manager
            .add_task("Completed 2".to_string(), TaskSection::Completed);

        // Delete from Backlog
        panel.focus.section = TaskSection::Backlog;
        panel.focus.index = 0;
        panel.key_delete_task();
        assert_eq!(panel.task_manager.section_len(TaskSection::Backlog), 1);
        assert_eq!(panel.task_manager.backlog()[0].text, "Backlog 2");
        assert_eq!(panel.focus.index, 0);

        // Delete from Current
        panel.focus.section = TaskSection::Current;
        panel.focus.index = 1;
        panel.key_delete_task();
        assert_eq!(panel.task_manager.section_len(TaskSection::Current), 1);
        assert_eq!(panel.task_manager.current()[0].text, "Current 1");
        assert_eq!(panel.focus.index, 0); // Clamped after deletion

        // Delete from Completed
        panel.focus.section = TaskSection::Completed;
        panel.focus.index = 0;
        panel.key_delete_task();
        assert_eq!(panel.task_manager.section_len(TaskSection::Completed), 1);
        assert_eq!(panel.task_manager.completed()[0].text, "Completed 2");
        assert_eq!(panel.focus.index, 0);
    }
}
