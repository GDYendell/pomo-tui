use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::audio::AudioPlayer;
use crate::digits::TIMER_MIN_WIDTH;
use crate::panel::{PanelId, Shortcut};
use crate::panels::{TasksPanel, TimerPanel};
use crate::timer::{SessionType, Timer, TimerState};

pub struct App {
    pub should_quit: bool,
    pub timer: Timer,
    pub timer_panel: TimerPanel,
    pub tasks_panel: TasksPanel,
    pub focused_panel: PanelId,
    pub tasks_visible: bool,
    pub shortcuts_visible: bool,
    pub two_columns: bool,
    audio: Option<AudioPlayer>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            timer: Timer::default(),
            timer_panel: TimerPanel::default(),
            tasks_panel: TasksPanel::default(),
            focused_panel: PanelId::Timer,
            tasks_visible: true,
            shortcuts_visible: true,
            two_columns: false,
            audio: AudioPlayer::new(),
        }
    }
}

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Global shortcuts first
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.should_quit = true;
                return;
            }
            // T to toggle tasks panel visibility
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.toggle_tasks_visibility();
                return;
            }
            // ? to toggle shortcuts bar visibility
            KeyCode::Char('?') => {
                self.shortcuts_visible = !self.shortcuts_visible;
                return;
            }
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.focus_previous();
                } else {
                    self.focus_next();
                }
                return;
            }
            KeyCode::BackTab => {
                self.focus_previous();
                return;
            }
            _ => {}
        }

        // Panel-specific keys
        match self.focused_panel {
            PanelId::Timer => self.handle_timer_key(key),
            PanelId::Tasks => {
                let _ = self.tasks_panel.handle_key(key);
            }
        }
    }

    fn handle_timer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(' ') => self.timer.toggle(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.timer.reset(),
            KeyCode::Char('w') | KeyCode::Char('W') if self.timer.state == TimerState::Idle => {
                self.timer.set_session_type(SessionType::Work);
            }
            KeyCode::Char('s') | KeyCode::Char('S') if self.timer.state == TimerState::Idle => {
                self.timer.set_session_type(SessionType::ShortBreak);
            }
            KeyCode::Char('l') | KeyCode::Char('L') if self.timer.state == TimerState::Idle => {
                self.timer.set_session_type(SessionType::LongBreak);
            }
            _ => {}
        }
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
            PanelId::Timer => self.timer_panel.shortcuts(&self.timer),
            PanelId::Tasks => self.tasks_panel.shortcuts(),
        }
    }

    fn focus_next(&mut self) {
        let all_panels = PanelId::all();
        let current_idx = all_panels
            .iter()
            .position(|&p| p == self.focused_panel)
            .unwrap_or(0);

        // Find next visible panel
        for i in 1..=all_panels.len() {
            let next_idx = (current_idx + i) % all_panels.len();
            let panel_id = all_panels[next_idx];
            if self.is_panel_visible(panel_id) {
                self.focused_panel = panel_id;
                return;
            }
        }
    }

    fn focus_previous(&mut self) {
        let all_panels = PanelId::all();
        let current_idx = all_panels
            .iter()
            .position(|&p| p == self.focused_panel)
            .unwrap_or(0);

        // Find previous visible panel
        for i in 1..=all_panels.len() {
            let prev_idx = (current_idx + all_panels.len() - i) % all_panels.len();
            let panel_id = all_panels[prev_idx];
            if self.is_panel_visible(panel_id) {
                self.focused_panel = panel_id;
                return;
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

    pub fn is_panel_visible(&self, id: PanelId) -> bool {
        match id {
            PanelId::Timer => self.two_columns || self.focused_panel == PanelId::Timer,
            PanelId::Tasks => self.two_columns || self.focused_panel == PanelId::Tasks,
        }
    }

    /// Called before render to update layout state based on terminal width
    pub fn update_layout(&mut self, width: u16) {
        self.two_columns = self.tasks_visible && (width / 2) >= TIMER_MIN_WIDTH;
    }
}
