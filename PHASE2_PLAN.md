# Phase 2 Implementation Plan: Timer Enhancements + Audio

## Overview
Enhance the timer panel with large block digits, running animation, manual mode switching, and audio notifications.

---

## Feature 1: Large Block Digits

Display time using block characters with straight edges and right angles.

### Design
```
██████  ██████     ██████  ██████
    ██  ██             ██      ██
██████  ██████     ██████  ██████
██          ██     ██          ██
██████  ██████     ██████  ██████

         ·  ●  ·  ·  ·
```

### Digit Patterns (5 lines tall, 6 chars wide)
```
0:        1:        2:        3:        4:
██████       ██   ██████   ██████   ██  ██
██  ██       ██       ██       ██   ██  ██
██  ██       ██   ██████   ██████   ██████
██  ██       ██   ██           ██       ██
██████       ██   ██████   ██████       ██

5:        6:        7:        8:        9:
██████   ██████   ██████   ██████   ██████
██       ██           ██   ██  ██   ██  ██
██████   ██████       ██   ██████   ██████
    ██   ██  ██       ██   ██  ██       ██
██████   ██████       ██   ██████   ██████
```

### New File: `src/digits.rs`

```rust
pub const DIGIT_HEIGHT: usize = 5;
pub const DIGIT_WIDTH: usize = 6;

pub fn digit_lines(d: u8) -> [&'static str; 5] { ... }
pub fn render_time(minutes: u64, seconds: u64) -> Vec<String> { ... }
```

---

## Feature 2: Running Indicator

5 dots pulsing in a wave pattern, moving back and forth.

### Design
```
Frame 0: ●  ·  ·  ·  ·
Frame 1: ·  ●  ·  ·  ·
Frame 2: ·  ·  ●  ·  ·
Frame 3: ·  ·  ·  ●  ·
Frame 4: ·  ·  ·  ·  ●
Frame 5: ·  ·  ·  ●  ·
Frame 6: ·  ·  ●  ·  ·
Frame 7: ·  ●  ·  ·  ·
(repeats)
```

### Characters
- Large dot: `●` (U+25CF)
- Small dot: `·` (U+00B7)

### Implementation
- Add `tick_count: u32` to `TimerPanel`
- Wave position cycles 0→1→2→3→4→3→2→1→0...
- Only animate when `TimerState::Running`
- Show static `·  ·  ·  ·  ·` when paused/idle

---

## Feature 3: Manual Mode Switching

Allow switching between work/short break/long break when timer is idle.

### Keybindings (Timer panel, only when idle)
| Key | Action |
|-----|--------|
| `w` | Switch to Work session |
| `s` | Switch to Short Break |
| `l` | Switch to Long Break |

### Implementation
- Add `set_session_type(SessionType)` method to `Timer`
- Only allow when `state == TimerState::Idle`
- Update `remaining` duration when switching
- Add shortcuts to `TimerPanel::shortcuts()` conditionally when idle

---

## Feature 4: Sound Notifications

Play audio alert when timer completes.

### Dependencies
Add to `Cargo.toml`:
```toml
rodio = "0.19"
```

### New File: `src/audio.rs`

```rust
pub struct AudioPlayer { ... }

impl AudioPlayer {
    pub fn new() -> Option<Self> { ... }
    pub fn play_notification(&self) { ... }
}
```

### Integration
- Add `audio: Option<AudioPlayer>` to `App`
- `Timer::tick()` returns `bool` indicating session completed
- App calls `audio.play_notification()` when session completes

### Audio File
- Create `assets/` directory
- Include a short notification sound (wav)

---

## Files to Create

1. `src/digits.rs` - Block digit rendering
2. `src/audio.rs` - Audio playback
3. `assets/notification.wav` - Notification sound

## Files to Modify

1. `src/main.rs` - Add module declarations
2. `src/timer.rs` - Add `set_session_type()`, return completion from `tick()`
3. `src/panels/timer.rs` - Block digits, wave animation, mode switching
4. `src/app.rs` - Audio player, handle session completion
5. `Cargo.toml` - Add rodio dependency

---

## Implementation Order

1. **Block digits** - Create `src/digits.rs` with straight-edge patterns
2. **Update timer rendering** - Integrate block digits into TimerPanel
3. **Wave indicator** - Add tick counter, 5-dot wave animation
4. **Manual mode switching** - Add w/s/l keybindings when idle
5. **Audio notifications** - Add rodio, create AudioPlayer, integrate

---

## Verification Checklist

- [ ] Block digits render with straight edges for 0-9
- [ ] Wave animation runs when timer is running
- [ ] Wave is static when paused/idle
- [ ] `w`/`s`/`l` switch modes when idle only
- [ ] Mode switching updates remaining time
- [ ] Sound plays on session completion
- [ ] App handles audio init failure gracefully
