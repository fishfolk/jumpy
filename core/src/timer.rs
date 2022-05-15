use hecs::World;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use crate::result::Result;

/// A simple timer implementation
#[derive(Debug)]
pub struct TimerImpl {
    duration: Duration,
    elapsed: Duration,
}

static mut NEXT_TIMER_INDEX: usize = 0;
static mut TIMERS: Option<HashMap<usize, TimerImpl>> = None;

fn timer_map() -> &'static mut HashMap<usize, TimerImpl> {
    unsafe { TIMERS.get_or_insert_with(HashMap::new) }
}

fn timer_index() -> usize {
    unsafe {
        let index = NEXT_TIMER_INDEX;
        NEXT_TIMER_INDEX += 1;
        index
    }
}

pub struct Timer(usize);

impl Timer {
    /// Create a new timer with the provided duration
    pub fn new(duration: Duration) -> Self {
        let timer_impl = TimerImpl {
            duration,
            elapsed: Duration::ZERO,
        };

        let index = timer_index();

        timer_map().insert(index, timer_impl);

        Timer(index)
    }

    /// Create a new timer with the provided duration
    pub fn from_secs_f32(duration_secs: f32) -> Self {
        Timer::new(Duration::from_secs_f32(duration_secs))
    }

    /// Get the duration of the timer
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Get the time that has been elapsed
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Return the percentage completion of the timer as a number between 0 and 1
    pub fn progress(&self) -> f32 {
        self.elapsed.as_secs_f32() / self.duration.as_secs_f32()
    }

    /// Get whether or not the timer has finished
    pub fn has_finished(&self) -> bool {
        self.elapsed > self.duration
    }

    /// Reset the time elapsed
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
    }
}

impl Deref for Timer {
    type Target = TimerImpl;

    fn deref(&self) -> &Self::Target {
        timer_map().get(&self.0).unwrap()
    }
}

impl DerefMut for Timer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        timer_map().get_mut(&self.0).unwrap()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        timer_map().remove_entry(&self.0).unwrap();
    }
}

pub fn update_timers(_world: &mut World, delta_time: f32) -> Result<()> {
    for timer in timer_map().values_mut() {
        timer.elapsed += Duration::from_secs_f32(delta_time);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_timer() {
        {
            let mut t = Timer::from_secs_f32(3.0);

            assert_eq!(t.duration(), Duration::from_secs_f32(3.0));

            t.elapsed += Duration::from_secs_f32(0.5);
            assert_eq!(t.elapsed(), Duration::from_secs_f32(0.5));
            assert_eq!(t.has_finished(), false);

            t.elapsed += Duration::from_secs_f32(2.0);
            assert_eq!(t.elapsed(), Duration::from_secs_f32(2.5));
            assert_eq!(t.has_finished(), false);

            t.elapsed += Duration::from_secs_f32(1.0);
            assert_eq!(t.elapsed(), Duration::from_secs_f32(3.5));
            assert_eq!(t.has_finished(), true);

            t.reset();
            assert_eq!(t.duration(), Duration::from_secs_f32(3.0));
            assert_eq!(t.elapsed(), Duration::from_secs_f32(0.0));
        }

        assert_eq!(timer_map().len(), 0);
    }
}
