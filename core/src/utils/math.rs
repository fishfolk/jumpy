/// Clamps a sin wave between a minimum and maximum value.
pub fn sine_between(min: f32, max: f32, t: f32) -> f32 {
    ((max - min) * t.sin() + max + min) / 2.
}
