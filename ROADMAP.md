# Pomodoro TUI Roadmap

## Phase 1: UI Foundation (Panel System)
**Goal**: Split-panel layout with borders and navigation

- [ ] Panel abstraction - `Panel` trait for render/input/shortcuts
- [ ] Split layout with borders - Panels visible simultaneously, bordered
- [ ] Panel focus - Tab/Shift+Tab or number keys to switch focus
- [ ] Dynamic bottom bar - Show shortcuts for focused panel
- [ ] Panel visibility toggle - Hide/show individual panels

**Initial panels**:
- Timer panel (center/prominent)
- Task backlog panel
- Current session tasks panel
- Completed tasks panel (top 5, newest first)

---

## Phase 2: Timer Enhancements + Audio
**Goal**: Visual polish and notifications

- [ ] Large block digits - Block characters like tty-clock
- [ ] Running indicator - Pulsing dots animation
- [ ] Manual mode switching - Switch work/short/long break when idle
- [ ] Sound notifications - Audio alert when timer completes

---

## Phase 3: Task Management
**Goal**: Full task workflow with markdown persistence

- [ ] Task data model - Status, timestamps, description
- [ ] Task backlog panel - Create, delete, queue for next session
- [ ] Current tasks panel - Mark done/undone, return to backlog
- [ ] Completed tasks panel - Top 5 displayed, newest first
- [ ] Markdown persistence - Load/save tasks to `.md` file

---

## Phase 4: Settings & Stats
**Goal**: Configuration and insights

- [ ] Settings panel - Full screen overlay
- [ ] Duration customization - Adjust work/short/long break lengths
- [ ] Session stats - View completed sessions, total focus time, etc.
- [ ] Stats persistence - Save alongside tasks
