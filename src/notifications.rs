use std::process::Command;

use crate::melodies::Melody;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink};

/// Send text notification via notify-send, returning any error message
pub fn send_notification(title: &str, message: &str) -> Option<String> {
    match Command::new("notify-send")
        .arg(title)
        .arg(message)
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .output()
    {
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Some(stderr.trim().to_string())
        }
        Err(e) => Some(format!("Failed to run notify-send: {e}")),
        _ => None,
    }
}

/// Renders a [`Melody`] into a [`SamplesBuffer`].
///
/// Each audible note gets a short linear fade-out to prevent inter-note clicks.
/// A tail of silence is appended so hardware output buffers flush cleanly.
fn load_melody(melody: Melody) -> SamplesBuffer<f32> {
    const SAMPLE_RATE: u32 = 44100;
    const AMP: f32 = 0.3;
    const FADE: usize = 400; // ~9ms linear fade-out per note
    #[allow(clippy::cast_precision_loss)]
    const FADE_STEP: f32 = 1.0 / FADE as f32;

    let mut samples = Vec::new();
    for &(freq, dur_ms) in melody {
        let n = SAMPLE_RATE as usize * dur_ms as usize / 1000;
        #[allow(clippy::cast_precision_loss)]
        let phase_inc = freq / (SAMPLE_RATE as f32);
        let mut phase = 0.0_f32;
        let mut gain = 1.0_f32;
        for i in 0..n {
            if freq > 0.0 && n - i <= FADE {
                gain = (gain - FADE_STEP).max(0.0);
            }
            samples.push((std::f32::consts::TAU * phase).sin() * AMP * gain);
            phase = (phase + phase_inc).fract();
        }
    }

    // Trailing silence lets the hardware output buffer drain before the stream ends
    samples.resize(samples.len() + SAMPLE_RATE as usize / 5, 0.0_f32);

    SamplesBuffer::new(1, SAMPLE_RATE, samples)
}

/// Play audio notifications using rodio for session completion alerts
pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl AudioPlayer {
    pub fn new() -> Option<Self> {
        let (stream, stream_handle) = OutputStream::try_default().ok()?;
        Some(Self {
            _stream: stream,
            stream_handle,
        })
    }

    /// Play a melody without blocking
    pub fn play_melody(&self, melody: Melody) {
        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            sink.append(load_melody(melody));
            sink.detach();
        }
    }
}
