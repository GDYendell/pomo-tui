use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
}

pub struct Timer {
    state: TimerState,
    session_type: SessionType,
    remaining: Duration,
    sessions_completed: u32,
    last_tick: Option<Instant>,

    work_duration: Duration,
    short_break_duration: Duration,
    long_break_duration: Duration,
}

impl Default for Timer {
    fn default() -> Self {
        let work_duration = Duration::from_secs(25 * 60);
        Self {
            state: TimerState::Idle,
            session_type: SessionType::Work,
            remaining: work_duration,
            sessions_completed: 0,
            last_tick: None,
            work_duration,
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
        }
    }
}

impl Timer {
    pub const fn session_type(&self) -> SessionType {
        self.session_type
    }

    pub fn is_idle(&self) -> bool {
        self.state == TimerState::Idle
    }

    pub fn is_running(&self) -> bool {
        self.state == TimerState::Running
    }

    pub fn start(&mut self) {
        if self.state != TimerState::Running {
            self.state = TimerState::Running;
            self.last_tick = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.state = TimerState::Paused;
            self.last_tick = None;
        }
    }

    pub fn toggle(&mut self) {
        match self.state {
            TimerState::Idle | TimerState::Paused => self.start(),
            TimerState::Running => self.pause(),
        }
    }

    pub fn reset(&mut self) {
        self.state = TimerState::Idle;
        self.last_tick = None;
        self.remaining = self.duration_for_session(self.session_type);
    }

    pub fn set_session_type(&mut self, session_type: SessionType) {
        if self.state == TimerState::Idle {
            self.session_type = session_type;
            self.remaining = self.duration_for_session(session_type);
        }
    }

    pub fn next_session_type(&mut self) {
        if self.state == TimerState::Idle {
            let next = match self.session_type {
                SessionType::Work => SessionType::ShortBreak,
                SessionType::ShortBreak => SessionType::LongBreak,
                SessionType::LongBreak => SessionType::Work,
            };
            self.session_type = next;
            self.remaining = self.duration_for_session(next);
        }
    }

    pub fn add_minute(&mut self) {
        if self.state == TimerState::Idle {
            self.remaining += Duration::from_secs(60);
        }
    }

    pub fn subtract_minute(&mut self) {
        if self.state == TimerState::Idle && self.remaining > Duration::from_secs(60) {
            self.remaining -= Duration::from_secs(60);
        }
    }

    /// Returns true if a session was completed during this tick
    pub fn tick(&mut self) -> bool {
        if self.state != TimerState::Running {
            return false;
        }

        if let Some(last) = self.last_tick {
            let elapsed = last.elapsed();
            self.last_tick = Some(Instant::now());

            if elapsed >= self.remaining {
                self.remaining = Duration::ZERO;
                self.complete_session();
                return true;
            }
            self.remaining -= elapsed;
        }
        false
    }

    fn complete_session(&mut self) {
        match self.session_type {
            SessionType::Work => {
                self.sessions_completed += 1;
                if self.sessions_completed.is_multiple_of(4) {
                    self.session_type = SessionType::LongBreak;
                } else {
                    self.session_type = SessionType::ShortBreak;
                }
            }
            SessionType::ShortBreak | SessionType::LongBreak => {
                self.session_type = SessionType::Work;
            }
        }
        self.remaining = self.duration_for_session(self.session_type);
        self.state = TimerState::Idle;
        self.last_tick = None;
    }

    const fn duration_for_session(&self, session: SessionType) -> Duration {
        match session {
            SessionType::Work => self.work_duration,
            SessionType::ShortBreak => self.short_break_duration,
            SessionType::LongBreak => self.long_break_duration,
        }
    }

    pub const fn minutes(&self) -> u64 {
        self.remaining.as_secs() / 60
    }

    pub const fn seconds(&self) -> u64 {
        self.remaining.as_secs() % 60
    }
}
