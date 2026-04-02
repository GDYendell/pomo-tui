use std::path::PathBuf;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui_input_manager::{keymap, KeyMap};

use crate::melodies::{TWO_TONE, VICTORY_FANFARE};
use crate::notifications::{send_notification, AudioPlayer};
use crate::panels::{PanelId, TasksPanel, TimerPanel, TIMER_MIN_WIDTH};
use crate::timer::{SessionType, Timer};

/// Main application state coordinating timer, tasks, panels, and overlays
pub struct App {
    audio: Option<AudioPlayer>,
    pub timer: Timer,
    pub focused_panel: PanelId,
    pub timer_panel: TimerPanel,
    pub tasks_panel: TasksPanel,
    /// Error message displayed in overlay, if Some
    pub error_message: Option<String>,
    /// Whether the shortcuts
    pub shortcuts_visible: bool,
    /// Whether the tasks panel is visible
    pub tasks_visible: bool,
    /// Whether in two column or single column layout
    pub two_columns: bool,
    /// Flag to trigger application exit
    pub should_quit: bool,
}

impl App {
    pub fn new(task_file: Option<PathBuf>) -> Self {
        let (tasks_panel, error_message) = TasksPanel::from_file(task_file);

        Self {
            should_quit: false,
            timer: Timer::default(),
            timer_panel: TimerPanel::default(),
            tasks_panel,
            focused_panel: PanelId::Timer,
            tasks_visible: true,
            shortcuts_visible: false,
            two_columns: false,
            error_message,
            audio: AudioPlayer::new(),
        }
    }

    /// Ticks the timer countdown and animation counter, notifying on session completion
    pub fn tick(&mut self) {
        let session_completed = self.timer.tick();

        if session_completed {
            if let Some(ref audio) = self.audio {
                // After completion the timer has already transitioned to the next session type.
                // If the new session is a break, a work session just finished → play the fanfare.
                if matches!(
                    self.timer.session_type(),
                    SessionType::ShortBreak | SessionType::LongBreak
                ) {
                    audio.play_melody(VICTORY_FANFARE);
                } else {
                    audio.play_melody(TWO_TONE);
                }
            }
            if let Some(err) = send_notification("Pomo-TUI", "Session completed!") {
                self.error_message = Some(err);
            }
        }

        self.timer_panel.next_animation_frame();
    }

    fn toggle_tasks_visibility(&mut self) {
        self.tasks_visible = !self.tasks_visible;

        if self.tasks_visible && !self.two_columns {
            self.focused_panel = PanelId::Tasks;
        }

        if !self.tasks_visible && self.focused_panel == PanelId::Tasks {
            self.focused_panel = PanelId::Timer;
        }
    }

    /// Compute the column layout based on terminal width
    pub fn compute_column_layout(&mut self, width: u16) {
        self.two_columns = self.tasks_visible && (width / 2) >= TIMER_MIN_WIDTH;
    }

    /// Handle a terminal event
    pub fn handle_event(&mut self, event: &Event) {
        if self.error_message.is_some() {
            if matches!(
                event,
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    ..
                })
            ) {
                self.error_message = None;
            }
            return;
        }

        if self.shortcuts_visible {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('?') | KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) = event
            {
                self.shortcuts_visible = false;
            }
            return;
        }

        if self.focused_panel == PanelId::Tasks {
            self.tasks_panel.handle_event(event);
            if let Some(error) = self.tasks_panel.take_error() {
                self.error_message = Some(error);
            }
        }

        self.handle(event);
    }
}

#[keymap(backend = "crossterm")]
impl App {
    /// Quit
    #[keybind(pressed(key=KeyCode::Char('q')))]
    #[keybind(pressed(key=KeyCode::Char('Q')))]
    #[keybind(pressed(key=KeyCode::Esc))]
    fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Toggle tasks panel visibility
    #[keybind(pressed(key=KeyCode::Char('T')))]
    fn toggle_tasks(&mut self) {
        self.toggle_tasks_visibility();
    }

    /// Switch panel focus
    #[keybind(pressed(key=KeyCode::Char('t')))]
    fn switch_focus(&mut self) {
        if self.tasks_visible {
            self.focused_panel = match self.focused_panel {
                PanelId::Timer => PanelId::Tasks,
                PanelId::Tasks => PanelId::Timer,
            };
        }
    }

    /// Toggle help overlay
    #[keybind(pressed(key=KeyCode::Char('?')))]
    fn toggle_help(&mut self) {
        self.shortcuts_visible = !self.shortcuts_visible;
    }

