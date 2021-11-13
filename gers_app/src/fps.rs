//! Tools for measuring and throttling FPS
use std::{
    thread,
    time::{Duration, Instant},
};

pub struct FpsThrottle {
    target: Duration,
    last_time: Instant,
    policy: FpsThrottlePolicy,
}

impl FpsThrottle {
    pub fn new(target_fps: u64, policy: FpsThrottlePolicy) -> Self {
        Self {
            target: Duration::from_secs_f64(1.0 / target_fps as f64),
            last_time: Instant::now(),
            policy,
        }
    }

    /// Block the current thread until the target delta time has passed.
    ///
    /// Provide the instant measurement given during the last frame's call.
    pub fn throttle(&mut self, last_time: Instant) {
        use FpsThrottlePolicy as P;

        self.last_time = last_time;
        let mut elapsed = Instant::now() - self.last_time;

        while elapsed <= self.target {
            match self.policy {
                P::Off => {
                    return;
                }
                P::Yield => {
                    thread::yield_now();
                }
                P::Sleep => {
                    let target_end = last_time + self.target;
                    let now = Instant::now();
                    if now < target_end {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            }

            elapsed = Instant::now() - self.last_time;
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum FpsThrottlePolicy {
    Off,
    Yield,
    Sleep,
}

/// Utility for measuring frame rate per second.
///
/// It takes periodic snapshots of the measured
/// fps to slow down the value being printed.
/// This makes it easier to read when presented to
/// the user.
pub struct FpsCounter {
    dt: Box<[f32; Self::DATA_POINT_COUNT]>,
    snapshot: f32,
    cursor: usize,
}

impl FpsCounter {
    const DATA_POINT_COUNT: usize = 100;

    pub fn new() -> Self {
        Self {
            dt: Box::new([0.0; Self::DATA_POINT_COUNT]),
            snapshot: 0.0,
            cursor: 0,
        }
    }

    pub fn add(&mut self, delta_time: Duration) {
        self.dt[self.cursor] = delta_time.as_secs_f32();
        if self.cursor == 0 {
            self.take_snapshot();
        }
        self.cursor = (self.cursor + 1) % self.dt.len();
    }

    fn take_snapshot(&mut self) {
        let sum: f32 = self.dt.iter().fold(0.0, |acc, el| acc + *el);
        let avg = sum / self.dt.len() as f32;
        // Approximately not zero
        if avg.abs() > f32::EPSILON {
            self.snapshot = 1.0 / avg;
        }
    }

    pub fn fps(&self) -> f32 {
        self.snapshot
    }
}
