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
    /// Load and parse a task file. Returns the TaskFile handle and parsed tasks.
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

    /// Re-read the task file from disk and return parsed tasks.
    pub fn read_tasks(&self) -> Result<ParsedTasks, io::Error> {
        let content = fs::read_to_string(&self.path)?;
        let lines: Vec<String> = content.lines().map(String::from).collect();
        Ok(parse_task_lines(&lines))
    }

    /// Write resolved sync items back to the file.
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

/// Parse markdown task lines into incomplete and complete text vectors.
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
