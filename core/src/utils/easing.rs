use std::f32::consts::PI;

/// Simple easing calculator
pub struct Ease {
    pub ease_in: bool,
    pub ease_out: bool,
    pub function: EaseFunction,
    pub progress: f32,
}

pub enum EaseFunction {
    Quadratic,
    Cubic,
    Sinusoidial,
}

impl Default for Ease {
    fn default() -> Self {
        Self {
            ease_in: true,
            ease_out: true,
            function: EaseFunction::Quadratic,
            progress: 0.0,
        }
    }
}

impl Ease {
    pub fn output(&self) -> f32 {
        let mut k = self.progress;

        // Reference for easing functions:
        // https://echarts.apache.org/examples/en/editor.html?c=line-easing&lang=ts
        //
        // TODO: Add tests to make sure easings are correct
        match (&self.function, self.ease_in, self.ease_out) {
            (EaseFunction::Quadratic, true, true) => {
                k *= 2.0;
                if k < 1.0 {
                    0.5 * k * k
                } else {
                    k -= 1.0;
                    -0.5 * (k * (k - 2.0) - 1.0)
                }
            }
            (EaseFunction::Quadratic, true, false) => k * k,
            (EaseFunction::Quadratic, false, true) => k * (2.0 - k),
            (EaseFunction::Cubic, true, true) => {
                k *= 2.0;
                if k < 1.0 {
                    0.5 * k * k * k
                } else {
                    k -= 2.0;
                    0.5 * (k * k * k + 2.0)
                }
            }
            (EaseFunction::Cubic, true, false) => k * k * k,
            (EaseFunction::Cubic, false, true) => {
                k -= 1.0;
                k * k * k + 1.0
            }
            (EaseFunction::Sinusoidial, true, true) => 0.5 * (1.0 - f32::cos(PI * k)),
            (EaseFunction::Sinusoidial, true, false) => 1.0 - f32::cos((k * PI) / 2.0),
            (EaseFunction::Sinusoidial, false, true) => f32::sin((k * PI) / 2.0),
            (_, false, false) => k,
        }
    }
}
