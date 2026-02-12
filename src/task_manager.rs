use std::io;
use std::path::PathBuf;

use crate::fileio::TaskFile;
use crate::overlays::{SyncItem, SyncResolution};
use crate::task::{Task, TaskSection};

pub struct TaskManager {
    file: Option<TaskFile>,
    backlog: Vec<Task>,
    current: Vec<Task>,
    completed: Vec<Task>,
}

impl TaskManager {
    pub const fn new() -> Self {
        Self {
            file: None,
            backlog: Vec::new(),
            current: Vec::new(),
            completed: Vec::new(),
        }
    }

    pub fn load(path: PathBuf) -> Result<Self, io::Error> {
        let (file, parsed) = TaskFile::load(path)?;
        Ok(Self {
            file: Some(file),
            backlog: parsed.incomplete.into_iter().map(Task::new).collect(),
            current: Vec::new(),
            completed: parsed.complete.into_iter().map(Task::new).collect(),
        })
    }

    pub fn compute_sync_items(&self) -> Result<Vec<SyncItem>, io::Error> {
        let Some(ref file) = self.file else {
            return Ok(Vec::new());
        };

        let file_tasks = file.read_tasks()?;

        let app_incomplete: Vec<String> = self
            .backlog
            .iter()
            .chain(self.current.iter())
            .map(|t| t.text.clone())
            .collect();
        let app_complete: Vec<String> = self.completed.iter().map(|t| t.text.clone()).collect();

        let mut items = Vec::new();

        // New incomplete tasks in file, not in app
        for text in &file_tasks.incomplete {
            if !app_incomplete.contains(text) && !app_complete.contains(text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Incomplete,
                });
            }
        }

        // New complete tasks in file, not in app
        for text in &file_tasks.complete {
            if !app_incomplete.contains(text) && !app_complete.contains(text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Complete,
                });
            }
        }

        // App incomplete but complete in file
        for text in &app_incomplete {
            if file_tasks.complete.contains(text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Complete,
                });
            }
        }

        // App complete but incomplete in file
        for text in &app_complete {
            if file_tasks.incomplete.contains(text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Complete,
                });
            }
        }

        let all_file_tasks: Vec<&String> = file_tasks
            .incomplete
            .iter()
            .chain(file_tasks.complete.iter())
            .collect();

        // App-only tasks not in file
        for text in &app_incomplete {
            if !all_file_tasks.contains(&text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Incomplete,
                });
            }
        }
        for text in &app_complete {
            if !all_file_tasks.contains(&text) {
                items.push(SyncItem {
                    text: text.clone(),
                    resolution: SyncResolution::Complete,
                });
            }
        }

        Ok(items)
    }

    pub fn apply_sync(&mut self, items: &[SyncItem]) -> Result<(), io::Error> {
        // Apply to app state
        for item in items {
            match item.resolution {
                SyncResolution::Incomplete => {
                    self.completed.retain(|t| t.text != item.text);
                    if !self.backlog.iter().any(|t| t.text == item.text)
                        && !self.current.iter().any(|t| t.text == item.text)
                    {
                        self.backlog.push(Task::new(item.text.clone()));
                    }
                }
                SyncResolution::Complete => {
                    self.backlog.retain(|t| t.text != item.text);
                    self.current.retain(|t| t.text != item.text);
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

        // Write to file
        if let Some(ref mut file) = self.file {
            file.write_sync(items)?;
        }

        Ok(())
    }

    pub fn add_task(&mut self, text: String, section: TaskSection) {
        self.section_tasks(section).push(Task::new(text));
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

    pub const fn has_file_path(&self) -> bool {
        self.file.is_some()
    }

    pub fn active_task(&self) -> Option<&Task> {
        self.current.first()
    }

    pub const fn section_len(&self, section: TaskSection) -> usize {
        match section {
            TaskSection::Backlog => self.backlog.len(),
            TaskSection::Current => self.current.len(),
            TaskSection::Completed => self.completed.len(),
        }
    }

    fn section_tasks(&mut self, section: TaskSection) -> &mut Vec<Task> {
        match section {
            TaskSection::Backlog => &mut self.backlog,
            TaskSection::Current => &mut self.current,
            TaskSection::Completed => &mut self.completed,
        }
    }

    pub fn reorder_down(&mut self, section: TaskSection, index: usize) {
        let tasks = self.section_tasks(section);
        if index + 1 < tasks.len() {
            tasks.swap(index, index + 1);
        }
    }

    pub fn reorder_up(&mut self, section: TaskSection, index: usize) {
        if index > 0 {
            let tasks = self.section_tasks(section);
            tasks.swap(index, index - 1);
        }
    }

    pub fn toggle_section(&mut self, section: TaskSection, index: usize) {
        match section {
            TaskSection::Backlog => {
                if index < self.backlog.len() {
                    let task = self.backlog.remove(index);
                    self.current.push(task);
                }
            }
            TaskSection::Current => {
                if index < self.current.len() {
                    let task = self.current.remove(index);
                    self.backlog.push(task);
                }
            }
            TaskSection::Completed => {}
        }
    }

    pub fn complete_focused(&mut self, section: TaskSection, index: usize) {
        match section {
            TaskSection::Current => {
                if index < self.current.len() {
                    let task = self.current.remove(index);
                    self.completed.push(task);
                }
            }
            TaskSection::Completed => {
                if index < self.completed.len() {
                    let task = self.completed.remove(index);
                    self.backlog.push(task);
                }
            }
            TaskSection::Backlog => {}
        }
    }

    pub fn complete_active(&mut self) {
        if !self.current.is_empty() {
            let task = self.current.remove(0);
            self.completed.push(task);
        }
    }

    pub fn delete_task(&mut self, section: TaskSection, index: usize) {
        let tasks = self.section_tasks(section);
        if index < tasks.len() {
            tasks.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task_manager() {
        let tm = TaskManager::new();
        assert_eq!(tm.backlog().len(), 0);
        assert_eq!(tm.current().len(), 0);
        assert_eq!(tm.completed().len(), 0);
        assert!(!tm.has_file_path());
    }

    #[test]
    fn test_add_task_to_sections() {
        let mut tm = TaskManager::new();

        tm.add_task("Task 1".to_string(), TaskSection::Backlog);
        tm.add_task("Task 2".to_string(), TaskSection::Current);
        tm.add_task("Task 3".to_string(), TaskSection::Completed);

        assert_eq!(tm.section_len(TaskSection::Backlog), 1);
        assert_eq!(tm.section_len(TaskSection::Current), 1);
        assert_eq!(tm.section_len(TaskSection::Completed), 1);
        assert_eq!(tm.backlog()[0].text, "Task 1");
        assert_eq!(tm.current()[0].text, "Task 2");
        assert_eq!(tm.completed()[0].text, "Task 3");
    }

    #[test]
    fn test_toggle_section() {
        let mut tm = TaskManager::new();
        tm.add_task("Task 1".to_string(), TaskSection::Backlog);
        tm.add_task("Task 2".to_string(), TaskSection::Backlog);

        // Backlog → Current
        tm.toggle_section(TaskSection::Backlog, 0);
        assert_eq!(tm.section_len(TaskSection::Backlog), 1);
        assert_eq!(tm.section_len(TaskSection::Current), 1);
        assert_eq!(tm.backlog()[0].text, "Task 2");
        assert_eq!(tm.current()[0].text, "Task 1");

        // Current → Backlog
        tm.toggle_section(TaskSection::Current, 0);
        assert_eq!(tm.section_len(TaskSection::Current), 0);
        assert_eq!(tm.section_len(TaskSection::Backlog), 2);
        assert_eq!(tm.backlog()[0].text, "Task 2");
        assert_eq!(tm.backlog()[1].text, "Task 1");
    }

    #[test]
    fn test_task_completion() {
        let mut tm = TaskManager::new();
        tm.add_task("Task 1".to_string(), TaskSection::Current);
        tm.add_task("Task 2".to_string(), TaskSection::Current);

        // Complete focused from current → completed
        tm.complete_focused(TaskSection::Current, 0);
        assert_eq!(tm.section_len(TaskSection::Current), 1);
        assert_eq!(tm.section_len(TaskSection::Completed), 1);
        assert_eq!(tm.current()[0].text, "Task 2");
        assert_eq!(tm.completed()[0].text, "Task 1");

        // Complete active (first in current)
        tm.complete_active();
        assert_eq!(tm.section_len(TaskSection::Current), 0);
        assert_eq!(tm.section_len(TaskSection::Completed), 2);
        assert_eq!(tm.completed()[1].text, "Task 2");

        // Un-complete: completed → backlog
        tm.complete_focused(TaskSection::Completed, 0);
        assert_eq!(tm.section_len(TaskSection::Completed), 1);
        assert_eq!(tm.section_len(TaskSection::Backlog), 1);
        assert_eq!(tm.backlog()[0].text, "Task 1");
    }

    #[test]
    fn test_active_task() {
        let mut tm = TaskManager::new();
        assert!(tm.active_task().is_none());

        tm.add_task("Task 1".to_string(), TaskSection::Current);
        assert_eq!(tm.active_task().map(|t| &t.text), Some(&"Task 1".to_string()));

        tm.add_task("Task 2".to_string(), TaskSection::Current);
        assert_eq!(tm.active_task().map(|t| &t.text), Some(&"Task 1".to_string()));
    }

    #[test]
    fn test_reorder_tasks() {
        let mut tm = TaskManager::new();
        tm.add_task("Task 1".to_string(), TaskSection::Backlog);
        tm.add_task("Task 2".to_string(), TaskSection::Backlog);
        tm.add_task("Task 3".to_string(), TaskSection::Backlog);

        // Reorder down (swap 0 and 1)
        tm.reorder_down(TaskSection::Backlog, 0);
        assert_eq!(tm.backlog()[0].text, "Task 2");
        assert_eq!(tm.backlog()[1].text, "Task 1");
        assert_eq!(tm.backlog()[2].text, "Task 3");

        // Reorder up (swap 1 and 2)
        tm.reorder_up(TaskSection::Backlog, 2);
        assert_eq!(tm.backlog()[0].text, "Task 2");
        assert_eq!(tm.backlog()[1].text, "Task 3");
        assert_eq!(tm.backlog()[2].text, "Task 1");

        // Try to move first item up (should do nothing)
        tm.reorder_up(TaskSection::Backlog, 0);
        assert_eq!(tm.backlog()[0].text, "Task 2");

        // Try to move last item down (should do nothing)
        tm.reorder_down(TaskSection::Backlog, 2);
        assert_eq!(tm.backlog()[2].text, "Task 1");
    }

    #[test]
    fn test_delete_task_from_backlog() {
        let mut tm = TaskManager::new();
        tm.add_task("Task 1".to_string(), TaskSection::Backlog);
        tm.add_task("Task 2".to_string(), TaskSection::Backlog);
        tm.add_task("Task 3".to_string(), TaskSection::Backlog);

        tm.delete_task(TaskSection::Backlog, 1);
        assert_eq!(tm.section_len(TaskSection::Backlog), 2);
        assert_eq!(tm.backlog()[0].text, "Task 1");
        assert_eq!(tm.backlog()[1].text, "Task 3");
    }

    #[test]
    fn test_delete_task_from_current() {
        let mut tm = TaskManager::new();
        tm.add_task("Task 1".to_string(), TaskSection::Current);
        tm.add_task("Task 2".to_string(), TaskSection::Current);

        tm.delete_task(TaskSection::Current, 0);
        assert_eq!(tm.section_len(TaskSection::Current), 1);
        assert_eq!(tm.current()[0].text, "Task 2");
    }

    #[test]
    fn test_delete_task_from_completed() {
        let mut tm = TaskManager::new();
        tm.add_task("Task 1".to_string(), TaskSection::Completed);
        tm.add_task("Task 2".to_string(), TaskSection::Completed);
        tm.add_task("Task 3".to_string(), TaskSection::Completed);

        tm.delete_task(TaskSection::Completed, 2);
        assert_eq!(tm.section_len(TaskSection::Completed), 2);
        assert_eq!(tm.completed()[0].text, "Task 1");
        assert_eq!(tm.completed()[1].text, "Task 2");
    }

    #[test]
    fn test_delete_task_invalid_index() {
        let mut tm = TaskManager::new();
        tm.add_task("Task 1".to_string(), TaskSection::Backlog);

        // Try to delete with invalid index (should do nothing)
        tm.delete_task(TaskSection::Backlog, 5);
        assert_eq!(tm.section_len(TaskSection::Backlog), 1);
        assert_eq!(tm.backlog()[0].text, "Task 1");
    }

    #[test]
    fn test_delete_from_empty_section() {
        let mut tm = TaskManager::new();

        // Try to delete from empty section (should do nothing)
        tm.delete_task(TaskSection::Backlog, 0);
        assert_eq!(tm.section_len(TaskSection::Backlog), 0);
    }
}
