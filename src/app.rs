use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::panel::{Panel, PanelId};
use crate::panels::{TasksPanel, TimerPanel};

pub struct PanelManager {
    pub timer: TimerPanel,
    pub tasks: TasksPanel,
}

impl Default for PanelManager {
    fn default() -> Self {
        Self {
            timer: TimerPanel::default(),
            tasks: TasksPanel::default(),
        }
    }
}

impl PanelManager {
    pub fn get(&self, id: PanelId) -> &dyn Panel {
        match id {
            PanelId::Timer => &self.timer,
            PanelId::Tasks => &self.tasks,
        }
    }

    pub fn get_mut(&mut self, id: PanelId) -> &mut dyn Panel {
        match id {
            PanelId::Timer => &mut self.timer,
            PanelId::Tasks => &mut self.tasks,
        }
    }
}

pub struct App {
    pub should_quit: bool,
    pub panels: PanelManager,
    pub focused_panel: PanelId,
    pub tasks_visible: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            panels: PanelManager::default(),
            focused_panel: PanelId::Timer,
            tasks_visible: true,
        }
    }
}

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Global shortcuts first
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                return;
            }
            // T to toggle tasks panel visibility (always active)
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.toggle_tasks_visibility();
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
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(n) = c.to_digit(10) {
                    if let Some(panel_id) = PanelId::from_number(n as u8) {
                        if self.is_panel_visible(panel_id) {
                            self.focused_panel = panel_id;
                        }
                    }
                }
                return;
            }
            _ => {}
        }

        // Pass to focused panel
        let panel = self.panels.get_mut(self.focused_panel);
        let _ = panel.handle_key(key);
    }

    pub fn tick(&mut self) {
        // Always tick the timer regardless of focus
        self.panels.timer.tick();
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

        // If tasks panel was focused and is now hidden, move focus to timer
        if !self.tasks_visible && self.focused_panel == PanelId::Tasks {
            self.focused_panel = PanelId::Timer;
        }
    }

    pub fn is_panel_visible(&self, id: PanelId) -> bool {
        match id {
            PanelId::Timer => true, // Timer is always visible
            PanelId::Tasks => self.tasks_visible,
        }
    }
}
