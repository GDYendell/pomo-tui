use std::fs;
use std::io;
use std::path::PathBuf;

use crate::overlays::{SyncItem, SyncResolution};

/// Parsed task file: incomplete and complete task text vectors.
pub struct ParsedTasks {
    pub incomplete: Vec<String>,
    pub complete: Vec<String>,
}

/// Handles reading/writing the markdown task file.
pub struct TaskFile {
    path: PathBuf,
    original_lines: Vec<String>,
}

impl TaskFile {
    /// Load and parse a task file
    ///
    /// Returns the `TaskFile` handle and parsed tasks.
    pub fn load(path: PathBuf) -> Result<(Self, ParsedTasks), io::Error> {
        let content = fs::read_to_string(&path)?;
        let original_lines: Vec<String> = content.lines().map(String::from).collect();
        let parsed = parse_task_lines(&original_lines);
        Ok((
            Self {
                path,
                original_lines,
            },
            parsed,
        ))
    }

    /// Re-read the task file from disk and return parsed tasks
    pub fn read_tasks(&self) -> Result<ParsedTasks, io::Error> {
        let content = fs::read_to_string(&self.path)?;
        let lines: Vec<String> = content.lines().map(String::from).collect();
        Ok(parse_task_lines(&lines))
    }

    /// Apply sync item resolutions to the file, preserving indentation and line order
    pub fn write_sync(&mut self, items: &[SyncItem]) -> Result<(), io::Error> {
        let content = fs::read_to_string(&self.path)?;
        let mut file_lines: Vec<String> = content.lines().map(String::from).collect();
        let mut used: Vec<usize> = Vec::new();
        let mut lines_to_remove: Vec<usize> = Vec::new();

        for item in items {
            if let Some(line_idx) = find_line_index(&item.text, &file_lines, &used) {
                let trimmed = file_lines[line_idx].trim();
                let indent = &file_lines[line_idx][..file_lines[line_idx].len() - trimmed.len()];
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
            } else if item.resolution != SyncResolution::Remove {
                let new_line = match item.resolution {
                    SyncResolution::Incomplete => format!("- [ ] {}", item.text),
                    SyncResolution::Complete => format!("- [x] {}", item.text),
                    SyncResolution::Remove => unreachable!(),
                };
                file_lines.push(new_line);
            }
        }

        lines_to_remove.sort_unstable();
        for idx in lines_to_remove.into_iter().rev() {
            file_lines.remove(idx);
        }

        let output = file_lines.join("\n");
        fs::write(&self.path, output)?;
        self.original_lines = file_lines;

        Ok(())
    }
}

/// Parse markdown task lines into incomplete and complete text vectors
fn parse_task_lines(lines: &[String]) -> ParsedTasks {
    let mut incomplete = Vec::new();
    let mut complete = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if let Some(text) = trimmed.strip_prefix("- [ ] ") {
            if !text.is_empty() {
                incomplete.push(text.to_string());
            }
        } else if let Some(text) = trimmed
            .strip_prefix("- [x] ")
            .or_else(|| trimmed.strip_prefix("- [X] "))
        {
            if !text.is_empty() {
                complete.push(text.to_string());
            }
        }
    }

    ParsedTasks {
        incomplete,
        complete,
    }
}

