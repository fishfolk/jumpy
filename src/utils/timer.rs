/// A simple timer utility
#[derive(Debug)]
pub struct Timer {
    duration: f32,
    elapsed: f32,
}

impl Timer {
    /// Create a new timer with the provider duration
    pub fn new(duration: f32) -> Self {
        Timer {
            duration,
            elapsed: f32::default(),
        }
    }

    /// Get the duration of the timer
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Get the time that has been elapsed
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Return the percentage completion of the timer as a number between 0 and 1
    pub fn progress(&self) -> f32 {
        self.elapsed / self.duration
    }

    /// Get whether or not the timer has finished
    pub fn has_finished(&self) -> bool {
        self.elapsed > self.duration
    }

    /// Reset the time elapsed
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Advanced the elapsed time by the macroquad frame time
    pub fn tick_frame_time(&mut self) {
        self.elapsed += macroquad::time::get_frame_time();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_timer() {
        let mut t = Timer::new(3.0);

        assert_eq!(t.duration(), 3.0);

        t.elapsed += 0.5;
        assert_eq!(t.elapsed(), 0.5);
        assert_eq!(t.has_finished(), false);

        t.elapsed += 2.0;
        assert_eq!(t.elapsed(), 2.5);
        assert_eq!(t.has_finished(), false);

        t.elapsed += 1.0;
        assert_eq!(t.elapsed(), 3.5);
        assert_eq!(t.has_finished(), true);

        t.reset();
        assert_eq!(t.duration(), 3.0);
        assert_eq!(t.elapsed(), 0.0);
    }
}
