# Phase 3: Task Management Implementation Plan

## Overview
Add task management to pomo-tui: load tasks from a markdown file, manage them across three sections (Backlog, Current, Completed), and integrate with the timer panel.

## Requirements Summary
- **File source**: CLI argument (e.g., `pomo-tui tasks.md`)
- **Persistence**: Sync completed tasks to file with `s` keybinding
- **Task completion**: Manual (user presses key to mark complete)
- **Keybindings**: Vim-style (j/k navigate, J/K reorder, Enter move sections, x complete)

---

## Files to Create

| File | Purpose |
|------|---------|
| `src/task.rs` | `Task` struct and `TaskSection` enum |
| `src/task_manager.rs` | `TaskManager` with file I/O, task manipulation, focus state |

## Files to Modify

| File | Changes |
|------|---------|
| `src/main.rs` | Add CLI argument parsing, pass file path to App |
| `src/app.rs` | Add `TaskManager`, coordinate timer-task integration |
| `src/panels/tasks.rs` | Full rewrite: render tasks, handle navigation, focus state |
| `src/panels/timer.rs` | Receive active task, render task text, handle `c` to complete |

---

## Data Structures

### `src/task.rs`
```rust
pub enum TaskSection {
    Backlog,
    Current,
    Completed,
}

pub struct Task {
    pub text: String,
    pub line_number: usize,  // For syncing back to file
}
```

### `src/task_manager.rs`
```rust
pub struct TaskManager {
    file_path: Option<PathBuf>,
    original_lines: Vec<String>,  // For sync
    backlog: Vec<Task>,
    current: Vec<Task>,
    completed: Vec<Task>,
}

pub struct TasksFocus {
    pub section: TaskSection,
    pub index: Option<usize>,
}
```

---

## Implementation Steps

### Step 1: Core Task Infrastructure
1. Create `src/task.rs` with `Task` and `TaskSection`
2. Create `src/task_manager.rs` with:
   - Markdown parser: matches `- [ ] text` and `- [x] text`
   - `load(path)` → loads file, populates backlog (unchecked) and completed (checked)
   - `sync_to_file()` → updates original lines, writes back
3. Add `mod task; mod task_manager;` to `main.rs`

### Step 2: CLI Argument Parsing
- Modify `main.rs` to accept optional positional argument for file path
- Use `std::env::args()` (no extra dependencies)
- Pass `Option<PathBuf>` to App constructor

### Step 3: Integrate TaskManager into App
- Add `task_manager: TaskManager` field to `App`
- Create `App::new(file_path: Option<PathBuf>)` constructor
- Load tasks on startup if file provided

### Step 4: Tasks Panel Rendering
- Add `focus: TasksFocus` to `TasksPanel`
- Render actual tasks using `ratatui::widgets::List` with `ListState`
- Highlight selected task, show focus indicator on section title
- Handle scrolling for long task lists

### Step 5: Tasks Panel Navigation
- `j` / `k`: Move selection within section, wrap to adjacent sections
- `J` / `K`: Reorder task within section
- `Enter`: Move task between Backlog ↔ Current
- `x`: Complete task (Current → Completed)
- `s`: Sync completed tasks to file

### Step 6: Timer Panel Integration
- Modify `render_current_task()` to accept `Option<&Task>`
- Display first task from Current section as active task
- Add `c` keybinding in timer panel to complete active task
- When completed: move to Completed, next Current task becomes active

### Step 7: Wire Up Key Handling
- `TasksPanel::handle_key()` returns `Option<TaskAction>` enum
- `App` matches on action and calls `TaskManager` methods
- Timer panel `c` key triggers task completion through App

### Step 8: Polish
- Update shortcuts bar with task panel shortcuts
- Handle edge cases (empty sections, no file, file errors)
- Error display for file load/sync failures

---

## Key Integration Points

```
┌─────────────┐      owns       ┌─────────────────┐
│    App      │───────────────→│  TaskManager    │
└─────────────┘                 └─────────────────┘
       │                               │
       │ passes reference              │ provides
       ▼                               ▼
┌─────────────┐  active_task()  ┌─────────────────┐
│ TimerPanel  │←────────────────│  Current[0]     │
└─────────────┘                 └─────────────────┘
       │
       │ 'c' key → complete
       ▼
┌─────────────┐
│    App      │ → TaskManager.complete_task()
└─────────────┘
```

---

## Keybindings Summary

### Tasks Panel (focused)
| Key | Action |
|-----|--------|
| `j` | Move selection down |
| `k` | Move selection up |
| `J` | Reorder task down |
| `K` | Reorder task up |
| `Enter` | Move task between Backlog ↔ Current |
| `x` | Complete task (Current → Completed) |
| `s` | Sync completed tasks to file |

### Timer Panel (focused)
| Key | Action |
|-----|--------|
| `c` | Complete current active task |

---

## Error Handling
- File not found: Show error, run without tasks
- Parse errors: Skip invalid lines, load valid tasks
- Sync errors: Display error message, keep tasks in memory

---

## Verification
1. Run `cargo check` after each step to ensure no errors/warnings
2. Test file loading: `cargo run -- test.md`
3. Test navigation: Focus tasks panel, use j/k to navigate
4. Test task movement: Enter to move between sections, x to complete
5. Test sync: Complete tasks, press s, verify file updated
6. Test timer integration: Add tasks to Current, verify active task shows in timer
7. Test completion from timer: Press c, verify task moves to Completed
