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

/// Pomodoro timer with work/break sessions and configurable durations
pub struct Timer {
    state: TimerState,
    session_type: SessionType,
    remaining: Duration,
    /// Completed work sessions
    sessions_completed: u32,
    /// Time of last tick - None when paused/idle, Some when running
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

    /// Cycle to the next session type (work → short break → long break → work)
    pub fn cycle_session_type(&mut self) {
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

    /// Decrement timer and return true if a session was completed
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

    /// Complete current session and transition to next session type
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_starts_in_idle() {
        let timer = Timer::default();
        assert_eq!(timer.state, TimerState::Idle);
        assert_eq!(timer.session_type, SessionType::Work);
        assert_eq!(timer.sessions_completed, 0);
        assert_eq!(timer.minutes(), 25);
    }

    #[test]
    fn test_start_pause_resume() {
        let mut timer = Timer::default();

        timer.start();
        assert_eq!(timer.state, TimerState::Running);
        assert!(timer.is_running());
        assert!(!timer.is_idle());

        timer.pause();
        assert_eq!(timer.state, TimerState::Paused);
        assert!(!timer.is_running());

        timer.start();
        assert_eq!(timer.state, TimerState::Running);
        assert!(timer.is_running());
    }

    #[test]
    fn test_toggle_states() {
        let mut timer = Timer::default();

        timer.toggle();
        assert_eq!(timer.state, TimerState::Running);

        timer.toggle();
        assert_eq!(timer.state, TimerState::Paused);

        timer.toggle();
        assert_eq!(timer.state, TimerState::Running);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut timer = Timer::default();
        timer.start();
        timer.reset();
        assert_eq!(timer.state, TimerState::Idle);
        assert_eq!(timer.minutes(), 25);
        assert_eq!(timer.remaining, Duration::from_secs(25 * 60));
    }

    #[test]
    fn test_time_adjustment() {
        let mut timer = Timer::default();
        assert_eq!(timer.minutes(), 25);

        // Add minute
        timer.add_minute();
        assert_eq!(timer.minutes(), 26);

        // Subtract minute
        timer.subtract_minute();
        assert_eq!(timer.minutes(), 25);
    }

    #[test]
    fn test_subtract_minute_minimum() {
        let mut timer = Timer {
            remaining: Duration::from_secs(60),
            ..Default::default()
        };
        timer.subtract_minute();
        // Should not go below 1 minute
        assert_eq!(timer.remaining, Duration::from_secs(60));
    }

    #[test]
    fn test_cannot_adjust_time_when_running() {
        let mut timer = Timer::default();
        timer.start();
        let initial = timer.remaining;
        timer.add_minute();
        assert_eq!(timer.remaining, initial); // No change
    }

    #[test]
    fn test_set_session_type_only_when_idle() {
        let mut timer = Timer::default();
        timer.set_session_type(SessionType::ShortBreak);
        assert_eq!(timer.session_type, SessionType::ShortBreak);
        assert_eq!(timer.minutes(), 5);

        timer.start();
        timer.set_session_type(SessionType::Work);
        assert_eq!(timer.session_type, SessionType::ShortBreak); // Unchanged
    }

    #[test]
    fn test_next_session_type_cycles() {
        let mut timer = Timer::default();
        assert_eq!(timer.session_type, SessionType::Work);

        timer.cycle_session_type();
        assert_eq!(timer.session_type, SessionType::ShortBreak);

        timer.cycle_session_type();
        assert_eq!(timer.session_type, SessionType::LongBreak);

        timer.cycle_session_type();
        assert_eq!(timer.session_type, SessionType::Work);
    }

    #[test]
    fn test_tick_when_paused_does_nothing() {
        let mut timer = Timer::default();
        timer.start();
        timer.pause();
        let initial = timer.remaining;
        std::thread::sleep(Duration::from_millis(100));
        let completed = timer.tick();
        assert!(!completed);
        assert_eq!(timer.remaining, initial);
    }

    #[test]
    fn test_session_completion_flow() {
        let mut timer = Timer::default();

        // Complete work session → short break
        timer.start();
        timer.remaining = Duration::from_secs(1);
        std::thread::sleep(Duration::from_millis(1100));
        let completed = timer.tick();
        assert!(completed);
        assert_eq!(timer.state, TimerState::Idle);
        assert_eq!(timer.session_type, SessionType::ShortBreak);
        assert_eq!(timer.sessions_completed, 1);
        assert_eq!(timer.minutes(), 5);

        // Complete short break → work
        timer.start();
        timer.remaining = Duration::from_secs(1);
        std::thread::sleep(Duration::from_millis(1100));
        timer.tick();
        assert_eq!(timer.session_type, SessionType::Work);
        assert_eq!(timer.minutes(), 25);

        // Complete 2 more work+break cycles (sessions 2-3)
        for _ in 0..2 {
            timer.start();
            timer.remaining = Duration::from_secs(1);
            std::thread::sleep(Duration::from_millis(1100));
            timer.tick();
            assert_eq!(timer.session_type, SessionType::ShortBreak);
            timer.start();
            timer.remaining = Duration::from_secs(1);
            std::thread::sleep(Duration::from_millis(1100));
            timer.tick();
        }
        assert_eq!(timer.sessions_completed, 3);

        // 4th work session → long break
        timer.start();
        timer.remaining = Duration::from_secs(1);
        std::thread::sleep(Duration::from_millis(1100));
        timer.tick();
        assert_eq!(timer.sessions_completed, 4);
        assert_eq!(timer.session_type, SessionType::LongBreak);
        assert_eq!(timer.minutes(), 15);
    }
}
