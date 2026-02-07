use std::fs;
use std::io;
use std::path::PathBuf;

use crate::task::{Task, TaskSection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncResolution {
    Incomplete,
    Complete,
    Remove,
}

#[derive(Debug, Clone)]
pub struct SyncItem {
    pub text: String,
    pub resolution: SyncResolution,
}

#[derive(Debug, Clone)]
pub struct TasksFocus {
    pub section: TaskSection,
    pub index: usize,
}

impl Default for TasksFocus {
    fn default() -> Self {
        Self {
            section: TaskSection::Backlog,
            index: 0,
        }
    }
}

pub struct TaskManager {
    file_path: Option<PathBuf>,
    original_lines: Vec<String>,
    backlog: Vec<Task>,
    current: Vec<Task>,
    completed: Vec<Task>,
    pub focus: TasksFocus,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            file_path: None,
            original_lines: Vec::new(),
            backlog: Vec::new(),
            current: Vec::new(),
            completed: Vec::new(),
            focus: TasksFocus::default(),
        }
    }

    pub fn load(path: PathBuf) -> Result<Self, io::Error> {
        let content = fs::read_to_string(&path)?;
        let original_lines: Vec<String> = content.lines().map(String::from).collect();

        let mut backlog = Vec::new();
        let mut completed = Vec::new();

        for line in &original_lines {
            let trimmed = line.trim();

            // Match unchecked tasks: - [ ] text
            if let Some(text) = trimmed.strip_prefix("- [ ] ") {
                if !text.is_empty() {
                    backlog.push(Task::new(text.to_string()));
                }
            }
            // Match checked tasks: - [x] text or - [X] text
            else if let Some(text) = trimmed
                .strip_prefix("- [x] ")
                .or_else(|| trimmed.strip_prefix("- [X] "))
            {
                if !text.is_empty() {
                    completed.push(Task::new(text.to_string()));
                }
            }
        }

        Ok(Self {
            file_path: Some(path),
            original_lines,
            backlog,
            current: Vec::new(),
            completed,
            focus: TasksFocus::default(),
        })
    }

    /// Compute sync items by comparing app state with file state.
    /// Only tasks where app and file disagree appear in the list.
    pub fn compute_sync_items(&self) -> Result<Vec<SyncItem>, io::Error> {
        let Some(path) = &self.file_path else {
            return Ok(Vec::new());
        };

        let content = fs::read_to_string(path)?;
        let file_lines: Vec<String> = content.lines().map(String::from).collect();

        // Parse file state
        let mut file_unchecked: Vec<String> = Vec::new();
        let mut file_checked: Vec<String> = Vec::new();

        for line in &file_lines {
            let trimmed = line.trim();
            if let Some(text) = trimmed.strip_prefix("- [ ] ") {
                if !text.is_empty() {
                    file_unchecked.push(text.to_string());
                }
            } else if let Some(text) = trimmed
                .strip_prefix("- [x] ")
                .or_else(|| trimmed.strip_prefix("- [X] "))
            {
                if !text.is_empty() {
                    file_checked.push(text.to_string());
                }
            }
        }

        // Collect app state
        let app_incomplete: Vec<String> = self
            .backlog
            .iter()
            .chain(self.current.iter())
            .map(|t| t.text.clone())
            .collect();
        let app_complete: Vec<String> = self.completed.iter().map(|t| t.text.clone()).collect();

        let mut items = Vec::new();

        // New unchecked tasks in file, not in app → default Incomplete
        for text in &file_unchecked {
            if !app_incomplete.contains(text) && !app_complete.contains(text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Incomplete,
                });
            }
        }

        // New checked tasks in file, not in app → default Complete
        for text in &file_checked {
            if !app_incomplete.contains(text) && !app_complete.contains(text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Complete,
                });
            }
        }

        // App incomplete but checked in file → default Complete
        for text in &app_incomplete {
            if file_checked.contains(text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Complete,
                });
            }
        }

        // App complete but unchecked in file → default Complete
        for text in &app_complete {
            if file_unchecked.contains(text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Complete,
                });
            }
        }

        let all_file_tasks: Vec<&String> =
            file_unchecked.iter().chain(file_checked.iter()).collect();

        // App incomplete, not in file at all → default Remove
        for text in &app_incomplete {
            if !all_file_tasks.contains(&text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Remove,
                });
            }
        }

        // App complete, not in file at all → default Remove
        for text in &app_complete {
            if !all_file_tasks.contains(&text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Remove,
                });
            }
        }

        Ok(items)
    }

    /// Apply resolved sync items to both app state and file.
    pub fn apply_sync(&mut self, items: &[SyncItem]) -> Result<(), io::Error> {
        // Apply to app state
        for item in items {
            match item.resolution {
                SyncResolution::Incomplete => {
                    // Remove from completed if present
                    self.completed.retain(|t| t.text != item.text);
                    // Add to backlog if not already in backlog or current
                    if !self.backlog.iter().any(|t| t.text == item.text)
                        && !self.current.iter().any(|t| t.text == item.text)
                    {
                        self.backlog.push(Task::new(item.text.clone()));
                    }
                }
                SyncResolution::Complete => {
                    // Remove from backlog and current
                    self.backlog.retain(|t| t.text != item.text);
                    self.current.retain(|t| t.text != item.text);
                    // Add to completed if not already there
                    if !self.completed.iter().any(|t| t.text == item.text) {
                        self.completed.push(Task::new(item.text.clone()));
                    }
                }
                SyncResolution::Remove => {
                    self.backlog.retain(|t| t.text != item.text);
                    self.current.retain(|t| t.text != item.text);
                    self.completed.retain(|t| t.text != item.text);
                }
            }
        }

        // Clamp focus
        let section_len = self.section_len(self.focus.section);
        if self.focus.index >= section_len {
            self.focus.index = section_len.saturating_sub(1);
        }

        // Apply to file
        if let Some(path) = &self.file_path {
            let content = fs::read_to_string(path)?;
            let mut file_lines: Vec<String> = content.lines().map(String::from).collect();
            let mut used: Vec<usize> = Vec::new();
            let mut lines_to_remove: Vec<usize> = Vec::new();

            for item in items {
                if let Some(line_idx) = self.find_line_index(&item.text, &file_lines, &used) {
                    let trimmed = file_lines[line_idx].trim();
                    let indent =
                        &file_lines[line_idx][..file_lines[line_idx].len() - trimmed.len()];
                    match item.resolution {
                        SyncResolution::Incomplete => {
                            file_lines[line_idx] = format!("{}- [ ] {}", indent, item.text);
                        }
                        SyncResolution::Complete => {
                            file_lines[line_idx] = format!("{}- [x] {}", indent, item.text);
                        }
                        SyncResolution::Remove => {
                            lines_to_remove.push(line_idx);
                        }
                    }
                    used.push(line_idx);
                }
            }

            // Remove lines in reverse order to preserve indices
            lines_to_remove.sort_unstable();
            for idx in lines_to_remove.into_iter().rev() {
                file_lines.remove(idx);
            }

            let output = file_lines.join("\n");
            fs::write(path, output)?;
            self.original_lines = file_lines;
        }

        Ok(())
    }

    fn find_line_index(
        &self,
        task_text: &str,
        file_lines: &[String],
        used_lines: &[usize],
    ) -> Option<usize> {
        for (idx, line) in file_lines.iter().enumerate() {
            if used_lines.contains(&idx) {
                continue;
            }
            let trimmed = line.trim();
            // Check both unchecked and checked formats
            if let Some(text) = trimmed.strip_prefix("- [ ] ") {
                if text == task_text {
                    return Some(idx);
                }
            } else if let Some(text) = trimmed
                .strip_prefix("- [x] ")
                .or_else(|| trimmed.strip_prefix("- [X] "))
            {
                if text == task_text {
                    return Some(idx);
                }
            }
        }
        None
    }

    pub fn backlog(&self) -> &[Task] {
        &self.backlog
    }

    pub fn current(&self) -> &[Task] {
        &self.current
    }

    pub fn completed(&self) -> &[Task] {
        &self.completed
    }

    pub fn active_task(&self) -> Option<&Task> {
        self.current.first()
    }

    fn section_len(&self, section: TaskSection) -> usize {
        match section {
            TaskSection::Backlog => self.backlog.len(),
            TaskSection::Current => self.current.len(),
            TaskSection::Completed => self.completed.len(),
        }
    }

    fn section_mut(&mut self, section: TaskSection) -> &mut Vec<Task> {
        match section {
            TaskSection::Backlog => &mut self.backlog,
            TaskSection::Current => &mut self.current,
            TaskSection::Completed => &mut self.completed,
        }
    }

    pub fn move_down(&mut self) {
        let section_len = self.section_len(self.focus.section);
        if section_len > 0 && self.focus.index + 1 < section_len {
            self.focus.index += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.focus.index > 0 {
            self.focus.index -= 1;
        }
    }

    pub fn next_section(&mut self) {
        self.focus.section = match self.focus.section {
            TaskSection::Backlog => TaskSection::Current,
            TaskSection::Current => TaskSection::Completed,
            TaskSection::Completed => TaskSection::Backlog,
        };
        // Clamp index to new section length
        let section_len = self.section_len(self.focus.section);
        if self.focus.index >= section_len {
            self.focus.index = section_len.saturating_sub(1);
        }
    }

    pub fn prev_section(&mut self) {
        self.focus.section = match self.focus.section {
            TaskSection::Backlog => TaskSection::Completed,
            TaskSection::Current => TaskSection::Backlog,
            TaskSection::Completed => TaskSection::Current,
        };
        // Clamp index to new section length
        let section_len = self.section_len(self.focus.section);
        if self.focus.index >= section_len {
            self.focus.index = section_len.saturating_sub(1);
        }
    }

    pub fn reorder_down(&mut self) {
        let idx = self.focus.index;
        let focus_section = self.focus.section;
        let section = self.section_mut(focus_section);
        if idx + 1 < section.len() {
            section.swap(idx, idx + 1);
            self.focus.index += 1;
        }
    }

    pub fn reorder_up(&mut self) {
        let idx = self.focus.index;
        if idx > 0 {
            let focus_section = self.focus.section;
            let section = self.section_mut(focus_section);
            section.swap(idx, idx - 1);
            self.focus.index -= 1;
        }
    }

    /// Move task between Backlog <-> Current (Enter key)
    pub fn toggle_section(&mut self) {
        match self.focus.section {
            TaskSection::Backlog => {
                if self.focus.index < self.backlog.len() {
                    let task = self.backlog.remove(self.focus.index);
                    self.current.push(task);
                    // Adjust focus if needed
                    if self.focus.index >= self.backlog.len() && !self.backlog.is_empty() {
                        self.focus.index = self.backlog.len() - 1;
                    }
                }
            }
            TaskSection::Current => {
                if self.focus.index < self.current.len() {
                    let task = self.current.remove(self.focus.index);
                    self.backlog.push(task);
                    // Adjust focus if needed
                    if self.focus.index >= self.current.len() && !self.current.is_empty() {
                        self.focus.index = self.current.len() - 1;
                    }
                }
            }
            TaskSection::Completed => {
                // Can't move completed tasks back with Enter
            }
        }
    }

    /// Toggle task completion (x key in tasks panel)
    /// Current -> Completed, Completed -> Backlog
    pub fn complete_focused(&mut self) {
        match self.focus.section {
            TaskSection::Current => {
                if self.focus.index < self.current.len() {
                    let task = self.current.remove(self.focus.index);
                    self.completed.push(task);
                    // Adjust focus if needed
                    if self.focus.index >= self.current.len() && !self.current.is_empty() {
                        self.focus.index = self.current.len() - 1;
                    }
                }
            }
            TaskSection::Completed => {
                if self.focus.index < self.completed.len() {
                    let task = self.completed.remove(self.focus.index);
                    self.backlog.push(task);
                    // Adjust focus if needed
                    if self.focus.index >= self.completed.len() && !self.completed.is_empty() {
                        self.focus.index = self.completed.len() - 1;
                    }
                }
            }
            TaskSection::Backlog => {
                // x does nothing in backlog - use Enter to move to Current
            }
        }
    }

    /// Complete the active (first current) task (c key in timer panel)
    pub fn complete_active(&mut self) {
        if !self.current.is_empty() {
            let task = self.current.remove(0);
            self.completed.push(task);
            // Adjust focus if in current section
            if self.focus.section == TaskSection::Current
                && self.focus.index >= self.current.len()
                && !self.current.is_empty()
            {
                self.focus.index = self.current.len() - 1;
            }
        }
    }
}
