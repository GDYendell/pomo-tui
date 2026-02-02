# Changelog

## [0.1.0] - 2026-02-02

### Added
- Initial pomodoro timer TUI using ratatui and crossterm
- Timer with standard pomodoro durations:
  - Work session: 25 minutes
  - Short break: 5 minutes
  - Long break: 15 minutes (after every 4 work sessions)
- Controls: Space (start/pause), r (reset), q (quit)
- Session counter tracking completed pomodoros
- Color-coded UI: red for work, green for short break, blue for long break

### Technical Decisions
- **ratatui** chosen as TUI framework for its active maintenance and documentation
- **crossterm** as terminal backend for cross-platform compatibility
- Modular architecture with separate timer, app, and ui modules
- 100ms tick rate for responsive UI updates
