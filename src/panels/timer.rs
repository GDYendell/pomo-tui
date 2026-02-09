use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crossterm::event::{KeyCode, KeyEvent};

use super::util::{panel_block, KeyHandleResult};
use crate::task::Task;
use crate::task_manager::TaskManager;
use crate::timer::{SessionType, Timer};
use crate::util::Shortcut;

pub struct TimerPanel {
    tick_count: u32,
}

impl Default for TimerPanel {
    fn default() -> Self {
        Self { tick_count: 0 }
    }
}

const TIMER_MIN_HEIGHT: u16 = 11; // digits + wave + blank + label + blank
const BOTTOM_BORDER: u16 = 1; // Borders::TOP
const BOTTOM_PAD: u16 = 2; // 1 row above + 1 row below text
/// Minimum width needed to display block digits with 1 char padding on each side
/// 4 digits × 6 + 3 spacings × 2 + colon × 2 + 2 colon spacings × 2 + 2 padding = 38
pub const TIMER_MIN_WIDTH: u16 = 38;
const DIGIT_HEIGHT: usize = 5;
const DIGIT_SPACING: u16 = 2;

const DIGITS: [[&str; 5]; 10] = [
    ["██████", "██  ██", "██  ██", "██  ██", "██████"],
    ["  ██  ", "  ██  ", "  ██  ", "  ██  ", "  ██  "],
    ["██████", "    ██", "██████", "██    ", "██████"],
    ["██████", "    ██", "██████", "    ██", "██████"],
    ["██  ██", "██  ██", "██████", "    ██", "    ██"],
    ["██████", "██    ", "██████", "    ██", "██████"],
    ["██████", "██    ", "██████", "██  ██", "██████"],
    ["██████", "    ██", "    ██", "    ██", "    ██"],
    ["██████", "██  ██", "██████", "██  ██", "██████"],
    ["██████", "██  ██", "██████", "    ██", "██████"],
];

const COLON: [&str; 5] = ["  ", "██", "  ", "██", "  "];

