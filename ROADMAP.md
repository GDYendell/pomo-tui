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
- [ ] Task line tracking - Sync by exact text match, warn if no match found
- [x] Task movement - Move tasks between sections (backlog/current/completed)
- [x] Completed tasks display - Show all completed tasks in section order
- [ ] Sync dialogue - `s` opens dialogue to write changes or read new tasks

---

## Phase 4: UI Polish
**Goal**: Layout refinements and visual enhancements

- [ ] Minimum terminal size - Enforce minimum size to fit timer with border
- [ ] Break mode ASCII art - Display art instead of current task during breaks
- [ ] Timer panel sizing - Main section sized to content, extra space to task/art area
- [ ] Consistent task panel sizing - Stable sizing when resizing terminal

---

## Phase 5: Settings & Stats
**Goal**: Configuration and insights

- [ ] Settings panel - Full screen overlay
- [ ] Duration customization - Adjust work/short/long break lengths
- [ ] Session stats - View completed sessions, total focus time, etc.
- [ ] Stats persistence - Save alongside tasks
