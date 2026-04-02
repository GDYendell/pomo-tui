/// A sequence of (`frequency_hz`, `duration_ms`) note pairs. Use `SILENCE` for rests.
pub type Melody = &'static [(f32, u64)];

// Note frequencies in Hz
const SILENCE: f32 = 0.0;
const AB4: f32 = 415.30;
const BB4: f32 = 466.16;
const C5: f32 = 523.25;
const A5: f32 = 880.0;
const CS6: f32 = 1108.0;

/// A5 and C#6 two-tone chime
pub const TWO_TONE: Melody = &[(A5, 150), (SILENCE, 50), (CS6, 200)];

/// Final Fantasy VII victory fanfare
pub const VICTORY_FANFARE: Melody = {
    const U: u64 = 150; // one beat unit (0.25 beats at 100 BPM) in ms
    const GAP: u64 = 15;
    &[
        (C5, U - GAP),
        (SILENCE, GAP),
        (C5, U - GAP),
        (SILENCE, GAP),
        (C5, U - GAP),
        (SILENCE, GAP),
        (C5, U * 3 - GAP),
        (SILENCE, GAP),
        (AB4, U * 3 - GAP),
        (SILENCE, GAP),
        (BB4, U * 3 - GAP),
        (SILENCE, GAP),
        (C5, U * 2 - GAP),
        (SILENCE, U / 2),
        (BB4, U - GAP),
        (SILENCE, GAP),
        (C5, U * 9),
    ]
};
