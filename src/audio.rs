use std::time::Duration;

use rodio::source::{SineWave, Source};
use rodio::{OutputStream, OutputStreamHandle, Sink};

pub struct AudioPlayer {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
}

impl AudioPlayer {
    pub fn new() -> Option<Self> {
        let (stream, stream_handle) = OutputStream::try_default().ok()?;
        Some(Self {
            _stream: stream,
            _stream_handle: stream_handle,
        })
    }

    pub fn play_notification(&self) {
        // Create a sink for playback
        if let Ok(sink) = Sink::try_new(&self._stream_handle) {
            // Generate a pleasant two-tone notification
            // First tone: 880 Hz (A5) for 150ms
            let tone1 = SineWave::new(880.0)
                .take_duration(Duration::from_millis(150))
                .amplify(0.3);

            // Brief pause
            let silence = SineWave::new(0.0)
                .take_duration(Duration::from_millis(50))
                .amplify(0.0);

            // Second tone: 1108 Hz (C#6) for 200ms
            let tone2 = SineWave::new(1108.0)
                .take_duration(Duration::from_millis(200))
                .amplify(0.3);

            sink.append(tone1);
            sink.append(silence);
            sink.append(tone2);

            // Detach so it plays without blocking
            sink.detach();
        }
    }
}
