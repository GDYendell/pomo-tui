# Pomodoro TUI Roadmap

## Phase 1: UI Foundation (Panel System)
**Goal**: Split-panel layout with borders and navigation

- [x] Panel abstraction - `Panel` trait for render/input/shortcuts
- [x] Split layout with borders - Panels visible simultaneously, bordered
- [x] Panel focus - Tab/Shift+Tab or number keys to switch focus
- [x] Dynamic bottom bar - Show shortcuts for focused panel
- [x] Panel visibility toggle - Hide/show individual panels

**Initial panels**:
- Timer panel (center/prominent)
- Task backlog panel
- Current session tasks panel
- Completed tasks panel (top 5, newest first)

---

## Phase 2: Timer Enhancements + Audio
**Goal**: Visual polish and notifications

- [x] Large block digits - Block characters like tty-clock
- [x] Running indicator - Pulsing dots animation
- [x] Manual mode switching - Switch work/short/long break when idle
- [x] Sound notifications - Audio alert when timer completes

---

## Phase 3: Task Management
**Goal**: Full task workflow with markdown persistence

- [x] Task data model - Status, ~~timestamps~~, description
- [x] Task backlog panel - ~~Create, delete,~~ queue for next session
- [x] Current tasks panel - Mark done/undone, return to backlog
- [x] Completed tasks panel - ~~Top 5 displayed, newest first~~
- [x] Markdown persistence - Load/save tasks to `.md` file
- [x] Task line tracking - Sync by exact text match, warn if no match found
- [x] Task movement - Move tasks between sections (backlog/current/completed)
- [x] Completed tasks display - Show all completed tasks in section order
- [x] Sync dialogue - `s` opens interactive dialogue to resolve differences between app state and file
- [x] Help overlay - `?` opens centered help overlay with panel-specific and global shortcuts
- [x] Page scrolling - `,`/`.` to page down/up in task sections

---

## Phase 4: UI Polish
**Goal**: Layout refinements and visual enhancements

- [x] Minimum window size - Enforced via Hyprland window rule (400x250px)
- [x] Timer panel adaptive layout - Digits always shown, wave/label centered, bottom section appears with space
- [x] Bottom section text wrapping - Wraps long task text with padding
- [x] Task text truncation - Truncated on whole words with `...` in task sections
- [x] Consistent task panel sizing - Manual height division instead of `Constraint::Ratio`

---

## Phase 5: Settings & Stats
**Goal**: Configuration and insights

- [ ] Settings panel - Full screen overlay
- [ ] Duration customization - Adjust work/short/long break lengths
- [ ] Session stats - View completed sessions, total focus time, etc.
- [ ] Stats persistence - Save alongside tasks
