use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};

use crate::audio::AudioPlayer;
use crate::digits::TIMER_MIN_WIDTH;
use crate::panel::{KeyHandleResult, PanelId, Shortcut};
use crate::panels::{TasksPanel, TimerPanel};
use crate::task::TaskSection;
use crate::task_manager::{SyncItem, SyncResolution, TaskManager};
use crate::timer::{SessionType, Timer, TimerState};

pub struct SyncDialogue {
    pub items: Vec<SyncItem>,
    pub focused: usize,
}

pub struct TaskInput {
    pub text: String,
    pub cursor: usize,
    pub section: TaskSection,
}

pub struct App {
    pub should_quit: bool,
    pub timer: Timer,
    pub timer_panel: TimerPanel,
    pub tasks_panel: TasksPanel,
    pub task_manager: TaskManager,
    pub focused_panel: PanelId,
    pub tasks_visible: bool,
    pub shortcuts_visible: bool,
    pub two_columns: bool,
    pub error_message: Option<String>,
    pub sync_dialogue: Option<SyncDialogue>,
    pub task_input: Option<TaskInput>,
    audio: Option<AudioPlayer>,
}

impl App {
    pub fn new(task_file: Option<PathBuf>) -> Self {
        let (task_manager, error_message) = match task_file {
            Some(path) => match TaskManager::load(path) {
                Ok(tm) => (tm, None),
                Err(e) => (
                    TaskManager::new(),
                    Some(format!("Failed to load tasks: {}", e)),
                ),
            },
            None => (TaskManager::new(), None),
        };

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
            sync_dialogue: None,
            task_input: None,
            audio: AudioPlayer::new(),
        }
    }
}

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Intercept if error overlay is active
        if self.error_message.is_some() {
            self.error_message = None;
            return;
        }

        // Intercept if task input is active
        if self.task_input.is_some() {
            self.handle_task_input_key(key);
            return;
        }

        // Intercept if sync dialogue is active
        if self.sync_dialogue.is_some() {
            self.handle_sync_dialogue_key(key);
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
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.should_quit = true;
                return;
            }
            // T to toggle tasks panel visibility
            KeyCode::Char('T') => {
                self.toggle_tasks_visibility();
                return;
            }
            // t to change panel focus (only if tasks panel is visible)
            KeyCode::Char('t') if self.tasks_visible => {
                self.focused_panel = match self.focused_panel {
                    PanelId::Timer => PanelId::Tasks,
                    PanelId::Tasks => PanelId::Timer,
                };
                return;
            }
            // ? to toggle shortcuts bar visibility
            KeyCode::Char('?') => {
                self.shortcuts_visible = !self.shortcuts_visible;
                return;
            }
            _ => {}
        }

        // Panel-specific keys (may consume Tab)
        let consumed = match self.focused_panel {
            PanelId::Timer => self.handle_timer_key(key),
            PanelId::Tasks => match self.tasks_panel.handle_key(key, &mut self.task_manager) {
                KeyHandleResult::Consumed => true,
                KeyHandleResult::AddTask => {
                    let section = self.task_manager.focus.section;
                    if section != TaskSection::Completed {
                        self.task_input = Some(TaskInput {
                            text: String::new(),
                            cursor: 0,
                            section,
                        });
                    }
                    true
                }
                KeyHandleResult::Ignored => false,
            },
        };

        if consumed {
            return;
        }

        // Handle sync key for tasks panel (after panel handling)
        if self.focused_panel == PanelId::Tasks {
            if let KeyCode::Char('s') | KeyCode::Char('S') = key.code {
                self.sync_tasks();
                return;
            }
        }

        // Panel focus switching is handled by 't' key only
    }

    fn sync_tasks(&mut self) {
        if !self.task_manager.has_file_path() {
            self.error_message =
                Some("No task file provided. Use --tasks <file> to enable sync.".to_string());
            return;
        }
        match self.task_manager.compute_sync_items() {
            Ok(items) => {
                self.sync_dialogue = Some(SyncDialogue { items, focused: 0 });
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Sync failed: {}", e));
            }
        }
    }

    fn handle_task_input_key(&mut self, key: KeyEvent) {
        let Some(input) = self.task_input.as_mut() else {
            return;
        };

        match key.code {
            KeyCode::Esc => {
                self.task_input = None;
            }
            KeyCode::Enter => {
                let text = input.text.trim().to_string();
                let section = input.section;
                if !text.is_empty() {
                    self.task_manager.add_task(text, section);
                }
                self.task_input = None;
            }
            KeyCode::Backspace => {
                if input.cursor > 0 {
                    input.text.remove(input.cursor - 1);
                    input.cursor -= 1;
                }
            }
            KeyCode::Left => {
                if input.cursor > 0 {
                    input.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if input.cursor < input.text.len() {
                    input.cursor += 1;
                }
            }
            KeyCode::Char(c) => {
                input.text.insert(input.cursor, c);
                input.cursor += 1;
            }
            _ => {}
        }
    }

    fn handle_sync_dialogue_key(&mut self, key: KeyEvent) {
        let Some(dialogue) = self.sync_dialogue.as_mut() else {
            return;
        };

        match key.code {
            KeyCode::Esc => {
                self.sync_dialogue = None;
            }
            KeyCode::Char('j') => {
                if !dialogue.items.is_empty() && dialogue.focused + 1 < dialogue.items.len() {
                    dialogue.focused += 1;
                }
            }
            KeyCode::Char('k') => {
                if dialogue.focused > 0 {
                    dialogue.focused -= 1;
                }
            }
            KeyCode::Char(' ') => {
                if let Some(item) = dialogue.items.get_mut(dialogue.focused) {
                    item.resolution = SyncResolution::Incomplete;
                }
            }
            KeyCode::Char('x') => {
                if let Some(item) = dialogue.items.get_mut(dialogue.focused) {
                    item.resolution = SyncResolution::Complete;
                }
            }
            KeyCode::Char('d') => {
                if let Some(item) = dialogue.items.get_mut(dialogue.focused) {
                    item.resolution = SyncResolution::Remove;
                }
            }
            KeyCode::Enter => {
                if let Some(dialogue) = self.sync_dialogue.take() {
                    if let Err(e) = self.task_manager.apply_sync(&dialogue.items) {
                        self.error_message = Some(format!("Sync failed: {}", e));
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_timer_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(' ') => self.timer.toggle(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.timer.reset(),
            KeyCode::Char('w') | KeyCode::Char('W') if self.timer.state == TimerState::Idle => {
                self.timer.set_session_type(SessionType::Work);
            }
            KeyCode::Char('b') | KeyCode::Char('B') if self.timer.state == TimerState::Idle => {
                self.timer.set_session_type(SessionType::LongBreak);
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.task_manager.complete_active();
            }
            KeyCode::Tab | KeyCode::BackTab if self.timer.state == TimerState::Idle => {
                self.timer.next_session_type();
                return true; // Consumed Tab
            }
            _ => {}
        }
        false
    }

    pub fn tick(&mut self) {
        // Always tick the timer regardless of focus
        let session_completed = self.timer.tick();

        // Play notification when session completes
        if session_completed {
            if let Some(ref audio) = self.audio {
                audio.play_notification();
            }
        }

        // Tick the panel for animation updates
        self.timer_panel.tick_animation();
    }

    pub fn focused_shortcuts(&self) -> Vec<Shortcut> {
        match self.focused_panel {
            PanelId::Timer => self
                .timer_panel
                .shortcuts(&self.timer, self.task_manager.active_task().is_some()),
            PanelId::Tasks => self.tasks_panel.shortcuts(),
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

    /// Called before render to update layout state based on terminal width
    pub fn update_layout(&mut self, width: u16) {
        self.two_columns = self.tasks_visible && (width / 2) >= TIMER_MIN_WIDTH;
    }
}
