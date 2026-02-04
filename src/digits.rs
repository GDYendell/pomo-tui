pub const DIGIT_HEIGHT: usize = 5;
pub const DIGIT_WIDTH: usize = 6;
pub const DIGIT_SPACING: usize = 2;

/// Minimum width needed to display block digits with 1 char padding on each side
/// 4 digits × 6 + 3 spacings × 2 + colon × 2 + 2 colon spacings × 2 + 2 padding = 38
pub const TIMER_MIN_WIDTH: u16 = 38;

const DIGITS: [[&str; 5]; 10] = [
    // 0
    [
        "██████",
        "██  ██",
        "██  ██",
        "██  ██",
        "██████",
    ],
    // 1
    [
        "  ██  ",
        "  ██  ",
        "  ██  ",
        "  ██  ",
        "  ██  ",
    ],
    // 2
    [
        "██████",
        "    ██",
        "██████",
        "██    ",
        "██████",
    ],
    // 3
    [
        "██████",
        "    ██",
        "██████",
        "    ██",
        "██████",
    ],
    // 4
    [
        "██  ██",
        "██  ██",
        "██████",
        "    ██",
        "    ██",
    ],
    // 5
    [
        "██████",
        "██    ",
        "██████",
        "    ██",
        "██████",
    ],
    // 6
    [
        "██████",
        "██    ",
        "██████",
        "██  ██",
        "██████",
    ],
    // 7
    [
        "██████",
        "    ██",
        "    ██",
        "    ██",
        "    ██",
    ],
    // 8
    [
        "██████",
        "██  ██",
        "██████",
        "██  ██",
        "██████",
    ],
    // 9
    [
        "██████",
        "██  ██",
        "██████",
        "    ██",
        "██████",
    ],
];

const COLON: [&str; 5] = [
    "  ",
    "██",
    "  ",
    "██",
    "  ",
];

pub fn digit_lines(d: u8) -> [&'static str; 5] {
    DIGITS[d as usize % 10]
}

pub fn render_time(minutes: u64, seconds: u64) -> Vec<String> {
    let m1 = (minutes / 10) as u8;
    let m2 = (minutes % 10) as u8;
    let s1 = (seconds / 10) as u8;
    let s2 = (seconds % 10) as u8;

    let d1 = digit_lines(m1);
    let d2 = digit_lines(m2);
    let d3 = digit_lines(s1);
    let d4 = digit_lines(s2);

    let spacing = " ".repeat(DIGIT_SPACING);
    let colon_spacing = " ".repeat(DIGIT_SPACING);

    (0..DIGIT_HEIGHT)
        .map(|i| {
            format!(
                "{}{}{}{}{}{}{}{}{}",
                d1[i], spacing, d2[i], colon_spacing, COLON[i], colon_spacing, d3[i], spacing, d4[i]
            )
        })
        .collect()
}

/// Render wave indicator with 5 dots
/// Position 0-4 indicates which dot is large, None for static display
pub fn render_wave(position: Option<usize>) -> String {
    const LARGE: char = '●';
    const SMALL: char = '·';
    const DOT_SPACING: &str = " ";

    match position {
        Some(pos) => {
            (0..5)
                .map(|i| if i == pos { LARGE } else { SMALL })
                .collect::<Vec<_>>()
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(DOT_SPACING)
        }
        None => {
            vec![SMALL; 5]
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(DOT_SPACING)
        }
    }
}

/// Calculate wave position from tick count (bounces back and forth)
/// Completes exactly one full oscillation per second (at 100ms tick rate)
pub fn wave_position(tick_count: u32) -> usize {
    // 5 dots = positions 0-4, bounce = 0,1,2,3,4,3,2,1 = 8 positions
    let tick = (tick_count % 8) as usize;
    if tick < 5 {
        tick
    } else {
        8 - tick
    }
}