/// Find the line index of a task, skipping already-used lines to handle duplicates
fn find_line_index(task_text: &str, file_lines: &[String], used_lines: &[usize]) -> Option<usize> {
    for (idx, line) in file_lines.iter().enumerate() {
        if used_lines.contains(&idx) {
            continue;
        }
        let trimmed = line.trim();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_task_lines_empty() {
        let lines = vec![];
        let parsed = parse_task_lines(&lines);
        assert_eq!(parsed.incomplete.len(), 0);
        assert_eq!(parsed.complete.len(), 0);
    }

    #[test]
    fn test_parse_task_lines_mixed() {
        let lines = vec![
            "# Tasks".to_string(),
            "- [ ] Incomplete task 1".to_string(),
            "- [x] Complete task 1".to_string(),
            "- [ ] Incomplete task 2".to_string(),
            "- [X] Complete task 2".to_string(), // Capital X
            "Some random text".to_string(),
            "  - [ ] Indented incomplete".to_string(),
            "  - [x] Indented complete".to_string(),
        ];
        let parsed = parse_task_lines(&lines);
        assert_eq!(parsed.incomplete.len(), 3);
        assert_eq!(parsed.complete.len(), 3);
        assert_eq!(parsed.incomplete[0], "Incomplete task 1");
        assert_eq!(parsed.incomplete[1], "Incomplete task 2");
        assert_eq!(parsed.incomplete[2], "Indented incomplete");
        assert_eq!(parsed.complete[0], "Complete task 1");
        assert_eq!(parsed.complete[1], "Complete task 2");
        assert_eq!(parsed.complete[2], "Indented complete");
    }

    #[test]
    fn test_parse_task_lines_ignores_empty() {
        let lines = vec![
            "- [ ] ".to_string(), // Empty task
            "- [x] ".to_string(), // Empty task
            "- [ ] Valid task".to_string(),
        ];
        let parsed = parse_task_lines(&lines);
        assert_eq!(parsed.incomplete.len(), 1);
        assert_eq!(parsed.complete.len(), 0);
        assert_eq!(parsed.incomplete[0], "Valid task");
    }

    #[test]
    fn test_find_line_index() {
        let lines = vec![
            "# Header".to_string(),
            "- [ ] Task 1".to_string(),
            "- [x] Task 2".to_string(),
            "- [ ] Task 3".to_string(),
        ];

        // Find incomplete task
        assert_eq!(find_line_index("Task 1", &lines, &[]), Some(1));

        // Find complete task
        assert_eq!(find_line_index("Task 2", &lines, &[]), Some(2));

        // Task not found
        assert_eq!(find_line_index("Nonexistent", &lines, &[]), None);

        // Task already used
        assert_eq!(find_line_index("Task 1", &lines, &[1]), None);
    }

    #[test]
    fn test_task_file_load() -> Result<(), io::Error> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_tasks.md");

        let content = "# Tasks\n- [ ] Task 1\n- [x] Task 2\n- [ ] Task 3";
        fs::write(&file_path, content)?;

        let (task_file, parsed) = TaskFile::load(file_path.clone())?;
        assert_eq!(task_file.path, file_path);
        assert_eq!(parsed.incomplete.len(), 2);
        assert_eq!(parsed.complete.len(), 1);
        assert_eq!(parsed.incomplete[0], "Task 1");
        assert_eq!(parsed.complete[0], "Task 2");

        Ok(())
    }

    #[test]
    fn test_task_file_read_tasks() -> Result<(), io::Error> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_tasks.md");

        let content = "- [ ] Initial task";
        fs::write(&file_path, content)?;

        let (task_file, _) = TaskFile::load(file_path.clone())?;

        // Modify file on disk
        let new_content = "- [ ] Initial task\n- [x] New completed task";
        fs::write(&file_path, new_content)?;

        let parsed = task_file.read_tasks()?;
        assert_eq!(parsed.incomplete.len(), 1);
        assert_eq!(parsed.complete.len(), 1);
        assert_eq!(parsed.complete[0], "New completed task");

        Ok(())
    }

    #[test]
    fn test_write_sync_mark_complete() -> Result<(), io::Error> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_tasks.md");

        let content = "- [ ] Task 1\n- [ ] Task 2";
        fs::write(&file_path, content)?;

        let (mut task_file, _) = TaskFile::load(file_path.clone())?;

        let sync_items = vec![SyncItem {
            text: "Task 1".to_string(),
            resolution: SyncResolution::Complete,
        }];

        task_file.write_sync(&sync_items)?;

        let result = fs::read_to_string(&file_path)?;
        assert!(result.contains("- [x] Task 1"));
        assert!(result.contains("- [ ] Task 2"));

        Ok(())
    }

    #[test]
    fn test_write_sync_mark_incomplete() -> Result<(), io::Error> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_tasks.md");

        let content = "- [x] Task 1\n- [ ] Task 2";
        fs::write(&file_path, content)?;

        let (mut task_file, _) = TaskFile::load(file_path.clone())?;

        let sync_items = vec![SyncItem {
            text: "Task 1".to_string(),
            resolution: SyncResolution::Incomplete,
        }];

        task_file.write_sync(&sync_items)?;

        let result = fs::read_to_string(&file_path)?;
        assert!(result.contains("- [ ] Task 1"));
        assert!(result.contains("- [ ] Task 2"));

        Ok(())
    }

    #[test]
    fn test_write_sync_add_new_task() -> Result<(), io::Error> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_tasks.md");

        let content = "- [ ] Task 1";
        fs::write(&file_path, content)?;

        let (mut task_file, _) = TaskFile::load(file_path.clone())?;

        let sync_items = vec![SyncItem {
            text: "New Task".to_string(),
            resolution: SyncResolution::Incomplete,
        }];

        task_file.write_sync(&sync_items)?;

        let result = fs::read_to_string(&file_path)?;
        assert!(result.contains("- [ ] Task 1"));
        assert!(result.contains("- [ ] New Task"));

        Ok(())
    }

    #[test]
    fn test_write_sync_remove_task() -> Result<(), io::Error> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_tasks.md");

        let content = "- [ ] Task 1\n- [ ] Task 2\n- [ ] Task 3";
        fs::write(&file_path, content)?;

        let (mut task_file, _) = TaskFile::load(file_path.clone())?;

        let sync_items = vec![SyncItem {
            text: "Task 2".to_string(),
            resolution: SyncResolution::Remove,
        }];

        task_file.write_sync(&sync_items)?;

        let result = fs::read_to_string(&file_path)?;
        assert!(result.contains("- [ ] Task 1"));
        assert!(!result.contains("Task 2"));
        assert!(result.contains("- [ ] Task 3"));

        Ok(())
    }

    #[test]
    fn test_write_sync_preserves_indentation() -> Result<(), io::Error> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_tasks.md");

        let content = "  - [ ] Indented task";
        fs::write(&file_path, content)?;

        let (mut task_file, _) = TaskFile::load(file_path.clone())?;

        let sync_items = vec![SyncItem {
            text: "Indented task".to_string(),
            resolution: SyncResolution::Complete,
        }];

        task_file.write_sync(&sync_items)?;

        let result = fs::read_to_string(&file_path)?;
        assert!(result.contains("  - [x] Indented task"));

        Ok(())
    }

    #[test]
    fn test_write_sync_multiple_operations() -> Result<(), io::Error> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_tasks.md");

        let content = "- [ ] Task 1\n- [x] Task 2\n- [ ] Task 3";
        fs::write(&file_path, content)?;

        let (mut task_file, _) = TaskFile::load(file_path.clone())?;

        let sync_items = vec![
            SyncItem {
                text: "Task 1".to_string(),
                resolution: SyncResolution::Complete,
            },
            SyncItem {
                text: "Task 2".to_string(),
                resolution: SyncResolution::Incomplete,
            },
            SyncItem {
                text: "Task 3".to_string(),
                resolution: SyncResolution::Remove,
            },
            SyncItem {
                text: "New Task 4".to_string(),
                resolution: SyncResolution::Incomplete,
            },
        ];

        task_file.write_sync(&sync_items)?;

        let result = fs::read_to_string(&file_path)?;
        assert!(result.contains("- [x] Task 1"));
        assert!(result.contains("- [ ] Task 2"));
        assert!(!result.contains("Task 3"));
        assert!(result.contains("- [ ] New Task 4"));

        Ok(())
    }
}
