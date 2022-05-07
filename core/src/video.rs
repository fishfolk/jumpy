use serde::de::Expected;
use serde::{Deserialize, Serialize};

pub use crate::backend_impl::video::*;

use crate::math::{ivec2, IVec2, Size, UVec2, Vec2, Zero};
use crate::result::Result;

pub const DEFAULT_MSAA_SAMPLES: Option<u16> = Some(1);
pub const DEFAULT_MAX_FPS: Option<u16> = Some(120);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    #[serde(
        default = "VideoConfig::default_msaa_samples",
        rename = "msaa-samples",
        skip_serializing_if = "Option::is_none"
    )]
    pub msaa_samples: Option<u16>,
    #[serde(
        default = "VideoConfig::default_max_fps",
        rename = "max-fps",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_fps: Option<u16>,
    #[serde(default, rename = "vsync")]
    pub is_vsync_enabled: bool,
    #[serde(default, rename = "show-fps")]
    pub should_show_fps: bool,
}

impl VideoConfig {
    pub(crate) fn default_msaa_samples() -> Option<u16> {
        DEFAULT_MSAA_SAMPLES
    }

    pub(crate) fn default_max_fps() -> Option<u16> {
        DEFAULT_MAX_FPS
    }
}

impl Default for VideoConfig {
    fn default() -> Self {
        VideoConfig {
            msaa_samples: DEFAULT_MSAA_SAMPLES,
            max_fps: DEFAULT_MAX_FPS,
            is_vsync_enabled: false,
            should_show_fps: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AspectRatio(f32);

impl AspectRatio {
    pub fn to_f32(self) -> f32 {
        self.into()
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        AspectRatio(aspect_ratios::SQUARE)
    }
}

impl From<f32> for AspectRatio {
    fn from(ratio: f32) -> Self {
        assert!(
            ratio > 0.0,
            "Aspect ratio must be a positive number (is '{}')!",
            ratio
        );

        AspectRatio(ratio)
    }
}

impl From<AspectRatio> for f32 {
    fn from(ratio: AspectRatio) -> Self {
        let ratio = ratio.0;

        assert!(
            ratio > 0.0,
            "Aspect ratio must be a positive number (is '{}')!",
            ratio
        );

        ratio
    }
}

pub type Resolution = Size<u32>;

impl Resolution {
    /// This will set height to be width divided by `ratio`
    pub fn height_to_ratio<R: Into<AspectRatio>>(&mut self, ratio: R) {
        assert_ne!(
            self.width, 0,
            "Unable to set height of resolution from ratio when width is 0!"
        );

        let ratio = ratio.into();

        self.height = (self.width as f32 / ratio.to_f32()) as u32;
    }

    /// This will set width to be height multiplied with `ratio`
    pub fn width_to_ratio<R: Into<AspectRatio>>(&mut self, ratio: R) {
        assert_ne!(
            self.height, 0,
            "Unable to set height of resolution from ratio when height is 0!"
        );

        let ratio = ratio.into();

        self.width = (self.height as f32 * ratio.to_f32()) as u32;
    }

    /// This returns the aspect ratio (`width` divided by `height`)
    pub fn aspect_ratio(&self) -> f32 {
        assert_ne!(
            self.width, 0,
            "Unable to calculate aspect ratio of resolution when width is 0"
        );
        assert_ne!(
            self.height, 0,
            "Unable to calculate aspect ratio of resolution when height is 0"
        );

        self.width as f32 / self.height as f32
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoMode {
    pub resolution: Resolution,
    pub bit_depth: u16,
    pub refresh_rate: u16,
    pub display: Option<Display>,
}

impl std::fmt::Display for VideoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}x{} @ {} Hz ({} bpp)",
            self.resolution.width, self.resolution.height, self.refresh_rate, self.bit_depth,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Display {
    pub name: Option<String>,
    pub resolution: Resolution,
    pub position: Vec2,
    pub scale_factor: f32,
    pub video_modes: Vec<VideoMode>,
}

pub mod aspect_ratios {
    pub const SQUARE_1_1: f32 = 1.0 / 1.0;
    pub const SQUARE: f32 = SQUARE_1_1;

    pub const STANDARD_4_3: f32 = 4.0 / 3.0;
    pub const STANDARD: f32 = STANDARD_4_3;

    pub const VERTICAL_9_16: f32 = 9.0 / 16.0;
    pub const VERTICAL: f32 = VERTICAL_9_16;

    pub const WIDESCREEN_16_9: f32 = 16.0 / 9.0;
    pub const WIDESCREEN: f32 = WIDESCREEN_16_9;

    pub const WIDESCREEN_16_10: f32 = 16.0 / 10.0;
}

pub mod resolutions {
    use super::Resolution;

    pub const VGA: Resolution = Resolution {
        width: 640,
        height: 480,
    };

    pub const WVGA: Resolution = Resolution {
        width: 768,
        height: 480,
    };

    pub const FWVGA: Resolution = Resolution {
        width: 854,
        height: 480,
    };

    pub const SVGA: Resolution = Resolution {
        width: 800,
        height: 600,
    };

    pub const DVGA: Resolution = Resolution {
        width: 960,
        height: 640,
    };

    pub const XGA: Resolution = Resolution {
        width: 1024,
        height: 768,
    };

    pub const WXGA: Resolution = Resolution {
        width: 1366,
        height: 768,
    };

    pub const XGAP: Resolution = Resolution {
        width: 1152,
        height: 864,
    };

    pub const WXGAP: Resolution = Resolution {
        width: 1440,
        height: 900,
    };

    pub const SXGA: Resolution = Resolution {
        width: 1280,
        height: 1024,
    };

    pub const WSXGAP: Resolution = Resolution {
        width: 1680,
        height: 1050,
    };

    pub const WUXGA: Resolution = Resolution {
        width: 1920,
        height: 1200,
    };

    pub const NHD: Resolution = Resolution {
        width: 640,
        height: 360,
    };

    pub const HD720: Resolution = Resolution {
        width: 1280,
        height: 720,
    };

    pub const HD: Resolution = HD720;

    pub const HD900: Resolution = Resolution {
        width: 1600,
        height: 900,
    };

    pub const HDP: Resolution = HD900;

    pub const HD1080: Resolution = Resolution {
        width: 1920,
        height: 1080,
    };

    pub const FHD: Resolution = HD1080;

    pub const HD1440: Resolution = Resolution {
        width: 2560,
        height: 1440,
    };

    pub const QHD: Resolution = HD1440;

    pub const UHD4K: Resolution = Resolution {
        width: 3840,
        height: 2160,
    };

    pub const DCI4K: Resolution = Resolution {
        width: 4096,
        height: 2160,
    };

    pub const UHD5K: Resolution = Resolution {
        width: 5120,
        height: 2880,
    };

    pub const UHD8K: Resolution = Resolution {
        width: 5120,
        height: 2880,
    };

    pub const UHD16K: Resolution = Resolution {
        width: 17280,
        height: 4320,
    };
}
