pub mod display;
mod constants;

pub use display::DisplayHandle;
pub use constants::*;

use serde::{Serialize, Deserialize};
use winit::dpi::LogicalSize;

#[cfg(feature = "winit")]
use winit::dpi::PhysicalSize;

use crate::Result;

pub const ASPECT_RATIO_SQUARE: f32 = 1.0 / 1.0;
pub const ASPECT_RATIO_STANDARD: f32 = 4.0 / 3.0;
pub const ASPECT_RATIO_VERTICAL: f32 = 9.0 / 16.0;
pub const ASPECT_RATIO_WIDESCREEN: f32 = 16.0 / 9.0;
pub const ASPECT_RATIO_WIDESCREEN_OLD: f32 = 16.0 / 10.0;

pub const RESOLUTION_VGA: Resolution = Resolution {
    width: 640,
    height: 480,
};

pub const RESOLUTION_WVGA: Resolution = Resolution {
    width: 768,
    height: 480,
};

pub const RESOLUTION_FWVGA: Resolution = Resolution {
    width: 854,
    height: 480,
};

pub const RESOLUTION_SVGA: Resolution = Resolution {
    width: 800,
    height: 600,
};

pub const RESOLUTION_DVGA: Resolution = Resolution {
    width: 960,
    height: 640,
};

pub const RESOLUTION_XGA: Resolution = Resolution {
    width: 1024,
    height: 768,
};

pub const RESOLUTION_WXGA: Resolution = Resolution {
    width: 1366,
    height: 768,
};

pub const RESOLUTION_XGAP: Resolution = Resolution {
    width: 1152,
    height: 864,
};

pub const RESOLUTION_WXGAP: Resolution = Resolution {
    width: 1440,
    height: 900,
};

pub const RESOLUTION_SXGA: Resolution = Resolution {
    width: 1280,
    height: 1024,
};

pub const RESOLUTION_WSXGAP: Resolution = Resolution {
    width: 1680,
    height: 1050,
};

pub const RESOLUTION_WUXGA: Resolution = Resolution {
    width: 1920,
    height: 1200,
};

pub const RESOLUTION_NHD: Resolution = Resolution {
    width: 640,
    height: 360,
};

pub const RESOLUTION_HD: Resolution = Resolution {
    width: 1280,
    height: 720,
};

pub const RESOLUTION_720: Resolution = RESOLUTION_HD;

pub const RESOLUTION_HDP: Resolution = Resolution {
    width: 1600,
    height: 900,
};

pub const RESOLUTION_900: Resolution = RESOLUTION_HDP;

pub const RESOLUTION_FHD: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

pub const RESOLUTION_1080: Resolution = RESOLUTION_FHD;

pub const RESOLUTION_QHD: Resolution = Resolution {
    width: 2560,
    height: 1440,
};

pub const RESOLUTION_1440: Resolution = RESOLUTION_QHD;

pub const RESOLUTION_4KUHD: Resolution = Resolution {
    width: 3840,
    height: 2160,
};

pub const RESOLUTION_DCI4K: Resolution = Resolution {
    width: 4096,
    height: 2160,
};

pub const RESOLUTION_5K: Resolution = Resolution {
    width: 5120,
    height: 2880,
};

pub const RESOLUTION_8K: Resolution = Resolution {
    width: 5120,
    height: 2880,
};

pub const RESOLUTION_16K: Resolution = Resolution {
    width: 17280,
    height: 4320,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AspectRatio(f32);

impl AspectRatio {
    pub fn to_f32(self) -> f32 {
        self.into()
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        AspectRatio(ASPECT_RATIO_SQUARE)
    }
}

impl From<f32> for AspectRatio {
    fn from(ratio: f32) -> Self {
        assert!(ratio > 0.0, "Aspect ratio must be a positive number (is '{}')!", ratio);

        AspectRatio(ratio)
    }
}

impl From<AspectRatio> for f32 {
    fn from(ratio: AspectRatio) -> Self {
        let ratio = ratio.0;

        assert!(ratio > 0.0, "Aspect ratio must be a positive number (is '{}')!", ratio);

        ratio
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        assert_ne!(width, 0, "Resolution width must be a positive number (is '{}')!", width);
        assert_ne!(height, 0, "Resolution height must be a positive number (is '{}')!", height);

        Resolution {
            width,
            height,
        }
    }

    /// This will set height to be width divided by `ratio`
    pub fn height_to_ratio<R: Into<AspectRatio>>(&mut self, ratio: R) {
        assert_ne!(self.width, 0, "Unable to set height of resolution from ratio when width is 0!");

        let ratio = ratio.into();

        self.height = (self.width as f32 / ratio.to_f32()) as u32;
    }

    /// This will set width to be height multiplied with `ratio`
    pub fn width_to_ratio<R: Into<AspectRatio>>(&mut self, ratio: R) {
        assert_ne!(self.height, 0, "Unable to set height of resolution from ratio when height is 0!");

        let ratio = ratio.into();

        self.width = (self.height as f32 * ratio.to_f32()) as u32;
    }

    /// This returns the aspect ratio (`width` divided by `height`)
    pub fn aspect_ratio(&self) -> f32 {
        assert_ne!(self.width, 0, "Unable to calculate aspect ratio of resolution when width is 0");
        assert_ne!(self.height, 0, "Unable to calculate aspect ratio of resolution when height is 0");

        self.width as f32 / self.height as f32
    }
}

#[cfg(feature = "winit")]
impl From<Resolution> for winit::dpi::PhysicalSize<u32> {
    fn from(res: Resolution) -> Self {
        PhysicalSize::new(res.width, res.height)
    }
}

#[cfg(feature = "winit")]
impl From<Resolution> for winit::dpi::LogicalSize<f64> {
    fn from(res: Resolution) -> Self {
        LogicalSize::new(res.width, res.height)
    }
}

#[derive(Debug, Clone)]
pub struct VideoMode {
    #[serde(flatten)]
    pub resolution: Resolution,
    pub bit_depth: u16,
    pub refresh_rate: u16,
    pub display: Option<DisplayHandle>
}

impl VideoMode {
    pub fn new<R: Into<Option<u16>>, D: Into<Option<DisplayHandle>>>(width: u32, height: u32, refresh_rate: R, display: D) -> Self {
        let refresh_rate = refresh_rate
            .into()
            .map(|ratio| if ratio > 0 { Some(ratio) } else { None })
            .flatten();

        VideoMode {
            resolution: Resolution::new(width, height),
            refresh_rate,
            display: display.into(),
        }
    }

    pub fn with_resolution(self, width: u32, height: u32) -> Self {
        VideoMode {
            resolution: Resolution::new(width, height),
            ..self
        }
    }

    pub fn with_refresh_rate<R: Into<Option<u16>>>(self, refresh_rate: R) -> Self {
        let refresh_rate = refresh_rate
            .into()
            .map(|ratio| if ratio > 0 { Some(ratio) } else { None })
            .flatten();

        VideoMode {
            refresh_rate,
            ..self
        }
    }

    pub fn with_display<D: Into<Option<DisplayHandle>>>(self, display: D) -> Self {
        VideoMode {
            display: display.into(),
            ..self
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.resolution.aspect_ratio()
    }
}

pub struct DisplayHandle {
    inner
}