use crossterm::event::{KeyCode, KeyEvent};

use crate::timer::Timer;

pub struct App {
    pub timer: Timer,
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            timer: Timer::default(),
            should_quit: false,
        }
    }
}

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char(' ') => self.timer.toggle(),
            KeyCode::Char('r') => self.timer.reset(),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.timer.tick();
    }
}
