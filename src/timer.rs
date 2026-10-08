use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Running,
    Paused,
    Expired,
}

pub struct Timer {
    pub original: Duration,
    remaining_at_start: Duration,
    started: Instant,
    pub status: Status,
}
impl Timer {
    pub fn new(duration: Duration, now: Instant) -> Self {
        Self {
            original: duration,
            remaining_at_start: duration,
            started: now,
            status: Status::Running,
        }
    }
    pub fn remaining(&self, now: Instant) -> Duration {
        match self.status {
            Status::Running => self
                .remaining_at_start
                .saturating_sub(now.saturating_duration_since(self.started)),
            Status::Paused => self.remaining_at_start,
            Status::Expired => Duration::ZERO,
        }
    }
    pub fn tick(&mut self, now: Instant) {
        if self.status == Status::Running && self.remaining(now).is_zero() {
            self.status = Status::Expired;
        }
    }
    pub fn toggle_pause(&mut self, now: Instant) {
        self.tick(now);
        match self.status {
            Status::Running => {
                self.remaining_at_start = self.remaining(now);
                self.status = Status::Paused;
            }
            Status::Paused => {
                self.started = now;
                self.status = Status::Running;
            }
            Status::Expired => {}
        }
    }
    pub fn restart(&mut self, now: Instant) {
        *self = Self::new(self.original, now);
    }
    pub fn progress(&self, now: Instant) -> f64 {
        (1.0 - self.remaining(now).as_secs_f64() / self.original.as_secs_f64()).clamp(0.0, 1.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pause_resume_and_restart_preserve_time() {
        let start = Instant::now();
        let mut timer = Timer::new(Duration::from_secs(10), start);
        timer.toggle_pause(start + Duration::from_secs(3));
        assert_eq!(
            timer.remaining(start + Duration::from_secs(100)),
            Duration::from_secs(7)
        );
        timer.toggle_pause(start + Duration::from_secs(100));
        timer.tick(start + Duration::from_secs(106));
        assert_eq!(
            timer.remaining(start + Duration::from_secs(106)),
            Duration::from_secs(1)
        );
        timer.tick(start + Duration::from_secs(107));
        assert_eq!(timer.status, Status::Expired);
        timer.toggle_pause(start + Duration::from_secs(108));
        assert_eq!(timer.status, Status::Expired);
        timer.restart(start + Duration::from_secs(109));
        assert_eq!(timer.status, Status::Running);
        assert_eq!(
            timer.remaining(start + Duration::from_secs(109)),
            Duration::from_secs(10)
        );
    }
    #[test]
    fn delayed_ticks_do_not_drift() {
        let start = Instant::now();
        let mut timer = Timer::new(Duration::from_secs(5), start);
        timer.tick(start + Duration::from_secs(20));
        assert_eq!(timer.status, Status::Expired);
        assert_eq!(timer.progress(start + Duration::from_secs(20)), 1.0);
    }
}
