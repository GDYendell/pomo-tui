# TODO

## Small Changes
- [x] Bug: `t` to toggle focus should not switch focus to tasks panel if it is not visible
- [x] Current task in timer panel is bold text
- [x] Bug: `Tab` moves focus from timer to tasks, only `t` should do this
- [x] Timer panel should not show current task in a break mode
- [x] Remove s keybind to swap to short break, only tab should be used for this
- [x] Reading from file should not clear current tasks (unless they are removed entirely by the read)
- [x] Turn the help panel into an overlay like the sync dialogue

## Features
- [x] `s` opens a dialogue. The dialogue allows writing changes to file, or reading new tasks.
- [x] Writing changes to file is done by exact match, not line number. If no match is found, then a warning is shown to the user.
- [ ] A minimum terminal size is enforced. It cannot be smaller than would allow to show just the timer, with one cell border.
- [ ] Some ascii art should be shown in place of the current task when in break mode
- [ ] In the timer panel, the main section should have a maximum size of the content plus one cell of border. Any additional space goes to the Current Task / ASCII art
- [ ] The size of the task sections should be kept as consistent as possible. Currently when reducing the height, they shrink inconsistently and sometimes even get bigger again. Explore other ratatui layout types to see if there are better options
