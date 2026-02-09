# TODO

## Small Changes
- [x] Bug: `t` to toggle focus should not switch focus to tasks panel if it is not visible
- [x] Current task in timer panel is bold text
- [x] Bug: `Tab` moves focus from timer to tasks, only `t` should do this
- [x] Timer panel should not show current task in a break mode
- [x] Remove s keybind to swap to short break, only tab should be used for this
- [x] Reading from file should not clear current tasks (unless they are removed entirely by the read)
- [x] Turn the help panel into an overlay like the sync dialogue
- [x] (empty) should be centred in the tasks sections
- [x] Keybind to complete in timer panel should be x to match tasks panel

## Features
- [x] `s` opens a dialogue. The dialogue allows writing changes to file, or reading new tasks.
- [x] Writing changes to file is done by exact match, not line number. If no match is found, then a warning is shown to the user.
- [x] A minimum window size is enforced via Hyprland window rule (400x250px matching title "pomo-tui")
- [x] The size of the task sections should be kept as consistent as possible. Fixed by using manual height division instead of `Constraint::Ratio`
- [x] Timer panel adaptive layout: digits always shown, wave/label centered in gap, bottom section appears with enough space
- [x] Bottom section wraps long task text with 2 col padding each side, 1 row pad above/below
- [x] Task text in tasks panel truncated on whole words with `...` when it doesn't leave 10 chars of space
- [x] Page scroll in tasks panel: `,` page down, `.` page up
