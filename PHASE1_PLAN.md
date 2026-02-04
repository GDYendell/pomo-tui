# Phase 1 Implementation Plan: Multi-Panel System

## Overview
Refactor the Pomodoro TUI from a single-panel layout to a multi-panel system with focus management, dynamic shortcuts, and visibility toggling.

## Layout Design
```
+--------------------------------------------------+
|                   POMODORO TIMER                 |  <- Title (3 lines)
+--------------------------------------------------+
|                    |         Task Backlog        |
|      TIMER         |        (placeholder)        |
|        +           +-----------------------------+
|   Current Tasks    |       Completed Tasks       |
|                    |          (top 5)            |
+--------------------------------------------------+
|  [Tab] Next  [1-3] Focus  [Ctrl+H] Hide  [q] Quit|  <- Shortcuts (3 lines)
+--------------------------------------------------+
```

- Timer + Current Tasks: 50% width left (combined panel)
- Task Backlog: 50% width right top, 50% height
- Completed Tasks: 50% width right bottom, 50% height

---

## New Files to Create

### 1. `src/panel.rs` - Core trait and types

```rust
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

#[derive(Clone)]
pub struct Shortcut {
    pub key: &'static str,
    pub description: &'static str,
}

pub enum KeyHandleResult {
    Consumed,
    Ignored,
}

pub trait Panel {
    fn id(&self) -> PanelId;
    fn title(&self) -> &str;
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool);
    fn handle_key(&mut self, key: KeyEvent) -> KeyHandleResult;
    fn shortcuts(&self) -> Vec<Shortcut>;
    fn tick(&mut self) {}
    fn focusable(&self) -> bool { true }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    Timer,        // Timer + Current Tasks combined
    TaskBacklog,
    CompletedTasks,
}

impl PanelId {
    pub fn all() -> &'static [PanelId] {
        &[
            PanelId::Timer,
            PanelId::TaskBacklog,
            PanelId::CompletedTasks,
        ]
    }

    pub fn from_number(n: u8) -> Option<PanelId> {
        match n {
            1 => Some(PanelId::Timer),
            2 => Some(PanelId::TaskBacklog),
            3 => Some(PanelId::CompletedTasks),
            _ => None,
        }
    }
}
```

### 2. `src/panels/mod.rs` - Re-exports

```rust
mod timer;
mod task_backlog;
mod completed_tasks;

pub use timer::TimerPanel;
pub use task_backlog::TaskBacklogPanel;
pub use completed_tasks::CompletedTasksPanel;
```

### 3. `src/panels/timer.rs` - TimerPanel

Wraps existing Timer, implements Panel trait:
- `render()`: Timer display at top, current tasks list below, focus-aware border color
- `handle_key()`: Space (toggle), r (reset)
- `shortcuts()`: Returns `[Space] Start/Pause`, `[r] Reset`
- `tick()`: Delegates to inner Timer
- Current tasks section shows "(Phase 3)" placeholder for now

### 4. `src/panels/task_backlog.rs` - Placeholder

Shows "(Phase 3)" centered, no shortcuts, no key handling.

### 5. `src/panels/completed_tasks.rs` - Placeholder

Shows "(Phase 3)" centered, no shortcuts, no key handling.

---

## Files to Modify

### `src/app.rs`

Add:
```rust
pub struct PanelManager {
    pub timer: TimerPanel,           // Includes current tasks display
    pub task_backlog: TaskBacklogPanel,
    pub completed_tasks: CompletedTasksPanel,
}

pub struct App {
    pub should_quit: bool,
    pub panels: PanelManager,
    pub focused_panel: PanelId,
    pub hidden_panels: HashSet<PanelId>,
}
```

Key handling:
- Tab: `focus_next()`
- Shift+Tab: `focus_previous()`
- 1-4: Jump to specific panel
- Ctrl+H: Toggle visibility (except Timer)
- Other keys: Pass to focused panel

### `src/ui.rs`

Add:
```rust
pub struct AppLayout {
    pub title: Rect,
    pub timer: Rect,              // Left side (50%), includes current tasks
    pub task_backlog: Rect,       // Right top (50% width, 50% height)
    pub completed_tasks: Rect,    // Right bottom (50% width, 50% height)
    pub shortcuts_bar: Rect,
}

pub fn create_layout(area: Rect) -> AppLayout { ... }
fn render_shortcuts_bar(frame: &mut Frame, area: Rect, app: &App) { ... }
```

Update `render()` to:
- Use `create_layout()` for panel areas
- Render each panel with focus state
- Skip hidden panels
- Render dynamic shortcuts bar

### `src/main.rs`

Add module declarations:
```rust
mod panel;
mod panels;
```

---

## Key Bindings

| Key | Scope | Action |
|-----|-------|--------|
| Tab | Global | Focus next panel |
| Shift+Tab | Global | Focus previous panel |
| 1-3 | Global | Focus specific panel |
| Ctrl+H | Global | Toggle hide current panel |
| q / Esc | Global | Quit |
| Space | Timer | Start/Pause |
| r | Timer | Reset |

---

## Implementation Order

1. **Create `src/panel.rs`** - trait, PanelId, Shortcut, KeyHandleResult
2. **Create `src/panels/` directory** with `mod.rs`
3. **Create `src/panels/timer.rs`** - move rendering from ui.rs, implement Panel (includes current tasks area)
4. **Create placeholder panels** - task_backlog.rs, completed_tasks.rs
5. **Update `src/app.rs`** - PanelManager, focus state, navigation, key routing
6. **Update `src/ui.rs`** - AppLayout, create_layout(), render_shortcuts_bar(), update render()
7. **Update `src/main.rs`** - add mod declarations

---

## Verification Checklist

After implementation, test:

- [ ] `cargo build` compiles without errors
- [ ] Timer works: Space starts/pauses, r resets
- [ ] Session transitions work correctly
- [ ] Tab cycles focus through 3 panels (border color changes)
- [ ] Shift+Tab cycles backwards
- [ ] Number keys 1-3 jump to panels
- [ ] Ctrl+H hides panels (except Timer)
- [ ] Hidden panels don't render
- [ ] Tab skips hidden panels
- [ ] Shortcuts bar shows global shortcuts
- [ ] Shortcuts bar updates with panel-specific shortcuts on focus change
- [ ] Layout renders correctly at various terminal sizes
- [ ] Timer panel shows current tasks placeholder area

---

## Notes

- Timer panel cannot be hidden (core functionality)
- Timer panel includes current tasks area (placeholder shows "(Phase 3)" for now)
- Task Backlog and Completed Tasks panels show "(Phase 3)" placeholder
- Focus indicated by cyan border, unfocused is dark gray
- Hidden panel space is preserved in layout (just not rendered)
- 3 panels total: Timer (with current tasks), Task Backlog, Completed Tasks
