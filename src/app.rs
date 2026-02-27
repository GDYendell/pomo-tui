use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};

use crate::notifications::{send_notification, AudioPlayer};
use crate::overlays::{SyncAction, SyncOverlay, TaskInputAction, TaskInputOverlay};
use crate::panels::{KeyHandleResult, PanelId, TasksPanel, TimerPanel, TIMER_MIN_WIDTH};
use crate::task::TaskSection;
use crate::task_manager::TaskManager;
use crate::timer::Timer;
use crate::util::Shortcut;

/// Main application state coordinating timer, tasks, panels, and overlays
pub struct App {
    audio: Option<AudioPlayer>,
    pub timer: Timer,
    pub focused_panel: PanelId,
    pub timer_panel: TimerPanel,
    pub tasks_panel: TasksPanel,
    pub task_manager: TaskManager,
    pub task_input_overlay: Option<TaskInputOverlay>,
    pub sync_overlay: Option<SyncOverlay>,
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
        let (task_manager, error_message) = task_file.map_or_else(
            || (TaskManager::new(), None),
            |path| {
                TaskManager::load(path).map_or_else(
                    |e| {
                        (
                            TaskManager::new(),
                            Some(format!("Failed to load tasks: {e}")),
                        )
                    },
                    |tm| (tm, None),
                )
            },
        );

        Self {
            should_quit: false,
            timer: Timer::default(),
            timer_panel: TimerPanel::default(),
            tasks_panel: TasksPanel::default(),
            task_manager,
            focused_panel: PanelId::Timer,
            tasks_visible: true,
            shortcuts_visible: false,
            two_columns: false,
            error_message,
            sync_overlay: None,
            task_input_overlay: None,
            audio: AudioPlayer::new(),
        }
    }

    /// Ticks the timer countdown and animation counter, notifying on session completion
    pub fn tick(&mut self) {
        let session_completed = self.timer.tick();

        if session_completed {
            if let Some(ref audio) = self.audio {
                audio.play_notification();
            }
            if let Some(err) = send_notification("Pomo-TUI", "Session completed!") {
                self.error_message = Some(err);
            }
        }

        self.timer_panel.next_animation_frame();
    }

    pub fn focused_shortcuts(&self) -> Vec<Shortcut> {
        match self.focused_panel {
            PanelId::Timer => self
                .timer_panel
                .shortcuts(&self.timer, self.task_manager.active_task().is_some()),
            PanelId::Tasks => self.tasks_panel.shortcuts(),
        }
    }

    fn sync_tasks(&mut self) {
        if !self.task_manager.has_file_path() {
            if let Err(e) = self.task_manager.create_default_file() {
                self.error_message = Some(format!("Failed to create default task file: {e}"));
                return;
            }
        }
        match self.task_manager.compute_sync_items() {
            Ok(items) => {
                self.sync_overlay = Some(SyncOverlay::new(items));
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Sync failed: {e}"));
            }
        }
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

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Intercept if error overlay is active
        if self.error_message.is_some() {
            self.error_message = None;
            return;
        }

        // Intercept if task input overlay is active
        if let Some(ref mut overlay) = self.task_input_overlay {
            match overlay.handle_key(key) {
                TaskInputAction::Consumed => {}
                TaskInputAction::Dismiss => {
                    self.task_input_overlay = None;
                }
                TaskInputAction::Submit { text, section } => {
                    self.task_manager.add_task(text, section);
                    self.task_input_overlay = None;
                }
            }
            return;
        }

        // Intercept if sync overlay is active
        if let Some(ref mut overlay) = self.sync_overlay {
            match overlay.handle_key(key) {
                SyncAction::Consumed => {}
                SyncAction::Dismiss => {
                    self.sync_overlay = None;
                }
                SyncAction::Apply(items) => {
                    if let Err(e) = self.task_manager.apply_sync(&items) {
                        self.error_message = Some(format!("Sync failed: {e}"));
                    }
                    self.tasks_panel.clamp_focus(&self.task_manager);
                    self.sync_overlay = None;
                }
            }
            return;
        }

        // Intercept if help overlay is active
        if self.shortcuts_visible {
            match key.code {
                KeyCode::Char('?') | KeyCode::Esc => self.shortcuts_visible = false,
                _ => {}
            }
            return;
        }

        // Global shortcuts first
        match key.code {
            KeyCode::Char('q' | 'Q') | KeyCode::Esc => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('T') => {
                self.toggle_tasks_visibility();
                return;
            }
            KeyCode::Char('t') if self.tasks_visible => {
                self.focused_panel = match self.focused_panel {
                    PanelId::Timer => PanelId::Tasks,
                    PanelId::Tasks => PanelId::Timer,
                };
                return;
            }
            KeyCode::Char('?') => {
                self.shortcuts_visible = !self.shortcuts_visible;
                return;
            }
            _ => {}
        }

        if self.focused_panel == PanelId::Tasks {
            if let KeyCode::Char('s' | 'S') = key.code {
                self.sync_tasks();
                return;
            }
        }

        match self.focused_panel {
            PanelId::Timer => {
                self.timer_panel
                    .handle_key(key, &mut self.timer, &mut self.task_manager);
            }
            PanelId::Tasks => {
                if self.tasks_panel.handle_key(key, &mut self.task_manager)
                    == KeyHandleResult::AddTask
                {
                    let section = self.tasks_panel.focused_section();
                    if section != TaskSection::Completed {
                        self.task_input_overlay = Some(TaskInputOverlay::new(section));
                    }
                }
            }
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