impl TimerPanel {
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
        timer: &Timer,
        active_task: Option<&Task>,
    ) {
        let block = panel_block(" Timer ", focused);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // In break mode, no bottom section — timer gets everything
        if timer.session_type() != SessionType::Work {
            self.render_timer_display(frame, inner, timer);
            return;
        }

        // Calculate bottom section height based on wrapped text
        let text_area_width = (inner.width as usize).saturating_sub(4); // 2 cols padding each side
        let text = match active_task {
            Some(task) => task.text.as_str(),
            None => "No task selected",
        };
        let wrapped_lines = if text_area_width > 0 {
            wrap_line_count(text, text_area_width)
        } else {
            1
        };
        let bottom_inner = BOTTOM_PAD + wrapped_lines as u16;
        let bottom_total = BOTTOM_BORDER + bottom_inner;

        let h = inner.height;

        // Need at least TIMER_MIN_HEIGHT for timer + bottom_total for bottom
        if h < TIMER_MIN_HEIGHT + bottom_total {
            // Not enough room — timer gets everything
            self.render_timer_display(frame, inner, timer);
        } else {
            let timer_h = h - bottom_total;
            let chunks = Layout::vertical([
                Constraint::Length(timer_h),
                Constraint::Length(bottom_total),
            ])
            .split(inner);
            self.render_timer_display(frame, chunks[0], timer);
            self.render_current_task(frame, chunks[1], active_task);
        }
    }

    pub fn shortcuts(&self, timer: &Timer, has_active_task: bool) -> Vec<Shortcut> {
        let mut shortcuts = vec![
            Shortcut {
                key: "Space",
                description: "Start/Pause",
            },
            Shortcut {
                key: "R",
                description: "Reset",
            },
        ];

        if timer.is_idle() {
            shortcuts.push(Shortcut {
                key: "Tab",
                description: "Mode",
            });
        }

        if has_active_task {
            shortcuts.push(Shortcut {
                key: "X",
                description: "Complete",
            });
        }

        shortcuts
    }

    pub fn handle_key(
        &self,
        key: KeyEvent,
        timer: &mut Timer,
        task_manager: &mut TaskManager,
    ) -> KeyHandleResult {
        match key.code {
            KeyCode::Char(' ') => {
                timer.toggle();
                KeyHandleResult::Consumed
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                timer.reset();
                KeyHandleResult::Consumed
            }
            KeyCode::Char('w') | KeyCode::Char('W') if timer.is_idle() => {
                timer.set_session_type(SessionType::Work);
                KeyHandleResult::Consumed
            }
            KeyCode::Char('b') | KeyCode::Char('B') if timer.is_idle() => {
                timer.set_session_type(SessionType::LongBreak);
                KeyHandleResult::Consumed
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                task_manager.complete_active();
                KeyHandleResult::Consumed
            }
            KeyCode::Tab | KeyCode::BackTab if timer.is_idle() => {
                timer.next_session_type();
                KeyHandleResult::Consumed
            }
            _ => KeyHandleResult::Ignored,
        }
    }

    /// Update animation tick counter without ticking the timer
    pub fn tick_animation(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    fn render_timer_display(&self, frame: &mut Frame, area: Rect, timer: &Timer) {
        let time_lines = render_time(timer.minutes(), timer.seconds());
        let session_color = session_color(timer.session_type());

        let wave = if timer.is_running() {
            render_wave(Some(wave_position(self.tick_count)))
        } else {
            render_wave(None)
        };

        let session_str = match timer.session_type() {
            SessionType::Work => "WORK",
            SessionType::ShortBreak => "SHORT BREAK",
            SessionType::LongBreak => "LONG BREAK",
        };

        // Fixed top: blank + 5 digit lines + blank = 7 lines
        let mut digits: Vec<Line> = vec![Line::from("")];
        for line in time_lines {
            digits.push(Line::from(Span::styled(
                line,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        digits.push(Line::from(""));

        // Bottom part: wave + blank + label = 3 lines, centered in remaining space
        let below: Vec<Line> = vec![
            Line::from(Span::styled(wave, Style::default().fg(session_color))),
            Line::from(""),
            Line::from(Span::styled(
                session_str,
                Style::default().fg(session_color),
            )),
        ];

        let remaining_h = area
            .height
            .saturating_sub(DIGIT_HEIGHT as u16 + DIGIT_SPACING);

        if remaining_h >= 3 {
            // Split: digits at top, wave+label centered in remaining space
            let chunks = Layout::vertical([
                Constraint::Length(DIGIT_HEIGHT as u16 + DIGIT_SPACING),
                Constraint::Length(remaining_h),
            ])
            .split(area);

            let digits_para = Paragraph::new(digits).alignment(Alignment::Center);
            frame.render_widget(digits_para, chunks[0]);

            // Center the 3 lines of wave+label within the remaining area
            let pad_top = (remaining_h.saturating_sub(3)) / 2;
            let mut below_content: Vec<Line> = Vec::new();
            for _ in 0..pad_top {
                below_content.push(Line::from(""));
            }
            below_content.extend(below);

            let below_para = Paragraph::new(below_content).alignment(Alignment::Center);
            frame.render_widget(below_para, chunks[1]);
        } else {
            // Not enough room — just render digits
            let digits_para = Paragraph::new(digits).alignment(Alignment::Center);
            frame.render_widget(digits_para, area);
        }
    }

    fn render_current_task(&self, frame: &mut Frame, area: Rect, active_task: Option<&Task>) {
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Current Task ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width < 5 {
            return;
        }

        let (text, style) = match active_task {
            Some(task) => (
                task.text.as_str(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            None => ("No task selected", Style::default().fg(Color::DarkGray)),
        };

        // 1 row pad top, text, 1 row pad bottom — with 2 cols padding each side
        let text_area = Rect::new(
            inner.x + 2,
            inner.y + 1,
            inner.width.saturating_sub(4),
            inner.height.saturating_sub(2),
        );

        if text_area.width == 0 || text_area.height == 0 {
            return;
        }

        let paragraph = Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, text_area);
    }
}

fn session_color(session_type: SessionType) -> Color {
    match session_type {
        SessionType::Work => Color::Red,
        SessionType::ShortBreak => Color::Green,
        SessionType::LongBreak => Color::Blue,
    }
}

/// Count how many lines text will wrap to at the given width.
fn wrap_line_count(text: &str, width: usize) -> usize {
    if text.is_empty() || width == 0 {
        return 1;
    }
    let mut lines = 1usize;
    let mut col = 0usize;
    for word in text.split_whitespace() {
        let wlen = word.chars().count();
        if col == 0 {
            col = wlen;
        } else if col + 1 + wlen <= width {
            col += 1 + wlen;
        } else {
            lines += 1;
            col = wlen;
        }
    }
    lines
}

// -- Block digits --

fn digit_lines(d: u8) -> [&'static str; 5] {
    DIGITS[d as usize % 10]
}

fn render_time(minutes: u64, seconds: u64) -> Vec<String> {
    let d1 = digit_lines((minutes / 10) as u8);
    let d2 = digit_lines((minutes % 10) as u8);
    let d3 = digit_lines((seconds / 10) as u8);
    let d4 = digit_lines((seconds % 10) as u8);

    let spacing = " ".repeat(DIGIT_SPACING as usize);
    let colon_spacing = " ".repeat(DIGIT_SPACING as usize);

    (0..DIGIT_HEIGHT)
        .map(|i| {
            format!(
                "{}{}{}{}{}{}{}{}{}",
                d1[i],
                spacing,
                d2[i],
                colon_spacing,
                COLON[i],
                colon_spacing,
                d3[i],
                spacing,
                d4[i]
            )
        })
        .collect()
}

fn render_wave(position: Option<usize>) -> String {
    const LARGE: char = '●';
    const SMALL: char = '·';
    const DOT_SPACING: &str = " ";

    match position {
        Some(pos) => (0..5)
            .map(|i| if i == pos { LARGE } else { SMALL })
            .collect::<Vec<_>>()
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(DOT_SPACING),
        None => vec![SMALL; 5]
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(DOT_SPACING),
    }
}

/// Calculate wave position from tick count (bounces back and forth)
fn wave_position(tick_count: u32) -> usize {
    let tick = (tick_count % 8) as usize;
    if tick < 5 {
        tick
    } else {
        8 - tick
    }
}
