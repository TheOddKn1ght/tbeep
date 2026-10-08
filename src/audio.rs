use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::{
    io::{self, Cursor, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub const WAV: &[u8] = include_bytes!("../assets/beep.wav");

#[derive(Default)]
struct Schedule {
    active: bool,
    due: Option<Instant>,
}
impl Schedule {
    fn ready(&mut self, active: bool, now: Instant, playing: bool) -> bool {
        if !active {
            self.active = false;
            self.due = None;
            return false;
        }
        if !self.active {
            self.active = true;
            self.due = Some(now);
        }
        if playing {
            self.due = None;
            return false;
        }
        let due = *self.due.get_or_insert(now + Duration::from_millis(500));
        if now >= due {
            self.due = None;
            true
        } else {
            false
        }
    }
}

pub struct Alarm {
    output: Option<MixerDeviceSink>,
    player: Option<Player>,
    attempted: bool,
    stream_failed: Arc<AtomicBool>,
    schedule: Schedule,
    pub warning: Option<String>,
}
impl Alarm {
    pub fn new() -> Self {
        Self {
            output: None,
            player: None,
            attempted: false,
            stream_failed: Arc::new(AtomicBool::new(false)),
            schedule: Schedule::default(),
            warning: None,
        }
    }
    fn initialize(&mut self) {
        if self.attempted {
            return;
        }
        self.attempted = true;
        let failed = Arc::clone(&self.stream_failed);
        let result = DeviceSinkBuilder::from_default_device().and_then(|builder| {
            builder
                .with_error_callback(move |_| {
                    failed.store(true, Ordering::Relaxed);
                })
                .open_sink_or_fallback()
        });
        match result {
            Ok(mut output) => {
                output.log_on_drop(false);
                self.output = Some(output);
            }
            Err(e) => {
                self.warning = Some(format!(
                    "Audio unavailable: {e}. Using terminal bell (may be silenced)."
                ))
            }
        }
    }
    pub fn tick(&mut self, active: bool, now: Instant) -> io::Result<()> {
        if !active {
            self.stop();
            return Ok(());
        }
        self.initialize();
        if self.stream_failed.swap(false, Ordering::Relaxed) {
            self.stop();
            self.output = None;
            self.warning =
                Some("Audio device disconnected. Using terminal bell (may be silenced).".into());
        }
        let playing = self.player.as_ref().is_some_and(|p| !p.empty());
        if self.schedule.ready(active, now, playing) {
            if let Some(output) = &self.output {
                match Decoder::try_from(Cursor::new(WAV)) {
                    Ok(source) => {
                        let player = Player::connect_new(output.mixer());
                        player.append(source);
                        self.player = Some(player);
                        return Ok(());
                    }
                    Err(e) => {
                        self.warning =
                            Some(format!("Cannot decode alarm: {e}. Using terminal bell."));
                        self.output = None;
                    }
                }
            }
            print!("\x07");
            io::stdout().flush()?;
        }
        Ok(())
    }
    pub fn stop(&mut self) {
        if let Some(player) = self.player.take() {
            player.stop();
        }
        self.schedule = Schedule::default();
    }
}
impl Drop for Alarm {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_alarm_decodes() {
        let source = Decoder::try_from(Cursor::new(WAV)).unwrap();
        assert!(source.count() > 0);
    }
    #[test]
    fn repeats_after_gap_and_cancels_immediately() {
        let now = Instant::now();
        let mut schedule = Schedule::default();
        assert!(schedule.ready(true, now, false));
        assert!(!schedule.ready(true, now + Duration::from_secs(1), true));
        assert!(!schedule.ready(true, now + Duration::from_secs(2), false));
        assert!(!schedule.ready(true, now + Duration::from_millis(2499), false));
        assert!(schedule.ready(true, now + Duration::from_millis(2500), false));
        assert!(!schedule.ready(false, now + Duration::from_secs(3), true));
        assert!(schedule.ready(true, now + Duration::from_secs(3), false));
    }
}