    /// Start or pause timer
    #[keybind(pressed(key=KeyCode::Char(' ')))]
    fn toggle_timer(&mut self) {
        if self.focused_panel == PanelId::Timer {
            self.timer.toggle();
        }
    }

    /// Reset timer
    #[keybind(pressed(key=KeyCode::Char('r')))]
    #[keybind(pressed(key=KeyCode::Char('R')))]
    fn reset_timer(&mut self) {
        if self.focused_panel == PanelId::Timer {
            self.timer.reset();
        }
    }

    /// Set work session mode
    #[keybind(pressed(key=KeyCode::Char('w')))]
    #[keybind(pressed(key=KeyCode::Char('W')))]
    fn set_work_mode(&mut self) {
        if self.focused_panel == PanelId::Timer && self.timer.is_idle() {
            self.timer.set_session_type(SessionType::Work);
        }
    }

    /// Set long break session mode
    #[keybind(pressed(key=KeyCode::Char('b')))]
    #[keybind(pressed(key=KeyCode::Char('B')))]
    fn set_break_mode(&mut self) {
        if self.focused_panel == PanelId::Timer && self.timer.is_idle() {
            self.timer.set_session_type(SessionType::LongBreak);
        }
    }

    /// Complete current task
    #[keybind(pressed(key=KeyCode::Char('x')))]
    #[keybind(pressed(key=KeyCode::Char('X')))]
    fn handle_complete(&mut self) {
        if self.focused_panel == PanelId::Timer {
            self.tasks_panel.complete_current_task();
        }
    }

    /// Cycle session type
    #[keybind(pressed(key=KeyCode::Tab))]
    #[keybind(pressed(key=KeyCode::BackTab))]
    fn cycle_session(&mut self) {
        if self.focused_panel == PanelId::Timer && self.timer.is_idle() {
            self.timer.cycle_session_type();
        }
    }

    /// Add one minute to timer
    #[keybind(pressed(key=KeyCode::Char('+')))]
    #[keybind(pressed(key=KeyCode::Char('=')))]
    fn add_minute(&mut self) {
        if self.focused_panel == PanelId::Timer && self.timer.is_idle() {
            self.timer.add_minute();
        }
    }

    /// Subtract one minute from timer
    #[keybind(pressed(key=KeyCode::Char('-')))]
    #[keybind(pressed(key=KeyCode::Char('_')))]
    fn subtract_minute(&mut self) {
        if self.focused_panel == PanelId::Timer && self.timer.is_idle() {
            self.timer.subtract_minute();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_tasks_visibility() {
        let mut app = App::new(None);
        app.tasks_visible = false;
        app.two_columns = false;
        app.focused_panel = PanelId::Timer;

        // Single-column mode: showing tasks switches focus to Tasks
        app.toggle_tasks_visibility();
        assert!(app.tasks_visible);
        assert_eq!(app.focused_panel, PanelId::Tasks);

        // Hiding tasks switches focus to Timer
        app.toggle_tasks_visibility();
        assert!(!app.tasks_visible);
        assert_eq!(app.focused_panel, PanelId::Timer);

        // Two-column mode: showing tasks keeps focus on Timer
        app.two_columns = true;
        app.toggle_tasks_visibility();
        assert!(app.tasks_visible);
        assert_eq!(app.focused_panel, PanelId::Timer);

        // Switch focus to Tasks, then hide while focused on Tasks
        app.focused_panel = PanelId::Tasks;
        app.toggle_tasks_visibility();
        assert!(!app.tasks_visible);
        assert_eq!(app.focused_panel, PanelId::Timer);
    }

    #[test]
    fn test_update_layout_two_column_threshold() {
        let mut app = App {
            tasks_visible: true,
            two_columns: false,
            ..App::new(None)
        };

        // Width below threshold: single column
        app.compute_column_layout(TIMER_MIN_WIDTH * 2 - 1);
        assert!(!app.two_columns);

        // Width at threshold: two columns
        app.compute_column_layout(TIMER_MIN_WIDTH * 2);
        assert!(app.two_columns);

        // Width above threshold: two columns
        app.compute_column_layout(TIMER_MIN_WIDTH * 2 + 10);
        assert!(app.two_columns);

        // Tasks hidden: always single column
        app.tasks_visible = false;
        app.compute_column_layout(TIMER_MIN_WIDTH * 2 + 100);
        assert!(!app.two_columns);
    }
}
